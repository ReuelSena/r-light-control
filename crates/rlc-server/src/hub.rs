//! Estado do servidor (A2): um único escritor, atrás de uma trava. Cada operação aplicada aumenta a
//! revisão e vira avisos `evt` difundidos a todos os clientes, na ordem; o motor e as saídas recebem
//! o que mudou para eles.

use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::extract::ws::Utf8Bytes;
use rlc_core::*;
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
        let universe = Arc::new(Mutex::new(None::<Id>));
        let (frames, patched) = (engine.frames.clone(), universe.clone());
        let output = rlc_dmx::spawn("dummy", rlc_dmx::Dummy, 44, move || {
            let id = patched.lock().ok()?.clone()?;
            frames.load().get(&id).copied()
        });
        let mut hub = Self {
            allow_lan: watch::channel(snapshot.remote.enabled).0,
            snapshot,
            rev: 0,
            events: broadcast::channel(1024).0,
            engine,
            outputs: vec![Slot {
                interface: "dummy".into(),
                universe,
                output,
                frames_seen: 0,
            }],
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
        let (mut topics, reply) = apply(&mut self.snapshot, op)?;
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
                let universe = p
                    .universes
                    .iter()
                    .find(|u| u.output.as_ref() == Some(&slot.interface));
                if let Ok(mut patched) = slot.universe.lock() {
                    *patched = universe.map(|u| u.id.clone());
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

    /// Números do Monitoramento, a cada segundo: o app, o motor e cada interface.
    pub fn refresh_stats(&mut self) {
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
            slot.frames_seen = frames;
        }
        self.publish(&[Topic::Stats, Topic::Interfaces]);
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
