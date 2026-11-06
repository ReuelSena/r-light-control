//! Estado do servidor (A2): um único escritor, atrás de uma trava. Cada operação aplicada aumenta a
//! revisão e vira avisos `evt` difundidos a todos os clientes, na ordem; o motor e as saídas recebem
//! o que mudou para eles. Também é aqui que ficam os arquivos (projetos, recuperação e biblioteca) e o
//! histórico do desfazer.

use std::mem::{Discriminant, discriminant};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::ws::Utf8Bytes;
use rlc_core::*;
use rlc_dmx::ftdi::{self, FtdiInfo, Protocol};
use rlc_engine::{Command, Engine, Frames, Published};
use serde::Deserialize;
use serde_json::{Value, json};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::{broadcast, watch};

use crate::Settings;
use crate::apply::{
    Topic, apply, console_op, empty_project, fixture_blackout, highlight, is_live, new_id,
    preview_levels, running_programs, script_matrix, universe_patch,
};
use crate::script::Previews;
use crate::{library, project};

/// Passos guardados no histórico do desfazer.
const HISTORY: usize = 100;
/// Alterações seguidas com menos que isso entre elas (um arrasto de fader) viram um passo só.
const MERGE: Duration = Duration::from_millis(500);

/// Um passo do desfazer: como estavam os pedaços do estado que a alteração mudou.
type Step = Vec<(Topic, Value)>;

/// Show de exemplo: gravado na pasta dos projetos na primeira vez (e aberto nos testes, que não têm
/// pastas). A biblioteca dele é a da primeira vez (as genéricas). Os controladores MIDI são simulados
/// até a F12.
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
        recovered: false,
        projects: vec![],
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
        chaser_steps: vec![],
        failures: vec![],
        paused: vec![],
        loaded_steps: vec![],
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
        preview: None,
        highlight: Locate {
            fixtures: vec![],
            others: 255,
        },
    }
}

/// Uma interface aberta: a thread dela e o universo que ela manda.
struct Slot {
    interface: Id,
    universe: Arc<Mutex<Option<Id>>>,
    output: rlc_dmx::Output,
    frames_seen: u64,
}

/// Macro rodando: o elemento, o próximo passo e quando ele vence.
struct RunningMacro {
    widget: Id,
    next: usize,
    at: Instant,
}

pub struct Hub {
    pub snapshot: Snapshot,
    rev: u64,
    events: broadcast::Sender<Utf8Bytes>,
    engine: Engine,
    outputs: Vec<Slot>,
    /// Pasta do app (biblioteca e recuperação) e dos projetos; sem elas (nos testes), nada vai ao disco.
    app_dir: Option<PathBuf>,
    projects_dir: Option<PathBuf>,
    undo: Vec<Step>,
    redo: Vec<Step>,
    /// Última alteração que entrou no histórico, para juntar as seguidas.
    last_edit: Option<(Discriminant<Op>, Vec<Topic>, Instant)>,
    /// Alteração ainda fora da cópia de recuperação.
    recovery_pending: bool,
    /// Acesso pela rede local, acompanhado pelos clientes de fora (que caem quando ele é desligado).
    pub allow_lan: watch::Sender<bool>,
    /// Elementos do console que cada aparelho está apertando (aparelho, elemento): se ele cair, o
    /// servidor os solta (o flash não fica preso).
    holds: Vec<(Id, Id)>,
    /// Macros do console esperando o próximo passo (E6).
    macros: Vec<RunningMacro>,
    /// Há macros esperando: o relógio de 40 ms só trava o estado para andá-las quando há.
    pub macros_waiting: Arc<AtomicBool>,
    /// As sessões de script das prévias dos RGB Matrix.
    previews: Previews,
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
        if let Some(dir) = &settings.app_dir {
            snapshot.library = library::load(&dir.join("fixtures"), snapshot.library);
        }

