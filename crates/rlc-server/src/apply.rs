//! Operações do contrato (A2): cada uma é validada, aplicada no estado e devolve os pedaços do
//! estado que mudaram, que viram os avisos `evt` (A3). As regras vêm do motor de mentira da F2.
//! Uma operação recusada não muda nada.

use rlc_core::*;
use rlc_engine::{
    ChaserProgram, EfxProgram, Program, Run, SceneLevel, SceneProgram, Span, Step, UniversePatch,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

/// Pedaço do estado levado por um aviso de mudança: o caminho dele no `Snapshot`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Topic {
    Project,
    ProjectName,
    Universes,
    FixtureDefs,
    Fixtures,
    FixtureGroups,
    ChannelGroups,
    Functions,
    Folders,
    Console,
    Stage,
    ProjectPath,
    Dirty,
    Recovered,
    Projects,
    Library,
    Interfaces,
    Remote,
    Running,
    ChaserSteps,
    Desk,
    Master,
    WidgetValues,
    Stats,
    TestPattern,
    Preview,
    Highlight,
}

impl Topic {
    pub fn path(self) -> &'static str {
        match self {
            Topic::Project => "project",
            Topic::ProjectName => "project.name",
            Topic::Universes => "project.universes",
            Topic::FixtureDefs => "project.fixtureDefs",
            Topic::Fixtures => "project.fixtures",
            Topic::FixtureGroups => "project.fixtureGroups",
            Topic::ChannelGroups => "project.channelGroups",
            Topic::Functions => "project.functions",
            Topic::Folders => "project.folders",
            Topic::Console => "project.console",
            Topic::Stage => "project.stage",
            Topic::ProjectPath => "projectPath",
            Topic::Dirty => "dirty",
            Topic::Recovered => "recovered",
            Topic::Projects => "projects",
            Topic::Library => "library",
            Topic::Interfaces => "interfaces",
            Topic::Remote => "remote",
            Topic::Running => "running",
            Topic::ChaserSteps => "chaserSteps",
            Topic::Desk => "desk",
            Topic::Master => "master",
            Topic::WidgetValues => "widgetValues",
            Topic::Stats => "stats",
            Topic::TestPattern => "testPattern",
            Topic::Preview => "preview",
            Topic::Highlight => "highlight",
        }
    }

    /// Faz parte do projeto (e deixa alterações não salvas).
    pub fn is_project(self) -> bool {
        self.path() == "project" || self.path().starts_with("project.")
    }

    /// Entra no desfazer: as partes do projeto e a biblioteca. O projeto inteiro só muda no "Novo" e
    /// no "Abrir", que começam um histórico novo.
    pub fn undoable(self) -> bool {
        (self.is_project() && self != Topic::Project) || self == Topic::Library
    }

    /// Devolve um pedaço guardado pelo desfazer.
    pub fn restore(self, s: &mut Snapshot, value: Value) -> Result<(), String> {
        fn set<T: DeserializeOwned>(slot: &mut T, value: Value) -> Result<(), String> {
            *slot = serde_json::from_value(value).map_err(|e| e.to_string())?;
            Ok(())
        }
        let p = &mut s.project;
        match self {
            Topic::ProjectName => set(&mut p.name, value),
            Topic::Universes => set(&mut p.universes, value),
            Topic::FixtureDefs => set(&mut p.fixture_defs, value),
            Topic::Fixtures => set(&mut p.fixtures, value),
            Topic::FixtureGroups => set(&mut p.fixture_groups, value),
            Topic::ChannelGroups => set(&mut p.channel_groups, value),
            Topic::Functions => set(&mut p.functions, value),
            Topic::Folders => set(&mut p.folders, value),
            Topic::Console => set(&mut p.console, value),
            Topic::Stage => set(&mut p.stage, value),
            Topic::Library => set(&mut s.library, value),
            _ => Err(format!("{} não se desfaz.", self.path())),
        }
    }

    pub fn data(self, s: &Snapshot) -> Value {
        let p = &s.project;
        match self {
            Topic::Project => json!(p),
            Topic::ProjectName => json!(p.name),
            Topic::Universes => json!(p.universes),
            Topic::FixtureDefs => json!(p.fixture_defs),
            Topic::Fixtures => json!(p.fixtures),
            Topic::FixtureGroups => json!(p.fixture_groups),
            Topic::ChannelGroups => json!(p.channel_groups),
            Topic::Functions => json!(p.functions),
            Topic::Folders => json!(p.folders),
            Topic::Console => json!(p.console),
            Topic::Stage => json!(p.stage),
            Topic::ProjectPath => json!(s.project_path),
            Topic::Dirty => json!(s.dirty),
            Topic::Recovered => json!(s.recovered),
            Topic::Projects => json!(s.projects),
            Topic::Library => json!(s.library),
            Topic::Interfaces => json!(s.interfaces),
            Topic::Remote => json!(s.remote),
            Topic::Running => json!(s.running),
            Topic::ChaserSteps => json!(s.chaser_steps),
            Topic::Desk => json!(s.desk),
            Topic::Master => json!(s.master),
            Topic::WidgetValues => json!(s.widget_values),
            Topic::Stats => json!(s.stats),
            Topic::TestPattern => json!(s.test_pattern),
            Topic::Preview => json!(s.preview),
            Topic::Highlight => json!(s.highlight),
        }
    }
}

/// Operação ao vivo: não entra no desfazer (um Ctrl + Z no show não mexe na luz).
pub fn is_live(op: &Op) -> bool {
    matches!(
        op,
        Op::SetFunctionRunning { .. }
            | Op::StepChaser { .. }
            | Op::StopAllFunctions
            | Op::Panic { .. }
            | Op::SetDeskLevel { .. }
            | Op::SetDeskGroupLevel { .. }
            | Op::ResetDesk { .. }
            | Op::SetWidgetValue { .. }
            | Op::SetGrandMaster { .. }
            | Op::SetBlackout { .. }
            | Op::SetBpm { .. }
            | Op::SetRemoteAccess { .. }
            | Op::SetTestPattern { .. }
            | Op::SetInterfaceKind { .. }
            | Op::SetPreview { .. }
            | Op::SetHighlight { .. }
    )
}

/// Tópicos que mudaram e a resposta da operação (`null` na maioria).
pub type Applied = Result<(Vec<Topic>, Value), String>;

