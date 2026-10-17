//! Estado do servidor (A2): um único escritor, atrás de uma trava. Cada operação aplicada aumenta a
//! revisão e vira avisos `evt` difundidos a todos os clientes, na ordem; o motor e as saídas recebem
//! o que mudou para eles.

use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::extract::ws::Utf8Bytes;
use rlc_core::*;
use rlc_dmx::ftdi::{self, FtdiInfo, Protocol};
use rlc_engine::{Command, Engine, Published};
use serde::Deserialize;
use serde_json::{Value, json};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::{broadcast, watch};

use crate::Settings;
use crate::apply::{Topic, apply, intensity_patch, new_id};

/// Show de exemplo com que o servidor abre até a F5 (salvar e abrir). Os controladores MIDI dele são
/// simulados até a F12.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Example {
    project: Project,
    library: Vec<FixtureDef>,
    midi_devices: Vec<MidiDevice>,
}

pub fn example_snapshot(lan_url: Option<String>) -> Snapshot {
    let example: Example =
        serde_json::from_str(include_str!("exemplo.json")).expect("exemplo.json inválido");
    Snapshot {
        project: example.project,
        project_path: None,
        dirty: false,
        recent_projects: vec![],
        library: example.library,
        interfaces: vec![DmxInterface {
            id: "dummy".into(),
            kind: InterfaceKind::Dummy,
            kind_forced: false,
            name: "Dummy (virtual)".into(),
            serial: String::new(),
            connected: true,
            fps: 0.0,
            jitter_ms: 0.0,
            errors: 0,
        }],
        midi_devices: example.midi_devices,
        remote: RemoteAccess {
            enabled: true,
            pin: None,
            lan_url,
            clients: vec![],
        },
        running: vec![],
        desk: Desk {
            levels: vec![],
            groups: vec![],
        },
        master: Master {
            grand_master: 255,
            blackout: false,
            bpm: 120,
        },
        widget_values: vec![],
        stats: Stats {
            cpu_percent: 0.0,
            memory_mb: 0.0,
            tick_ms: 0.0,
            uptime_s: 0,
        },
        test_pattern: None,
    }
}

/// Uma interface aberta: a thread dela e o universo que ela manda.
struct Slot {
    interface: Id,
    universe: Arc<Mutex<Option<Id>>>,
    output: rlc_dmx::Output,
    frames_seen: u64,
}

pub struct Hub {
    pub snapshot: Snapshot,
    rev: u64,
    events: broadcast::Sender<Utf8Bytes>,
    engine: Engine,
    outputs: Vec<Slot>,
    /// Acesso pela rede local, acompanhado pelos clientes de fora (que caem quando ele é desligado).
    pub allow_lan: watch::Sender<bool>,
    started: Instant,
    last_stats: Instant,
    system: System,
}

impl Hub {
    pub fn new(settings: &Settings, port: u16) -> Self {
        let lan_url = local_ip_address::local_ip()
            .ok()
            .map(|ip| format!("http://{ip}:{port}"));
        let mut snapshot = example_snapshot(lan_url);
        snapshot.remote.enabled = settings.allow_lan;
        snapshot.remote.pin = settings.pin.clone();

        let engine = Engine::spawn();
        let dummy = start_output(&engine, &snapshot.project, &snapshot.interfaces[0]);
        let mut hub = Self {
            allow_lan: watch::channel(snapshot.remote.enabled).0,
            snapshot,
            rev: 0,
            events: broadcast::channel(1024).0,
            engine,
            outputs: vec![dummy],
            started: Instant::now(),
            last_stats: Instant::now(),
            system: System::new(),
        };
        hub.sync_engine(&[Topic::Project, Topic::Desk, Topic::Master]);
        hub
    }

    pub fn frames(&self) -> Published {
        self.engine.frames.clone()
    }

