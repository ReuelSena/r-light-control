//! Operações do contrato (A2): cada uma é validada, aplicada no estado e devolve os pedaços do
//! estado que mudaram, que viram os avisos `evt` (A3). As regras vêm do motor de mentira da F2.
//! Uma operação recusada não muda nada.

use rlc_core::*;
use rlc_engine::{
    CellOutput, ChaserProgram, EfxProgram, Imposed, MatrixCell, MatrixProgram, MatrixSource,
    PhaserProgram, PhaserTrack, Program, Run, SceneLevel, SceneProgram, Span, Step, Times,
    UniversePatch,
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
    Scripts,
    ProjectPath,
    Dirty,
    Recovered,
    Projects,
    Library,
    Interfaces,
    Remote,
    Running,
    ChaserSteps,
    Paused,
    Failures,
    LoadedSteps,
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
            Topic::Scripts => "project.scripts",
            Topic::ProjectPath => "projectPath",
            Topic::Dirty => "dirty",
            Topic::Recovered => "recovered",
            Topic::Projects => "projects",
            Topic::Library => "library",
            Topic::Interfaces => "interfaces",
            Topic::Remote => "remote",
            Topic::Running => "running",
            Topic::ChaserSteps => "chaserSteps",
            Topic::Paused => "paused",
            Topic::Failures => "failures",
            Topic::LoadedSteps => "loadedSteps",
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
            Topic::Scripts => set(&mut p.scripts, value),
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
            Topic::Scripts => json!(p.scripts),
            Topic::ProjectPath => json!(s.project_path),
            Topic::Dirty => json!(s.dirty),
            Topic::Recovered => json!(s.recovered),
            Topic::Projects => json!(s.projects),
            Topic::Library => json!(s.library),
            Topic::Interfaces => json!(s.interfaces),
            Topic::Remote => json!(s.remote),
            Topic::Running => json!(s.running),
            Topic::ChaserSteps => json!(s.chaser_steps),
            Topic::Paused => json!(s.paused),
            Topic::Failures => json!(s.failures),
            Topic::LoadedSteps => json!(s.loaded_steps),
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
            | Op::SetFunctionPaused { .. }
            | Op::StepChaser { .. }
            | Op::LoadStep { .. }
            | Op::StopAllFunctions
            | Op::Panic { .. }
            | Op::SetDeskLevel { .. }
            | Op::SetDeskGroupLevel { .. }
            | Op::ResetDesk { .. }
            | Op::SetWidgetValue { .. }
            | Op::PressWidget { .. }
            | Op::SetGrandMaster { .. }
            | Op::SetBlackout { .. }
            | Op::SetBpm { .. }
            | Op::SetTempo { .. }
            | Op::SetRemoteAccess { .. }
            | Op::SetTestPattern { .. }
            | Op::SetInterfaceKind { .. }
            | Op::SetPreview { .. }
            | Op::SetHighlight { .. }
            | Op::PreviewMatrix { .. }
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
            s.failures.clear();
            s.widget_values.clear();
            s.desk = Desk {
                levels: vec![],
                groups: vec![],
            };
            s.preview = None;
            s.highlight.fixtures.clear();
            vec![
                T::Project,
                T::ProjectPath,
                T::Dirty,
                T::Running,
                T::Failures,
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
                    FunctionKind::Efx(efx) => efx
                        .fixtures
                        .retain(|x| !matches!(&x.target, EfxTarget::Fixture(id) if gone(id))),
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
            s.highlight.fixtures.retain(|id| !gone(id));
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
        // O grupo fica congelado onde era usado, e a luz não muda: os valores dele nas cenas passam
        // para as fixtures, e o EFX, o grupo de canais, o blackout de fixtures e o XY Pad ficam com as
        // fixtures que estavam nele. Só o RGB Matrix e o Phaser ficam sem grupo.
        Op::RemoveFixtureGroup { id } => {
            let members: Vec<Id> = group_fixtures(p, &id)
                .iter()
                .map(|f| f.id.clone())
                .collect();
            // Nas cenas, os canais que mudariam sem o grupo ganham o valor de agora
            let frozen: Vec<(Id, Vec<SceneValue>)> = p
                .functions
                .iter()
                .filter_map(|f| match &f.kind {
                    FunctionKind::Scene(scene) if scene.groups.iter().any(|g| g.group == id) => {
                        let mut without = scene.clone();
                        without.groups.retain(|g| g.group != id);
                        let after = scene_values(p, &without);
                        let values = scene_values(p, scene)
                            .into_iter()
                            .filter(|v| !after.contains(v));
                        Some((f.id.clone(), values.collect()))
                    }
                    _ => None,
                })
                .collect();
            // No EFX, o item do grupo vira as fixtures dele, com a defasagem já no início de cada uma
            let frozen_efx: Vec<(Id, Vec<EfxFixture>)> = p
                .functions
                .iter()
                .filter_map(|f| match &f.kind {
                    FunctionKind::Efx(efx) => {
                        let fixtures = efx_expand(p, efx, Some(&id));
                        (fixtures != efx.fixtures).then(|| (f.id.clone(), fixtures))
                    }
                    _ => None,
                })
                .collect();
            p.fixture_groups.retain(|g| g.id != id);
            for f in &mut p.functions {
                match &mut f.kind {
                    FunctionKind::Scene(scene) => {
                        let Some((_, values)) = frozen.iter().find(|(s, _)| *s == f.id) else {
                            continue;
                        };
                        scene.groups.retain(|g| g.group != id);
                        for v in values {
                            scene
                                .values
                                .retain(|o| o.fixture != v.fixture || o.channel != v.channel);
                            scene.values.push(v.clone());
                        }
                    }
                    FunctionKind::Efx(efx) => {
                        if let Some((_, fixtures)) = frozen_efx.iter().find(|(e, _)| *e == f.id) {
                            efx.fixtures = fixtures.clone();
                        }
                    }
                    FunctionKind::RgbMatrix(m) if m.group.as_ref() == Some(&id) => m.group = None,
                    FunctionKind::Phaser(ph) if ph.group.as_ref() == Some(&id) => ph.group = None,
                    _ => {}
                }
            }
            for g in &mut p.channel_groups {
                if g.fixture_group.as_ref() == Some(&id) {
                    g.fixture_group = None;
                }
            }
            for w in &mut p.console.widgets {
                let (groups, fixtures) = match &mut w.kind {
                    VcWidgetKind::Button(b) => (&mut b.groups, &mut b.fixtures),
                    VcWidgetKind::XyPad(xy) => (&mut xy.groups, &mut xy.fixtures),
                    _ => continue,
                };
                if groups.contains(&id) {
                    groups.retain(|g| *g != id);
                    for m in &members {
                        if !fixtures.contains(m) {
                            fixtures.push(m.clone());
                        }
                    }
                }
            }
            vec![T::FixtureGroups, T::ChannelGroups, T::Functions, T::Console]
        }
        Op::MoveFixtureGroup { id, to } => {
            move_to(&mut p.fixture_groups, &id, to, |g| &g.id)?;
            vec![T::FixtureGroups]
        }
        Op::SaveChannelGroup { group } => {
            if let Some(g) = &group.fixture_group
                && !p.fixture_groups.iter().any(|x| x.id == *g)
            {
                return Err("Grupo de fixtures não encontrado.".into());
            }
            upsert(&mut p.channel_groups, group, |g| &g.id)?;
            vec![T::ChannelGroups]
        }
        Op::MoveChannelGroup { id, to } => {
            move_to(&mut p.channel_groups, &id, to, |g| &g.id)?;
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
            match &function.kind {
                FunctionKind::Efx(efx) => check_efx(p, efx)?,
                FunctionKind::RgbMatrix(m) => check_matrix(p, m)?,
                FunctionKind::Phaser(ph) => check_phaser(p, ph)?,
                FunctionKind::Scene(scene) => {
                    for (i, g) in scene.groups.iter().enumerate() {
                        if !p.fixture_groups.iter().any(|x| x.id == g.group) {
                            return Err("Grupo de fixtures não encontrado.".into());
                        }
                        if scene.groups[..i].iter().any(|o| o.group == g.group) {
                            return Err("Grupo repetido na cena.".into());
                        }
                    }
                }
                _ => {}
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
                    VcWidgetKind::Slider(slider) => {
                        clear(&mut slider.function);
                        slider.functions.retain(|f| *f != id);
                    }
                    VcWidgetKind::CueList(cues) => clear(&mut cues.chaser),
                    VcWidgetKind::Animation(a) => clear(&mut a.matrix),
                    VcWidgetKind::SpeedDial(dial) => dial.functions.retain(|f| *f != id),
                    VcWidgetKind::Macro(m) => m.steps.retain(|step| {
                        !matches!(step, MacroStep::Start { function } | MacroStep::Stop { function }
                            | MacroStep::Go { chaser: function } if *function == id)
                    }),
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
            check_folder(p, &folder)?;
            upsert(&mut p.folders, folder, |d| &d.id)?;
            vec![T::Folders]
        }
        // O que estava dentro sobe para a pasta de cima
        Op::RemoveFolder { id } => {
            let parent = p
                .folders
                .iter()
                .find(|d| d.id == id)
                .and_then(|d| d.parent.clone());
            p.folders.retain(|d| d.id != id);
            for d in &mut p.folders {
                if d.parent.as_ref() == Some(&id) {
                    d.parent = parent.clone();
                }
            }
            for f in &mut p.functions {
                if f.folder.as_ref() == Some(&id) {
                    f.folder = parent.clone();
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
        // Só uma criação rodando pausa
        Op::SetFunctionPaused { id, paused } => {
            if (paused && !s.running.contains(&id)) || paused == s.paused.contains(&id) {
                return Ok((vec![], Value::Null));
            }
            s.paused.retain(|r| *r != id);
            if paused {
                s.paused.push(id);
            }
            vec![T::Paused]
        }
        // O passo pedido vai ao motor (no hub); parado, o chaser passa a rodar e, pausado, continua.
        // Ir para o passo carregado o desfaz
        Op::StepChaser { id, action } => {
            let chaser = |f: &Function| {
                f.id == id && matches!(f.kind, FunctionKind::Chaser(_) | FunctionKind::Sequence(_))
            };
            if !p.functions.iter().any(chaser) {
                return Err("A cue list só anda chasers e sequências.".into());
            }
            let mut topics = vec![];
            if let StepAction::Go(i) = action
                && s.loaded_steps.contains(&CurrentStep {
                    chaser: id.clone(),
                    step: i,
                })
            {
                s.loaded_steps.retain(|l| l.chaser != id);
                topics.push(T::LoadedSteps);
            }
            if s.paused.contains(&id) {
                s.paused.retain(|r| *r != id);
                topics.push(T::Paused);
            }
            if !s.running.contains(&id) {
                s.running.push(id);
                topics.push(T::Running);
            }
            topics
        }
        Op::LoadStep { id, step } => {
            let steps = p
                .functions
                .iter()
                .find_map(|f| match &f.kind {
                    FunctionKind::Chaser(c) | FunctionKind::Sequence(c) if f.id == id => {
                        Some(c.steps.len())
                    }
                    _ => None,
                })
                .ok_or("A cue list só anda chasers e sequências.")?;
            if step.is_some_and(|i| usize::from(i) >= steps) {
                return Err("Passo fora do chaser.".into());
            }
            s.loaded_steps.retain(|l| l.chaser != id);
            if let Some(step) = step {
                s.loaded_steps.push(CurrentStep { chaser: id, step });
            }
            vec![T::LoadedSteps]
        }
        // Parar também baixa os sliders de playback. O pânico corta as cenas na hora (no hub); com
        // `all`, também zera a Mesa simples e desliga o padrão de teste, a prévia e o "Localizar".
        Op::StopAllFunctions | Op::Panic { .. } => {
            s.running.clear();
            // O pânico de dois toques também zera os sliders de nível e os XY Pads, como zera a Mesa
            // simples
            let lowered: Vec<&Id> = p
                .console
                .widgets
                .iter()
                .filter(|w| {
                    playback(w).is_some()
                        || (panic_all
                            && matches!(
                                &w.kind,
                                VcWidgetKind::XyPad(_)
                                    | VcWidgetKind::Slider(VcSlider {
                                        mode: SliderMode::Level,
                                        ..
                                    })
                            ))
                })
                .map(|w| &w.id)
                .collect();
            s.widget_values.retain(|v| !lowered.contains(&&v.widget));
            let mut topics = vec![T::Running, T::WidgetValues];
            if panic_all {
                s.desk = Desk {
                    levels: vec![],
                    groups: vec![],
                };
                s.test_pattern = None;
                s.preview = None;
                s.highlight.fixtures.clear();
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
        Op::SetHighlight {
            mut fixtures,
            others,
        } => {
            fixtures.retain(|id| p.fixtures.iter().any(|f| &f.id == id));
            s.highlight = Locate { fixtures, others };
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
            // O elemento "Tempos" que trocou de unidade (segundos ou batidas) e o slider que trocou de
            // modo ou de criação (e o Animation que trocou de RGB Matrix) voltam ao valor inicial: o
            // valor antigo não serve no novo. O playback que estava aceso desliga a criação dele
            let unit = |w: &VcWidget| match &w.kind {
                VcWidgetKind::SpeedDial(dial) => format!("{}", dial.beats),
                VcWidgetKind::Slider(slider) => format!("{:?} {:?}", slider.mode, slider.function),
                VcWidgetKind::Animation(a) => format!("{:?}", a.matrix),
                _ => String::new(),
            };
            let changed: Vec<VcWidget> = p
                .console
                .widgets
                .iter()
                .filter(|old| {
                    widgets
                        .iter()
                        .any(|w| w.id == old.id && unit(old) != unit(w))
                })
                .cloned()
                .collect();
            let lit = |id: &Id| {
                s.widget_values
                    .iter()
                    .any(|v| v.widget == *id && v.value.first().is_some_and(|&x| x > 0))
            };
            let stopped: Vec<Id> = changed
                .iter()
                .filter(|old| lit(&old.id))
                .filter_map(|old| playback(old).cloned())
                .collect();
            for w in widgets {
                upsert(&mut p.console.widgets, w, |w| &w.id)?;
            }
            if changed.is_empty() {
                vec![T::Console]
            } else {
                s.widget_values
                    .retain(|v| !changed.iter().any(|w| w.id == v.widget));
                let mut topics = vec![T::Console, T::WidgetValues];
                if !stopped.is_empty() {
                    s.running.retain(|r| !stopped.contains(r));
                    topics.push(T::Running);
                }
                topics
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
            // O playback (slider ou Animation) liga a criação ao sair do zero e desliga ao voltar a ele
            let playback = p
                .console
                .widgets
                .iter()
                .find(|w| w.id == widget)
                .and_then(playback)
                .cloned();
            let up = value.first().is_some_and(|&v| v > 0);
            s.widget_values.retain(|v| v.widget != widget);
            if !value.is_empty() {
                s.widget_values.push(WidgetValue { widget, value });
            }
            let mut topics = vec![T::WidgetValues];
            if let Some(f) = playback
                && up != s.running.contains(&f)
            {
                s.running.retain(|r| *r != f);
                if up {
                    s.running.push(f);
                }
                topics.push(T::Running);
            }
            topics
        }
        Op::PressWidget {
            widget,
            control,
            down,
        } => press(s, &widget, control, down)?,
        Op::SetGrandMaster { value } => {
            s.master.grand_master = value;
            vec![T::Master]
        }
        Op::SetBlackout { on } => {
            s.master.blackout = on;
            vec![T::Master]
        }
        // O BPM vai ao motor no hub: o tap recomeça a batida, e o fader (`SetTempo`) não
        Op::SetBpm { bpm } | Op::SetTempo { bpm } => {
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
        Op::ImportScript { file_name, source } => {
            let meta = rlc_script::inspect(&source).map_err(|e| format!("{file_name}: {e}"))?;
            let script = RgbScript {
                id: meta.name.clone(),
                name: meta.name,
                author: meta.author,
                api: meta.api,
                accepts: meta.accepts,
                properties: meta.properties,
                source,
            };
            match p.scripts.iter_mut().find(|s| s.id == script.id) {
                Some(old) => *old = script.clone(),
                None => p.scripts.push(script.clone()),
            }
            // As criações que usavam o script antigo seguem com o novo; as propriedades que ele não
            // tem saem, para o projeto continuar válido
            for f in &mut p.functions {
                if let FunctionKind::RgbMatrix(m) = &mut f.kind
                    && let RgbAlgorithm::Script {
                        script: id,
                        properties,
                    } = &mut m.algorithm
                    && *id == script.id
                {
                    properties.retain(|v| script.properties.iter().any(|x| x.name == v.name));
                }
            }
            return Ok((vec![T::Scripts, T::Functions], json!(script)));
        }
        Op::RemoveScript { id } => {
            let used = |f: &Function| match &f.kind {
                FunctionKind::RgbMatrix(m) => {
                    matches!(&m.algorithm, RgbAlgorithm::Script { script, .. } if *script == id)
                }
                _ => false,
            };
            if p.functions.iter().any(used) {
                return Err("Um RGB Matrix usa este script.".into());
            }
            p.scripts.retain(|s| s.id != id);
            vec![T::Scripts]
        }
        Op::PreviewMatrix { id, step } => {
            let Some(FunctionKind::RgbMatrix(m)) =
                p.functions.iter().find(|f| f.id == id).map(|f| &f.kind)
            else {
                return Err("RGB Matrix não encontrado.".into());
            };
            let matrix = matrix_program(p, &id, m, Span::Ms(0), [Span::Ms(0); 2]);
            let (Some(steps), Some(map)) = (matrix.steps(), matrix.map(step as usize)) else {
                return Err("A prévia de um script vem do hub.".into());
            };
            let preview = MatrixPreview {
                steps: steps as u32,
                map,
            };
            return Ok((vec![], json!(preview)));
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
    let mut topics = topics;
    // A criação que parou deixa de estar pausada, e sai o passo carregado que não existe mais
    let paused = s.paused.len();
    s.paused.retain(|id| s.running.contains(id));
    if s.paused.len() < paused && !topics.contains(&T::Paused) {
        topics.push(T::Paused);
    }
    let loaded = s.loaded_steps.len();
    let functions = &s.project.functions;
    s.loaded_steps.retain(|l| {
        functions.iter().any(|f| match &f.kind {
            FunctionKind::Chaser(c) | FunctionKind::Sequence(c) => {
                f.id == l.chaser && usize::from(l.step) < c.steps.len()
            }
            _ => false,
        })
    });
    if s.loaded_steps.len() < loaded && !topics.contains(&T::LoadedSteps) {
        topics.push(T::LoadedSteps);
    }
    // Os grupos de canais que acompanham um grupo de fixtures refazem os canais (N1)
    let patch = [
        T::Fixtures,
        T::FixtureDefs,
        T::FixtureGroups,
        T::ChannelGroups,
    ];
    if topics.iter().any(|t| patch.contains(t))
        && follow_groups(&mut s.project)
        && !topics.contains(&T::ChannelGroups)
    {
        topics.push(T::ChannelGroups);
    }
    Ok((topics, Value::Null))
}

/// Refaz os canais dos grupos de canais que acompanham um grupo de fixtures: os atributos deles (sem
/// os finos) nas fixtures que estão no grupo agora. Diz se algum mudou.
fn follow_groups(p: &mut Project) -> bool {
    let mut changed = false;
    for i in 0..p.channel_groups.len() {
        let g = &p.channel_groups[i];
        let Some(group) = &g.fixture_group else {
            continue;
        };
        let channels: Vec<FixtureChannel> = group_fixtures(p, group)
            .into_iter()
            .flat_map(|f| {
                mode_channels(p, f)
                    .enumerate()
                    .filter(|(_, c)| !c.fine && g.attributes.contains(&c.attribute))
                    .map(|(channel, _)| FixtureChannel {
                        fixture: f.id.clone(),
                        channel: channel as u16,
                    })
            })
            .collect();
        if channels != g.channels {
            p.channel_groups[i].channels = channels;
            changed = true;
        }
    }
    changed
}

/// Fixtures de um grupo de fixtures, na ordem da grade (linha por linha), cada uma uma vez.
fn group_fixtures<'a>(p: &'a Project, group: &Id) -> Vec<&'a Fixture> {
    let mut cells: Vec<&GroupCell> = p
        .fixture_groups
        .iter()
        .filter(|g| g.id == *group)
        .flat_map(|g| &g.cells)
        .collect();
    cells.sort_by_key(|c| (c.y, c.x));
    let mut fixtures: Vec<&Fixture> = vec![];
    for c in cells {
        if let Some(f) = p.fixtures.iter().find(|f| f.id == c.fixture)
            && !fixtures.iter().any(|x| x.id == f.id)
        {
            fixtures.push(f);
        }
    }
    fixtures
}

/// Valores de uma cena por fixture (N1): os dos grupos dela nas fixtures que estão em cada um agora
/// (o grupo de depois vence) e, por cima, os valores próprios das fixtures.
fn scene_values(p: &Project, scene: &Scene) -> Vec<SceneValue> {
    let mut values: Vec<SceneValue> = vec![];
    for group in &scene.groups {
        for f in group_fixtures(p, &group.group) {
            let channels: Vec<&Channel> = mode_channels(p, f).collect();
            for (i, c) in channels.iter().enumerate() {
                // O k-ésimo canal do atributo recebe o k-ésimo valor dele
                let index = channels[..i]
                    .iter()
                    .filter(|o| o.attribute == c.attribute)
                    .count();
                if let Some(v) = group
                    .values
                    .iter()
                    .find(|v| v.attribute == c.attribute && usize::from(v.index) == index)
                {
                    values.push(SceneValue {
                        fixture: f.id.clone(),
                        channel: i as u16,
                        value: v.value,
                    });
                }
            }
        }
    }
    let mut merged: Vec<SceneValue> = vec![];
    for v in values.into_iter().chain(scene.values.iter().cloned()) {
        merged.retain(|o| o.fixture != v.fixture || o.channel != v.channel);
        merged.push(v);
    }
    merged
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
        scripts: vec![],
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
/// e o tilt dos EFX nos endereços DMX, e os chasers e as coleções com o que disparam, os tempos que
/// valem agora, a intensidade, a prioridade e a pausa. Depois delas, os sliders de nível acesos, na
/// prioridade Normal.
pub fn running_programs(s: &Snapshot) -> Vec<Run> {
    let mut runs: Vec<Run> = s
        .running
        .iter()
        .filter_map(|id| {
            // O Flash e o slider de playback são o próprio fade: a criação entra e sai sem os gravados
            let flash = flashed(s, id);
            let cut = flash || playback_level(s, id).is_some();
            let play = Play { rate: 1.0, cut };
            let function = s.project.functions.iter().find(|f| f.id == *id)?;
            Some(Run {
                id: id.clone(),
                program: program(s, id, [None; 3], &mut vec![], play)?,
                intensity: if flash { 255 } else { intensity(s, id) },
                priority: function.priority,
                paused: s.paused.contains(id),
            })
        })
        .collect();
    runs.extend(level_sliders(s));
    runs.extend(xy_pads(s));
    runs
}

/// Como uma criação está sendo tocada: a velocidade (os sliders de velocidade, que multiplicam os
/// tempos em segundos) e se ela entra e sai sem os fades gravados (o Flash e o slider de playback).
#[derive(Clone, Copy)]
struct Play {
    rate: f64,
    cut: bool,
}

/// O que uma criação acende. `dial` traz os tempos (fade in, fade out e duração) de um elemento
/// "Tempos" que controla uma coleção de fora; o que controla a própria criação vale antes. `path` são
/// as criações de fora: uma que contenha a si mesma não entra.
fn program(
    s: &Snapshot,
    id: &Id,
    dial: [Option<Span>; 3],
    path: &mut Vec<Id>,
    play: Play,
) -> Option<Program> {
    let p = &s.project;
    let function = p
        .functions
        .iter()
        .find(|f| f.id == *id && !path.contains(id))?;
    let dial: [Option<Span>; 3] = std::array::from_fn(|i| live_time(s, id, i).or(dial[i]));
    let play = Play {
        rate: play.rate * rate(s, id),
        ..play
    };
    // A velocidade só muda os tempos em segundos (menos o hold infinito): nos em batidas, manda o BPM
    let ms = |span: Span| match span {
        Span::Ms(ms) if ms != INFINITE => Span::Ms((f64::from(ms) / play.rate).round() as u32),
        span => span,
    };
    let fade = |dial: Option<Span>, recorded: u32| match play.cut {
        true => Span::Ms(0),
        false => ms(dial.unwrap_or(Span::Ms(recorded))),
    };
    path.push(id.clone());
    let program = match &function.kind {
        FunctionKind::Scene(scene) => {
            let (times, parts) = scene_times(scene, [dial[0], dial[1]], fade);
            Some(scene_program(p, &scene_values(p, scene), times, &parts))
        }
        FunctionKind::Collection(c) => Some(Program::Collection(
            c.functions
                .iter()
                .filter_map(|f| program(s, f, dial, path, play))
                .collect(),
        )),
        FunctionKind::Chaser(c) | FunctionKind::Sequence(c) => {
            let unit = |v: u32| if c.beats { Span::Beats(v) } else { Span::Ms(v) };
            let time = |mode: SpeedMode, common: u32, step: u32| match mode {
                SpeedMode::Default => None,
                SpeedMode::Common => Some(unit(common)),
                SpeedMode::PerStep => Some(unit(step)),
            };
            // Na sequência, os passos acendem a cena dela, e o "Padrão" são os tempos da cena
            let scene = c.scene.as_ref().and_then(|scene| {
                p.functions.iter().find_map(|f| match &f.kind {
                    FunctionKind::Scene(sc) if f.id == *scene => {
                        let live = [live_time(s, &f.id, 0), live_time(s, &f.id, 1)];
                        Some(scene_times(sc, live, |dial, recorded| {
                            ms(dial.unwrap_or(Span::Ms(recorded)))
                        }))
                    }
                    _ => None,
                })
            });
            let sequence = matches!(function.kind, FunctionKind::Sequence(_));
            // Os passos andam com os tempos deles: entrar sem fade vale só para a entrada
            let inner = Play { cut: false, ..play };
            let steps = c
                .steps
                .iter()
                .map(|step| Step {
                    program: if sequence {
                        let none = (Times::fades(Span::Ms(0), Span::Ms(0)), vec![]);
                        let (times, parts) = scene.clone().unwrap_or(none);
                        scene_program(p, &step.values, times, &parts)
                    } else {
                        // O passo de uma criação que não existe mais conta o tempo sem acender nada
                        program(s, &step.function, [None; 3], path, inner)
                            .unwrap_or(Program::Collection(vec![]))
                    },
                    // O delay de entrada e a curva vão com o fade in; o de saída, com o fade out
                    times: Imposed {
                        delay_in: time(
                            c.fade_in_mode,
                            c.common.delay_in_ms,
                            step.timing.delay_in_ms,
                        )
                        .map(ms),
                        fade_in: dial[0]
                            .or(time(
                                c.fade_in_mode,
                                c.common.fade_in_ms,
                                step.timing.fade_in_ms,
                            ))
                            .map(ms),
                        delay_out: time(
                            c.fade_out_mode,
                            c.common.delay_out_ms,
                            step.timing.delay_out_ms,
                        )
                        .map(ms),
                        fade_out: dial[1]
                            .or(time(
                                c.fade_out_mode,
                                c.common.fade_out_ms,
                                step.timing.fade_out_ms,
                            ))
                            .map(ms),
                        curve: match c.fade_in_mode {
                            SpeedMode::Default => None,
                            SpeedMode::Common => Some(c.common.curve),
                            SpeedMode::PerStep => Some(step.timing.curve),
                        },
                    },
                    // No hold, "Padrão" vale como "Comum". A duração do elemento em batidas vale em
                    // todos os chasers; a em segundos, só nos que não estão em batidas (neles, manda
                    // o BPM)
                    hold: ms(match dial[2] {
                        Some(span @ Span::Beats(_)) => span,
                        Some(span) if !c.beats => span,
                        _ if c.hold_mode == SpeedMode::PerStep => unit(step.timing.hold_ms),
                        _ => unit(c.common.hold_ms),
                    }),
                })
                .collect();
            Some(Program::Chaser(ChaserProgram {
                steps,
                run_order: c.run_order,
                direction: c.direction,
                keep_last: c.keep_last,
            }))
        }
        // Só as fixtures com pan ou tilt andam no desenho, e um grupo entra com as que estão nele agora
        // (cada fixture uma vez); a duração do elemento "Tempos" é a da volta
        FunctionKind::Efx(efx) => {
            let fixtures = efx_expand(p, efx, None);
            let mut efx = Efx {
                fixtures: vec![],
                ..efx.clone()
            };
            let mut heads = vec![];
            for x in fixtures {
                let EfxTarget::Fixture(id) = &x.target else {
                    continue;
                };
                let head = pan_tilt(p, id);
                if head.iter().any(Option::is_some) {
                    heads.push(head);
                    efx.fixtures.push(x);
                }
            }
            Some(Program::Efx(EfxProgram {
                cycle: ms(dial[2].unwrap_or(Span::Ms(efx.timing.hold_ms))),
                fade_in: fade(dial[0], efx.timing.fade_in_ms),
                fade_out: fade(dial[1], efx.timing.fade_out_ms),
                heads,
                efx,
            }))
        }
        // A duração do elemento "Tempos" é a do passo
        FunctionKind::RgbMatrix(m) => Some(Program::RgbMatrix(matrix_program(
            p,
            id,
            m,
            ms(dial[2].unwrap_or(Span::Ms(m.timing.hold_ms))),
            [
                fade(dial[0], m.timing.fade_in_ms),
                fade(dial[1], m.timing.fade_out_ms),
            ],
        ))),
        // A duração do elemento "Tempos" é a do ciclo (em batidas, seguindo o master)
        FunctionKind::Phaser(ph) => {
            let unit = |v: u32| {
                if ph.beats {
                    Span::Beats(v)
                } else {
                    Span::Ms(v)
                }
            };
            Some(Program::Phaser(PhaserProgram {
                tracks: ph
                    .layers
                    .iter()
                    .map(|layer| PhaserTrack {
                        layer: layer.clone(),
                        targets: phaser_targets(p, ph.group.as_ref(), layer.attribute),
                    })
                    .collect(),
                spread: ph.spread,
                cycle: ms(dial[2].unwrap_or(unit(ph.timing.hold_ms))),
                fade_in: fade(dial[0], ph.timing.fade_in_ms),
                fade_out: fade(dial[1], ph.timing.fade_out_ms),
            }))
        }
    };
    path.pop();
    program
}

/// O RGB Matrix para o motor: a grade do grupo nos endereços DMX, com o passo e os fades que valem.
/// Os valores das propriedades de um script vão como texto; as que não foram mexidas ficam com o
/// padrão do script.
pub fn matrix_program(
    p: &Project,
    id: &Id,
    m: &RgbMatrix,
    step: Span,
    fades: [Span; 2],
) -> MatrixProgram {
    let group = m
        .group
        .as_ref()
        .and_then(|id| p.fixture_groups.iter().find(|g| g.id == *id));
    let (source, accepts) = match &m.algorithm {
        RgbAlgorithm::Plain => (MatrixSource::Plain, 1),
        // Duas cores: o texto muda de uma para a outra de um passo ao seguinte
        RgbAlgorithm::Text { text, style, x, y } => (
            MatrixSource::Text {
                text: text.clone(),
                style: *style,
                x: *x,
                y: *y,
            },
            2,
        ),
        RgbAlgorithm::Image { image, style, x, y } => {
            let (width, height, pixels) = image.as_ref().map_or((0, 0, vec![]), image_pixels);
            let source = MatrixSource::Image {
                width,
                height,
                pixels: pixels.into(),
                style: *style,
                x: *x,
                y: *y,
            };
            (source, 0)
        }
        // Um script que saiu do projeto vale como a cor fixa (o servidor não deixa isso acontecer)
        RgbAlgorithm::Script { script, properties } => {
            match p.scripts.iter().find(|s| s.id == *script) {
                Some(s) => (
                    MatrixSource::Script {
                        source: s.source.as_str().into(),
                        properties: properties
                            .iter()
                            .map(|v| (v.name.clone(), v.value.clone()))
                            .collect(),
                    },
                    usize::from(s.accepts),
                ),
                None => (MatrixSource::Plain, 1),
            }
        }
    };
    MatrixProgram {
        function: id.clone(),
        source,
        accepts,
        colors: m.colors.iter().map(|c| hex_rgb(c)).collect(),
        run_order: m.run_order,
        direction: m.direction,
        width: group.map_or(0, |g| g.width),
        height: group.map_or(0, |g| g.height),
        cells: group.map_or(vec![], |g| matrix_cells(p, g, m.control)),
        step,
        fade_in: fades[0],
        fade_out: fades[1],
    }
}

/// Um botão do console apertado ou solto (A2). O botão de Flash ou Temp marca `[1]` no valor dele
/// enquanto segura a criação que ele mesmo acendeu: só ela apaga ao soltar. A cue list chega já
/// traduzida (`console_op`), e os outros elementos não têm o que apertar.
fn press(
    s: &mut Snapshot,
    id: &str,
    control: InputControl,
    down: bool,
) -> Result<Vec<Topic>, String> {
    use Topic as T;
    let w = s
        .project
        .console
        .widgets
        .iter()
        .find(|w| w.id == id)
        .ok_or("Elemento não encontrado.")?;
    let (VcWidgetKind::Button(b), InputControl::Main) = (&w.kind, control) else {
        return Ok(vec![]);
    };
    let (b, frame) = (b.clone(), w.frame.clone());
    let momentary = b.mode != ButtonMode::Toggle;
    let held = s.widget_values.iter().any(|v| v.widget == id);
    let hold = |s: &mut Snapshot, on: bool| {
        s.widget_values.retain(|v| v.widget != id);
        if on {
            s.widget_values.push(WidgetValue {
                widget: id.into(),
                value: vec![1],
            });
        }
    };
    // O liga/desliga e as ações de um toque agem ao apertar; soltar não faz nada
    if !momentary && !down {
        return Ok(vec![]);
    }
    Ok(match b.action {
        ButtonAction::Function | ButtonAction::Exclusive => {
            let Some(f) = b.function else {
                return Ok(vec![]);
            };
            let running = s.running.contains(&f);
            if (momentary && !down && held) || (!momentary && running) {
                s.running.retain(|r| *r != f);
                hold(s, false);
            } else if down && !running {
                if b.action == ButtonAction::Exclusive {
                    // "Só esta": as outras param (com o fade out), e os playbacks baixam
                    s.running.clear();
                    let lowered: Vec<Id> = s
                        .project
                        .console
                        .widgets
                        .iter()
                        .filter(|w| playback(w).is_some())
                        .map(|w| w.id.clone())
                        .collect();
                    s.widget_values.retain(|v| !lowered.contains(&v.widget));
                } else {
                    solo(s, frame.as_ref(), &f);
                }
                s.running.push(f);
                hold(s, momentary);
            } else {
                return Ok(vec![]);
            }
            vec![T::Running, T::WidgetValues]
        }
        ButtonAction::Blackout => {
            s.master.blackout = if momentary { down } else { !s.master.blackout };
            vec![T::Master]
        }
        ButtonAction::BlackoutFixtures => {
            hold(s, if momentary { down } else { !held });
            vec![T::WidgetValues]
        }
        ButtonAction::StopAll if down => apply(s, Op::StopAllFunctions)?.0,
        // O Voltar e o Pausar chegam já traduzidos (`console_op`)
        ButtonAction::StopAll | ButtonAction::Previous | ButtonAction::Pause => vec![],
    })
}

/// Numa moldura solo, ligar a criação de um botão para as dos outros botões da moldura.
fn solo(s: &mut Snapshot, frame: Option<&Id>, keep: &Id) {
    let widgets = &s.project.console.widgets;
    let solo = widgets
        .iter()
        .any(|x| Some(&x.id) == frame && matches!(&x.kind, VcWidgetKind::Frame(fr) if fr.solo));
    if !solo {
        return;
    }
    let others: Vec<Id> = widgets
        .iter()
        .filter(|x| x.frame.as_ref() == frame)
        .filter_map(|x| match &x.kind {
            VcWidgetKind::Button(b) => b.function.clone().filter(|f| f != keep),
            _ => None,
        })
        .collect();
    s.running.retain(|r| !others.contains(r));
}

/// O que a cue list e os botões Voltar e Pausar fazem vira a operação que faz: o passo vai direto ao
/// motor, então a tradução acontece no hub, antes de aplicar. Na cue list, o Go (o principal ou o
/// "próximo") começa o chaser parado, continua o pausado e, rodando, vai para o passo seguinte; com um
/// passo carregado, vai para ele.
pub fn console_op(s: &Snapshot, op: Op) -> Op {
    let (id, control) = match &op {
        Op::PressWidget {
            widget,
            control,
            down: true,
        } => {
            let target = s
                .project
                .console
                .widgets
                .iter()
                .find_map(|w| match &w.kind {
                    VcWidgetKind::CueList(c) if w.id == *widget => {
                        Some((c.chaser.clone(), *control))
                    }
                    VcWidgetKind::Button(b)
                        if w.id == *widget && *control == InputControl::Main =>
                    {
                        let control = match b.action {
                            ButtonAction::Previous => InputControl::Previous,
                            ButtonAction::Pause => InputControl::Pause,
                            _ => return None,
                        };
                        Some((b.function.clone(), control))
                    }
                    _ => None,
                });
            let Some((Some(id), control)) = target else {
                return op;
            };
            (id, control)
        }
        Op::StepChaser {
            id,
            action: StepAction::Next,
        } => (id.clone(), InputControl::Next),
        _ => return op,
    };
    let paused = s.paused.contains(&id);
    let loaded = s.loaded_steps.iter().find(|l| l.chaser == id);
    match control {
        InputControl::Main if paused => Op::SetFunctionPaused { id, paused: false },
        InputControl::Main | InputControl::Next => match loaded {
            Some(l) => Op::StepChaser {
                id,
                action: StepAction::Go(l.step),
            },
            None if control == InputControl::Main && !s.running.contains(&id) => {
                Op::SetFunctionRunning { id, running: true }
            }
            None => Op::StepChaser {
                id,
                action: StepAction::Next,
            },
        },
        InputControl::Previous => Op::StepChaser {
            id,
            action: StepAction::Previous,
        },
        InputControl::Pause => Op::SetFunctionPaused {
            id,
            paused: !paused,
        },
        InputControl::Stop => Op::SetFunctionRunning { id, running: false },
        _ => op,
    }
}

/// Valor ao vivo de um elemento do console (o primeiro número; 0 a 255 nos sliders).
fn widget_level(s: &Snapshot, widget: &str) -> Option<u32> {
    s.widget_values
        .iter()
        .find(|v| v.widget == widget)
        .and_then(|v| v.value.first().copied())
}

/// Um botão de Flash está segurando a criação (o Flash é cheio e sem fades).
fn flashed(s: &Snapshot, id: &Id) -> bool {
    s.project.console.widgets.iter().any(|w| {
        matches!(&w.kind, VcWidgetKind::Button(b) if b.mode == ButtonMode::Flash && b.function.as_ref() == Some(id))
            && s.widget_values.iter().any(|v| v.widget == w.id)
    })
}

/// A criação que um elemento toca pelo fader: a do slider de playback e o RGB Matrix do Animation.
fn playback(w: &VcWidget) -> Option<&Id> {
    match &w.kind {
        VcWidgetKind::Slider(slider) if slider.mode == SliderMode::Playback => {
            slider.function.as_ref()
        }
        VcWidgetKind::Animation(a) => a.matrix.as_ref(),
        _ => None,
    }
}

/// Nível do playback (slider ou Animation) que está tocando a criação (acima do zero).
fn playback_level(s: &Snapshot, id: &Id) -> Option<u32> {
    s.project
        .console
        .widgets
        .iter()
        .filter(|w| playback(w) == Some(id))
        .find_map(|w| widget_level(s, &w.id).filter(|&v| v > 0))
}

/// Velocidade de uma criação: os sliders de velocidade que a têm na lista, de ×0,25 a ×4 (sem valor,
/// ×1, que fica em 128, o meio do slider).
fn rate(s: &Snapshot, id: &Id) -> f64 {
    s.project
        .console
        .widgets
        .iter()
        .filter_map(|w| match &w.kind {
            VcWidgetKind::Slider(slider)
                if slider.mode == SliderMode::Rate && slider.functions.contains(id) =>
            {
                Some(widget_level(s, &w.id).unwrap_or(128))
            }
            _ => None,
        })
        .map(|v| 4f64.powf((f64::from(v) - 128.0) / 127.0).clamp(0.25, 4.0))
        .product()
}

/// Submasters que alcançam um elemento: os da moldura dele e das molduras de fora, e os soltos na
/// página (a página é a moldura de fora de todas).
fn submasters<'a>(p: &'a Project, w: &'a VcWidget) -> impl Iterator<Item = &'a VcWidget> {
    let mut frames: Vec<&Id> = vec![];
    let mut frame = w.frame.as_ref();
    while let Some(id) = frame.filter(|id| !frames.contains(id)) {
        frames.push(id);
        frame = p
            .console
            .widgets
            .iter()
            .find(|x| x.id == *id)
            .and_then(|x| x.frame.as_ref());
    }
    p.console.widgets.iter().filter(move |x| {
        x.page == w.page
            && matches!(&x.kind, VcWidgetKind::Slider(s) if s.mode == SliderMode::Submaster)
            && x.frame.as_ref().is_none_or(|f| frames.contains(&f))
    })
}

/// Fração de um submaster (sem valor, cheio).
fn submaster_level(s: &Snapshot, w: &VcWidget) -> f64 {
    f64::from(widget_level(s, &w.id).unwrap_or(255).min(255)) / 255.0
}

/// Intensidade de uma criação (0 a 255): o playback que a toca e os submasters que alcançam os
/// elementos que a controlam (botões, playbacks e cue lists), multiplicados.
fn intensity(s: &Snapshot, id: &Id) -> u8 {
    let p = &s.project;
    let mut factor = playback_level(s, id).map_or(1.0, |v| f64::from(v.min(255)) / 255.0);
    let mut seen: Vec<&Id> = vec![];
    for w in p.console.widgets.iter().filter(|w| match &w.kind {
        VcWidgetKind::Button(b) => b.function.as_ref() == Some(id),
        VcWidgetKind::CueList(c) => c.chaser.as_ref() == Some(id),
        _ => playback(w) == Some(id),
    }) {
        for sub in submasters(p, w) {
            if !seen.contains(&&sub.id) {
                seen.push(&sub.id);
                factor *= submaster_level(s, sub);
            }
        }
    }
    (factor * 255.0).round() as u8
}

/// Os XY Pads mexidos, como cenas sem fade no pan (X) e no tilt (Y) das fixtures deles e das que
/// estão nos grupos deles, pelo curso inteiro e em 16 bits com o fino: contam da hora em que foram
/// mexidos, como um slider de nível (LTP).
fn xy_pads(s: &Snapshot) -> impl Iterator<Item = Run> {
    let p = &s.project;
    p.console.widgets.iter().filter_map(move |w| {
        let VcWidgetKind::XyPad(xy) = &w.kind else {
            return None;
        };
        let value = &s.widget_values.iter().find(|v| v.widget == w.id)?.value;
        let [x, y] = [0, 1].map(|i| value.get(i).copied().unwrap_or(128).min(255) as u16);
        let mut fixtures: Vec<&Id> = xy.fixtures.iter().collect();
        for f in xy.groups.iter().flat_map(|g| group_fixtures(p, g)) {
            if !fixtures.contains(&&f.id) {
                fixtures.push(&f.id);
            }
        }
        let mut levels: Vec<SceneLevel> = vec![];
        for id in fixtures {
            for (level, v) in pan_tilt(p, id).into_iter().zip([x, y]) {
                if let Some(mut level) = level {
                    level.value = if level.fine.is_some() { v * 257 } else { v };
                    levels.push(level);
                }
            }
        }
        Some(Run {
            id: w.id.clone(),
            program: Program::Scene(SceneProgram {
                levels,
                times: Times::fades(Span::Ms(0), Span::Ms(0)),
            }),
            intensity: 255,
            priority: Priority::Normal,
            paused: false,
        })
    })
}

/// Os sliders de nível acesos, como cenas sem fade nos canais dos grupos de canais deles: na
/// intensidade misturam com as criações pelo maior valor; no resto, contam da hora em que saíram do
/// zero, como uma criação. Os submasters que os alcançam escalam a intensidade.
fn level_sliders(s: &Snapshot) -> impl Iterator<Item = Run> {
    let p = &s.project;
    p.console.widgets.iter().filter_map(move |w| {
        let VcWidgetKind::Slider(slider) = &w.kind else {
            return None;
        };
        let value =
            widget_level(s, &w.id).filter(|&v| v > 0 && slider.mode == SliderMode::Level)?;
        let mut levels: Vec<SceneLevel> = vec![];
        for (universe, channel) in slider
            .channel_groups
            .iter()
            .flat_map(|g| group_addresses(p, g))
        {
            if !levels
                .iter()
                .any(|l| l.universe == universe && l.channel == channel)
            {
                levels.push(SceneLevel {
                    universe,
                    channel,
                    fine: None,
                    value: value.min(255) as u16,
                });
            }
        }
        Some(Run {
            id: w.id.clone(),
            program: Program::Scene(SceneProgram {
                levels,
                times: Times::fades(Span::Ms(0), Span::Ms(0)),
            }),
            intensity: (submasters(p, w)
                .map(|sub| submaster_level(s, sub))
                .product::<f64>()
                * 255.0)
                .round() as u8,
            priority: Priority::Normal,
            paused: false,
        })
    })
}

/// Canais das fixtures apagadas pelos botões de blackout de fixtures ligados (o motor zera só a
/// intensidade deles).
pub fn fixture_blackout(s: &Snapshot) -> Vec<(Id, u16)> {
    let p = &s.project;
    let mut fixtures: Vec<&Id> = vec![];
    for w in &p.console.widgets {
        let VcWidgetKind::Button(b) = &w.kind else {
            continue;
        };
        if b.action != ButtonAction::BlackoutFixtures || widget_level(s, &w.id).is_none() {
            continue;
        }
        fixtures.extend(&b.fixtures);
        for g in p.fixture_groups.iter().filter(|g| b.groups.contains(&g.id)) {
            fixtures.extend(g.cells.iter().map(|c| &c.fixture));
        }
    }
    p.fixtures
        .iter()
        .filter(|f| fixtures.contains(&&f.id))
        .flat_map(|f| {
            (0..channel_count(p, f))
                .filter_map(|i| channel_address(p, &f.id, i))
                .map(|l| (l.universe, l.channel))
        })
        .collect()
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

/// Tempos de uma cena para o motor: os dela e os de cada tipo de atributo com tempos próprios (R3).
/// `fades` são o fade in e o fade out ao vivo (o elemento "Tempos"), que valem em todos; `time` leva
/// cada tempo ao motor (`time(ao vivo, gravado)`, com a velocidade e o "sem fades").
fn scene_times(
    scene: &Scene,
    fades: [Option<Span>; 2],
    time: impl Fn(Option<Span>, u32) -> Span,
) -> (Times, Vec<(AttributeKind, Times)>) {
    let times = |delay_in, fade_in, delay_out, fade_out| Times {
        delay_in: time(None, delay_in),
        fade_in: time(fades[0], fade_in),
        delay_out: time(None, delay_out),
        fade_out: time(fades[1], fade_out),
        curve: scene.curve,
    };
    let parts = scene
        .attribute_times
        .iter()
        .map(|a| {
            let t = times(a.delay_in_ms, a.fade_in_ms, a.delay_out_ms, a.fade_out_ms);
            (a.kind, t)
        })
        .collect();
    let own = times(
        scene.delay_in_ms,
        scene.fade_in_ms,
        scene.delay_out_ms,
        scene.fade_out_ms,
    );
    (own, parts)
}

/// Valores de uma cena (ou de um passo de sequência) nos endereços DMX. O canal fino vai junto do
/// grosso, para o fade em 16 bits; sem o fino na cena, ele vale 0. Com tempos próprios de tipos de
/// atributo (`parts`), a cena vira uma coleção, que o motor acende junta: uma parte com os tempos da
/// cena e uma para cada tipo.
fn scene_program(
    p: &Project,
    values: &[SceneValue],
    times: Times,
    parts: &[(AttributeKind, Times)],
) -> Program {
    let mut levels: Vec<Vec<SceneLevel>> = vec![vec![]; parts.len() + 1];
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
            let part = attribute_kind(channel.attribute)
                .and_then(|kind| parts.iter().position(|(k, _)| *k == kind))
                .map_or(0, |i| i + 1);
            levels[part].push(SceneLevel {
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
    let all = std::iter::once(times).chain(parts.iter().map(|(_, t)| *t));
    let mut scenes = levels
        .into_iter()
        .zip(all)
        .map(|(levels, times)| Program::Scene(SceneProgram { levels, times }));
    match parts.is_empty() {
        true => scenes.next().expect("a parte com os tempos da cena"),
        false => Program::Collection(scenes.collect()),
    }
}

/// Tipo de atributo dos tempos próprios da cena (R3): as abas do editor de cena.
fn attribute_kind(attribute: Attribute) -> Option<AttributeKind> {
    use Attribute::*;
    match attribute {
        Dimmer => Some(AttributeKind::Intensity),
        Red | Green | Blue | White | Amber | Uv | Lime | Indigo | Cyan | Magenta | Yellow
        | ColorWheel | ColorMacro | ColorTemperature => Some(AttributeKind::Color),
        Pan | Tilt => Some(AttributeKind::Position),
        Shutter | Gobo | GoboRotation | Prism | Focus | Zoom | Iris => Some(AttributeKind::Effects),
        Speed | Effect | Maintenance | Other => None,
    }
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

/// Os pixels de uma imagem do projeto como 0xRRGGBB, linha a linha, com o transparente em 0 (o QLC+ trata
/// o alfa 0 assim). Uma imagem que não bate com o tamanho que diz ter fica vazia.
fn image_pixels(image: &RgbImage) -> (u16, u16, Vec<u32>) {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&image.pixels)
        .unwrap_or_default();
    let (width, height) = (image.width, image.height);
    if bytes.len() != usize::from(width) * usize::from(height) * 4 {
        return (0, 0, vec![]);
    }
    let pixels = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| match p[3] {
            0 => 0,
            _ => u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2]),
        })
        .collect();
    (width, height, pixels)
}

/// O RGB Matrix `id` quando o algoritmo dele é um script: a prévia dele vem do hub, que espera a thread
/// do script.
pub fn script_matrix(p: &Project, id: &Id) -> Option<MatrixProgram> {
    let function = p.functions.iter().find(|f| f.id == *id)?;
    let FunctionKind::RgbMatrix(m) = &function.kind else {
        return None;
    };
    matches!(m.algorithm, RgbAlgorithm::Script { .. })
        .then(|| matrix_program(p, id, m, Span::Ms(0), [Span::Ms(0); 2]))
}

/// "#rrggbb" em [r, g, b]; o que não é cor vale preto.
fn hex_rgb(color: &str) -> [u8; 3] {
    let hex = color.strip_prefix('#').filter(|h| h.len() == 6);
    let part = |i: usize| hex.and_then(|h| u8::from_str_radix(&h[i..i + 2], 16).ok());
    [0, 2, 4].map(|i| part(i).unwrap_or(0))
}

/// Os canais de uma fixture e, entre eles, as posições dos da cabeça `head` (sem cabeças no modo, a
/// fixture é uma cabeça só, com todos os canais).
fn head_channels<'a>(
    p: &'a Project,
    fixture: &Fixture,
    head: u16,
) -> (Vec<&'a Channel>, Vec<usize>) {
    let def = p.fixture_defs.iter().find(|d| d.id == fixture.def_id);
    let mode = def.and_then(|d| d.modes.iter().find(|m| m.name == fixture.mode));
    let channels: Vec<&Channel> = mode_channels(p, fixture).collect();
    let positions = match mode.and_then(|m| m.heads.get(usize::from(head))) {
        Some(head) => head.iter().map(|&i| usize::from(i)).collect(),
        None => (0..channels.len()).collect(),
    };
    (channels, positions)
}

/// O canal do atributo (com o fino, se houver) entre as `positions` dos canais da fixture, no endereço
/// DMX; o valor não conta.
fn attribute_level(
    p: &Project,
    fixture: &Id,
    channels: &[&Channel],
    positions: &[usize],
    attribute: Attribute,
) -> Option<SceneLevel> {
    let find = |fine: bool| {
        let at = |&i: &usize| channels[i].attribute == attribute && channels[i].fine == fine;
        positions.iter().copied().find(at)
    };
    let coarse = channel_address(p, fixture, find(false)? as u16)?;
    let fine = find(true).and_then(|i| channel_address(p, fixture, i as u16));
    Some(SceneLevel {
        universe: coarse.universe,
        channel: coarse.channel,
        fine: fine.map(|l| l.channel),
        value: 0,
    })
}

/// O canal de `attribute` em cada cabeça do grupo, na ordem da grade (linha por linha): o da cabeça
/// ou, sem ele, o da fixture. Um canal entra uma vez só (a fixture de várias cabeças que só tem o
/// atributo na fixture inteira).
fn phaser_targets(p: &Project, group: Option<&Id>, attribute: Attribute) -> Vec<SceneLevel> {
    let mut cells: Vec<&GroupCell> = p
        .fixture_groups
        .iter()
        .filter(|g| Some(&g.id) == group)
        .flat_map(|g| &g.cells)
        .collect();
    cells.sort_by_key(|c| (c.y, c.x));
    let mut targets: Vec<SceneLevel> = vec![];
    for cell in cells {
        let Some(fixture) = p.fixtures.iter().find(|f| f.id == cell.fixture) else {
            continue;
        };
        let (channels, head) = head_channels(p, fixture, cell.head);
        let all: Vec<usize> = (0..channels.len()).collect();
        let level =
            |positions: &[usize]| attribute_level(p, &fixture.id, &channels, positions, attribute);
        if let Some(l) = level(&head).or_else(|| level(&all))
            && !targets
                .iter()
                .any(|t| t.universe == l.universe && t.channel == l.channel)
        {
            targets.push(l);
        }
    }
    targets
}

/// O Phaser salvo: um grupo que existe, de 1 a 8 camadas em atributos contínuos (um por tipo), com o
/// piso abaixo do teto, a largura do pulso, a fase e a defasagem nos limites.
fn check_phaser(p: &Project, ph: &Phaser) -> Result<(), String> {
    use Attribute::*;
    if ph
        .group
        .as_ref()
        .is_some_and(|g| !p.fixture_groups.iter().any(|x| x.id == *g))
    {
        return Err("Grupo de fixtures não encontrado.".into());
    }
    if !(1..=8).contains(&ph.layers.len()) {
        return Err("O Phaser tem de 1 a 8 camadas.".into());
    }
    if !(-360..=360).contains(&ph.spread) {
        return Err("A defasagem do Phaser vai de -360° a 360°.".into());
    }
    for (i, l) in ph.layers.iter().enumerate() {
        if !is_intensity(l.attribute)
            && !matches!(l.attribute, Pan | Tilt | Zoom | Focus | Iris | GoboRotation)
        {
            return Err("O Phaser só leva atributos contínuos (dimmer, cores, pan, tilt, zoom, foco, íris e rotação do gobo).".into());
        }
        if !(1..=99).contains(&l.width) {
            return Err("A largura do pulso vai de 1 a 99%.".into());
        }
        if l.phase > 359 {
            return Err("A fase da camada vai até 359°.".into());
        }
        if l.low > l.high {
            return Err("O piso da onda não pode passar do teto.".into());
        }
        if ph.layers[..i]
            .iter()
            .any(|o| o.attribute == l.attribute && o.relative == l.relative)
        {
            return Err("Atributo repetido no Phaser.".into());
        }
    }
    Ok(())
}

/// As cabeças de um grupo de fixtures com os canais que recebem a cor, nos endereços DMX, como o
/// `updateMapChannels` do QLC+. No RGB, os três canais de cor da cabeça (sem eles, ciano, magenta e
/// amarelo; sem nenhum dos dois, a cabeça não acende); nos outros, o canal do atributo (o da cabeça
/// ou, sem ele, o da fixture). Com o canal fino, o fade é em 16 bits.
fn matrix_cells(p: &Project, group: &FixtureGroup, control: RgbControl) -> Vec<MatrixCell> {
    let mut cells = vec![];
    for cell in &group.cells {
        let Some(fixture) = p.fixtures.iter().find(|f| f.id == cell.fixture) else {
            continue;
        };
        let (channels, head) = head_channels(p, fixture, cell.head);
        let all: Vec<usize> = (0..channels.len()).collect();
        let level = |attribute: Attribute, positions: &[usize]| {
            attribute_level(p, &fixture.id, &channels, positions, attribute)
        };
        let three = |attributes: [Attribute; 3]| {
            let levels = attributes.map(|a| level(a, &head));
            levels
                .iter()
                .all(Option::is_some)
                .then(|| levels.map(Option::unwrap))
        };
        let output = match control {
            RgbControl::Rgb => three([Attribute::Red, Attribute::Green, Attribute::Blue])
                .map(CellOutput::Rgb)
                .or_else(|| {
                    three([Attribute::Cyan, Attribute::Magenta, Attribute::Yellow])
                        .map(CellOutput::Cmy)
                }),
            _ => {
                let attribute = match control {
                    RgbControl::Dimmer => Attribute::Dimmer,
                    RgbControl::White => Attribute::White,
                    RgbControl::Amber => Attribute::Amber,
                    _ => Attribute::Uv,
                };
                level(attribute, &head)
                    .or_else(|| level(attribute, &all))
                    .map(CellOutput::Grey)
            }
        };
        if let Some(output) = output {
            cells.push(MatrixCell {
                x: cell.x,
                y: cell.y,
                output,
            });
        }
    }
    cells
}

/// O RGB Matrix salvo: de 1 a 5 cores #rrggbb, um grupo que existe e uma ordem que ele tem (o do QLC+
/// não tem Aleatória).
fn check_matrix(p: &Project, m: &RgbMatrix) -> Result<(), String> {
    if m.colors.is_empty() || m.colors.len() > 5 || m.colors.iter().any(|c| !is_hex_color(c)) {
        return Err("O RGB Matrix tem de 1 a 5 cores #rrggbb.".into());
    }
    if m.group
        .as_ref()
        .is_some_and(|g| !p.fixture_groups.iter().any(|x| x.id == *g))
    {
        return Err("Grupo de fixtures não encontrado.".into());
    }
    if m.run_order == RunOrder::Random {
        return Err("O RGB Matrix não tem a ordem Aleatória.".into());
    }
    if let RgbAlgorithm::Text { text, .. } = &m.algorithm
        && text.chars().count() > 256
    {
        return Err("O texto do RGB Matrix tem até 256 letras.".into());
    }
    if let RgbAlgorithm::Image {
        image: Some(image), ..
    } = &m.algorithm
    {
        let ok = (1..=128).contains(&image.width)
            && (1..=128).contains(&image.height)
            && image.pixels.len() <= 256 * 1024
            && image_pixels(image).0 == image.width;
        if !ok {
            return Err("A imagem tem até 128 × 128 pontos e 256 KB.".into());
        }
    }
    if let RgbAlgorithm::Script { script, properties } = &m.algorithm {
        let Some(script) = p.scripts.iter().find(|s| s.id == *script) else {
            return Err("Script não encontrado.".into());
        };
        for v in properties {
            let property = script.properties.iter().find(|x| x.name == v.name);
            let ok = match property.map(|x| &x.kind) {
                Some(PropertyKind::List { values }) => values.contains(&v.value),
                Some(PropertyKind::Range { min, max }) => v
                    .value
                    .parse::<i32>()
                    .is_ok_and(|n| (*min..=*max).contains(&n)),
                Some(PropertyKind::Float) => v.value.parse::<f64>().is_ok(),
                Some(PropertyKind::Text) => true,
                None => false,
            };
            if !ok {
                return Err(format!("Propriedade {} inválida no script.", v.name));
            }
        }
    }
    Ok(())
}

fn is_hex_color(color: &str) -> bool {
    color
        .strip_prefix('#')
        .is_some_and(|h| h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit()))
}

/// A lista do EFX com os grupos (todos, ou só `only`) trocados pelas fixtures deles que têm pan ou tilt,
/// na ordem da grade, cada uma `spread` graus depois da anterior (a defasagem vai para o início de cada
/// uma). Cada fixture fica no primeiro lugar em que aparece; a que tem item próprio na lista usa os
/// ajustes dele, também dentro de um grupo (a exceção, como na cena).
fn efx_expand(p: &Project, efx: &Efx, only: Option<&Id>) -> Vec<EfxFixture> {
    let mut list: Vec<EfxFixture> = vec![];
    for x in &efx.fixtures {
        let items = match &x.target {
            EfxTarget::Group(group) if only.is_none_or(|id| id == group) => {
                group_fixtures(p, group)
                    .into_iter()
                    .filter(|f| pan_tilt(p, &f.id) != [None, None])
                    .enumerate()
                    .map(|(k, f)| EfxFixture {
                        target: EfxTarget::Fixture(f.id.clone()),
                        start_offset: ((usize::from(x.start_offset) + k * usize::from(x.spread))
                            % 360) as u16,
                        spread: 0,
                        ..x.clone()
                    })
                    .collect()
            }
            _ => vec![x.clone()],
        };
        for x in items {
            let own = efx.fixtures.iter().find(|o| o.target == x.target);
            if !list.iter().any(|o| o.target == x.target) {
                list.push(own.cloned().unwrap_or(x));
            }
        }
    }
    list
}

/// Os limites do QLC+ para o EFX e as ordens que um desenho tem; só fixtures com pan ou tilt, sem
/// repetir.
fn check_efx(p: &Project, efx: &Efx) -> Result<(), String> {
    let own = efx.fixtures.iter().filter_map(|x| x.shape.as_ref());
    for shape in std::iter::once(&efx.shape).chain(own) {
        if shape.width > 127 || shape.height > 127 {
            return Err("A largura e a altura do EFX vão até 127.".into());
        }
        if shape.x_frequency > 32 || shape.y_frequency > 32 {
            return Err("As frequências do EFX vão até 32.".into());
        }
        if [shape.rotation, shape.x_phase, shape.y_phase]
            .iter()
            .any(|&a| a > 359)
        {
            return Err("Os ângulos do EFX vão até 359°.".into());
        }
        if shape.path.len() > 64 || shape.path.iter().any(|&(x, y)| x < -127 || y < -127) {
            return Err("O desenho à mão tem até 64 pontos, de -127 a 127.".into());
        }
        // A curva desenhada: 16 pontos, de 0 à volta inteira, sem descer
        if let Some(points) = &shape.speed_points
            && (points.len() != 16
                || points[0] != 0
                || points[15] != 255
                || points.windows(2).any(|w| w[1] < w[0]))
        {
            return Err("A curva de velocidade vai de 0 à volta inteira, sem descer.".into());
        }
    }
    let starts = efx.fixtures.iter().flat_map(|x| [x.start_offset, x.spread]);
    if starts.into_iter().any(|a| a > 359) {
        return Err("Os ângulos do EFX vão até 359°.".into());
    }
    if efx.run_order == RunOrder::Random {
        return Err("O EFX não tem a ordem aleatória.".into());
    }
    for (i, x) in efx.fixtures.iter().enumerate() {
        if efx.fixtures[..i].iter().any(|o| o.target == x.target) {
            return Err("Fixture ou grupo repetido no EFX.".into());
        }
        match &x.target {
            EfxTarget::Fixture(f) if pan_tilt(p, f) == [None, None] => {
                return Err("Só fixtures com pan ou tilt entram no EFX.".into());
            }
            EfxTarget::Group(g) if !p.fixture_groups.iter().any(|x| x.id == *g) => {
                return Err("Grupo de fixtures não encontrado.".into());
            }
            _ => {}
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

/// A pasta de cima existe, é do mesmo tipo e a pasta não acaba dentro de si mesma.
fn check_folder(p: &Project, folder: &Folder) -> Result<(), String> {
    let mut up = folder.parent.as_ref();
    while let Some(id) = up {
        if *id == folder.id {
            return Err("Uma pasta não pode ficar dentro de si mesma.".into());
        }
        let parent = p
            .folders
            .iter()
            .find(|d| d.id == *id)
            .ok_or("A pasta de cima não existe.")?;
        if parent.kind != folder.kind {
            return Err("A pasta de cima é de outro tipo.".into());
        }
        up = parent.parent.as_ref();
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
    scene_values(p, scene)
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

/// O "Localizar" para o motor: dimmer e RGBW a 100% e o shutter no padrão (o strobo em "luz ativa");
/// o resto (pan, tilt, gobo…) fica como está. Os canais de todas as fixtures localizadas vão à parte,
/// para o motor reduzir só a intensidade das outras.
pub fn highlight(s: &Snapshot) -> rlc_engine::Highlight {
    use Attribute::*;
    let p = &s.project;
    let mut highlight = rlc_engine::Highlight {
        others: s.highlight.others,
        ..Default::default()
    };
    for f in p
        .fixtures
        .iter()
        .filter(|f| s.highlight.fixtures.contains(&f.id))
    {
        for (i, channel) in mode_channels(p, f).enumerate() {
            let Some(level) = channel_address(p, &f.id, i as u16) else {
                continue;
            };
            highlight
                .located
                .push((level.universe.clone(), level.channel));
            let value = match channel.attribute {
                _ if channel.fine => continue,
                Dimmer | Red | Green | Blue | White => 255,
                Shutter => channel.default,
                _ => continue,
            };
            highlight.levels.push(DeskLevel { value, ..level });
        }
    }
    highlight
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

/// Leva o item de id `id` para a posição `to` (no fim, se passar do tamanho da lista).
fn move_to<T>(
    list: &mut Vec<T>,
    id: &Id,
    to: usize,
    key: impl Fn(&T) -> &Id,
) -> Result<(), String> {
    let from = list
        .iter()
        .position(|x| key(x) == id)
        .ok_or("Grupo não encontrado.")?;
    let item = list.remove(from);
    list.insert(to.min(list.len()), item);
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
    fn groups_are_reordered_in_the_list() {
        let mut s = example();
        let ids = |s: &Snapshot| -> Vec<Id> {
            s.project
                .channel_groups
                .iter()
                .map(|g| g.id.clone())
                .collect()
        };
        let before = ids(&s);
        assert!(before.len() >= 2);
        let first = before[0].clone();
        let (topics, _) = apply(
            &mut s,
            Op::MoveChannelGroup {
                id: first.clone(),
                to: 1,
            },
        )
        .unwrap();
        assert_eq!(topics, vec![Topic::ChannelGroups]);
        assert_eq!(ids(&s)[1], first);
        // Passar do fim leva para o último lugar; um id que não existe é recusado
        apply(
            &mut s,
            Op::MoveChannelGroup {
                id: first.clone(),
                to: 99,
            },
        )
        .unwrap();
        assert_eq!(ids(&s).last(), Some(&first));
        assert!(
            apply(
                &mut s,
                Op::MoveFixtureGroup {
                    id: "nao-existe".into(),
                    to: 0
                }
            )
            .is_err()
        );
    }

    #[test]
    fn folders_nest_and_removing_one_lifts_its_content() {
        let mut s = example();
        let folder = |id: &str, kind, parent: Option<&str>| Folder {
            id: id.into(),
            name: id.into(),
            kind,
            parent: parent.map(Into::into),
        };
        let save = |s: &mut Snapshot, f: Folder| apply(s, Op::SaveFolder { folder: f });
        save(&mut s, folder("a", FunctionType::Scene, Some("d1"))).unwrap();
        save(&mut s, folder("b", FunctionType::Scene, Some("a"))).unwrap();
        // Recusadas: dentro de si mesma, dentro de uma descendente, pai que não existe, pai de outro tipo
        assert!(save(&mut s, folder("a", FunctionType::Scene, Some("a"))).is_err());
        assert!(save(&mut s, folder("d1", FunctionType::Scene, Some("b"))).is_err());
        assert!(save(&mut s, folder("c", FunctionType::Scene, Some("x"))).is_err());
        assert!(save(&mut s, folder("c", FunctionType::Chaser, Some("a"))).is_err());

        // Remover "a" leva "b" e as criações dela para "d1"
        let scene = s
            .project
            .functions
            .iter_mut()
            .find(|f| f.id == "s4")
            .unwrap();
        scene.folder = Some("a".into());
        apply(&mut s, Op::RemoveFolder { id: "a".into() }).unwrap();
        let parent_of = |id: &str| {
            s.project
                .folders
                .iter()
                .find(|d| d.id == id)
                .unwrap()
                .parent
                .clone()
        };
        assert_eq!(parent_of("b"), Some("d1".into()));
        let scene = s.project.functions.iter().find(|f| f.id == "s4").unwrap();
        assert_eq!(scene.folder, Some("d1".into()));
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
                fixtures: vec![beam.clone(), "nao-existe".into()],
                others: 51,
            },
        )
        .unwrap();
        assert_eq!(s.highlight.fixtures.len(), 1);
        let h = highlight(&s);
        let channels: Vec<_> = h.levels.iter().map(|l| (l.channel, l.value)).collect();
        assert_eq!(channels, [(2, 255), (3, 255)]);
        // A intensidade das outras vai ao motor, junto de todos os canais do Beam (não só os acesos)
        let beam = s.project.fixtures.iter().find(|f| f.id == beam).unwrap();
        let count = usize::from(channel_count(&s.project, beam));
        assert_eq!(h.others, 51);
        assert_eq!(h.located.len(), count);
        assert!(count > channels.len());
        assert!(h.located.iter().all(|(universe, _)| universe == "u3"));
        assert_eq!(h.located[0].1, 1);
        // O segundo toque do pânico desliga os dois
        apply(&mut s, Op::Panic { all: true }).unwrap();
        assert!(s.preview.is_none() && s.highlight.fixtures.is_empty());
    }

    #[test]
    fn the_bpm_of_the_tap_and_of_the_fader_stays_within_30_and_300() {
        let mut s = example();
        let ops: [fn(u16) -> Op; 2] = [|bpm| Op::SetBpm { bpm }, |bpm| Op::SetTempo { bpm }];
        for op in ops {
            apply(&mut s, op(90)).unwrap();
            assert_eq!(s.master.bpm, 90);
            assert!(apply(&mut s, op(29)).is_err() && apply(&mut s, op(301)).is_err());
            assert_eq!(s.master.bpm, 90);
        }
    }

    fn run(s: &mut Snapshot, id: &str) {
        let op = Op::SetFunctionRunning {
            id: id.into(),
            running: true,
        };
        apply(s, op).unwrap();
    }

    /// Aperta (`down`) ou solta um elemento do console.
    fn hit(s: &mut Snapshot, widget: &str, down: bool) {
        let op = Op::PressWidget {
            widget: widget.into(),
            control: InputControl::Main,
            down,
        };
        apply(s, op).unwrap();
    }

    fn widget(s: &Snapshot, id: &str) -> VcWidget {
        s.project
            .console
            .widgets
            .iter()
            .find(|w| w.id == id)
            .cloned()
            .unwrap()
    }

    fn save_widget(s: &mut Snapshot, widget: VcWidget) {
        let op = Op::SaveWidgets {
            widgets: vec![widget],
        };
        apply(s, op).unwrap();
    }

    fn set_value(s: &mut Snapshot, widget: &str, value: Vec<u32>) {
        let op = Op::SetWidgetValue {
            widget: widget.into(),
            value,
        };
        apply(s, op).unwrap();
    }

    /// Fade in, fade out e intensidade de uma cena rodando.
    fn scene_run(s: &Snapshot, id: &str) -> (Span, Span, u8) {
        let run = running_programs(s)
            .into_iter()
            .find(|r| r.id == id)
            .unwrap();
        let times = scene_of(&run.program).times;
        (times.fade_in, times.fade_out, run.intensity)
    }

    fn button(w: &mut VcWidget) -> &mut VcButton {
        match &mut w.kind {
            VcWidgetKind::Button(b) => b,
            _ => panic!("não é botão"),
        }
    }

    fn slider(w: &mut VcWidget) -> &mut VcSlider {
        match &mut w.kind {
            VcWidgetKind::Slider(slider) => slider,
            _ => panic!("não é slider"),
        }
    }

    #[test]
    fn buttons_toggle_flash_temp_and_only_this() {
        let mut s = example();
        let ms = Span::Ms;
        // Liga/desliga ("b1", Frente âmbar, numa moldura solo): age ao apertar; soltar não faz nada
        hit(&mut s, "b1", true);
        hit(&mut s, "b1", false);
        assert_eq!(s.running, ["s1"]);
        // Na moldura solo, ligar a "b2" para a da "b1"; apertar de novo desliga
        hit(&mut s, "b2", true);
        assert_eq!(s.running, ["s2"]);
        hit(&mut s, "b2", true);
        assert!(s.running.is_empty());
        // Temp ("b11", Haze leve): só enquanto pressionado, com os fades gravados
        hit(&mut s, "b11", true);
        assert_eq!(s.running, ["s5"]);
        assert_eq!(scene_run(&s, "s5"), (ms(1000), ms(1000), 255));
        hit(&mut s, "b11", false);
        assert!(s.running.is_empty() && s.widget_values.is_empty());

        // A "b1" em Flash, com o submaster da moldura a 50%: na hora e cheia
        let mut b1 = widget(&s, "b1");
        button(&mut b1).mode = ButtonMode::Flash;
        save_widget(&mut s, b1.clone());
        set_value(&mut s, "sl1", vec![128]);
        hit(&mut s, "b1", true);
        assert_eq!(scene_run(&s, "s1"), (ms(0), ms(0), 255));
        hit(&mut s, "b1", false);
        assert!(s.running.is_empty());
        // O Flash não solta o que ele não acendeu
        run(&mut s, "s1");
        hit(&mut s, "b1", true);
        hit(&mut s, "b1", false);
        assert_eq!(s.running, ["s1"]);
        // Em Temp, o submaster vale
        apply(&mut s, Op::StopAllFunctions).unwrap();
        button(&mut b1).mode = ButtonMode::Temp;
        save_widget(&mut s, b1);
        hit(&mut s, "b1", true);
        assert_eq!(scene_run(&s, "s1"), (ms(1000), ms(1000), 128));
        hit(&mut s, "b1", false);

        // "Só esta" na "b6" (o chaser c1): para as outras e baixa o slider de playback
        let mut b6 = widget(&s, "b6");
        button(&mut b6).action = ButtonAction::Exclusive;
        save_widget(&mut s, b6);
        run(&mut s, "s1");
        set_value(&mut s, "sl3", vec![200]);
        assert_eq!(s.running, ["s1", "s4"]);
        hit(&mut s, "b6", true);
        assert_eq!(s.running, ["c1"]);
        assert!(!s.widget_values.iter().any(|v| v.widget == "sl3"));
        // Apertar de novo desliga só ela
        hit(&mut s, "b6", true);
        assert!(s.running.is_empty());
    }

    #[test]
    fn blackout_buttons_and_stop_all() {
        let mut s = example();
        // "b4": blackout dos PARs da frente (o grupo g1), liga/desliga
        hit(&mut s, "b4", true);
        let dark = fixture_blackout(&s);
        assert!(dark.contains(&("u1".into(), 1)) && !dark.contains(&("u1".into(), 101)));
        hit(&mut s, "b4", true);
        assert!(fixture_blackout(&s).is_empty());
        // Parar tudo ("b12")
        run(&mut s, "s1");
        hit(&mut s, "b12", true);
        assert!(s.running.is_empty());
        // Blackout geral em Temp: só enquanto pressionado
        let mut b12 = widget(&s, "b12");
        button(&mut b12).action = ButtonAction::Blackout;
        button(&mut b12).mode = ButtonMode::Temp;
        save_widget(&mut s, b12);
        hit(&mut s, "b12", true);
        assert!(s.master.blackout);
        hit(&mut s, "b12", false);
        assert!(!s.master.blackout);
    }

    #[test]
    fn playback_slider_is_the_fade_and_the_intensity() {
        let mut s = example();
        let ms = Span::Ms;
        // "sl3" toca a s4 (MHs no centro): sai do zero e liga, sem os fades gravados
        set_value(&mut s, "sl3", vec![128]);
        assert_eq!(s.running, ["s4"]);
        assert_eq!(scene_run(&s, "s4"), (ms(0), ms(0), 128));
        set_value(&mut s, "sl3", vec![255]);
        assert_eq!(scene_run(&s, "s4").2, 255);
        set_value(&mut s, "sl3", vec![0]);
        assert!(s.running.is_empty());
        // Trocar o modo com o slider aceso desliga a criação e solta o valor
        set_value(&mut s, "sl3", vec![200]);
        let mut sl3 = widget(&s, "sl3");
        slider(&mut sl3).mode = SliderMode::Level;
        save_widget(&mut s, sl3);
        assert!(s.running.is_empty() && s.widget_values.is_empty());
    }

    #[test]
    fn submasters_reach_the_frame_the_frames_inside_and_the_page() {
        let mut s = example();
        // A s1 pela "b1" (na moldura fr1, com o submaster sl1) e a s4 pelo slider de fora
        hit(&mut s, "b1", true);
        set_value(&mut s, "sl3", vec![255]);
        let intensities = |s: &Snapshot| [scene_run(s, "s1").2, scene_run(s, "s4").2];
        // Sem valor, o submaster fica cheio
        assert_eq!(intensities(&s), [255, 255]);
        set_value(&mut s, "sl1", vec![128]);
        assert_eq!(intensities(&s), [128, 255]);
        // Um submaster solto na página alcança os dois
        let mut page = widget(&s, "sl1");
        page.id = "sub".into();
        page.frame = None;
        save_widget(&mut s, page);
        set_value(&mut s, "sub", vec![128]);
        assert_eq!(intensities(&s), [64, 128]);
        // Uma moldura dentro da fr1: os dois submasters multiplicam
        let mut inner = widget(&s, "fr1");
        inner.id = "fr2".into();
        inner.frame = Some("fr1".into());
        let VcWidgetKind::Frame(frame) = &mut inner.kind else {
            panic!("fr1 não é moldura")
        };
        frame.solo = false;
        save_widget(&mut s, inner);
        let mut bx = widget(&s, "b2");
        bx.id = "bx".into();
        bx.frame = Some("fr2".into());
        save_widget(&mut s, bx);
        hit(&mut s, "bx", true);
        assert_eq!(scene_run(&s, "s2").2, 64);
    }

    #[test]
    fn level_slider_lights_its_channel_groups_without_fade() {
        let mut s = example();
        // "sl2": o grupo "Dimmer PARs" (seis canais)
        set_value(&mut s, "sl2", vec![200]);
        let (fade_in, fade_out, intensity) = scene_run(&s, "sl2");
        assert_eq!(
            (fade_in, fade_out, intensity),
            (Span::Ms(0), Span::Ms(0), 255)
        );
        let runs = running_programs(&s);
        let levels = &scene_of(&runs.iter().find(|r| r.id == "sl2").unwrap().program).levels;
        assert_eq!(levels.len(), 6);
        assert!(levels.iter().all(|l| l.value == 200));
        // O pânico de um toque não mexe nele; o de dois zera
        apply(&mut s, Op::Panic { all: false }).unwrap();
        assert!(running_programs(&s).iter().any(|r| r.id == "sl2"));
        apply(&mut s, Op::Panic { all: true }).unwrap();
        assert!(running_programs(&s).is_empty());
    }

    #[test]
    fn xy_pad_moves_the_pan_and_tilt_of_its_group_in_16_bits() {
        let mut s = example();
        // "xy1": o grupo "Moving heads" (4 Beams, pan e tilt com o fino)
        assert!(!running_programs(&s).iter().any(|r| r.id == "xy1"));
        set_value(&mut s, "xy1", vec![255, 0]);
        let runs = running_programs(&s);
        let levels = &scene_of(&runs.iter().find(|r| r.id == "xy1").unwrap().program).levels;
        assert_eq!(levels.len(), 8);
        assert!(levels.iter().all(|l| l.fine.is_some()));
        let values: Vec<u16> = levels.iter().map(|l| l.value).collect();
        assert_eq!(values, [0xFFFF, 0].repeat(4));
        // O pânico de dois toques solta o pad
        apply(&mut s, Op::Panic { all: true }).unwrap();
        assert!(!running_programs(&s).iter().any(|r| r.id == "xy1"));
        // Remover o grupo deixa no pad as fixtures que estavam nele
        apply(&mut s, Op::RemoveFixtureGroup { id: "g3".into() }).unwrap();
        let VcWidgetKind::XyPad(xy) = widget(&s, "xy1").kind else {
            panic!("xy1 não é XY Pad")
        };
        assert_eq!((xy.groups.len(), xy.fixtures.len()), (0, 4));
    }

    #[test]
    fn animation_fader_plays_its_matrix() {
        let mut s = example();
        // "an1" toca o r1: sai do zero e liga, com a intensidade do fader
        set_value(&mut s, "an1", vec![128]);
        assert_eq!(s.running, ["r1"]);
        let run = |s: &Snapshot| running_programs(s).into_iter().find(|r| r.id == "r1");
        assert_eq!(run(&s).unwrap().intensity, 128);
        set_value(&mut s, "an1", vec![0]);
        assert!(s.running.is_empty());
        // Ligado pelo Play, sem valor, fica cheio
        apply(
            &mut s,
            Op::SetFunctionRunning {
                id: "r1".into(),
                running: true,
            },
        )
        .unwrap();
        assert_eq!(run(&s).unwrap().intensity, 255);
    }

    #[test]
    fn rate_scales_the_times_in_seconds_but_not_in_beats() {
        let mut s = example();
        // Um slider de velocidade com a s1 e o chaser c1 (em segundos)
        let mut rate = widget(&s, "sl3");
        rate.id = "rt".into();
        let rt = slider(&mut rate);
        rt.mode = SliderMode::Rate;
        rt.function = None;
        rt.functions = vec!["s1".into(), "c1".into()];
        save_widget(&mut s, rate);
        run(&mut s, "s1");
        run(&mut s, "c1");
        let times = |s: &Snapshot| {
            let runs = running_programs(s);
            let scene = scene_of(&runs[0].program);
            let step = &chaser_of(&runs[1].program).steps[0];
            [
                Some(scene.times.fade_in),
                step.times.fade_in,
                Some(step.hold),
            ]
        };
        let ms = |v| Some(Span::Ms(v));
        let normal = times(&s);
        assert_eq!(normal[0], ms(1000));
        // ×4 (o topo): os tempos caem a um quarto; ×0,25 (embaixo): quadruplicam
        set_value(&mut s, "rt", vec![255]);
        let fast = times(&s);
        set_value(&mut s, "rt", vec![1]);
        let slow = times(&s);
        let scaled = |times: [Option<Span>; 3], by: f64| {
            times.map(|t| match t {
                Some(Span::Ms(v)) => ms((f64::from(v) * by).round() as u32),
                other => other,
            })
        };
        assert_eq!(fast, scaled(normal, 0.25));
        assert_eq!(slow, scaled(normal, 4.0));
        // No meio (128), ×1
        set_value(&mut s, "rt", vec![128]);
        assert_eq!(times(&s), normal);
        // Os tempos em batidas do elemento "Tempos" ("sd1" segue o BPM) não mudam
        set_value(&mut s, "rt", vec![255]);
        set_value(&mut s, "sd1", vec![2000, 500, 0]);
        assert_eq!(times(&s)[0], Some(Span::Beats(2000)));
    }

    #[test]
    fn cue_list_press_becomes_go_back_pause_and_stop() {
        let mut s = example();
        let op = |widget: &str, control, down| Op::PressWidget {
            widget: widget.into(),
            control,
            down,
        };
        let go = |s: &Snapshot| console_op(s, op("cl3", InputControl::Main, true));
        let play = |running| Op::SetFunctionRunning {
            id: "c2".into(),
            running,
        };
        let step = |action| Op::StepChaser {
            id: "c2".into(),
            action,
        };
        let pause = |paused| Op::SetFunctionPaused {
            id: "c2".into(),
            paused,
        };
        // Parado, o Go começa; rodando, vai para o passo seguinte; pausado, continua
        assert_eq!(go(&s), play(true));
        run(&mut s, "c2");
        assert_eq!(go(&s), step(StepAction::Next));
        let back = op("cl3", InputControl::Previous, true);
        assert_eq!(console_op(&s, back), step(StepAction::Previous));
        let paused = console_op(&s, op("cl3", InputControl::Pause, true));
        assert_eq!(paused, pause(true));
        apply(&mut s, paused).unwrap();
        assert_eq!(go(&s), pause(false));
        let stop = console_op(&s, op("cl3", InputControl::Stop, true));
        assert_eq!(stop, play(false));
        // Os botões Voltar e Pausar do roteiro fazem o mesmo
        assert_eq!(
            console_op(&s, op("b13", InputControl::Main, true)),
            step(StepAction::Previous)
        );
        assert_eq!(
            console_op(&s, op("b14", InputControl::Main, true)),
            pause(false)
        );
        // Com um passo carregado, o Go (e o "próximo" de outro lugar) vai para ele
        let load = Op::LoadStep {
            id: "c2".into(),
            step: Some(3),
        };
        apply(&mut s, load).unwrap();
        apply(&mut s, pause(false)).unwrap();
        assert_eq!(go(&s), step(StepAction::Go(3)));
        assert_eq!(
            console_op(&s, step(StepAction::Next)),
            step(StepAction::Go(3))
        );
        // Soltar não vira nada
        let up = op("cl3", InputControl::Main, false);
        assert_eq!(console_op(&s, up.clone()), up);
    }

    #[test]
    fn pause_and_the_loaded_step_follow_the_running_chaser() {
        let mut s = example();
        let pause = |paused| Op::SetFunctionPaused {
            id: "c2".into(),
            paused,
        };
        // Parada, a criação não pausa
        assert!(apply(&mut s, pause(true)).unwrap().0.is_empty());
        run(&mut s, "c2");
        apply(&mut s, pause(true)).unwrap();
        assert_eq!(s.paused, ["c2"]);
        assert!(
            running_programs(&s)
                .iter()
                .any(|r| r.id == "c2" && r.paused)
        );
        // Ir para um passo continua a criação pausada; ir para o carregado o desfaz
        let load = |step| Op::LoadStep {
            id: "c2".into(),
            step,
        };
        assert!(apply(&mut s, load(Some(9))).is_err());
        apply(&mut s, load(Some(2))).unwrap();
        let go = |i| Op::StepChaser {
            id: "c2".into(),
            action: StepAction::Go(i),
        };
        let (topics, _) = apply(&mut s, go(1)).unwrap();
        assert_eq!(topics, [Topic::Paused]);
        assert_eq!(s.loaded_steps.len(), 1);
        apply(&mut s, go(2)).unwrap();
        assert!(s.loaded_steps.is_empty());
        // Parada, deixa de estar pausada; o passo carregado fica para o próximo Go
        apply(&mut s, pause(true)).unwrap();
        apply(&mut s, load(Some(1))).unwrap();
        let stop = Op::SetFunctionRunning {
            id: "c2".into(),
            running: false,
        };
        let (topics, _) = apply(&mut s, stop).unwrap();
        assert_eq!(topics, [Topic::Running, Topic::Paused]);
        assert_eq!(s.loaded_steps.len(), 1);
        // O passo carregado sai quando o passo deixa de existir
        let mut f = s
            .project
            .functions
            .iter()
            .find(|f| f.id == "c2")
            .unwrap()
            .clone();
        if let FunctionKind::Chaser(c) = &mut f.kind {
            c.steps.truncate(1);
        }
        let (topics, _) = apply(&mut s, Op::SaveFunction { function: f }).unwrap();
        assert!(topics.contains(&Topic::LoadedSteps) && s.loaded_steps.is_empty());
    }

    fn phaser_program(s: &Snapshot, id: &str) -> PhaserProgram {
        match running_programs(s)
            .into_iter()
            .find(|r| r.id == id)
            .map(|r| r.program)
        {
            Some(Program::Phaser(p)) => p,
            _ => panic!("{id} não é um Phaser rodando"),
        }
    }

    fn phaser_mut<'a>(s: &'a mut Snapshot, id: &str) -> &'a mut Phaser {
        match s.project.functions.iter_mut().find(|f| f.id == id) {
            Some(Function {
                kind: FunctionKind::Phaser(ph),
                ..
            }) => ph,
            _ => panic!("{id} não é um Phaser"),
        }
    }

    #[test]
    fn the_phaser_reaches_the_attribute_channels_of_each_head_in_grid_order() {
        let mut s = example();
        for id in ["ph1", "ph3", "ph4"] {
            run(&mut s, id);
        }
        // O endereço de cada fixture do grupo "ParLeds", na ordem da grade
        let group = |g: &str| {
            let mut cells = s
                .project
                .fixture_groups
                .iter()
                .find(|x| x.id == g)
                .unwrap()
                .cells
                .clone();
            cells.sort_by_key(|c| (c.y, c.x));
            let address = |c: &GroupCell| {
                let f = s.project.fixtures.iter().find(|f| f.id == c.fixture);
                f.unwrap().address
            };
            cells.iter().map(address).collect::<Vec<u16>>()
        };
        let channels = |track: &PhaserTrack| track.targets.iter().map(|t| t.channel).collect();
        let parleds = group("g4");
        assert_eq!(parleds.len(), 12);
        // Onda de dimmer: o dimmer (o primeiro canal) de cada ParLed, com o ciclo em 4 batidas
        let wave = phaser_program(&s, "ph1");
        assert_eq!((wave.spread, wave.cycle), (30, Span::Beats(4000)));
        assert_eq!(channels(&wave.tracks[0]), parleds);
        // Arco-íris: vermelho, verde e azul são o segundo, o terceiro e o quarto canal
        let rainbow = phaser_program(&s, "ph3");
        for (track, k) in rainbow.tracks.iter().zip(1..) {
            let channels: Vec<u16> = channels(track);
            assert_eq!(channels, parleds.iter().map(|a| a + k).collect::<Vec<_>>());
        }
        // Balanço: o pan em 16 bits (o fino logo depois), o tilt no terceiro canal, o ciclo em segundos
        let sway = phaser_program(&s, "ph4");
        assert_eq!(sway.cycle, Span::Ms(4000));
        let heads = group("g3");
        let first = &sway.tracks[0].targets[0];
        assert_eq!((first.channel, first.fine), (heads[0], Some(heads[0] + 1)));
        assert_eq!(channels(&sway.tracks[1])[0], heads[0] + 2);
        assert!(sway.tracks.iter().all(|t| t.layer.relative));
    }

    #[test]
    fn a_phaser_layer_skips_the_fixtures_without_its_attribute() {
        // O grupo "Moving heads" não tem cores: a camada de vermelho não acha nenhuma fixture
        let mut s = example();
        let ph = phaser_mut(&mut s, "ph1");
        ph.group = Some("g3".into());
        ph.layers[0].attribute = Attribute::Red;
        run(&mut s, "ph1");
        assert!(phaser_program(&s, "ph1").tracks[0].targets.is_empty());
        // Sem grupo, o Phaser não acende nada
        phaser_mut(&mut s, "ph1").group = None;
        assert!(phaser_program(&s, "ph1").tracks[0].targets.is_empty());
    }

    #[test]
    fn the_saved_phaser_is_checked() {
        let mut s = example();
        type Edit = fn(&mut Phaser);
        let save = |s: &mut Snapshot, edit: Edit| {
            let mut f = s
                .project
                .functions
                .iter()
                .find(|f| f.id == "ph3")
                .unwrap()
                .clone();
            if let FunctionKind::Phaser(ph) = &mut f.kind {
                edit(ph);
            }
            apply(s, Op::SaveFunction { function: f }).map(|_| ())
        };
        assert!(save(&mut s, |_| {}).is_ok());
        let cases: [(&str, Edit); 9] = [
            ("grupo", |ph| ph.group = Some("nao-existe".into())),
            ("sem camadas", |ph| ph.layers.clear()),
            ("camadas demais", |ph| {
                let layer = ph.layers[0].clone();
                ph.layers = vec![layer; 9];
            }),
            ("atributo que não é contínuo", |ph| {
                ph.layers[0].attribute = Attribute::Gobo
            }),
            ("atributo repetido", |ph| {
                ph.layers[1].attribute = ph.layers[0].attribute
            }),
            ("largura do pulso", |ph| ph.layers[0].width = 0),
            ("fase", |ph| ph.layers[0].phase = 360),
            ("piso acima do teto", |ph| {
                (ph.layers[0].low, ph.layers[0].high) = (200, 100)
            }),
            ("defasagem", |ph| ph.spread = 361),
        ];
        for (what, edit) in cases {
            assert!(save(&mut s, edit).is_err(), "{what}");
        }
        // O mesmo atributo vale uma vez absoluto e outra relativo
        let relative_too = |ph: &mut Phaser| {
            let mut layer = ph.layers[0].clone();
            layer.relative = true;
            ph.layers.push(layer);
        };
        assert!(save(&mut s, relative_too).is_ok());
    }

    #[test]
    fn removing_the_fixture_group_leaves_the_phaser_without_a_group() {
        let mut s = example();
        let op = Op::RemoveFixtureGroup { id: "g4".into() };
        apply(&mut s, op).unwrap();
        assert!(phaser_mut(&mut s, "ph1").group.is_none());
        assert_eq!(phaser_mut(&mut s, "ph4").group.as_deref(), Some("g3"));
    }

    #[test]
    fn the_rgb_matrix_reaches_the_color_channels_of_each_head() {
        let mut s = example();
        for id in ["r1", "r2"] {
            let op = Op::SetFunctionRunning {
                id: id.into(),
                running: true,
            };
            apply(&mut s, op).unwrap();
        }
        let runs = running_programs(&s);
        let matrix = |id: &str| match &runs.iter().find(|r| r.id == id).unwrap().program {
            Program::RgbMatrix(m) => m.clone(),
            _ => panic!("não é RGB Matrix"),
        };
        let channels = |cell: &MatrixCell| match &cell.output {
            CellOutput::Rgb(l) => l.clone().map(|l| l.channel),
            _ => panic!("não é RGB"),
        };
        let fixture = |g: &str, x: u16, y: u16| {
            let cell =
                |group: &FixtureGroup| group.cells.iter().find(|c| (c.x, c.y) == (x, y)).cloned();
            let cell = s
                .project
                .fixture_groups
                .iter()
                .filter(|x| x.id == g)
                .find_map(cell)
                .unwrap();
            let f = s
                .project
                .fixtures
                .iter()
                .find(|f| f.id == cell.fixture)
                .unwrap();
            (f.address, cell.head)
        };
        // "Barras LED": 8 × 2 pixels de 3 canais, uma cabeça por célula
        let bars = matrix("r1");
        assert_eq!((bars.width, bars.height, bars.cells.len()), (8, 2, 16));
        for cell in &bars.cells {
            let (address, head) = fixture("g2", cell.x, cell.y);
            assert_eq!(channels(cell), [0, 1, 2].map(|k| address + 3 * head + k));
        }
        // ParLed 18x15w (dimmer, vermelho, verde, azul…): o RGB começa no segundo canal
        let pars = matrix("r2");
        assert_eq!((pars.width, pars.height, pars.cells.len()), (12, 1, 12));
        let (address, _) = fixture("g4", 0, 0);
        let first = pars.cells.iter().find(|c| (c.x, c.y) == (0, 0)).unwrap();
        assert_eq!(channels(first), [1, 2, 3].map(|k| address + k));
        // O dimmer: o canal do atributo vira o cinza
        let g4 = s
            .project
            .fixture_groups
            .iter()
            .find(|g| g.id == "g4")
            .unwrap();
        let grey = matrix_cells(&s.project, g4, RgbControl::Dimmer);
        assert!(matches!(&grey[0].output, CellOutput::Grey(l) if l.channel == address));
        // Sem o canal do atributo (a barra não tem UV), a cabeça não acende
        let g2 = s
            .project
            .fixture_groups
            .iter()
            .find(|g| g.id == "g2")
            .unwrap();
        assert!(matrix_cells(&s.project, g2, RgbControl::Uv).is_empty());
    }

    #[test]
    fn text_and_image_matrices_draw_and_are_checked() {
        use base64::Engine;
        let mut s = example();
        let Some(FunctionKind::RgbMatrix(m)) = s
            .project
            .functions
            .iter()
            .find(|f| f.id == "r1")
            .map(|f| f.kind.clone())
        else {
            panic!("r1 não é RGB Matrix")
        };
        let save = |s: &mut Snapshot, algorithm: RgbAlgorithm| {
            let mut function = s
                .project
                .functions
                .iter()
                .find(|f| f.id == "r1")
                .unwrap()
                .clone();
            function.kind = FunctionKind::RgbMatrix(RgbMatrix {
                algorithm,
                ..m.clone()
            });
            apply(s, Op::SaveFunction { function })
        };
        let preview = |s: &mut Snapshot, step: u32| {
            let (_, reply) = apply(
                s,
                Op::PreviewMatrix {
                    id: "r1".into(),
                    step,
                },
            )
            .unwrap();
            let map: Vec<Vec<u64>> = serde_json::from_value(reply["map"].clone()).unwrap();
            (reply["steps"].as_u64().unwrap(), map)
        };
        // Texto rolando: 2 letras de 6 pontos, 12 passos, numa grade de 8 × 2
        let text = RgbAlgorithm::Text {
            text: "OI".into(),
            style: ScrollStyle::Horizontal,
            x: 0,
            y: 0,
        };
        save(&mut s, text).unwrap();
        let (steps, map) = preview(&mut s, 0);
        assert_eq!((steps, map.len(), map[0].len()), (12, 2, 8));
        // Imagem de 2 × 2: vermelho, transparente / verde, azul; dá a volta na grade de 8 × 2
        let pixels = [255, 0, 0, 255, 0, 0, 0, 0, 0, 255, 0, 255, 0, 0, 255, 255];
        let image = |width, height, bytes: &[u8]| RgbImage {
            name: "teste".into(),
            width,
            height,
            pixels: base64::engine::general_purpose::STANDARD.encode(bytes),
        };
        let with = |style| RgbAlgorithm::Image {
            image: Some(image(2, 2, &pixels)),
            style,
            x: 0,
            y: 0,
        };
        save(&mut s, with(ScrollStyle::Static)).unwrap();
        let (steps, map) = preview(&mut s, 0);
        assert_eq!(steps, 1);
        assert_eq!((map[0][0], map[0][1], map[0][2]), (0xFF0000, 0, 0xFF0000));
        assert_eq!((map[1][0], map[1][1]), (0x00FF00, 0x0000FF));
        // Rolando para o lado: uma coluna por passo, e 2 passos (a largura da imagem)
        save(&mut s, with(ScrollStyle::Horizontal)).unwrap();
        let (steps, map) = preview(&mut s, 1);
        assert_eq!((steps, map[0][0], map[0][1]), (2, 0, 0xFF0000));
        // Recusados: texto grande demais, imagem com bytes que não batem, imagem maior que 128
        let long = RgbAlgorithm::Text {
            text: "A".repeat(257),
            style: ScrollStyle::Static,
            x: 0,
            y: 0,
        };
        assert!(save(&mut s, long).is_err());
        let short = RgbAlgorithm::Image {
            image: Some(image(2, 2, &pixels[..12])),
            style: ScrollStyle::Static,
            x: 0,
            y: 0,
        };
        assert!(save(&mut s, short).is_err());
        let wide = RgbAlgorithm::Image {
            image: Some(image(129, 1, &[0; 129 * 4])),
            style: ScrollStyle::Static,
            x: 0,
            y: 0,
        };
        assert!(save(&mut s, wide).is_err());
        // Sem imagem, a grade fica apagada
        let none = RgbAlgorithm::Image {
            image: None,
            style: ScrollStyle::Static,
            x: 0,
            y: 0,
        };
        save(&mut s, none).unwrap();
        assert!(preview(&mut s, 0).1.iter().flatten().all(|&c| c == 0));
    }

    #[test]
    fn the_example_text_and_image_matrices_light_the_panel() {
        let mut s = example();
        for id in ["r6", "r7"] {
            let mut lit = false;
            let steps = {
                let (_, reply) = apply(
                    &mut s,
                    Op::PreviewMatrix {
                        id: id.into(),
                        step: 0,
                    },
                )
                .unwrap();
                reply["steps"].as_u64().unwrap() as u32
            };
            for step in 0..steps {
                let (_, reply) = apply(
                    &mut s,
                    Op::PreviewMatrix {
                        id: id.into(),
                        step,
                    },
                )
                .unwrap();
                let map: Vec<Vec<u64>> = serde_json::from_value(reply["map"].clone()).unwrap();
                assert_eq!((map.len(), map[0].len()), (8, 8), "{id}");
                lit |= map.iter().flatten().any(|&c| c != 0);
            }
            assert!(lit, "{id} não acendeu o painel em {steps} passos");
        }
    }

    #[test]
    fn the_example_scripts_match_what_the_import_reads() {
        let scripts = example().project.scripts;
        assert!(!scripts.is_empty());
        for s in scripts {
            let meta = rlc_script::inspect(&s.source).unwrap();
            let read = (
                meta.name,
                meta.author,
                meta.api,
                meta.accepts,
                meta.properties,
            );
            assert_eq!(read, (s.name, s.author, s.api, s.accepts, s.properties));
        }
    }

    #[test]
    fn a_script_is_imported_checked_and_removed_only_when_unused() {
        use crate::script::TEST_SCRIPT;
        let mut s = example();
        let import = |s: &mut Snapshot, source: &str| {
            let op = Op::ImportScript {
                file_name: "teste.js".into(),
                source: source.into(),
            };
            apply(s, op)
        };
        let (topics, reply) = import(&mut s, TEST_SCRIPT).unwrap();
        assert_eq!(topics, vec![Topic::Scripts, Topic::Functions]);
        assert_eq!(reply["id"], "Teste");
        assert_eq!(
            reply["properties"][0]["kind"],
            json!({ "type": "range", "min": 1, "max": 5 })
        );
        // Importar de novo troca o script, sem duplicar; o que não é script é recusado
        let count = s.project.scripts.len();
        import(&mut s, TEST_SCRIPT).unwrap();
        assert_eq!(s.project.scripts.len(), count);
        assert!(import(&mut s, "1 +").unwrap_err().starts_with("teste.js: "));

        // Uma criação com o script e uma propriedade; valor fora da faixa, propriedade que o script
        // não tem e script que não existe são recusados
        let Some(FunctionKind::RgbMatrix(m)) = s
            .project
            .functions
            .iter()
            .find(|f| f.id == "r1")
            .map(|f| f.kind.clone())
        else {
            panic!("r1 não é RGB Matrix")
        };
        let with = |script: &str, name: &str, value: &str| RgbMatrix {
            algorithm: RgbAlgorithm::Script {
                script: script.into(),
                properties: vec![PropertyValue {
                    name: name.into(),
                    value: value.into(),
                }],
            },
            ..m.clone()
        };
        let save = |s: &mut Snapshot, m: RgbMatrix| {
            let mut function = s
                .project
                .functions
                .iter()
                .find(|f| f.id == "r1")
                .unwrap()
                .clone();
            function.kind = FunctionKind::RgbMatrix(m);
            apply(s, Op::SaveFunction { function })
        };
        assert!(save(&mut s, with("Teste", "level", "9")).is_err());
        assert!(save(&mut s, with("Teste", "nao-existe", "1")).is_err());
        assert!(save(&mut s, with("outro", "level", "1")).is_err());
        save(&mut s, with("Teste", "level", "3")).unwrap();

        // O motor recebe o texto e o valor da propriedade; o script usa as cores que aceita (2, o padrão)
        apply(
            &mut s,
            Op::SetFunctionRunning {
                id: "r1".into(),
                running: true,
            },
        )
        .unwrap();
        let runs = running_programs(&s);
        let Program::RgbMatrix(program) = &runs.iter().find(|r| r.id == "r1").unwrap().program
        else {
            panic!("não é RGB Matrix")
        };
        assert_eq!(program.accepts, 2);
        assert!(
            matches!(&program.source, MatrixSource::Script { properties, .. } if *properties == [("level".to_string(), "3".to_string())])
        );

        // Usado, o script não sai; solto, sai
        assert!(apply(&mut s, Op::RemoveScript { id: "Teste".into() }).is_err());
        save(
            &mut s,
            RgbMatrix {
                algorithm: RgbAlgorithm::Plain,
                ..m
            },
        )
        .unwrap();
        let (topics, _) = apply(&mut s, Op::RemoveScript { id: "Teste".into() }).unwrap();
        assert_eq!(topics, vec![Topic::Scripts]);
        assert!(s.project.scripts.iter().all(|s| s.id != "Teste"));
    }

    #[test]
    fn the_preview_of_a_matrix_is_its_color_map() {
        let mut s = example();
        let op = Op::PreviewMatrix {
            id: "r1".into(),
            step: 7,
        };
        let (topics, reply) = apply(&mut s, op).unwrap();
        // "Cor fixa nas barras": a grade de 8 × 2 inteira no roxo, num passo só
        assert!(topics.is_empty());
        assert_eq!(reply["steps"], 1);
        assert_eq!(
            reply["map"],
            json!([vec![0x8000ff_u32; 8], vec![0x8000ff_u32; 8]])
        );
        assert!(
            apply(
                &mut s,
                Op::PreviewMatrix {
                    id: "s1".into(),
                    step: 0
                }
            )
            .is_err()
        );
    }

    #[test]
    fn a_saved_rgb_matrix_needs_valid_colors_a_group_and_no_random_order() {
        let mut s = example();
        let Some(FunctionKind::RgbMatrix(m)) = s
            .project
            .functions
            .iter()
            .find(|f| f.id == "r1")
            .map(|f| f.kind.clone())
        else {
            panic!("r1 não é RGB Matrix")
        };
        let save = |s: &mut Snapshot, m: RgbMatrix| {
            let mut function = s
                .project
                .functions
                .iter()
                .find(|f| f.id == "r1")
                .unwrap()
                .clone();
            function.kind = FunctionKind::RgbMatrix(m);
            apply(s, Op::SaveFunction { function })
        };
        assert!(save(&mut s, m.clone()).is_ok());
        for bad in [
            RgbMatrix {
                colors: vec![],
                ..m.clone()
            },
            RgbMatrix {
                colors: vec!["vermelho".into()],
                ..m.clone()
            },
            RgbMatrix {
                colors: vec!["#000000".into(); 6],
                ..m.clone()
            },
            RgbMatrix {
                group: Some("nao-existe".into()),
                ..m.clone()
            },
            RgbMatrix {
                run_order: RunOrder::Random,
                ..m.clone()
            },
        ] {
            assert!(save(&mut s, bad).is_err());
        }
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
        assert_eq!(scene_of(&runs[0].program).times.fade_in, Span::Ms(1000));
        // MH 1 no 101: pan e tilt levam o fino junto, em 16 bits (sem o fino na cena, ele vale 0)
        let at = |runs: &[Run], channel: u16| {
            let levels = &scene_of(&runs[0].program).levels;
            let level = levels.iter().find(|l| l.channel == channel);
            level.map(|l| (l.fine, l.value))
        };
        assert_eq!(at(&runs, 101), Some((Some(102), 128 << 8)));
        assert_eq!(at(&runs, 103), Some((Some(104), 128 << 8)));
        assert_eq!(at(&runs, 106), Some((None, 255)));

        // Com o fino na cena, ele vai junto do grosso; só o fino (MH 2 no 115) muda só o fino. A cena
        // (do grupo dos MHs) passa a ter os valores por fixture, para o MH 2 ficar sem o tilt grosso
        let s4 = s.project.functions.iter_mut().find(|f| f.id == "s4");
        let Some(FunctionKind::Scene(scene)) = s4.map(|f| &mut f.kind) else {
            panic!("s4 não é cena");
        };
        let mut values = scene_values(&example().project, scene);
        let value = |fixture: &str, channel, value| SceneValue {
            fixture: fixture.into(),
            channel,
            value,
        };
        values.push(value("mh1", 1, 7));
        values.retain(|v| v.fixture != "mh2" || v.channel != 2);
        values.push(value("mh2", 3, 9));
        scene.groups.clear();
        scene.values = values;
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
                (scene.times.fade_in, scene.times.fade_out)
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
    fn scenes_go_with_delays_curve_priority_and_times_by_attribute() {
        let mut s = example();
        for id in ["s6", "s7", "s8", "s9"] {
            run(&mut s, id);
        }
        let runs = running_programs(&s);
        let run_of = |id: &str| runs.iter().find(|r| r.id == id).unwrap();
        let priorities = ["s6", "s7", "s8"].map(|id| run_of(id).priority);
        assert_eq!(
            priorities,
            [Priority::Low, Priority::High, Priority::Normal]
        );
        let t = scene_of(&run_of("s8").program).times;
        assert_eq!(
            (t.delay_in, t.fade_in, t.delay_out, t.fade_out, t.curve),
            (
                Span::Ms(1000),
                Span::Ms(3000),
                Span::Ms(2000),
                Span::Ms(3000),
                Curve::EaseInOut
            )
        );
        // "Cor no escuro" vira três partes: a com os tempos da cena (o strobo), a da intensidade (o
        // dimmer) e a da cor (as cores e a macro). O ParLed #01 vai do 63 ao 71
        let Program::Collection(parts) = &run_of("s9").program else {
            panic!("a cena não foi dividida");
        };
        let channels = |i: usize| -> Vec<u16> {
            let levels = &scene_of(&parts[i]).levels;
            let first = levels.iter().filter(|l| (63..=71).contains(&l.channel));
            first.map(|l| l.channel).collect()
        };
        assert_eq!(
            [channels(0), channels(1), channels(2)],
            [vec![70], vec![63], vec![64, 65, 66, 67, 68, 69, 71]]
        );
        let times = |i: usize| scene_of(&parts[i]).times;
        assert_eq!(
            (times(1).delay_in, times(1).fade_in, times(1).fade_out),
            (Span::Ms(2000), Span::Ms(3000), Span::Ms(3000))
        );
        assert_eq!(
            (times(2).fade_in, times(2).delay_out),
            (Span::Ms(0), Span::Ms(3000))
        );

        // O roteiro: o hold infinito não muda com a velocidade, e o passo usa os tempos da cena
        let mut rate = widget(&s, "sl3");
        rate.id = "rt".into();
        let rt = slider(&mut rate);
        (rt.mode, rt.functions) = (SliderMode::Rate, vec!["c2".into()]);
        save_widget(&mut s, rate);
        set_value(&mut s, "rt", vec![255]);
        run(&mut s, "c2");
        let roteiro = |s: &Snapshot| {
            let runs = running_programs(s);
            let c2 = runs.iter().find(|r| r.id == "c2").unwrap();
            chaser_of(&c2.program).steps[2].clone()
        };
        let step = roteiro(&s);
        assert_eq!(step.hold, Span::Ms(INFINITE));
        assert_eq!(step.times, Imposed::default());
        // Em "Comum", o passo impõe o delay, o fade e a curva comuns a todas as partes da cena
        chaser_mut(&mut s, "c2").fade_in_mode = SpeedMode::Common;
        let imposed = roteiro(&s).times;
        assert_eq!(
            (imposed.delay_in, imposed.fade_in, imposed.curve),
            (Some(Span::Ms(0)), Some(Span::Ms(0)), Some(Curve::Linear))
        );
    }

    #[test]
    fn chasers_go_with_their_steps_and_the_times_that_hold() {
        let mut s = example();
        run(&mut s, "c1");
        let times = |s: &Snapshot| {
            let runs = running_programs(s);
            let step = &chaser_of(&runs[0].program).steps[0];
            (step.times.fade_in, step.hold, step.times.fade_out)
        };
        // "Troca de cores": fade in e hold por passo (1 s e 2 s), fade out da cena
        let c1 = &running_programs(&s)[0].program;
        assert_eq!(chaser_of(c1).steps.len(), 3);
        assert_eq!(
            scene_of(&chaser_of(c1).steps[0].program).times.fade_out,
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
        assert_eq!(first.times.fade_in, Span::Ms(1000));
        assert_eq!(q1.steps[0].times.fade_in, Some(Span::Ms(800)));
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
        assert!(save(&mut s, |e| e.shape.width = 127).is_ok());
        let refused: [fn(&mut Efx); 12] = [
            |e| e.shape.width = 128,
            |e| e.shape.y_frequency = 33,
            |e| e.fixtures[0].start_offset = 360,
            // O desenho próprio do item também é conferido
            |e| {
                e.fixtures[0].shape = Some(EfxShape {
                    rotation: 360,
                    ..e.shape.clone()
                })
            },
            |e| e.shape.path = vec![(0, 0); 65],
            |e| e.shape.path = vec![(-128, 0)],
            |e| e.shape.speed_points = Some(vec![0; 16]),
            // Sobe e desce
            |e| {
                let points = (0..16).map(|k| if k == 3 || k == 15 { 255 } else { 0 });
                e.shape.speed_points = Some(points.collect())
            },
            |e| e.run_order = RunOrder::Random,
            |e| e.fixtures.push(e.fixtures[0].clone()),
            // O PAR não tem pan nem tilt
            |e| e.fixtures[0].target = EfxTarget::Fixture("par1".into()),
            |e| e.fixtures[0].target = EfxTarget::Group("inexistente".into()),
        ];
        for change in refused {
            assert!(save(&mut s, change).is_err());
        }
        assert_eq!(efx_mut(&mut s, "e1").shape.width, 127);
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

    fn scene_mut<'a>(s: &'a mut Snapshot, id: &str) -> &'a mut Scene {
        let f = s.project.functions.iter_mut().find(|f| f.id == id);
        match f.map(|f| &mut f.kind) {
            Some(FunctionKind::Scene(scene)) => scene,
            _ => panic!("{id} não é cena"),
        }
    }

    fn fixture_group(s: &Snapshot, id: &str) -> FixtureGroup {
        let group = s.project.fixture_groups.iter().find(|g| g.id == id);
        group.cloned().unwrap()
    }

    /// Valores (canal DMX, valor) da primeira criação rodando, uma cena.
    fn scene_levels(s: &Snapshot) -> Vec<(u16, u16)> {
        let runs = running_programs(s);
        let mut levels: Vec<(u16, u16)> = scene_of(&runs[0].program)
            .levels
            .iter()
            .map(|l| (l.channel, l.value))
            .collect();
        levels.sort();
        levels
    }

    #[test]
    fn a_group_scene_lights_who_is_in_the_group_now() {
        let mut s = example();
        run(&mut s, "s1");
        let level = |s: &Snapshot, channel| {
            let levels = scene_levels(s);
            levels.iter().find(|l| l.0 == channel).map(|l| l.1)
        };
        // "Frente âmbar" no grupo "PARs frente": dimmer, vermelho e verde nos seis PARs
        assert_eq!(scene_levels(&s).len(), 18);
        assert_eq!(
            (level(&s, 1), level(&s, 2), level(&s, 3)),
            (Some(255), Some(255), Some(140))
        );
        // O PAR 7 (só RGB, no 41) entra no grupo e acende, sem editar a cena; "RGB PARs" acompanha
        let mut g1 = fixture_group(&s, "g1");
        g1.width = 7;
        g1.cells.push(GroupCell {
            x: 6,
            y: 0,
            fixture: "par7".into(),
            head: 0,
        });
        let (topics, _) = apply(&mut s, Op::SaveFixtureGroup { group: g1 }).unwrap();
        assert!(topics.contains(&Topic::ChannelGroups));
        assert_eq!((level(&s, 41), level(&s, 42)), (Some(255), Some(140)));
        // A exceção vence o grupo: o PAR 2 (no 8) sem o vermelho
        scene_mut(&mut s, "s1").values.push(SceneValue {
            fixture: "par2".into(),
            channel: 1,
            value: 0,
        });
        assert_eq!((level(&s, 9), level(&s, 16)), (Some(0), Some(255)));
        // Com o PAR 1 em dois grupos da cena, vence o de depois
        let group = FixtureGroup {
            id: "g9".into(),
            name: "PAR 1".into(),
            width: 1,
            height: 1,
            cells: vec![GroupCell {
                x: 0,
                y: 0,
                fixture: "par1".into(),
                head: 0,
            }],
        };
        apply(&mut s, Op::SaveFixtureGroup { group }).unwrap();
        let red = |value| AttributeValue {
            attribute: Attribute::Red,
            index: 0,
            value,
        };
        scene_mut(&mut s, "s1").groups.push(SceneGroup {
            group: "g9".into(),
            values: vec![red(10)],
        });
        assert_eq!((level(&s, 2), level(&s, 16)), (Some(10), Some(255)));
        // Um grupo repetido ou que não existe é recusado
        for group in ["g9", "nada"] {
            let mut s1 = s.project.functions.iter().find(|f| f.id == "s1").cloned();
            if let Some(FunctionKind::Scene(scene)) = s1.as_mut().map(|f| &mut f.kind) {
                scene.groups.push(SceneGroup {
                    group: group.into(),
                    values: vec![],
                });
            }
            let op = Op::SaveFunction {
                function: s1.unwrap(),
            };
            assert!(apply(&mut s, op).is_err());
        }
    }

    #[test]
    fn each_channel_of_the_same_kind_gets_its_own_value() {
        let s = example();
        // Beam 7R: prisma e prisma (4 e 5), pan e pan fino (9 e 10)
        let value = |attribute, index, value| AttributeValue {
            attribute,
            index,
            value,
        };
        let scene = Scene {
            values: vec![],
            groups: vec![SceneGroup {
                group: "g5".into(),
                values: vec![
                    value(Attribute::Prism, 0, 50),
                    value(Attribute::Prism, 1, 200),
                    value(Attribute::Pan, 1, 7),
                ],
            }],
            channel_groups: vec![],
            delay_in_ms: 0,
            fade_in_ms: 0,
            delay_out_ms: 0,
            fade_out_ms: 0,
            curve: Curve::Linear,
            attribute_times: vec![],
        };
        let values = scene_values(&s.project, &scene);
        let of = |channel| {
            let v = values
                .iter()
                .find(|v| v.fixture == "rbeam1" && v.channel == channel);
            v.map(|v| v.value)
        };
        assert_eq!(values.len(), 12);
        assert_eq!(
            (of(4), of(5), of(9), of(10)),
            (Some(50), Some(200), None, Some(7))
        );
    }

    #[test]
    fn channel_groups_follow_their_fixture_group() {
        let mut s = example();
        let cg6 = |s: &Snapshot| {
            let group = s.project.channel_groups.iter().find(|g| g.id == "cg6");
            group.cloned().unwrap()
        };
        // "Dimmer ParLeds" acompanha o grupo "ParLeds": uma ParLed a menos, um canal a menos
        let mut g4 = fixture_group(&s, "g4");
        g4.cells.pop();
        let (topics, _) = apply(&mut s, Op::SaveFixtureGroup { group: g4 }).unwrap();
        assert!(topics.contains(&Topic::ChannelGroups));
        assert_eq!(cg6(&s).channels.len(), 11);
        // A Mesa simples segue
        let op = Op::SetDeskGroupLevel {
            group: "cg6".into(),
            value: 9,
        };
        apply(&mut s, op).unwrap();
        assert_eq!(s.desk.levels.len(), 11);
        // Com mais um atributo, o servidor refaz a lista
        let mut group = cg6(&s);
        group.attributes.push(Attribute::Red);
        group.channels.clear();
        apply(&mut s, Op::SaveChannelGroup { group }).unwrap();
        assert_eq!(cg6(&s).channels.len(), 22);
        // Um grupo de fixtures que não existe é recusado
        let mut group = cg6(&s);
        group.fixture_group = Some("nada".into());
        assert!(apply(&mut s, Op::SaveChannelGroup { group }).is_err());
    }

    #[test]
    fn efx_takes_its_groups_in_grid_order() {
        let mut s = example();
        run(&mut s, "e1");
        let efx = |s: &Snapshot| match running_programs(s).remove(0).program {
            Program::Efx(efx) => efx,
            _ => panic!("e1 não é EFX"),
        };
        // Pan de cada fixture do desenho, o "Inverter" e o início dela
        let pans = |e: &EfxProgram| -> Vec<(Option<u16>, bool, u16)> {
            let pan = e.heads.iter().map(|h| h[0].as_ref().map(|l| l.channel));
            let set = e.efx.fixtures.iter().map(|x| (x.reverse, x.start_offset));
            pan.zip(set)
                .map(|(pan, (r, start))| (pan, r, start))
                .collect()
        };
        // "Círculo nos MHs" no grupo "Moving heads": MH 1 a 4, com o pan no 101, 115, 129 e 143
        assert_eq!(efx(&s).heads.len(), 4);
        // O MH 4 vai para o começo da grade, o MH 1 para o fim, e o MH 2 sai do grupo
        let mut g3 = fixture_group(&s, "g3");
        g3.cells[0].x = 3;
        g3.cells[3].x = 0;
        g3.cells.remove(1);
        apply(&mut s, Op::SaveFixtureGroup { group: g3 }).unwrap();
        let e = efx(&s);
        assert_eq!(
            pans(&e),
            [
                (Some(143), false, 0),
                (Some(129), false, 0),
                (Some(101), false, 0)
            ]
        );
        // A defasagem do grupo: cada fixture começa 100° depois da anterior, a partir do início dele
        let e1 = efx_mut(&mut s, "e1");
        e1.fixtures[0].start_offset = 10;
        e1.fixtures[0].spread = 100;
        assert_eq!(
            pans(&efx(&s)),
            [
                (Some(143), false, 10),
                (Some(129), false, 110),
                (Some(101), false, 210)
            ]
        );
        // Uma fixture solta antes do grupo vem primeiro, sem repetir; o "Inverter" do grupo vale para
        // as fixtures dele
        let e1 = efx_mut(&mut s, "e1");
        e1.fixtures[0].reverse = true;
        e1.fixtures.insert(
            0,
            EfxFixture {
                target: EfxTarget::Fixture("mh1".into()),
                reverse: false,
                start_offset: 0,
                spread: 0,
                shape: None,
                invert_pan: false,
                invert_tilt: false,
                swap_axes: false,
            },
        );
        assert_eq!(
            pans(&efx(&s)),
            [
                (Some(101), false, 0),
                (Some(143), true, 10),
                (Some(129), true, 110)
            ]
        );
        // O item próprio de uma fixture do grupo, depois dele, é a exceção: ela fica no lugar do grupo,
        // com o "Inverter" e o início do item dela
        efx_mut(&mut s, "e1").fixtures.push(EfxFixture {
            target: EfxTarget::Fixture("mh4".into()),
            reverse: false,
            start_offset: 77,
            spread: 0,
            shape: None,
            invert_pan: false,
            invert_tilt: false,
            swap_axes: false,
        });
        assert_eq!(
            pans(&efx(&s)),
            [
                (Some(101), false, 0),
                (Some(143), false, 77),
                (Some(129), true, 110)
            ]
        );
        // O desenho próprio, o espelho e a troca dos eixos do grupo vão para as fixtures dele; a
        // exceção fica com os do item dela
        let e1 = efx_mut(&mut s, "e1");
        let own = EfxShape {
            algorithm: EfxAlgorithm::Eight,
            ..e1.shape.clone()
        };
        e1.fixtures[1].shape = Some(own.clone());
        (e1.fixtures[1].invert_pan, e1.fixtures[1].swap_axes) = (true, true);
        let looks = |e: &EfxProgram| -> Vec<(Option<EfxShape>, bool, bool)> {
            let items = e.efx.fixtures.iter();
            items
                .map(|x| (x.shape.clone(), x.invert_pan, x.swap_axes))
                .collect()
        };
        assert_eq!(
            looks(&efx(&s)),
            [
                (None, false, false),
                (None, false, false),
                (Some(own), true, true)
            ]
        );
        // A defasagem também tem o limite de 359°
        let mut e1 = s.project.functions.iter().find(|f| f.id == "e1").cloned();
        if let Some(FunctionKind::Efx(e)) = e1.as_mut().map(|f| &mut f.kind) {
            e.fixtures[1].spread = 360;
        }
        let op = Op::SaveFunction {
            function: e1.unwrap(),
        };
        assert!(apply(&mut s, op).is_err());
    }

    #[test]
    fn removing_a_fixture_group_freezes_it_where_it_was_used() {
        let mut s = example();
        run(&mut s, "s1");
        let scene = scene_levels(&s);
        // Um botão de blackout de fixtures no grupo dos MHs, e o EFX dele invertido
        let mut widgets = s.project.console.widgets.iter();
        let mut w = widgets
            .find(|w| matches!(w.kind, VcWidgetKind::Button(_)))
            .cloned()
            .unwrap();
        button(&mut w).action = ButtonAction::BlackoutFixtures;
        button(&mut w).groups = vec!["g3".into()];
        button(&mut w).fixtures = vec![];
        save_widget(&mut s, w.clone());
        efx_mut(&mut s, "e1").fixtures[0].reverse = true;
        efx_mut(&mut s, "e1").fixtures[0].spread = 30;
        // O MH 2 com item próprio (a exceção) mantém os ajustes dele
        efx_mut(&mut s, "e1").fixtures.push(EfxFixture {
            target: EfxTarget::Fixture("mh2".into()),
            reverse: false,
            start_offset: 5,
            spread: 0,
            shape: None,
            invert_pan: false,
            invert_tilt: false,
            swap_axes: false,
        });
        for id in ["g1", "g3"] {
            let op = Op::RemoveFixtureGroup { id: id.into() };
            apply(&mut s, op).unwrap();
        }
        // A cena fica com os valores nas fixtures, e a luz não muda
        assert!(scene_mut(&mut s, "s1").groups.is_empty());
        assert_eq!(scene_levels(&s), scene);
        // O EFX fica com as fixtures, com o ajuste e a defasagem do grupo no início de cada uma
        let targets: Vec<(EfxTarget, bool, u16)> = [
            ("mh1", true, 0),
            ("mh2", false, 5),
            ("mh3", true, 60),
            ("mh4", true, 90),
        ]
        .map(|(id, reverse, start)| (EfxTarget::Fixture(id.into()), reverse, start))
        .into();
        let efx = efx_mut(&mut s, "e1");
        let after: Vec<(EfxTarget, bool, u16)> = efx
            .fixtures
            .iter()
            .map(|x| (x.target.clone(), x.reverse, x.start_offset))
            .collect();
        assert_eq!(after, targets);
        // Os grupos de canais ficam com os canais que tinham, e o botão com as fixtures
        for id in ["cg1", "cg3", "cg4"] {
            let g = s
                .project
                .channel_groups
                .iter()
                .find(|g| g.id == id)
                .unwrap();
            assert!(g.fixture_group.is_none() && !g.channels.is_empty());
        }
        let VcWidgetKind::Button(b) = widget(&s, &w.id).kind else {
            panic!("não é botão")
        };
        assert!(b.groups.is_empty() && b.fixtures.len() == 4);
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