pub fn apply(s: &mut Snapshot, op: Op) -> Applied {
    use Topic as T;
    let panic_all = op == (Op::Panic { all: true });
    let p = &mut s.project;
    let topics = match op {
        Op::NewProject => {
            s.project = empty_project();
            s.project_path = None;
            s.dirty = false;
            s.running.clear();
            s.widget_values.clear();
            s.desk = Desk {
                levels: vec![],
                groups: vec![],
            };
            s.preview = None;
            s.highlight.clear();
            vec![
                T::Project,
                T::ProjectPath,
                T::Dirty,
                T::Running,
                T::WidgetValues,
                T::Desk,
                T::Preview,
                T::Highlight,
            ]
        }
        // Mexem em arquivos e no histórico: ficam no hub
        Op::OpenProject { .. } | Op::SaveProject { .. } | Op::Undo | Op::Redo => {
            return Err("Operação do servidor.".into());
        }
        Op::AddFixtures {
            def_id,
            mode,
            name,
            color,
            count,
            universe,
            address,
        } => {
            let def = s
                .library
                .iter()
                .chain(&p.fixture_defs)
                .find(|d| d.id == def_id)
                .cloned()
                .ok_or(format!("Definição {def_id} não encontrada."))?;
            let size = def
                .modes
                .iter()
                .find(|m| m.name == mode)
                .ok_or(format!("A definição não tem o modo {mode}."))?
                .channels
                .len() as u16;
            check_universe(p, &universe)?;
            if !(1..=512).contains(&count) {
                return Err("Quantidade fora de 1 a 512.".into());
            }
            if let Some(start) = address {
                fits(start, u32::from(count) * u32::from(size))?;
            }
            let mut topics = vec![T::Fixtures];
            if !p.fixture_defs.iter().any(|d| d.id == def.id) {
                p.fixture_defs.push(def.clone());
                topics.push(T::FixtureDefs);
            }
            for i in 0..count {
                // Sem endereço, cada uma vai para o primeiro lugar livre (se não couber, fica no 1 e o
                // Patch mostra o conflito)
                let address = match address {
                    Some(start) => start + i * size,
                    None => next_free_address(p, &universe, size).unwrap_or(1),
                };
                p.fixtures.push(Fixture {
                    id: new_id(),
                    def_id: def.id.clone(),
                    mode: mode.clone(),
                    name: if count > 1 {
                        format!("{name} {}", i + 1)
                    } else {
                        name.clone()
                    },
                    color: color.clone(),
                    universe: universe.clone(),
                    address,
                });
            }
            topics
        }
        Op::UpdateFixture { fixture } => {
            check_universe(p, &fixture.universe)?;
            fits(fixture.address, 1)?;
            upsert(&mut p.fixtures, fixture, |f| &f.id)?;
            vec![T::Fixtures]
        }
        Op::RemoveFixtures { ids } => {
            let gone = |id: &Id| ids.contains(id);
            p.fixtures.retain(|f| !gone(&f.id));
            for g in &mut p.fixture_groups {
                g.cells.retain(|c| !gone(&c.fixture));
            }
            for g in &mut p.channel_groups {
                g.channels.retain(|c| !gone(&c.fixture));
            }
            for f in &mut p.functions {
                match &mut f.kind {
                    FunctionKind::Scene(scene) => scene.values.retain(|v| !gone(&v.fixture)),
                    FunctionKind::Sequence(sequence) => {
                        for step in &mut sequence.steps {
                            step.values.retain(|v| !gone(&v.fixture));
                        }
                    }
                    FunctionKind::Efx(efx) => efx.fixtures.retain(|x| !gone(&x.fixture)),
                    _ => {}
                }
            }
            for w in &mut p.console.widgets {
                match &mut w.kind {
                    VcWidgetKind::Button(b) => b.fixtures.retain(|id| !gone(id)),
                    VcWidgetKind::XyPad(xy) => xy.fixtures.retain(|id| !gone(id)),
                    _ => {}
                }
            }
            p.stage.fixtures.retain(|x| !gone(&x.fixture));
            s.highlight.retain(|id| !gone(id));
            vec![
                T::Fixtures,
                T::FixtureGroups,
                T::ChannelGroups,
                T::Functions,
                T::Console,
                T::Stage,
                T::Highlight,
            ]
        }
        Op::AutoAddress {
            ids,
            universe,
            start,
        } => {
            check_universe(p, &universe)?;
            let mut address = start;
            for id in &ids {
                let Some(i) = p.fixtures.iter().position(|f| &f.id == id) else {
                    continue;
                };
                let size = channel_count(p, &p.fixtures[i]);
                let fixture = &mut p.fixtures[i];
                fixture.universe = universe.clone();
                fixture.address = address;
                address = address.saturating_add(size);
            }
            vec![T::Fixtures]
        }
        Op::SaveFixtureGroup { group } => {
            check_group(p, &group)?;
            upsert(&mut p.fixture_groups, group, |g| &g.id)?;
            vec![T::FixtureGroups]
        }
        Op::RemoveFixtureGroup { id } => {
            p.fixture_groups.retain(|g| g.id != id);
            for w in &mut p.console.widgets {
                if let VcWidgetKind::Button(b) = &mut w.kind {
                    b.groups.retain(|g| *g != id);
                }
            }
            for f in &mut p.functions {
                if let FunctionKind::RgbMatrix(m) = &mut f.kind
                    && m.group.as_ref() == Some(&id)
                {
                    m.group = None;
                }
            }
            vec![T::FixtureGroups, T::Functions, T::Console]
        }
        Op::SaveChannelGroup { group } => {
            upsert(&mut p.channel_groups, group, |g| &g.id)?;
            vec![T::ChannelGroups]
        }
        Op::RemoveChannelGroup { id } => {
            p.channel_groups.retain(|g| g.id != id);
            for w in &mut p.console.widgets {
                if let VcWidgetKind::Slider(slider) = &mut w.kind {
                    slider.channel_groups.retain(|g| *g != id);
                }
            }
            vec![T::ChannelGroups, T::Console]
        }
        Op::SaveFunction { function } => {
            // Como no QLC+, uma criação não contém a si mesma, direto ou por outra
            if contains(&p.functions, &function.kind, &function.id) {
                return Err("Uma criação não pode conter a si mesma.".into());
            }
            if let FunctionKind::Efx(efx) = &function.kind {
                check_efx(p, efx)?;
            }
            upsert(&mut p.functions, function, |f| &f.id)?;
            vec![T::Functions]
        }
        Op::RemoveFunction { id } => {
            p.functions.retain(|f| f.id != id);
            for f in &mut p.functions {
                match &mut f.kind {
                    FunctionKind::Chaser(chaser) => chaser.steps.retain(|step| step.function != id),
                    FunctionKind::Collection(c) => c.functions.retain(|f| *f != id),
                    _ => {}
                }
            }
            let clear = |reference: &mut Option<Id>| {
                if reference.as_ref() == Some(&id) {
                    *reference = None;
                }
            };
            for w in &mut p.console.widgets {
                match &mut w.kind {
                    VcWidgetKind::Button(b) => clear(&mut b.function),
                    VcWidgetKind::Slider(slider) => clear(&mut slider.function),
                    VcWidgetKind::CueList(cues) => clear(&mut cues.chaser),
                    VcWidgetKind::Animation(a) => clear(&mut a.matrix),
                    VcWidgetKind::SpeedDial(dial) => dial.functions.retain(|f| *f != id),
                    _ => {}
                }
            }
            s.running.retain(|r| *r != id);
            if s.preview.as_ref() == Some(&id) {
                s.preview = None;
            }
            vec![T::Functions, T::Console, T::Running, T::Preview]
        }
        Op::SaveFolder { folder } => {
            upsert(&mut p.folders, folder, |d| &d.id)?;
            vec![T::Folders]
        }
        Op::RemoveFolder { id } => {
            p.folders.retain(|d| d.id != id);
            for f in &mut p.functions {
                if f.folder.as_ref() == Some(&id) {
                    f.folder = None;
                }
            }
            vec![T::Folders, T::Functions]
        }
        Op::SetFunctionRunning { id, running } => {
            s.running.retain(|r| *r != id);
            if running {
                s.running.push(id);
            }
            vec![T::Running]
        }
        // O passo pedido vai ao motor (no hub); parado, o chaser passa a rodar
        Op::StepChaser { id, .. } => {
            let chaser = |f: &Function| {
                f.id == id && matches!(f.kind, FunctionKind::Chaser(_) | FunctionKind::Sequence(_))
            };
            if !p.functions.iter().any(chaser) {
                return Err("A cue list só anda chasers e sequências.".into());
            }
            if s.running.contains(&id) {
                return Ok((vec![], Value::Null));
            }
            s.running.push(id);
            vec![T::Running]
        }
        // Parar também baixa os sliders de playback. O pânico corta as cenas na hora (no hub); com
        // `all`, também zera a Mesa simples e desliga o padrão de teste, a prévia e o "Localizar".
        Op::StopAllFunctions | Op::Panic { .. } => {
            s.running.clear();
            let playback: Vec<&Id> = p
                .console
                .widgets
                .iter()
                .filter(|w| {
                    matches!(&w.kind, VcWidgetKind::Slider(slider) if slider.mode == SliderMode::Playback)
                })
                .map(|w| &w.id)
                .collect();
            s.widget_values.retain(|v| !playback.contains(&&v.widget));
            let mut topics = vec![T::Running, T::WidgetValues];
            if panic_all {
                s.desk = Desk {
                    levels: vec![],
                    groups: vec![],
                };
                s.test_pattern = None;
                s.preview = None;
                s.highlight.clear();
                topics.extend([T::Desk, T::TestPattern, T::Preview, T::Highlight]);
            }
            topics
        }
        Op::SetPreview { function } => {
            let scene = |id: &Id| {
                p.functions
                    .iter()
                    .any(|f| &f.id == id && matches!(f.kind, FunctionKind::Scene(_)))
            };
            if function.as_ref().is_some_and(|id| !scene(id)) {
                return Err("A prévia é só de cenas.".into());
            }
            s.preview = function;
            vec![T::Preview]
        }
        Op::SetHighlight { mut fixtures } => {
            fixtures.retain(|id| p.fixtures.iter().any(|f| &f.id == id));
            s.highlight = fixtures;
            vec![T::Highlight]
        }
        Op::SetDeskLevel {
            universe,
            channel,
            value,
        } => {
            check_universe(p, &universe)?;
            fits(channel, 1)?;
            set_desk_level(&mut s.desk, universe, channel, value);
            vec![T::Desk]
        }
        // O grupo grava o próprio nível e o valor nos canais dele, que os faders de canal mostram
        Op::SetDeskGroupLevel { group, value } => {
            let addresses = group_addresses(p, &group);
            s.desk.groups.retain(|g| g.group != group);
            s.desk.groups.push(GroupLevel { group, value });
            for (universe, channel) in addresses {
                set_desk_level(&mut s.desk, universe, channel, value);
            }
            vec![T::Desk]
        }
        // Zerar os grupos também solta os canais deles
        Op::ResetDesk { universe } => {
            match universe {
                Some(universe) => s.desk.levels.retain(|l| l.universe != universe),
                None => {
                    let released: Vec<(Id, u16)> = s
                        .desk
                        .groups
                        .iter()
                        .flat_map(|g| group_addresses(p, &g.group))
                        .collect();
                    s.desk
                        .levels
                        .retain(|l| !released.contains(&(l.universe.clone(), l.channel)));
                    s.desk.groups.clear();
                }
            }
            vec![T::Desk]
        }
        Op::SaveConsolePage { page } => {
            upsert(&mut p.console.pages, page, |page| &page.id)?;
            vec![T::Console]
        }
        Op::RemoveConsolePage { id } => {
            p.console.pages.retain(|page| page.id != id);
            p.console.widgets.retain(|w| w.page != id);
            vec![T::Console]
        }
        Op::SaveWidgets { widgets } => {
            unique(widgets.iter().map(|w| &w.id))?;
            // O elemento "Tempos" que trocou de unidade (segundos ou batidas) volta aos tempos
            // gravados: o valor de uma unidade não serve na outra
            let unit = |w: &VcWidget| match &w.kind {
                VcWidgetKind::SpeedDial(dial) => Some(dial.beats),
                _ => None,
            };
            let changed: Vec<Id> = widgets
                .iter()
                .filter(|w| {
                    let old = p.console.widgets.iter().find(|old| old.id == w.id);
                    old.is_some_and(|old| unit(old) != unit(w))
                })
                .map(|w| w.id.clone())
                .collect();
            for w in widgets {
                upsert(&mut p.console.widgets, w, |w| &w.id)?;
            }
            if changed.is_empty() {
                vec![T::Console]
            } else {
                s.widget_values.retain(|v| !changed.contains(&v.widget));
                vec![T::Console, T::WidgetValues]
            }
        }
        Op::RemoveWidgets { ids } => {
            p.console.widgets.retain(|w| {
                !ids.contains(&w.id) && !w.frame.as_ref().is_some_and(|f| ids.contains(f))
            });
            let widgets = &p.console.widgets;
            s.widget_values
                .retain(|v| widgets.iter().any(|w| w.id == v.widget));
            vec![T::Console, T::WidgetValues]
        }
        Op::SetWidgetValue { widget, value } => {
            s.widget_values.retain(|v| v.widget != widget);
            if !value.is_empty() {
                s.widget_values.push(WidgetValue { widget, value });
            }
            vec![T::WidgetValues]
        }
        Op::SetGrandMaster { value } => {
            s.master.grand_master = value;
            vec![T::Master]
        }
        Op::SetBlackout { on } => {
            s.master.blackout = on;
            vec![T::Master]
        }
        // O BPM vai ao motor no hub, para a batida recomeçar só no tap (e não a cada mudança do master)
        Op::SetBpm { bpm } => {
            if !(30..=300).contains(&bpm) {
                return Err("BPM fora de 30 a 300.".into());
            }
            s.master.bpm = bpm;
            vec![T::Master]
        }
        Op::SaveStage { stage } => {
            unique(stage.elements.iter().map(|e| &e.id))?;
            p.stage = stage;
            vec![T::Stage]
        }
        Op::SaveFixtureDef { def } => {
            // O id vira o nome do arquivo na biblioteca
            if def.id.is_empty()
                || !def
                    .id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err(format!("Id de definição inválido: \"{}\".", def.id));
            }
            let in_project = p.fixture_defs.iter().any(|d| d.id == def.id);
            upsert(&mut s.library, def.clone(), |d| &d.id)?;
            if !in_project {
                return Ok((vec![T::Library], Value::Null));
            }
            upsert(&mut p.fixture_defs, def, |d| &d.id)?;
            vec![T::Library, T::FixtureDefs]
        }
        Op::RemoveFixtureDef { id } => {
            let count = s.library.len();
            s.library.retain(|d| d.id != id);
            if s.library.len() == count {
                return Err(format!("Definição {id} não encontrada."));
            }
            vec![T::Library]
        }
        Op::ImportQxf { file_name, content } => {
            let (def, mut warnings) =
                rlc_qxf::import(&content).map_err(|e| format!("{file_name}: {e}"))?;
            // A mesma fixture importada de novo substitui a anterior, como o "Salvar" do editor
            if s.library
                .iter()
                .chain(&p.fixture_defs)
                .any(|d| d.id == def.id)
            {
                warnings.insert(
                    0,
                    "Substituiu a definição que já existia (na biblioteca ou no projeto).".into(),
                );
            }
            let report = ImportReport {
                def_id: def.id.clone(),
                channels: def.channels.len() as u16,
                modes: def.modes.len() as u16,
                warnings,
            };
            let (topics, _) = apply(s, Op::SaveFixtureDef { def })?;
            return Ok((topics, json!(report)));
        }
        Op::AddUniverse => {
            p.universes.push(Universe {
                id: new_id(),
                name: format!("Universo {}", p.universes.len() + 1),
                output: None,
            });
            vec![T::Universes]
        }
        Op::UpdateUniverse { universe } => {
            if let Some(output) = &universe.output {
                if !s.interfaces.iter().any(|i| &i.id == output) {
                    return Err("Interface não encontrada.".into());
                }
                // Uma interface manda um universo só: escolhida aqui, sai do outro
                for u in p.universes.iter_mut().filter(|u| u.id != universe.id) {
                    if u.output.as_ref() == Some(output) {
                        u.output = None;
                    }
                }
            }
            upsert(&mut p.universes, universe, |u| &u.id)?;
            vec![T::Universes]
        }
        Op::RemoveUniverse { id } => {
            if p.fixtures.iter().any(|f| f.universe == id) {
                return Err("O universo tem fixtures: tire-as antes.".into());
            }
            p.universes.retain(|u| u.id != id);
            s.desk.levels.retain(|l| l.universe != id);
            vec![T::Universes, T::Desk]
        }
        Op::SetRemoteAccess { enabled, pin } => {
            s.remote.enabled = enabled;
            s.remote.pin = pin.filter(|p| !p.is_empty());
            vec![T::Remote]
        }
        Op::SetTestPattern { pattern } => {
            s.test_pattern = pattern;
            vec![T::TestPattern]
        }
        Op::SetInterfaceKind { interface, kind } => {
            let usb = s
                .interfaces
                .iter_mut()
                .find(|i| i.id == interface && i.kind != InterfaceKind::Dummy)
                .ok_or("Interface USB não encontrada.")?;
            if kind == Some(InterfaceKind::Dummy) {
                return Err("Uma interface USB não vira Dummy.".into());
            }
            usb.kind_forced = kind.is_some();
            usb.kind = kind.unwrap_or_else(|| crate::hub::kind_by_name(&usb.name));
            vec![T::Interfaces]
        }
    };
    Ok((topics, Value::Null))
}