    /// Aplica uma operação de um cliente. Recusada, nada muda e ninguém é avisado.
    pub fn command(&mut self, op: Op) -> Result<Value, String> {
        let new_project = op == Op::NewProject;
        let kind_change = match &op {
            Op::SetInterfaceKind { interface, kind } => Some((interface.clone(), *kind)),
            _ => None,
        };
        let (mut topics, reply) = apply(&mut self.snapshot, op)?;
        // Tipo escolhido à mão: guardado pelo id da interface e com a saída reaberta no protocolo novo
        if let Some((id, kind)) = kind_change {
            let mut settings = Settings::load();
            match kind {
                Some(kind) => settings.interface_kinds.insert(id.clone(), kind),
                None => settings.interface_kinds.remove(&id),
            };
            settings.save();
            if let Some(interface) = self.snapshot.interfaces.iter().find(|i| i.id == id) {
                let slot = start_output(&self.engine, &self.snapshot.project, interface);
                self.outputs.retain(|s| s.interface != id);
                self.outputs.push(slot);
            }
        }
        if !new_project && !self.snapshot.dirty && topics.iter().any(|t| t.is_project()) {
            self.snapshot.dirty = true;
            topics.push(Topic::Dirty);
        }
        if topics.contains(&Topic::Remote) {
            let remote = &self.snapshot.remote;
            self.allow_lan.send_replace(remote.enabled);
            let mut settings = Settings::load();
            settings.allow_lan = remote.enabled;
            settings.pin = remote.pin.clone();
            settings.save();
        }
        self.publish(&topics);
        self.sync_engine(&topics);
        Ok(reply)
    }

    /// Avisa todos os clientes do que mudou, numa revisão nova.
    fn publish(&mut self, topics: &[Topic]) {
        self.rev += 1;
        for topic in topics {
            let event = json!({ "t": "evt", "rev": self.rev, "topic": topic.path(), "data": topic.data(&self.snapshot) });
            // Sem clientes, ninguém recebe: não é erro
            let _ = self.events.send(event.to_string().into());
        }
    }

    /// Estado completo, para quem acabou de conectar (ou ficou para trás), e a fila dos avisos
    /// seguintes: os dois saem juntos, então nenhum aviso se perde nem chega repetido.
    pub fn snapshot(&self) -> (Utf8Bytes, broadcast::Receiver<Utf8Bytes>) {
        let message = json!({ "t": "snapshot", "rev": self.rev, "data": &self.snapshot });
        (message.to_string().into(), self.events.subscribe())
    }

    /// Cliente novo: entra na lista dos aparelhos conectados.
    pub fn join(&mut self, address: SocketAddr, user_agent: &str) -> Id {
        let id = new_id();
        self.snapshot.remote.clients.push(RemoteClient {
            id: id.clone(),
            name: device_name(address, user_agent).into(),
            address: address.ip().to_string(),
        });
        self.publish(&[Topic::Remote]);
        id
    }

    pub fn leave(&mut self, id: &str) {
        self.snapshot.remote.clients.retain(|c| c.id != id);
        self.publish(&[Topic::Remote]);
    }

    /// Leva ao motor e às saídas o que mudou para eles.
    fn sync_engine(&mut self, topics: &[Topic]) {
        let any = |of: &[Topic]| topics.iter().any(|t| of.contains(t));
        let p = &self.snapshot.project;
        if any(&[
            Topic::Project,
            Topic::Universes,
            Topic::Fixtures,
            Topic::FixtureDefs,
        ]) {
            self.engine.send(Command::Patch(intensity_patch(p)));
            for slot in &self.outputs {
                if let Ok(mut patched) = slot.universe.lock() {
                    *patched = patched_universe(p, &slot.interface);
                }
            }
        }
        if any(&[Topic::Project, Topic::Desk]) {
            self.engine
                .send(Command::Desk(self.snapshot.desk.levels.clone()));
        }
        if any(&[Topic::Master]) {
            self.engine
                .send(Command::GrandMaster(self.snapshot.master.grand_master));
            self.engine
                .send(Command::Blackout(self.snapshot.master.blackout));
        }
        if any(&[Topic::TestPattern]) {
            self.engine.send(Command::Test(self.snapshot.test_pattern));
        }
    }

