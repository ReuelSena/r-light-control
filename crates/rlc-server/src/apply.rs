//! Operações do contrato (A2): cada uma é validada, aplicada no estado e devolve os pedaços do
//! estado que mudaram, que viram os avisos `evt` (A3). As regras vêm do motor de mentira da F2.
//! Uma operação recusada não muda nada.

use rlc_core::*;
use rlc_engine::UniversePatch;
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
            | Op::StopAllFunctions
            | Op::Panic
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
    let panic = op == Op::Panic;
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
            vec![T::FixtureGroups, T::Console]
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
        // Parar também baixa os sliders de playback e solta os passos escolhidos nas cue lists.
        // O pânico também zera a Mesa simples e desliga o padrão de teste.
        Op::StopAllFunctions | Op::Panic => {
            s.running.clear();
            let playback: Vec<&Id> = p
                .console
                .widgets
                .iter()
                .filter(|w| match &w.kind {
                    VcWidgetKind::Slider(slider) => slider.mode == SliderMode::Playback,
                    VcWidgetKind::CueList(_) => true,
                    _ => false,
                })
                .map(|w| &w.id)
                .collect();
            s.widget_values.retain(|v| !playback.contains(&&v.widget));
            let mut topics = vec![T::Running, T::WidgetValues];
            if panic {
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
            for w in widgets {
                upsert(&mut p.console.widgets, w, |w| &w.id)?;
            }
            vec![T::Console]
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
        Op::SetBpm { bpm } => {
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
                defaults: Box::new([0; 512]),
            };
            for f in p.fixtures.iter().filter(|f| f.universe == u.id) {
                for (i, channel) in mode_channels(p, f).enumerate() {
                    let index = (usize::from(f.address) + i).checked_sub(1);
                    if let Some(c) = index.filter(|&c| c < 512) {
                        patch.intensity[c] = is_intensity(channel.attribute);
                        patch.defaults[c] = channel.default;
                    }
                }
            }
            patch
        })
        .collect()
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
        // O pânico desliga os dois
        apply(&mut s, Op::Panic).unwrap();
        assert!(s.preview.is_none() && s.highlight.is_empty());
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