/// UUIDv7 (A4), para o que o servidor cria sem receber o id (fixtures, universos).
pub fn new_id() -> Id {
    uuid::Uuid::now_v7().to_string()
}

pub fn empty_project() -> Project {
    Project {
        name: "Novo projeto".into(),
        universes: vec![Universe {
            id: new_id(),
            name: "Universo 1".into(),
            output: Some("dummy".into()),
        }],
        fixture_defs: vec![],
        fixtures: vec![],
        fixture_groups: vec![],
        channel_groups: vec![],
        functions: vec![],
        folders: vec![],
        console: Console {
            pages: vec![VcPage {
                id: new_id(),
                name: "Página 1".into(),
            }],
            widgets: vec![],
        },
        stage: Stage {
            width: 10.0,
            depth: 8.0,
            height: 6.0,
            fixtures: vec![],
            elements: vec![],
        },
    }
}

/// Os universos para o motor, na ordem do projeto: os canais de intensidade (o Grand Master e o
/// blackout só agem neles) e o valor padrão de cada canal. Canal sem fixture conta como intensidade,
/// para o blackout apagar a luz ligada só pela Mesa simples.
pub fn universe_patch(p: &Project) -> Vec<UniversePatch> {
    p.universes
        .iter()
        .map(|u| {
            let mut patch = UniversePatch {
                id: u.id.clone(),
                intensity: Box::new([true; 512]),
                fades: Box::new([true; 512]),
                defaults: Box::new([0; 512]),
            };
            for f in p.fixtures.iter().filter(|f| f.universe == u.id) {
                for (i, channel) in mode_channels(p, f).enumerate() {
                    let index = (usize::from(f.address) + i).checked_sub(1);
                    if let Some(c) = index.filter(|&c| c < 512) {
                        use Attribute::*;
                        patch.intensity[c] = is_intensity(channel.attribute);
                        // Rodas, gobo, prisma, strobo, macros… mudam de uma vez
                        patch.fades[c] = patch.intensity[c]
                            || matches!(
                                channel.attribute,
                                Pan | Tilt | Focus | Zoom | Iris | ColorTemperature
                            );
                        patch.defaults[c] = channel.default;
                    }
                }
            }
            patch
        })
        .collect()
}

/// Criações disparadas pelo usuário, na ordem de início, já resolvidas para o motor: as cenas e o pan
/// e o tilt dos EFX nos endereços DMX, e os chasers e as coleções com o que disparam e os tempos que
/// valem agora. O RGB Matrix só acende na F9.
pub fn running_programs(s: &Snapshot) -> Vec<Run> {
    s.running
        .iter()
        .filter_map(|id| {
            let program = program(s, id, [None; 3], &mut vec![])?;
            Some(Run {
                id: id.clone(),
                program,
            })
        })
        .collect()
}