    /// A cada segundo: as interfaces USB ligadas agora (as novas ganham a sua saída) e os números do
    /// Monitoramento (o app, o motor e cada interface).
    pub fn refresh(&mut self, devices: &[FtdiInfo]) {
        for device in devices {
            let id = format!("ftdi-{}", device.serial);
            if self.snapshot.interfaces.iter().any(|i| i.id == id) {
                continue;
            }
            let forced = Settings::load().interface_kinds.get(&id).copied();
            let interface = DmxInterface {
                kind: forced.unwrap_or_else(|| kind_by_name(&device.description)),
                kind_forced: forced.is_some(),
                id,
                name: device.description.clone(),
                serial: device.serial.clone(),
                connected: true,
                fps: 0.0,
                jitter_ms: 0.0,
                errors: 0,
            };
            tracing::info!(
                "interface USB: {} ({}), {:?}",
                interface.name,
                interface.serial,
                interface.kind
            );
            self.outputs.push(start_output(
                &self.engine,
                &self.snapshot.project,
                &interface,
            ));
            self.snapshot.interfaces.push(interface);
        }
        // Desplugada, continua na lista (e no patch) até voltar
        for interface in &mut self.snapshot.interfaces {
            if interface.kind != InterfaceKind::Dummy {
                interface.connected = devices.iter().any(|d| d.serial == interface.serial);
            }
        }
        let seconds = self.last_stats.elapsed().as_secs_f32().max(0.001);
        self.last_stats = Instant::now();
        let pid = Pid::from_u32(std::process::id());
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );
        // O sistema mede a CPU por núcleo; o Monitoramento mostra a parte da máquina inteira
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get()) as f32;
        let (cpu, memory) = self
            .system
            .process(pid)
            .map(|p| (p.cpu_usage() / cores, p.memory() as f32 / 1_048_576.0))
            .unwrap_or_default();
        self.snapshot.stats = Stats {
            cpu_percent: cpu,
            memory_mb: memory.round(),
            tick_ms: self.engine.frames.load().tick_time.as_secs_f32() * 1000.0,
            uptime_s: self.started.elapsed().as_secs() as u32,
        };
        for slot in &mut self.outputs {
            let Some(interface) = self
                .snapshot
                .interfaces
                .iter_mut()
                .find(|i| i.id == slot.interface)
            else {
                continue;
            };
            let stats = &slot.output.stats;
            let frames = stats.frames.load(Ordering::Relaxed);
            interface.fps = ((frames - slot.frames_seen) as f32 / seconds * 10.0).round() / 10.0;
            interface.jitter_ms = stats.jitter_us.swap(0, Ordering::Relaxed) as f32 / 1000.0;
            interface.errors = stats.errors.load(Ordering::Relaxed);
            // Respondeu como Pro ao abrir: é uma Pro compatível, mesmo com outro nome
            if !interface.kind_forced && stats.pro.load(Ordering::Relaxed) {
                interface.kind = InterfaceKind::EnttecPro;
            }
            slot.frames_seen = frames;
        }
        self.publish(&[Topic::Stats, Topic::Interfaces]);
    }
}

/// Thread de saída de uma interface, já com o universo patchado nela.
fn start_output(engine: &Engine, project: &Project, interface: &DmxInterface) -> Slot {
    let universe = Arc::new(Mutex::new(patched_universe(project, &interface.id)));
    let (frames, patched) = (engine.frames.clone(), universe.clone());
    let source = move || {
        let id = patched.lock().ok()?.clone()?;
        frames.load().get(&id).copied()
    };
    let output = match interface.kind {
        InterfaceKind::Dummy => rlc_dmx::spawn("dummy", |_| Ok(Box::new(rlc_dmx::Dummy)), source),
        kind => {
            let serial = interface.serial.clone();
            let protocol = match kind {
                InterfaceKind::EnttecPro => Protocol::EnttecPro,
                _ => Protocol::OpenDmx,
            };
            let auto = !interface.kind_forced;
            rlc_dmx::spawn(
                &interface.serial,
                move |stats| ftdi::open(&serial, protocol, auto, stats),
                source,
            )
        }
    };
    Slot {
        interface: interface.id.clone(),
        universe,
        output,
        frames_seen: 0,
    }
}

/// Universo que manda na interface (uma interface manda um universo só).
fn patched_universe(p: &Project, interface: &str) -> Option<Id> {
    p.universes
        .iter()
        .find(|u| u.output.as_deref() == Some(interface))
        .map(|u| u.id.clone())
}

/// Tipo pelo nome USB, como no QLC+ (a Open DMX ainda pode virar Pro se responder como uma).
pub fn kind_by_name(description: &str) -> InterfaceKind {
    match ftdi::protocol_by_name(description) {
        Protocol::EnttecPro => InterfaceKind::EnttecPro,
        Protocol::OpenDmx => InterfaceKind::OpenDmx,
    }
}

/// Nome do aparelho na lista do ⚙: este computador ou o sistema do navegador.
fn device_name(address: SocketAddr, user_agent: &str) -> &'static str {
    if address.ip().is_loopback() {
        return "Este computador";
    }
    [
        ("iPhone", "iPhone"),
        ("iPad", "iPad"),
        ("Android", "Android"),
        ("Windows", "Windows"),
        ("Macintosh", "Mac"),
        ("Linux", "Linux"),
    ]
    .into_iter()
    .find(|(key, _)| user_agent.contains(key))
    .map_or("Navegador", |(_, name)| name)
}