        let engine = Engine::spawn();
        let dummy = start_output(&engine, &snapshot.project, &snapshot.interfaces[0]);
        let mut hub = Self {
            allow_lan: watch::channel(snapshot.remote.enabled).0,
            snapshot,
            rev: 0,
            events: broadcast::channel(1024).0,
            engine,
            outputs: vec![dummy],
            app_dir: settings.app_dir.clone(),
            projects_dir: settings.projects_dir.clone(),
            undo: vec![],
            redo: vec![],
            last_edit: None,
            recovery_pending: false,
            holds: vec![],
            macros: vec![],
            macros_waiting: Arc::default(),
            previews: Previews::default(),
            started: Instant::now(),
            last_stats: Instant::now(),
            system: System::new(),
        };
        hub.start_project(settings.last_project.as_deref());
        hub.sync_engine(&[Topic::Project, Topic::Desk, Topic::Master]);
        hub
    }

    /// O que abre ao iniciar: a recuperação; senão, o último projeto; senão, um projeto em branco.
    fn start_project(&mut self, last: Option<&str>) {
        let Some(dir) = self.projects_dir.clone() else {
            return;
        };
        // Primeira vez: a pasta nasce com o show de exemplo, para abrir pela lista
        if !dir.exists()
            && let Err(e) = project::save(&dir.join("Show de exemplo.rlc"), &self.snapshot.project)
        {
            tracing::warn!("não foi possível criar {}: {e}", dir.display());
        }
        let recovered = self.recovery_file().and_then(|f| project::read(&f).ok());
        if let Some((project, origin)) = recovered {
            tracing::info!("projeto recuperado da última sessão");
            let s = &mut self.snapshot;
            (s.project, s.project_path, s.dirty, s.recovered) = (project, origin, true, true);
        } else if last.is_none_or(|name| self.open(name).is_err()) {
            self.snapshot.project = empty_project();
        }
        self.snapshot.projects = project::list(&dir);
    }

    pub fn frames(&self) -> Published {
        self.engine.frames.clone()
    }

    /// Operação de um aparelho: anota o que ele aperta e solta no console, para soltar se ele cair.
    pub fn command_from(&mut self, client: &str, op: Op) -> Result<Value, String> {
        if let Op::PressWidget {
            widget,
            control: InputControl::Main,
            down,
        } = &op
        {
            self.holds.retain(|(_, w)| w != widget);
            if *down {
                self.holds.push((client.into(), widget.clone()));
            }
        }
        self.command(op)
    }

    /// Aplica uma operação de um cliente. Recusada, nada muda e ninguém é avisado.
    pub fn command(&mut self, op: Op) -> Result<Value, String> {
        let kind = |id: &Id| {
            let widgets = &self.snapshot.project.console.widgets;
            widgets.iter().find(|w| w.id == *id).map(|w| &w.kind)
        };
        // A macro anda aqui: o toque a começa ou, rodando, a interrompe; soltar não faz nada
        if let Op::PressWidget {
            widget,
            control: InputControl::Main,
            down,
        } = &op
            && let Some(VcWidgetKind::Macro(_)) = kind(widget)
        {
            if *down {
                self.toggle_macro(widget.clone());
            }
            return Ok(Value::Null);
        }
        // O pânico, o "Novo", o "Abrir" e o Stop All interrompem as macros (o Stop All de um passo
        // não interrompe a macro dele, que está fora da lista enquanto executa)
        let cancel = match &op {
            Op::Panic { .. } | Op::NewProject | Op::OpenProject { .. } | Op::StopAllFunctions => {
                true
            }
            Op::PressWidget {
                widget,
                control: InputControl::Main,
                down: true,
            } => {
                matches!(kind(widget), Some(VcWidgetKind::Button(b)) if b.action == ButtonAction::StopAll)
            }
            _ => false,
        };
        if cancel {
            let gone: Vec<Id> = self.macros.drain(..).map(|m| m.widget).collect();
            self.snapshot
                .widget_values
                .retain(|v| !gone.contains(&v.widget));
        }
        let op = console_op(&self.snapshot, op);
        // O pânico, o "Novo" e o "Abrir" param as cenas sem fade out
        let cut = matches!(
            op,
            Op::Panic { .. } | Op::NewProject | Op::OpenProject { .. }
        );
        // O passo da cue list e o BPM (do tap e do fader) vão direto ao motor, depois das criações rodando
        let direct = match &op {
            Op::StepChaser { id, action } => Some(Command::Step(id.clone(), *action)),
            Op::SetBpm { bpm } => Some(Command::Bpm(*bpm)),
            Op::SetTempo { bpm } => Some(Command::Tempo(*bpm)),
            _ => None,
        };
        // A prévia de um script espera a thread dele: fica fora do `apply`, que só vê o estado
        if let Op::PreviewMatrix { id, step } = &op
            && let Some(matrix) = script_matrix(&self.snapshot.project, id)
        {
            let preview = self
                .previews
                .preview(&self.snapshot.project, id, &matrix, *step)?;
            return Ok(json!(preview));
        }
        let (mut topics, reply) = match op {
            Op::OpenProject { name } => (self.open(&name)?, Value::Null),
            Op::SaveProject { name } => (self.save(name)?, Value::Null),
            Op::Undo => (self.step_back(true)?, Value::Null),
            Op::Redo => (self.step_back(false)?, Value::Null),
            op => self.edit(op)?,
        };
        // A macro removida (ou que virou outro elemento) para
        if topics.contains(&Topic::Console) {
            let widgets = &self.snapshot.project.console.widgets;
            let gone: Vec<Id> = self
                .macros
                .iter()
                .filter(|m| {
                    !widgets
                        .iter()
                        .any(|w| w.id == m.widget && matches!(w.kind, VcWidgetKind::Macro(_)))
                })
                .map(|m| m.widget.clone())
                .collect();
            if !gone.is_empty() {
                self.macros.retain(|m| !gone.contains(&m.widget));
                self.snapshot
                    .widget_values
                    .retain(|v| !gone.contains(&v.widget));
                topics.push(Topic::WidgetValues);
            }
        }
        if cut {
            self.engine.send(Command::Cut);
        }
        self.publish(&topics);
        self.sync_engine(&topics);
        if let Some(command) = direct {
            self.engine.send(command);
        }
        Ok(reply)
    }

    /// O toque numa macro: rodando, ela é interrompida; senão, começa, acesa enquanto roda.
    fn toggle_macro(&mut self, widget: Id) {
        let waiting = self.macros.len();
        self.macros.retain(|m| m.widget != widget);
        let start = self.macros.len() == waiting;
        self.light_macro(&widget, start);
        if start {
            let at = Instant::now();
            self.run_macro(RunningMacro {
                widget,
                next: 0,
                at,
            });
        }
    }

    fn light_macro(&mut self, widget: &Id, on: bool) {
        let values = &mut self.snapshot.widget_values;
        values.retain(|v| v.widget != *widget);
        if on {
            values.push(WidgetValue {
                widget: widget.clone(),
                value: vec![1],
            });
        }
        self.publish(&[Topic::WidgetValues]);
    }

    /// Executa os passos da macro até uma espera, quando ela volta à lista, ou até o fim. Cada passo
    /// é uma operação comum (avisos e motor); um passo recusado não para a macro.
    fn run_macro(&mut self, mut m: RunningMacro) {
        let steps = self
            .snapshot
            .project
            .console
            .widgets
            .iter()
            .find_map(|w| match &w.kind {
                VcWidgetKind::Macro(mc) if w.id == m.widget => Some(mc.steps.clone()),
                _ => None,
            })
            .unwrap_or_default();
        while let Some(step) = steps.get(m.next) {
            m.next += 1;
            let run = |id: &Id, running| Op::SetFunctionRunning {
                id: id.clone(),
                running,
            };
            let op = match step {
                MacroStep::Wait { ms } => {
                    m.at += Duration::from_millis((*ms).into());
                    self.macros.push(m);
                    self.macros_waiting.store(true, Ordering::Relaxed);
                    return;
                }
                MacroStep::Start { function }
                | MacroStep::Stop { function }
                | MacroStep::Go { chaser: function }
                    if !self
                        .snapshot
                        .project
                        .functions
                        .iter()
                        .any(|f| f.id == *function) =>
                {
                    tracing::warn!(
                        "macro {}, passo {}: criação não encontrada",
                        m.widget,
                        m.next
                    );
                    continue;
                }
                MacroStep::Start { function } => run(function, true),
                MacroStep::Stop { function } => run(function, false),
                MacroStep::Go { chaser } if self.snapshot.running.contains(chaser) => {
                    Op::StepChaser {
                        id: chaser.clone(),
                        action: StepAction::Next,
                    }
                }
                MacroStep::Go { chaser } => run(chaser, true),
                MacroStep::StopAll => Op::StopAllFunctions,
                MacroStep::Blackout { on } => Op::SetBlackout { on: *on },
                MacroStep::Bpm { bpm } => Op::SetTempo { bpm: *bpm },
            };
            if let Err(e) = self.command(op) {
                tracing::warn!("macro {}, passo {}: {e}", m.widget, m.next);
            }
        }
        self.light_macro(&m.widget, false);
    }

    /// Relógio das macros (a cada 40 ms): executa os passos das que venceram a espera.
    pub fn advance_macros(&mut self) {
        let now = Instant::now();
        while let Some(i) = self.macros.iter().position(|m| m.at <= now) {
            let m = self.macros.remove(i);
            self.run_macro(m);
        }
        self.macros_waiting
            .store(!self.macros.is_empty(), Ordering::Relaxed);
    }

    /// Acompanha o motor: o passo atual de cada chaser e as criações que terminaram sozinhas (o
    /// Single Shot que apagou no fim), que saem das criações rodando.
    pub fn follow_engine(&mut self, frames: &Frames) {
        let s = &mut self.snapshot;
        let mut topics = vec![];
        if s.running.iter().any(|id| frames.done.contains(id)) {
            s.running.retain(|id| !frames.done.contains(id));
            topics.push(Topic::Running);
            if s.paused.iter().any(|id| frames.done.contains(id)) {
                s.paused.retain(|id| !frames.done.contains(id));
                topics.push(Topic::Paused);
            }
        }
        // Uma criação que falhou fica na lista até rodar de novo (ou sair do projeto)
        let mut failures: Vec<RunFailure> = s
            .failures
            .iter()
            .filter(|f| !s.running.contains(&f.function))
            .cloned()
            .collect();
        for (id, message) in &frames.failed {
            if !failures.iter().any(|f| f.function == *id) {
                failures.push(RunFailure {
                    function: id.clone(),
                    message: message.clone(),
                });
            }
        }
        failures.retain(|f| s.project.functions.iter().any(|x| x.id == f.function));
        if failures != s.failures {
            s.failures = failures;
            topics.push(Topic::Failures);
        }
        let steps: Vec<CurrentStep> = frames
            .chasers
            .iter()
            .filter(|(id, _)| s.running.contains(id))
            .map(|(id, step)| CurrentStep {
                chaser: id.clone(),
                step: *step,
            })
            .collect();
        if steps != s.chaser_steps {
            s.chaser_steps = steps;
            topics.push(Topic::ChaserSteps);
        }
        if !topics.is_empty() {
            self.publish(&topics);
            self.sync_engine(&topics);
        }
    }

    /// Operações do contrato (`apply`), com o histórico do desfazer e o que vai para o disco.
    fn edit(&mut self, op: Op) -> Result<(Vec<Topic>, Value), String> {
        let new_project = op == Op::NewProject;
        let kind_change = match &op {
            Op::SetInterfaceKind { interface, kind } => Some((interface.clone(), *kind)),
            _ => None,
        };
        let before = (!is_live(&op) && !new_project).then(|| self.snapshot.clone());
        let kind = discriminant(&op);
        let (mut topics, reply) = apply(&mut self.snapshot, op)?;
        if let Some(before) = &before {
            self.record(kind, &topics, before);
            if topics.contains(&Topic::Library) {
                self.sync_library(&before.library);
            }
        }
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
        if new_project {
            self.undo.clear();
            self.redo.clear();
            self.clean(&mut topics);
        } else {
            self.changed(&mut topics);
        }
        if topics.contains(&Topic::Remote) {
            let remote = &self.snapshot.remote;
            self.allow_lan.send_replace(remote.enabled);
            let mut settings = Settings::load();
            settings.allow_lan = remote.enabled;
            settings.pin = remote.pin.clone();
            settings.save();
        }
        Ok((topics, reply))
    }

    /// Guarda no histórico como estavam os pedaços que a alteração mudou. Alterações seguidas da mesma
    /// operação nos mesmos pedaços (um arrasto de fader) viram um passo só.
    fn record(&mut self, kind: Discriminant<Op>, topics: &[Topic], before: &Snapshot) {
        let topics: Vec<Topic> = topics.iter().copied().filter(|t| t.undoable()).collect();
        if topics.is_empty() {
            return;
        }
        self.redo.clear();
        let now = Instant::now();
        let same = self
            .last_edit
            .as_ref()
            .is_some_and(|(k, t, at)| *k == kind && *t == topics && now - *at < MERGE);
        if !same || self.undo.is_empty() {
            self.undo
                .push(topics.iter().map(|t| (*t, t.data(before))).collect());
            if self.undo.len() > HISTORY {
                self.undo.remove(0);
            }
        }
        self.last_edit = Some((kind, topics, now));
    }

    /// Desfaz (ou refaz) um passo: devolve os pedaços guardados e guarda os de agora para o caminho
    /// de volta.
    fn step_back(&mut self, undo: bool) -> Result<Vec<Topic>, String> {
        let (from, to) = match undo {
            true => (&mut self.undo, &mut self.redo),
            false => (&mut self.redo, &mut self.undo),
        };
        let step = from.pop().ok_or(match undo {
            true => "Nada para desfazer.",
            false => "Nada para refazer.",
        })?;
        let library = self.snapshot.library.clone();
        let mut back = Vec::new();
        for (topic, value) in step {
            back.push((topic, topic.data(&self.snapshot)));
            topic.restore(&mut self.snapshot, value)?;
        }
        let mut topics: Vec<Topic> = back.iter().map(|(t, _)| *t).collect();
        to.push(back);
        self.last_edit = None;
        if topics.contains(&Topic::Library) {
            self.sync_library(&library);
        }
        self.changed(&mut topics);
        Ok(topics)
    }

    /// Alteração no projeto: fica marcada como não salva e vai para a cópia de recuperação.
    fn changed(&mut self, topics: &mut Vec<Topic>) {
        if !topics.iter().any(|t| t.is_project()) {
            return;
        }
        self.recovery_pending = true;
        if !self.snapshot.dirty {
            self.snapshot.dirty = true;
            topics.push(Topic::Dirty);
        }
    }

    /// Projeto novo, aberto ou salvo: a cópia de recuperação não vale mais.
    fn clean(&mut self, topics: &mut Vec<Topic>) {
        self.recovery_pending = false;
        self.snapshot.recovered = false;
        topics.push(Topic::Recovered);
        if let Some(file) = self.recovery_file().filter(|f| f.exists())
            && let Err(e) = std::fs::remove_file(&file)
        {
            tracing::warn!("não foi possível apagar {}: {e}", file.display());
        }
    }

    fn recovery_file(&self) -> Option<PathBuf> {
        self.app_dir.as_ref().map(|d| d.join("recuperacao.rlc"))
    }

    /// Último projeto aberto ou salvo, que o app abre ao iniciar (só com as pastas do usuário).
    fn remember(&self, name: &str) {
        if self.app_dir.is_some() {
            let mut settings = Settings::load();
            settings.last_project = Some(name.into());
            settings.save();
        }
    }

    /// Abre um projeto da pasta. Um arquivo inválido é recusado, e o projeto aberto continua como está.
    fn open(&mut self, name: &str) -> Result<Vec<Topic>, String> {
        let dir = self.projects_dir.clone().ok_or("Sem pasta de projetos.")?;
        let path = project::path(&dir, name)?;
        let (mut project, _) = project::read(&path)?;
        project.name = project::name(&path);
        // O "Novo" solta o que é ao vivo (mesa, criações rodando, prévia); depois entra o projeto lido
        let (mut topics, _) = apply(&mut self.snapshot, Op::NewProject)?;
        self.remember(&project.name);
        self.snapshot.project = project;
        self.snapshot.project_path = Some(path.display().to_string());
        self.undo.clear();
        self.redo.clear();
        self.clean(&mut topics);
        Ok(topics)
    }

    /// Salva no arquivo atual ou, com nome, como um projeto novo na pasta (o "Salvar como").
    fn save(&mut self, name: Option<String>) -> Result<Vec<Topic>, String> {
        let dir = self.projects_dir.clone().ok_or("Sem pasta de projetos.")?;
        let mut topics = vec![
            Topic::ProjectName,
            Topic::ProjectPath,
            Topic::Dirty,
            Topic::Projects,
        ];
        let path = match name {
            Some(name) => {
                let path = project::path(&dir, &name)?;
                if path.exists() {
                    return Err(format!(
                        "Já existe um projeto chamado \"{}\".",
                        project::name(&path)
                    ));
                }
                path
            }
            None => self
                .snapshot
                .project_path
                .as_ref()
                .map(PathBuf::from)
                .ok_or("O projeto ainda não tem nome: use \"Salvar como\".")?,
        };
        let mut project = self.snapshot.project.clone();
        project.name = project::name(&path);
        project::save(&path, &project).map_err(|e| format!("Não foi possível salvar: {e}"))?;
        self.remember(&project.name);
        let s = &mut self.snapshot;
        s.project.name = project.name;
        s.project_path = Some(path.display().to_string());
        s.dirty = false;
        s.projects = project::list(&dir);
        self.clean(&mut topics);
        Ok(topics)
    }

    /// Leva ao disco o que mudou na biblioteca (salvar, importar, remover ou desfazer).
    fn sync_library(&self, before: &[FixtureDef]) {
        let Some(dir) = self.app_dir.as_ref().map(|d| d.join("fixtures")) else {
            return;
        };
        let now = &self.snapshot.library;
        for def in now.iter().filter(|d| !before.contains(d)) {
            library::save(&dir, def);
        }
        for def in before.iter().filter(|d| !now.iter().any(|x| x.id == d.id)) {
            library::remove(&dir, &def.id);
        }
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

    /// O aparelho saiu (ou caiu): o que ele estava apertando no console é solto.
    pub fn leave(&mut self, id: &str) {
        self.snapshot.remote.clients.retain(|c| c.id != id);
        self.publish(&[Topic::Remote]);
        let held: Vec<Id> = self
            .holds
            .iter()
            .filter(|(client, _)| client == id)
            .map(|(_, widget)| widget.clone())
            .collect();
        self.holds.retain(|(client, _)| client != id);
        for widget in held {
            let release = Op::PressWidget {
                widget,
                control: InputControl::Main,
                down: false,
            };
            // Um elemento removido nesse meio tempo não tem o que soltar
            let _ = self.command(release);
        }
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
            self.engine.send(Command::Patch(universe_patch(p)));
            for slot in &self.outputs {
                if let Ok(mut patched) = slot.universe.lock() {
                    *patched = patched_universe(p, &slot.interface);
                }
            }
        }
        // As criações rodando acompanham as edições, inclusive a delas mesmas e a dos grupos (N1), e os
        // tempos ajustados ao vivo no console
        if any(&[
            Topic::Project,
            Topic::Universes,
            Topic::Fixtures,
            Topic::FixtureDefs,
            Topic::FixtureGroups,
            Topic::ChannelGroups,
            Topic::Functions,
            Topic::Running,
            Topic::Paused,
            Topic::Console,
            Topic::WidgetValues,
        ]) {
            self.engine
                .send(Command::Running(running_programs(&self.snapshot)));
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
        // A prévia acompanha cada mudança da cena em edição e dos grupos dela
        if any(&[
            Topic::Project,
            Topic::Universes,
            Topic::Fixtures,
            Topic::FixtureDefs,
            Topic::FixtureGroups,
            Topic::Functions,
            Topic::Preview,
            Topic::Highlight,
        ]) {
            self.engine
                .send(Command::Preview(preview_levels(&self.snapshot)));
            self.engine
                .send(Command::Highlight(highlight(&self.snapshot)));
        }
        if any(&[
            Topic::Project,
            Topic::Universes,
            Topic::Fixtures,
            Topic::FixtureDefs,
            Topic::FixtureGroups,
            Topic::Console,
            Topic::WidgetValues,
        ]) {
            self.engine
                .send(Command::FixtureBlackout(fixture_blackout(&self.snapshot)));
        }
    }

    /// A cada segundo: as interfaces USB ligadas agora (as novas ganham a sua saída), os números do
    /// Monitoramento (o app, o motor e cada interface), a lista dos projetos e a cópia de recuperação.
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
        let mut topics = vec![Topic::Stats, Topic::Interfaces];
        // Projetos copiados para a pasta (ou apagados dela) aparecem na lista sem reiniciar o app
        if let Some(dir) = &self.projects_dir {
            let projects = project::list(dir);
            if projects != self.snapshot.projects {
                self.snapshot.projects = projects;
                topics.push(Topic::Projects);
            }
        }
        self.publish(&topics);
        // Cópia de recuperação das alterações não salvas (no máximo uma vez por segundo)
        if std::mem::take(&mut self.recovery_pending)
            && self.snapshot.dirty
            && let Some(file) = self.recovery_file()
        {
            let s = &self.snapshot;
            project::save_recovery(&file, &s.project, s.project_path.as_deref());
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Hub com uma pasta de projetos temporária (sem a pasta do app: nada vai para o settings.json).
    fn hub() -> (Hub, PathBuf) {
        let dir = std::env::temp_dir().join(format!("rlc-hub-{}", new_id()));
        let settings = Settings {
            projects_dir: Some(dir.clone()),
            ..Default::default()
        };
        (Hub::new(&settings, 0), dir)
    }

    #[test]
    fn the_preview_of_a_script_matrix_comes_from_its_own_session() {
        use crate::script::TEST_SCRIPT;
        let (mut hub, dir) = hub();
        hub.command(Op::OpenProject {
            name: "Show de exemplo".into(),
        })
        .unwrap();
        let source = TEST_SCRIPT.to_string();
        hub.command(Op::ImportScript {
            file_name: "teste.js".into(),
            source,
        })
        .unwrap();
        // O RGB Matrix "r1" do exemplo passa a usar o script
        let mut function = hub
            .snapshot
            .project
            .functions
            .iter()
            .find(|f| f.id == "r1")
            .unwrap()
            .clone();
        if let FunctionKind::RgbMatrix(m) = &mut function.kind {
            m.algorithm = RgbAlgorithm::Script {
                script: "Teste".into(),
                properties: vec![],
            };
        }
        hub.command(Op::SaveFunction {
            function: function.clone(),
        })
        .unwrap();
        // 3 passos; o passo 1 pinta de verde (1 + 1) × 20 × 1 = 40, na grade 8 × 2 do grupo
        let preview = hub
            .command(Op::PreviewMatrix {
                id: "r1".into(),
                step: 1,
            })
            .unwrap();
        assert_eq!(preview["steps"], 3);
        assert_eq!(preview["map"], json!([vec![40 << 8; 8], vec![40 << 8; 8]]));
        // O passo passa do último: volta ao primeiro
        let preview = hub
            .command(Op::PreviewMatrix {
                id: "r1".into(),
                step: 3,
            })
            .unwrap();
        assert_eq!(preview["map"][0][0], 20 << 8);
        // Um script que falha vira o erro da prévia, sem derrubar o servidor
        let broken = "(function () { throw new Error('quebrado'); })()".to_string();
        let i = hub
            .snapshot
            .project
            .scripts
            .iter()
            .position(|s| s.id == "Teste")
            .unwrap();
        let old = hub.snapshot.project.scripts[i].clone();
        hub.snapshot.project.scripts[i] = RgbScript {
            source: broken,
            ..old
        };
        let error = hub
            .command(Op::PreviewMatrix {
                id: "r1".into(),
                step: 0,
            })
            .unwrap_err();
        assert!(error.contains("quebrado"), "{error}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_example_script_matrices_draw_their_grids() {
        let (mut hub, dir) = hub();
        hub.command(Op::OpenProject {
            name: "Show de exemplo".into(),
        })
        .unwrap();
        for (id, width, height) in [("r3", 8, 2), ("r4", 12, 1), ("r5", 8, 2)] {
            let first = hub
                .command(Op::PreviewMatrix {
                    id: id.into(),
                    step: 0,
                })
                .unwrap();
            let steps = first["steps"].as_u64().unwrap() as u32;
            let mut lit = false;
            for step in 0..steps {
                let preview = hub
                    .command(Op::PreviewMatrix {
                        id: id.into(),
                        step,
                    })
                    .unwrap();
                let map = preview["map"].as_array().unwrap();
                assert_eq!(map.len(), height, "{id}");
                for row in map {
                    assert_eq!(row.as_array().unwrap().len(), width, "{id}");
                    lit |= row
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|c| c.as_u64() != Some(0));
                }
            }
            assert!(lit, "{id} não acendeu nada em {steps} passos");
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn first_start_is_blank_with_the_example_in_the_folder() {
        let (hub, dir) = hub();
        assert_eq!(hub.snapshot.project.name, "Novo projeto");
        assert_eq!(hub.snapshot.projects, ["Show de exemplo"]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn save_as_and_open_round_trip() {
        let (mut hub, dir) = hub();
        hub.command(Op::OpenProject {
            name: "Show de exemplo".into(),
        })
        .unwrap();
        let example = hub.snapshot.project.clone();
        assert!(!example.fixtures.is_empty());
        hub.command(Op::AddUniverse).unwrap();
        assert!(hub.snapshot.dirty);
        hub.command(Op::SaveProject {
            name: Some("Culto".into()),
        })
        .unwrap();
        assert!(!hub.snapshot.dirty);
        assert_eq!(hub.snapshot.project.name, "Culto");
        assert_eq!(hub.snapshot.projects[0], "Culto");
        let saved = hub.snapshot.project.clone();
        // Nome repetido é recusado; um arquivo inválido também, sem mexer no projeto aberto
        assert!(
            hub.command(Op::SaveProject {
                name: Some("Culto".into())
            })
            .is_err()
        );
        std::fs::write(dir.join("Ruim.rlc"), "{").unwrap();
        assert!(
            hub.command(Op::OpenProject {
                name: "Ruim".into()
            })
            .is_err()
        );
        assert_eq!(hub.snapshot.project, saved);
        // Abrir de novo dá o mesmo projeto, e o histórico começa vazio
        hub.command(Op::NewProject).unwrap();
        hub.command(Op::OpenProject {
            name: "Culto".into(),
        })
        .unwrap();
        assert_eq!(hub.snapshot.project, saved);
        assert!(hub.command(Op::Undo).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn what_a_device_holds_is_released_when_it_leaves() {
        let (mut hub, dir) = hub();
        hub.command(Op::OpenProject {
            name: "Show de exemplo".into(),
        })
        .unwrap();
        let press = |widget: &str| Op::PressWidget {
            widget: widget.into(),
            control: InputControl::Main,
            down: true,
        };
        // O aparelho segura o Temp "b11" (Haze leve) e liga a "b1" (liga/desliga)
        hub.command_from("aparelho", press("b11")).unwrap();
        hub.command_from("aparelho", press("b1")).unwrap();
        assert_eq!(hub.snapshot.running, ["s5", "s1"]);
        // Ele cai: o Temp solta, e o liga/desliga continua
        hub.leave("aparelho");
        assert_eq!(hub.snapshot.running, ["s1"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Show de exemplo com a macro "m1": liga s1, espera 1 min, liga s5 e para s1.
    fn hub_with_macro() -> (Hub, PathBuf) {
        let (mut hub, dir) = hub();
        hub.command(Op::OpenProject {
            name: "Show de exemplo".into(),
        })
        .unwrap();
        let widgets = &hub.snapshot.project.console.widgets;
        let mut m = widgets.iter().find(|w| w.id == "b1").unwrap().clone();
        m.id = "m1".into();
        m.inputs.clear();
        m.kind = VcWidgetKind::Macro(VcMacro {
            steps: vec![
                MacroStep::Start {
                    function: "s1".into(),
                },
                MacroStep::Wait { ms: 60_000 },
                MacroStep::Start {
                    function: "s5".into(),
                },
                MacroStep::Stop {
                    function: "s1".into(),
                },
            ],
            icon: None,
        });
        hub.command(Op::SaveWidgets { widgets: vec![m] }).unwrap();
        (hub, dir)
    }

    fn press_macro(hub: &mut Hub) {
        let op = Op::PressWidget {
            widget: "m1".into(),
            control: InputControl::Main,
            down: true,
        };
        hub.command(op).unwrap();
    }

    fn lit(hub: &Hub) -> bool {
        hub.snapshot.widget_values.iter().any(|v| v.widget == "m1")
    }

    #[test]
    fn a_macro_runs_its_steps_and_waits() {
        let (mut hub, dir) = hub_with_macro();
        press_macro(&mut hub);
        assert_eq!(hub.snapshot.running, ["s1"]);
        assert!(lit(&hub));
        // Antes da espera vencer, nada anda; vencida, o resto executa e a macro apaga
        hub.advance_macros();
        assert_eq!(hub.snapshot.running, ["s1"]);
        hub.macros[0].at = Instant::now();
        hub.advance_macros();
        assert_eq!(hub.snapshot.running, ["s5"]);
        assert!(!lit(&hub) && hub.macros.is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_second_touch_or_the_panic_interrupts_the_macro() {
        let (mut hub, dir) = hub_with_macro();
        press_macro(&mut hub);
        press_macro(&mut hub);
        assert!(!lit(&hub) && hub.macros.is_empty());
        assert_eq!(hub.snapshot.running, ["s1"]);
        press_macro(&mut hub);
        assert!(lit(&hub));
        hub.command(Op::Panic { all: false }).unwrap();
        assert!(!lit(&hub) && hub.macros.is_empty());
        assert!(hub.snapshot.running.is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn engine_steps_reach_the_state_and_finished_creations_stop() {
        let (mut hub, dir) = hub();
        hub.command(Op::OpenProject {
            name: "Show de exemplo".into(),
        })
        .unwrap();
        for id in ["c1", "q1"] {
            let run = Op::SetFunctionRunning {
                id: id.into(),
                running: true,
            };
            hub.command(run).unwrap();
        }
        // A sequência terminou sozinha; um chaser que já foi parado não aparece
        let frames = Frames {
            chasers: vec![("c1".into(), 2), ("q1".into(), 0), ("parado".into(), 1)],
            done: vec!["q1".into()],
            ..Default::default()
        };
        hub.follow_engine(&frames);
        let step = CurrentStep {
            chaser: "c1".into(),
            step: 2,
        };
        assert_eq!(hub.snapshot.running, ["c1"]);
        assert_eq!(hub.snapshot.chaser_steps, [step]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn undo_and_redo_restore_the_state_and_skip_live_operations() {
        let (mut hub, dir) = hub();
        let blank = hub.snapshot.project.clone();
        hub.command(Op::AddUniverse).unwrap();
        let with_two = hub.snapshot.project.clone();
        hub.command(Op::SetGrandMaster { value: 10 }).unwrap();
        // Um arrasto: palcos seguidos viram um passo só
        for width in [11.0, 12.0, 13.0] {
            let mut stage = hub.snapshot.project.stage.clone();
            stage.width = width;
            hub.command(Op::SaveStage { stage }).unwrap();
        }
        hub.command(Op::Undo).unwrap();
        assert_eq!(hub.snapshot.project, with_two);
        hub.command(Op::Undo).unwrap();
        assert_eq!(hub.snapshot.project, blank);
        assert_eq!(hub.snapshot.master.grand_master, 10);
        assert_eq!(hub.command(Op::Undo).unwrap_err(), "Nada para desfazer.");
        hub.command(Op::Redo).unwrap();
        hub.command(Op::Redo).unwrap();
        assert_eq!(hub.snapshot.project.stage.width, 13.0);
        assert!(hub.command(Op::Redo).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