/// O que uma criação acende. `dial` traz os tempos (fade in, fade out e duração) de um elemento
/// "Tempos" que controla uma coleção de fora; o que controla a própria criação vale antes. `path` são
/// as criações de fora: uma que contenha a si mesma não entra.
fn program(s: &Snapshot, id: &Id, dial: [Option<Span>; 3], path: &mut Vec<Id>) -> Option<Program> {
    let p = &s.project;
    let function = p
        .functions
        .iter()
        .find(|f| f.id == *id && !path.contains(id))?;
    let dial: [Option<Span>; 3] = std::array::from_fn(|i| live_time(s, id, i).or(dial[i]));
    path.push(id.clone());
    let program = match &function.kind {
        FunctionKind::Scene(scene) => Some(scene_program(
            p,
            &scene.values,
            dial[0].unwrap_or(Span::Ms(scene.fade_in_ms)),
            dial[1].unwrap_or(Span::Ms(scene.fade_out_ms)),
        )),
        FunctionKind::Collection(c) => Some(Program::Collection(
            c.functions
                .iter()
                .filter_map(|f| program(s, f, dial, path))
                .collect(),
        )),
        FunctionKind::Chaser(c) | FunctionKind::Sequence(c) => {
            let unit = |v: u32| if c.beats { Span::Beats(v) } else { Span::Ms(v) };
            let time = |mode: SpeedMode, common: u32, step: u32| match mode {
                SpeedMode::Default => None,
                SpeedMode::Common => Some(unit(common)),
                SpeedMode::PerStep => Some(unit(step)),
            };
            // Na sequência, os passos acendem a cena dela, e o "Padrão" são os fades da cena
            let scene = c.scene.as_ref().and_then(|scene| {
                p.functions.iter().find_map(|f| match &f.kind {
                    FunctionKind::Scene(sc) if f.id == *scene => {
                        let live = |i| live_time(s, &f.id, i);
                        Some((
                            live(0).unwrap_or(Span::Ms(sc.fade_in_ms)),
                            live(1).unwrap_or(Span::Ms(sc.fade_out_ms)),
                        ))
                    }
                    _ => None,
                })
            });
            let sequence = matches!(function.kind, FunctionKind::Sequence(_));
            let steps = c
                .steps
                .iter()
                .map(|step| Step {
                    program: if sequence {
                        let (fade_in, fade_out) = scene.unwrap_or((Span::Ms(0), Span::Ms(0)));
                        scene_program(p, &step.values, fade_in, fade_out)
                    } else {
                        // O passo com um EFX (ou um RGB Matrix) conta o tempo sem acender nada
                        program(s, &step.function, [None; 3], path)
                            .unwrap_or(Program::Collection(vec![]))
                    },
                    fade_in: dial[0].or(time(
                        c.fade_in_mode,
                        c.common.fade_in_ms,
                        step.timing.fade_in_ms,
                    )),
                    // No hold, "Padrão" vale como "Comum". A duração do elemento em batidas vale em
                    // todos os chasers; a em segundos, só nos que não estão em batidas (neles, manda
                    // o BPM)
                    hold: match dial[2] {
                        Some(span @ Span::Beats(_)) => span,
                        Some(span) if !c.beats => span,
                        _ if c.hold_mode == SpeedMode::PerStep => unit(step.timing.hold_ms),
                        _ => unit(c.common.hold_ms),
                    },
                    fade_out: dial[1].or(time(
                        c.fade_out_mode,
                        c.common.fade_out_ms,
                        step.timing.fade_out_ms,
                    )),
                })
                .collect();
            Some(Program::Chaser(ChaserProgram {
                steps,
                run_order: c.run_order,
                direction: c.direction,
                keep_last: c.keep_last,
            }))
        }
        // Só as fixtures com pan ou tilt andam no desenho; a duração do elemento "Tempos" é a da volta
        FunctionKind::Efx(efx) => {
            let mut efx = efx.clone();
            let mut heads = vec![];
            efx.fixtures.retain(|x| {
                let head = pan_tilt(p, &x.fixture);
                let moves = head.iter().any(Option::is_some);
                if moves {
                    heads.push(head);
                }
                moves
            });
            Some(Program::Efx(EfxProgram {
                cycle: dial[2].unwrap_or(Span::Ms(efx.timing.hold_ms)),
                fade_in: dial[0].unwrap_or(Span::Ms(efx.timing.fade_in_ms)),
                fade_out: dial[1].unwrap_or(Span::Ms(efx.timing.fade_out_ms)),
                heads,
                efx,
            }))
        }
        FunctionKind::RgbMatrix(_) => None,
    };
    path.pop();
    program
}

/// Tempo ajustado ao vivo (VC-3) de uma criação: fade in, fade out ou duração (0, 1 ou 2), em
/// segundos ou, no elemento que segue o BPM, em batidas. Vale o último elemento "Tempos" mexido que
/// tem a criação na lista e controla esse tempo.
fn live_time(s: &Snapshot, id: &Id, index: usize) -> Option<Span> {
    s.widget_values.iter().rev().find_map(|v| {
        let widget = s
            .project
            .console
            .widgets
            .iter()
            .find(|w| w.id == v.widget)?;
        match &widget.kind {
            VcWidgetKind::SpeedDial(dial)
                if dial.functions.contains(id)
                    && [dial.fade_in, dial.fade_out, dial.hold][index] =>
            {
                let unit = if dial.beats { Span::Beats } else { Span::Ms };
                v.value.get(index).map(|&v| unit(v))
            }
            _ => None,
        }
    })
}

/// Valores de uma cena (ou de um passo de sequência) nos endereços DMX. O canal fino vai junto do
/// grosso, para o fade em 16 bits; sem o fino na cena, ele vale 0.
fn scene_program(p: &Project, values: &[SceneValue], fade_in: Span, fade_out: Span) -> Program {
    let mut levels = Vec::new();
    for fixture in &p.fixtures {
        // Valores da cena nesta fixture, pela posição do canal no modo
        let values: Vec<(usize, u8)> = values
            .iter()
            .filter(|v| v.fixture == fixture.id)
            .map(|v| (usize::from(v.channel), v.value))
            .collect();
        let value = |pos: usize| values.iter().find(|(i, _)| *i == pos).map(|(_, v)| *v);
        let channels: Vec<&Channel> = mode_channels(p, fixture).collect();
        for &(pos, v) in &values {
            let Some(channel) = channels.get(pos) else {
                continue;
            };
            // O k-ésimo fino de um atributo é do k-ésimo canal grosso dele ("Pan, Tilt, Pan fino,
            // Tilt fino"); a conta acha o par dos dois lados
            let same = channels
                .iter()
                .enumerate()
                .filter(|(_, c)| c.attribute == channel.attribute);
            let k = same
                .clone()
                .filter(|(i, c)| *i < pos && c.fine == channel.fine)
                .count();
            let pair = same.filter(|(_, c)| c.fine != channel.fine).nth(k);
            let pair = pair.map(|(i, _)| i);
            // Com os dois na cena, o fino vai junto do grosso
            if channel.fine && pair.and_then(value).is_some() {
                continue;
            }
            let Some(address) = channel_address(p, &fixture.id, pos as u16) else {
                continue;
            };
            let fine = pair
                .filter(|_| !channel.fine)
                .and_then(|i| channel_address(p, &fixture.id, i as u16));
            levels.push(SceneLevel {
                universe: address.universe,
                channel: address.channel,
                value: match fine {
                    Some(_) => u16::from(v) << 8 | u16::from(pair.and_then(value).unwrap_or(0)),
                    None => u16::from(v),
                },
                fine: fine.map(|l| l.channel),
            });
        }
    }
    Program::Scene(SceneProgram {
        levels,
        fade_in,
        fade_out,
    })
}

/// Pan e tilt de uma fixture nos endereços DMX: o primeiro canal grosso de cada um, com o fino dele.
fn pan_tilt(p: &Project, fixture: &Id) -> [Option<SceneLevel>; 2] {
    let Some(f) = p.fixtures.iter().find(|f| f.id == *fixture) else {
        return [None, None];
    };
    let channels: Vec<&Channel> = mode_channels(p, f).collect();
    let address = |attribute, fine| {
        let i = channels
            .iter()
            .position(|c| c.attribute == attribute && c.fine == fine)?;
        channel_address(p, fixture, i as u16)
    };
    [Attribute::Pan, Attribute::Tilt].map(|attribute| {
        let coarse = address(attribute, false)?;
        Some(SceneLevel {
            universe: coarse.universe,
            channel: coarse.channel,
            fine: address(attribute, true).map(|l| l.channel),
            value: 0,
        })
    })
}

/// Os limites do QLC+ para o EFX e as ordens que um desenho tem; só fixtures com pan ou tilt, sem
/// repetir.
fn check_efx(p: &Project, efx: &Efx) -> Result<(), String> {
    if efx.width > 127 || efx.height > 127 {
        return Err("A largura e a altura do EFX vão até 127.".into());
    }
    if efx.x_frequency > 32 || efx.y_frequency > 32 {
        return Err("As frequências do EFX vão até 32.".into());
    }
    let angles = [efx.rotation, efx.x_phase, efx.y_phase];
    let starts = efx.fixtures.iter().map(|x| x.start_offset);
    if angles.into_iter().chain(starts).any(|a| a > 359) {
        return Err("Os ângulos do EFX vão até 359°.".into());
    }
    if efx.run_order == RunOrder::Random {
        return Err("O EFX não tem a ordem aleatória.".into());
    }
    for (i, x) in efx.fixtures.iter().enumerate() {
        if efx.fixtures[..i].iter().any(|o| o.fixture == x.fixture) {
            return Err("Fixture repetida no EFX.".into());
        }
        if pan_tilt(p, &x.fixture) == [None, None] {
            return Err("Só fixtures com pan ou tilt entram no EFX.".into());
        }
    }
    Ok(())
}

/// Grade de 1 a 64 por lado, cada célula dentro dela com uma cabeça que existe, e cada cabeça uma vez
/// só, como no QLC+.
fn check_group(p: &Project, group: &FixtureGroup) -> Result<(), String> {
    if !(1..=64).contains(&group.width) || !(1..=64).contains(&group.height) {
        return Err("A grade do grupo vai de 1 a 64 células por lado.".into());
    }
    for (i, c) in group.cells.iter().enumerate() {
        let before = &group.cells[..i];
        if c.x >= group.width || c.y >= group.height {
            return Err("Célula fora da grade do grupo.".into());
        }
        if before.iter().any(|o| (o.x, o.y) == (c.x, c.y)) {
            return Err("Duas cabeças na mesma célula do grupo.".into());
        }
        if before
            .iter()
            .any(|o| o.fixture == c.fixture && o.head == c.head)
        {
            return Err("Cabeça repetida no grupo.".into());
        }
        // Fixture sem cabeças no modo conta como uma cabeça só
        let heads = p.fixtures.iter().find(|f| f.id == c.fixture).and_then(|f| {
            let def = p.fixture_defs.iter().find(|d| d.id == f.def_id)?;
            let mode = def.modes.iter().find(|m| m.name == f.mode)?;
            Some(mode.heads.len().max(1))
        });
        if heads.is_none_or(|n| usize::from(c.head) >= n) {
            return Err("O grupo tem uma cabeça que não existe.".into());
        }
    }
    Ok(())
}

/// Criações que uma criação dispara direto: os passos do chaser e as da coleção.
fn members(kind: &FunctionKind) -> Vec<&Id> {
    match kind {
        FunctionKind::Chaser(c) => c.steps.iter().map(|step| &step.function).collect(),
        FunctionKind::Collection(c) => c.functions.iter().collect(),
        _ => vec![],
    }
}

/// A criação dispara `target`, direto ou por outra.
fn contains<'a>(functions: &'a [Function], kind: &'a FunctionKind, target: &Id) -> bool {
    let mut stack = members(kind);
    let mut seen = vec![];
    while let Some(id) = stack.pop() {
        if id == target {
            return true;
        }
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        if let Some(f) = functions.iter().find(|f| f.id == *id) {
            stack.extend(members(&f.kind));
        }
    }
    false
}

fn is_intensity(attribute: Attribute) -> bool {
    use Attribute::*;
    matches!(
        attribute,
        Dimmer | Red | Green | Blue | White | Amber | Uv | Lime | Indigo | Cyan | Magenta | Yellow
    )
}

fn mode_channels<'a>(p: &'a Project, f: &Fixture) -> impl Iterator<Item = &'a Channel> {
    let def = p.fixture_defs.iter().find(|d| d.id == f.def_id);
    let mode = def.and_then(|d| d.modes.iter().find(|m| m.name == f.mode));
    mode.into_iter()
        .flat_map(|m| &m.channels)
        .filter_map(move |&i| def?.channels.get(usize::from(i)))
}

fn channel_count(p: &Project, f: &Fixture) -> u16 {
    mode_channels(p, f).count() as u16
}

/// Primeiro endereço com `count` canais livres seguidos no universo.
fn next_free_address(p: &Project, universe: &str, count: u16) -> Option<u16> {
    let mut used = [false; 513];
    for f in p.fixtures.iter().filter(|f| f.universe == universe) {
        for c in f.address..(f.address + channel_count(p, f)).min(513) {
            used[usize::from(c)] = true;
        }
    }
    (1..=513u16.saturating_sub(count))
        .find(|&start| (start..start + count).all(|c| !used[usize::from(c)]))
}

/// Endereço (universo, canal DMX) de um canal de uma fixture, pela posição no modo.
fn channel_address(p: &Project, fixture: &str, channel: u16) -> Option<DeskLevel> {
    let f = p.fixtures.iter().find(|f| f.id == fixture)?;
    let address = f.address + channel;
    (address <= 512).then(|| DeskLevel {
        universe: f.universe.clone(),
        channel: address,
        value: 0,
    })
}

/// Endereços (universo, canal DMX) dos canais de um grupo de canais.
fn group_addresses(p: &Project, group: &str) -> Vec<(Id, u16)> {
    let Some(group) = p.channel_groups.iter().find(|g| g.id == group) else {
        return vec![];
    };
    group
        .channels
        .iter()
        .filter_map(|c| channel_address(p, &c.fixture, c.channel))
        .map(|l| (l.universe, l.channel))
        .collect()
}

/// Valores da cena em prévia, nos endereços DMX (o motor não conhece fixtures).
pub fn preview_levels(s: &Snapshot) -> Vec<DeskLevel> {
    let p = &s.project;
    let scene = p
        .functions
        .iter()
        .find(|f| s.preview.as_ref() == Some(&f.id));
    let Some(FunctionKind::Scene(scene)) = scene.map(|f| &f.kind) else {
        return vec![];
    };
    scene
        .values
        .iter()
        .filter_map(|v| {
            let level = channel_address(p, &v.fixture, v.channel)?;
            Some(DeskLevel {
                value: v.value,
                ..level
            })
        })
        .collect()
}

/// Valores do "Localizar": dimmer e RGBW a 100% e o shutter no padrão (o strobo em "luz ativa");
/// o resto (pan, tilt, gobo…) fica como está.
pub fn highlight_levels(s: &Snapshot) -> Vec<DeskLevel> {
    use Attribute::*;
    let p = &s.project;
    let mut levels = Vec::new();
    for f in p.fixtures.iter().filter(|f| s.highlight.contains(&f.id)) {
        for (i, channel) in mode_channels(p, f).enumerate() {
            let value = match channel.attribute {
                _ if channel.fine => continue,
                Dimmer | Red | Green | Blue | White => 255,
                Shutter => channel.default,
                _ => continue,
            };
            if let Some(level) = channel_address(p, &f.id, i as u16) {
                levels.push(DeskLevel { value, ..level });
            }
        }
    }
    levels
}

fn set_desk_level(desk: &mut Desk, universe: Id, channel: u16, value: u8) {
    desk.levels
        .retain(|l| l.universe != universe || l.channel != channel);
    desk.levels.push(DeskLevel {
        universe,
        channel,
        value,
    });
}

fn check_universe(p: &Project, universe: &str) -> Result<(), String> {
    match p.universes.iter().any(|u| u.id == universe) {
        true => Ok(()),
        false => Err("Universo não encontrado.".into()),
    }
}

/// Endereço inicial e quantidade de canais dentro do universo (1 a 512).
fn fits(start: u16, count: u32) -> Result<(), String> {
    match start >= 1 && u32::from(start) + count <= 513 {
        true => Ok(()),
        false => Err(format!(
            "Não cabe no universo a partir do endereço {start}."
        )),
    }
}

/// Substitui o item de mesmo id ou o acrescenta ao fim.
fn upsert<T>(list: &mut Vec<T>, item: T, id: impl Fn(&T) -> &Id) -> Result<(), String> {
    if id(&item).is_empty() {
        return Err("Item sem id.".into());
    }
    match list.iter().position(|x| id(x) == id(&item)) {
        Some(i) => list[i] = item,
        None => list.push(item),
    }
    Ok(())
}

/// Recusa uma lista com ids repetidos (A4).
fn unique<'a>(ids: impl Iterator<Item = &'a Id>) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    match ids.into_iter().all(|id| seen.insert(id)) {
        true => Ok(()),
        false => Err("Id repetido na mesma operação.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Snapshot {
        crate::hub::example_snapshot(None)
    }

    #[test]
    fn desk_group_writes_its_channels_and_reset_releases_them() {
        let mut s = example();
        // "Dimmer PARs": canal 0 dos seis PARs (endereços 1, 8, 15…)
        let (topics, _) = apply(
            &mut s,
            Op::SetDeskGroupLevel {
                group: "cg1".into(),
                value: 200,
            },
        )
        .unwrap();
        assert_eq!(topics, vec![Topic::Desk]);
        let channels: Vec<u16> = s.desk.levels.iter().map(|l| l.channel).collect();
        assert_eq!(channels, vec![1, 8, 15, 22, 29, 36]);
        apply(&mut s, Op::ResetDesk { universe: None }).unwrap();
        assert!(s.desk.levels.is_empty() && s.desk.groups.is_empty());
    }

    #[test]
    fn removing_a_fixture_cleans_references() {
        let mut s = example();
        let (topics, _) = apply(
            &mut s,
            Op::RemoveFixtures {
                ids: vec!["mh1".into()],
            },
        )
        .unwrap();
        assert!(topics.contains(&Topic::Console) && topics.contains(&Topic::Stage));
        let p = &s.project;
        assert!(!p.fixtures.iter().any(|f| f.id == "mh1"));
        assert!(!p.stage.fixtures.iter().any(|x| x.fixture == "mh1"));
        assert!(
            p.channel_groups
                .iter()
                .all(|g| g.channels.iter().all(|c| c.fixture != "mh1"))
        );
    }

    #[test]
    fn add_fixtures_creates_ids_on_the_server_and_takes_free_addresses() {
        let mut s = example();
        let before = s.project.fixtures.len();
        apply(
            &mut s,
            Op::AddFixtures {
                def_id: "generico-par-rgb".into(),
                mode: "3 canais".into(),
                name: "Novo".into(),
                color: "#ff0000".into(),
                count: 2,
                universe: "u1".into(),
                address: None,
            },
        )
        .unwrap();
        let added = &s.project.fixtures[before..];
        assert_eq!(added.len(), 2);
        assert_ne!(added[0].id, added[1].id);
        assert_eq!(added[0].id.len(), 36);
        assert_eq!(
            (added[0].name.as_str(), added[1].name.as_str()),
            ("Novo 1", "Novo 2")
        );
        // Os PARs ocupam 1 a 43; o primeiro lugar livre com 3 canais é o 44
        assert_eq!((added[0].address, added[1].address), (44, 47));
    }

    #[test]
    fn invalid_operations_are_refused_without_changes() {
        let mut s = example();
        let before = serde_json::to_value(&s).unwrap();
        let refused = [
            Op::SetDeskLevel {
                universe: "nao-existe".into(),
                channel: 1,
                value: 1,
            },
            Op::SetDeskLevel {
                universe: "u1".into(),
                channel: 513,
                value: 1,
            },
            Op::RemoveUniverse { id: "u1".into() },
            Op::SaveProject { name: None },
            Op::SaveWidgets {
                widgets: vec![
                    s.project.console.widgets[0].clone(),
                    s.project.console.widgets[0].clone(),
                ],
            },
        ];
        for op in refused {
            assert!(apply(&mut s, op).is_err());
        }
        assert_eq!(serde_json::to_value(&s).unwrap(), before);
    }

    #[test]
    fn choosing_an_interface_moves_it_from_the_other_universe() {
        let mut s = example();
        let mut fundo = s.project.universes[1].clone();
        fundo.output = Some("dummy".into());
        apply(&mut s, Op::UpdateUniverse { universe: fundo }).unwrap();
        let outputs: Vec<_> = s
            .project
            .universes
            .iter()
            .map(|u| u.output.clone())
            .collect();
        assert_eq!(outputs[..2], [None, Some("dummy".into())]);
    }

    #[test]
    fn interface_kind_is_forced_and_back_to_automatic() {
        let mut s = example();
        let mut usb = s.interfaces[0].clone();
        (usb.id, usb.kind, usb.name) = (
            "ftdi-A1".into(),
            InterfaceKind::OpenDmx,
            "INTERFACE DMX - NUMBSTAR".into(),
        );
        s.interfaces.push(usb);
        let set = |kind| Op::SetInterfaceKind {
            interface: "ftdi-A1".into(),
            kind,
        };
        apply(&mut s, set(Some(InterfaceKind::EnttecPro))).unwrap();
        assert_eq!(
            (s.interfaces[1].kind, s.interfaces[1].kind_forced),
            (InterfaceKind::EnttecPro, true)
        );
        apply(&mut s, set(None)).unwrap();
        assert_eq!(
            (s.interfaces[1].kind, s.interfaces[1].kind_forced),
            (InterfaceKind::OpenDmx, false)
        );
        assert!(apply(&mut s, set(Some(InterfaceKind::Dummy))).is_err());
        let dummy = Op::SetInterfaceKind {
            interface: "dummy".into(),
            kind: Some(InterfaceKind::OpenDmx),
        };
        assert!(apply(&mut s, dummy).is_err());
    }

    #[test]
    fn universe_patch_marks_intensity_and_takes_the_defaults() {
        let s = example();
        let patch = universe_patch(&s.project);
        let palco = &patch[0];
        assert_eq!(palco.id, "u1");
        // Moving head no 101: o primeiro canal dele é pan
        let mh = s.project.fixtures.iter().find(|f| f.id == "mh1").unwrap();
        let first = mode_channels(&s.project, mh).next().unwrap();
        assert_eq!(palco.intensity[100], is_intensity(first.attribute));
        assert!(!palco.intensity.iter().all(|i| *i));
        // Canal sem fixture conta como intensidade
        assert!(palco.intensity[511]);
        // Rig RSena: o Beam 7R no endereço 1 tem o Strobo (canal 2) em 255, "luz ativa"
        let rig = &patch[2];
        assert_eq!((rig.defaults[1], rig.intensity[1]), (255, false));
    }

    #[test]
    fn import_replaces_the_same_fixture_and_reports_it() {
        let mut s = example();
        let import = |name: &str, content: &str| Op::ImportQxf {
            file_name: name.into(),
            content: content.into(),
        };
        let strobe = include_str!("../../rlc-qxf/tests/qxf/RSena-Strobo-1000w.qxf");
        let (topics, report) = apply(&mut s, import("Strobo.qxf", strobe)).unwrap();
        assert_eq!(topics, vec![Topic::Library]);
        assert_eq!(report["defId"], "rsena-strobo-1000w");
        assert_eq!(report["warnings"], json!([]));
        // O projeto já usa o Beam: a cópia dele é substituída, com aviso
        let count = s.library.len();
        s.project.fixture_defs[7].model = "Velho".into();
        let beam = include_str!("../../rlc-qxf/tests/qxf/RSena-Beam-7r.qxf");
        let (topics, report) = apply(&mut s, import("Beam.qxf", beam)).unwrap();
        assert_eq!(s.library.len(), count + 1);
        assert!(
            report["warnings"][0]
                .as_str()
                .unwrap()
                .starts_with("Substituiu")
        );
        assert_eq!(topics, vec![Topic::Library, Topic::FixtureDefs]);
        assert_eq!(s.project.fixture_defs[7].model, "Beam 7r");
        let refused = Op::ImportQxf {
            file_name: "show.qxw".into(),
            content: "<Workspace/>".into(),
        };
        assert!(
            apply(&mut s, refused)
                .unwrap_err()
                .starts_with("show.qxw: ")
        );
    }

    #[test]
    fn preview_and_highlight_go_to_dmx_addresses() {
        let mut s = example();
        // Uma cena do exemplo, com valores nos PARs
        let scene = s
            .project
            .functions
            .iter()
            .find(|f| matches!(&f.kind, FunctionKind::Scene(sc) if !sc.values.is_empty()))
            .unwrap()
            .clone();
        apply(
            &mut s,
            Op::SetPreview {
                function: Some(scene.id.clone()),
            },
        )
        .unwrap();
        let FunctionKind::Scene(values) = &scene.kind else {
            unreachable!()
        };
        assert_eq!(preview_levels(&s).len(), values.values.len());
        // Uma criação que não é cena não entra na prévia
        let chaser = s
            .project
            .functions
            .iter()
            .find(|f| matches!(f.kind, FunctionKind::Chaser(_)))
            .unwrap()
            .id
            .clone();
        assert!(
            apply(
                &mut s,
                Op::SetPreview {
                    function: Some(chaser)
                }
            )
            .is_err()
        );
        // Localizar o Beam no endereço 1 do Rig RSena: Strobo (2) no padrão e Dimmer (3) a 100%; pan parado
        let beam = s
            .project
            .fixtures
            .iter()
            .find(|f| f.universe == "u3" && f.address == 1)
            .unwrap()
            .id
            .clone();
        apply(
            &mut s,
            Op::SetHighlight {
                fixtures: vec![beam, "nao-existe".into()],
            },
        )
        .unwrap();
        assert_eq!(s.highlight.len(), 1);
        let channels: Vec<_> = highlight_levels(&s)
            .iter()
            .map(|l| (l.channel, l.value))
            .collect();
        assert_eq!(channels, [(2, 255), (3, 255)]);
        // O segundo toque do pânico desliga os dois
        apply(&mut s, Op::Panic { all: true }).unwrap();
        assert!(s.preview.is_none() && s.highlight.is_empty());
    }

    fn run(s: &mut Snapshot, id: &str) {
        let op = Op::SetFunctionRunning {
            id: id.into(),
            running: true,
        };
        apply(s, op).unwrap();
    }

    /// A cena de uma criação rodando.
    fn scene_of(program: &Program) -> &SceneProgram {
        match program {
            Program::Scene(scene) => scene,
            _ => panic!("não é cena"),
        }
    }

    fn chaser_of(program: &Program) -> &ChaserProgram {
        match program {
            Program::Chaser(chaser) => chaser,
            _ => panic!("não é chaser"),
        }
    }

    fn chaser_mut<'a>(s: &'a mut Snapshot, id: &str) -> &'a mut Chaser {
        let f = s.project.functions.iter_mut().find(|f| f.id == id);
        match f.map(|f| &mut f.kind) {
            Some(FunctionKind::Chaser(c) | FunctionKind::Sequence(c)) => c,
            _ => panic!("{id} não é chaser"),
        }
    }

    #[test]
    fn running_scenes_go_to_dmx_addresses_in_start_order() {
        let mut s = example();
        for id in ["s4", "s1"] {
            run(&mut s, id);
        }
        let runs = running_programs(&s);
        let ids: Vec<&str> = runs.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["s4", "s1"]);
        assert_eq!(scene_of(&runs[0].program).fade_in, Span::Ms(1000));
        // MH 1 no 101: pan e tilt levam o fino junto, em 16 bits (sem o fino na cena, ele vale 0)
        let at = |runs: &[Run], channel: u16| {
            let levels = &scene_of(&runs[0].program).levels;
            let level = levels.iter().find(|l| l.channel == channel);
            level.map(|l| (l.fine, l.value))
        };
        assert_eq!(at(&runs, 101), Some((Some(102), 128 << 8)));
        assert_eq!(at(&runs, 103), Some((Some(104), 128 << 8)));
        assert_eq!(at(&runs, 106), Some((None, 255)));

        // Com o fino na cena, ele vai junto do grosso; só o fino (MH 2 no 115) muda só o fino
        let Some(FunctionKind::Scene(scene)) = s
            .project
            .functions
            .iter_mut()
            .find(|f| f.id == "s4")
            .map(|f| &mut f.kind)
        else {
            panic!("s4 não é cena");
        };
        let value = |fixture: &str, channel, value| SceneValue {
            fixture: fixture.into(),
            channel,
            value,
        };
        scene.values.push(value("mh1", 1, 7));
        scene
            .values
            .retain(|v| v.fixture != "mh2" || v.channel != 2);
        scene.values.push(value("mh2", 3, 9));
        let runs = running_programs(&s);
        assert_eq!(at(&runs, 101), Some((Some(102), 128 << 8 | 7)));
        assert_eq!(at(&runs, 102), None);
        assert_eq!((at(&runs, 117), at(&runs, 118)), (None, Some((None, 9))));

        // Fixture removida sai da cena rodando
        apply(
            &mut s,
            Op::RemoveFixtures {
                ids: vec!["mh1".into()],
            },
        )
        .unwrap();
        let runs = running_programs(&s);
        assert!(
            scene_of(&runs[0].program)
                .levels
                .iter()
                .all(|l| l.channel > 114)
        );
    }

    #[test]
    fn speed_dial_sets_the_fades_of_its_scenes_live() {
        let mut s = example();
        for id in ["s1", "s4"] {
            run(&mut s, id);
        }
        let fades = |s: &Snapshot| {
            let times = |r: &Run| {
                let scene = scene_of(&r.program);
                (scene.fade_in, scene.fade_out)
            };
            running_programs(s).iter().map(times).collect::<Vec<_>>()
        };
        let dial = |value: Vec<u32>| Op::SetWidgetValue {
            widget: "sd1".into(),
            value,
        };
        let ms = Span::Ms;
        // Sem valor, valem os tempos gravados (1 s)
        assert_eq!(fades(&s), [(ms(1000), ms(1000)), (ms(1000), ms(1000))]);
        // "Fades da frente" segue o BPM e controla a s1: os fades dela em batidas; a s4 fica com os
        // gravados
        apply(&mut s, dial(vec![2000, 500, 0])).unwrap();
        let beats = (Span::Beats(2000), Span::Beats(500));
        assert_eq!(fades(&s), [beats, (ms(1000), ms(1000))]);
        // Em segundos: trocar a unidade solta o valor, que não serve na outra
        let mut widget = s
            .project
            .console
            .widgets
            .iter()
            .find(|w| w.id == "sd1")
            .cloned()
            .unwrap();
        let VcWidgetKind::SpeedDial(d) = &mut widget.kind else {
            panic!("sd1 não é Tempos")
        };
        d.beats = false;
        let (topics, _) = apply(
            &mut s,
            Op::SaveWidgets {
                widgets: vec![widget.clone()],
            },
        )
        .unwrap();
        assert_eq!(topics, [Topic::Console, Topic::WidgetValues]);
        assert_eq!(fades(&s)[0], (ms(1000), ms(1000)));
        apply(&mut s, dial(vec![3000, 500, 0])).unwrap();
        assert_eq!(fades(&s)[0], (ms(3000), ms(500)));
        // Gravar o elemento sem trocar a unidade mantém o valor
        apply(
            &mut s,
            Op::SaveWidgets {
                widgets: vec![widget],
            },
        )
        .unwrap();
        assert_eq!(fades(&s)[0], (ms(3000), ms(500)));
        // Com o fade out desligado no elemento, o fade out volta ao gravado
        let widget = s.project.console.widgets.iter_mut().find(|w| w.id == "sd1");
        if let Some(VcWidgetKind::SpeedDial(d)) = widget.map(|w| &mut w.kind) {
            d.fade_out = false;
        }
        assert_eq!(fades(&s)[0], (ms(3000), ms(1000)));
        // "Gravados" solta o valor
        apply(&mut s, dial(vec![])).unwrap();
        assert_eq!(fades(&s)[0], (ms(1000), ms(1000)));
    }

    #[test]
    fn chasers_go_with_their_steps_and_the_times_that_hold() {
        let mut s = example();
        run(&mut s, "c1");
        let times = |s: &Snapshot| {
            let runs = running_programs(s);
            let step = &chaser_of(&runs[0].program).steps[0];
            (step.fade_in, step.hold, step.fade_out)
        };
        // "Troca de cores": fade in e hold por passo (1 s e 2 s), fade out da cena
        let c1 = &running_programs(&s)[0].program;
        assert_eq!(chaser_of(c1).steps.len(), 3);
        assert_eq!(
            scene_of(&chaser_of(c1).steps[0].program).fade_out,
            Span::Ms(1000)
        );
        assert_eq!(times(&s), (Some(Span::Ms(1000)), Span::Ms(2000), None));
        // Comum e, no hold, "Padrão" valendo como "Comum"
        let c = chaser_mut(&mut s, "c1");
        (c.fade_in_mode, c.hold_mode) = (SpeedMode::Common, SpeedMode::Default);
        c.common.hold_ms = 3000;
        assert_eq!(times(&s), (Some(Span::Ms(500)), Span::Ms(3000), None));
        // Em batidas
        chaser_mut(&mut s, "c1").beats = true;
        assert_eq!(times(&s), (Some(Span::Beats(500)), Span::Beats(3000), None));

        // O elemento "Tempos" vale para todos os passos. Em segundos, a duração dele não vale num
        // chaser em batidas
        let set_dial = |s: &mut Snapshot, beats: bool| {
            let dial = s.project.console.widgets.iter_mut().find(|w| w.id == "sd1");
            if let Some(VcWidgetKind::SpeedDial(d)) = dial.map(|w| &mut w.kind) {
                (d.functions, d.hold, d.beats) = (vec!["c1".into()], true, beats);
            }
            let value = Op::SetWidgetValue {
                widget: "sd1".into(),
                value: vec![300, 200, 900],
            };
            apply(s, value).unwrap();
        };
        set_dial(&mut s, false);
        let fades = (Some(Span::Ms(300)), Span::Beats(3000), Some(Span::Ms(200)));
        assert_eq!(times(&s), fades);
        chaser_mut(&mut s, "c1").beats = false;
        assert_eq!(times(&s).1, Span::Ms(900));
        // Seguindo o BPM, tudo em batidas, também num chaser em segundos
        set_dial(&mut s, true);
        let beats = (
            Some(Span::Beats(300)),
            Span::Beats(900),
            Some(Span::Beats(200)),
        );
        assert_eq!(times(&s), beats);
    }

    #[test]
    fn sequences_and_collections_light_what_they_hold() {
        let mut s = example();
        for id in ["q1", "k1"] {
            run(&mut s, id);
        }
        let runs = running_programs(&s);
        // "Subida da frente": cada passo acende os valores dele na cena da sequência
        let q1 = chaser_of(&runs[0].program);
        let first = scene_of(&q1.steps[0].program);
        let Some(FunctionKind::Sequence(sequence)) = s
            .project
            .functions
            .iter()
            .find(|f| f.id == "q1")
            .map(|f| &f.kind)
        else {
            panic!("q1 não é sequência")
        };
        assert_eq!(first.levels.len(), sequence.steps[0].values.len());
        assert_eq!(first.fade_in, Span::Ms(1000));
        assert_eq!(q1.steps[0].fade_in, Some(Span::Ms(800)));
        // "Abertura": o chaser, as duas cenas e o EFX
        let Program::Collection(k1) = &runs[1].program else {
            panic!("k1 não é coleção")
        };
        assert_eq!(k1.len(), 4);
        assert!(matches!(k1[0], Program::Chaser(_)) && matches!(k1[2], Program::Efx(_)));
    }

    #[test]
    fn a_creation_cannot_contain_itself() {
        let mut s = example();
        // "Abertura" (k1) já tem o "Troca de cores" (c1): c1 não pode ter um passo com k1
        let mut c1 = s
            .project
            .functions
            .iter()
            .find(|f| f.id == "c1")
            .cloned()
            .unwrap();
        let FunctionKind::Chaser(c) = &mut c1.kind else {
            panic!("c1 não é chaser")
        };
        let mut step = c.steps[0].clone();
        step.function = "k1".into();
        c.steps.push(step);
        let refused = apply(&mut s, Op::SaveFunction { function: c1 });
        assert_eq!(
            refused.unwrap_err(),
            "Uma criação não pode conter a si mesma."
        );
        let mut k1 = s
            .project
            .functions
            .iter()
            .find(|f| f.id == "k1")
            .cloned()
            .unwrap();
        if let FunctionKind::Collection(c) = &mut k1.kind {
            c.functions.push("k1".into());
        }
        assert!(apply(&mut s, Op::SaveFunction { function: k1 }).is_err());
    }

    fn efx_mut<'a>(s: &'a mut Snapshot, id: &str) -> &'a mut Efx {
        let f = s.project.functions.iter_mut().find(|f| f.id == id);
        match f.map(|f| &mut f.kind) {
            Some(FunctionKind::Efx(efx)) => efx,
            _ => panic!("{id} não é EFX"),
        }
    }

    #[test]
    fn efx_goes_with_pan_and_tilt_addresses_and_the_live_times() {
        let mut s = example();
        run(&mut s, "e1");
        let efx = |s: &Snapshot| match running_programs(s).remove(0).program {
            Program::Efx(efx) => efx,
            _ => panic!("e1 não é EFX"),
        };
        // "Círculo nos MHs": os 4 MHs, o MH 1 no 101 com pan, pan fino, tilt e tilt fino
        let e = efx(&s);
        assert_eq!(e.heads.len(), 4);
        let at = |channel, fine| SceneLevel {
            universe: "u1".into(),
            channel,
            fine: Some(fine),
            value: 0,
        };
        assert_eq!(e.heads[0], [Some(at(101, 102)), Some(at(103, 104))]);
        assert_eq!((e.cycle, e.fade_in), (Span::Ms(4000), Span::Ms(0)));
        // Uma fixture que perdeu o pan e o tilt (outro modo) fica fora do desenho
        s.project.fixtures[7].mode = "inexistente".into();
        assert_eq!(efx(&s).efx.fixtures.len(), 3);

        // Um elemento "Tempos" que segue o BPM: a volta em batidas
        let mut dial = s.project.console.widgets[0].clone();
        dial.id = "sd-efx".into();
        dial.kind = VcWidgetKind::SpeedDial(VcSpeedDial {
            functions: vec!["e1".into()],
            fade_in: true,
            fade_out: true,
            hold: true,
            beats: true,
        });
        s.project.console.widgets.push(dial);
        let op = Op::SetWidgetValue {
            widget: "sd-efx".into(),
            value: vec![1000, 0, 8000],
        };
        apply(&mut s, op).unwrap();
        let e = efx(&s);
        assert_eq!((e.cycle, e.fade_in), (Span::Beats(8000), Span::Beats(1000)));
    }

    #[test]
    fn efx_is_checked_when_saved() {
        let mut s = example();
        let save = |s: &mut Snapshot, change: fn(&mut Efx)| {
            let mut copy = s.clone();
            change(efx_mut(&mut copy, "e1"));
            let function = copy.project.functions.into_iter().find(|f| f.id == "e1");
            apply(
                s,
                Op::SaveFunction {
                    function: function.unwrap(),
                },
            )
        };
        assert!(save(&mut s, |e| e.width = 127).is_ok());
        let refused: [fn(&mut Efx); 6] = [
            |e| e.width = 128,
            |e| e.y_frequency = 33,
            |e| e.fixtures[0].start_offset = 360,
            |e| e.run_order = RunOrder::Random,
            |e| e.fixtures[1].fixture = "mh1".into(),
            // O PAR não tem pan nem tilt
            |e| e.fixtures[0].fixture = "par1".into(),
        ];
        for change in refused {
            assert!(save(&mut s, change).is_err());
        }
        assert_eq!(efx_mut(&mut s, "e1").width, 127);
    }

    #[test]
    fn fixture_groups_are_checked_and_removing_one_cleans_the_matrix() {
        let mut s = example();
        let groups = s.project.fixture_groups.clone();
        for group in groups.clone() {
            apply(&mut s, Op::SaveFixtureGroup { group }).unwrap();
        }
        // A barra de 8 pixels tem as cabeças 0 a 7
        let refused: [fn(&mut FixtureGroup); 5] = [
            |g| g.width = 0,
            |g| g.cells[0].x = g.width,
            |g| g.cells[1].x = g.cells[0].x,
            |g| g.cells[1].head = g.cells[0].head,
            |g| g.cells[0].head = 8,
        ];
        for change in refused {
            let mut group = groups[1].clone();
            change(&mut group);
            assert!(apply(&mut s, Op::SaveFixtureGroup { group }).is_err());
        }
        assert_eq!(s.project.fixture_groups, groups);
        // "Degradê nas barras" usava o grupo removido
        apply(&mut s, Op::RemoveFixtureGroup { id: "g2".into() }).unwrap();
        let matrix = s.project.functions.iter().find(|f| f.id == "r1");
        let Some(FunctionKind::RgbMatrix(m)) = matrix.map(|f| &f.kind) else {
            panic!("r1 não é RGB Matrix")
        };
        assert_eq!(m.group, None);
    }

    #[test]
    fn cue_list_runs_a_stopped_chaser_and_only_chasers() {
        let mut s = example();
        let step = |id: &str| Op::StepChaser {
            id: id.into(),
            action: StepAction::Next,
        };
        let (topics, _) = apply(&mut s, step("c1")).unwrap();
        assert_eq!(
            (topics, s.running.clone()),
            (vec![Topic::Running], vec!["c1".into()])
        );
        // Já rodando, o passo vai só ao motor
        assert_eq!(apply(&mut s, step("c1")).unwrap().0, vec![]);
        assert!(apply(&mut s, step("s1")).is_err());
    }

    #[test]
    fn only_intensity_position_and_beam_size_fade() {
        let s = example();
        let patch = universe_patch(&s.project);
        // MH 1 no 101: pan, pan fino e dimmer fazem fade; a roda de cor (108) e o gobo (109), não
        let fades: Vec<bool> = [101, 102, 106, 108, 109]
            .map(|c| patch[0].fades[c - 1])
            .into();
        assert_eq!(fades, [true, true, true, false, false]);
    }

    #[test]
    fn undoable_topics_restore_their_part() {
        let mut s = example();
        let before = s.clone();
        apply(
            &mut s,
            Op::RemoveFixtures {
                ids: vec!["mh1".into()],
            },
        )
        .unwrap();
        for topic in [
            Topic::Fixtures,
            Topic::Functions,
            Topic::Console,
            Topic::Stage,
        ] {
            assert!(topic.undoable());
            topic.restore(&mut s, topic.data(&before)).unwrap();
        }
        assert_eq!(s.project.fixtures, before.project.fixtures);
        assert!(
            !Topic::Desk.undoable() && !Topic::Project.undoable() && !Topic::Projects.is_project()
        );
        assert!(is_live(&Op::SetGrandMaster { value: 1 }) && !is_live(&Op::NewProject));
    }

    #[test]
    fn removing_a_definition_keeps_the_project_copy() {
        let mut s = example();
        let id = "generico-par-rgbw";
        apply(&mut s, Op::RemoveFixtureDef { id: id.into() }).unwrap();
        assert!(!s.library.iter().any(|d| d.id == id));
        assert!(s.project.fixture_defs.iter().any(|d| d.id == id));
        assert!(apply(&mut s, Op::RemoveFixtureDef { id: id.into() }).is_err());
        // O id vira nome de arquivo: só minúsculas, números e hífens
        let mut def = s.project.fixture_defs[0].clone();
        def.id = "../fora".into();
        assert!(apply(&mut s, Op::SaveFixtureDef { def }).is_err());
    }
}
