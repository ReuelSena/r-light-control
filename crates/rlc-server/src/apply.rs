//! Operações do contrato (A2): cada uma é validada, aplicada no estado e devolve os pedaços do
//! estado que mudaram, que viram os avisos `evt` (A3). As regras vêm do motor de mentira da F2.
//! Uma operação recusada não muda nada.

use rlc_core::*;
use serde_json::{Value, json};

/// Pedaço do estado levado por um aviso de mudança: o caminho dele no `Snapshot`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Topic {
    Project,
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
    Library,
    Interfaces,
    Remote,
    Running,
    Desk,
    Master,
    WidgetValues,
    Stats,
    TestPattern,
}

impl Topic {
    pub fn path(self) -> &'static str {
        match self {
            Topic::Project => "project",
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
            Topic::Library => "library",
            Topic::Interfaces => "interfaces",
            Topic::Remote => "remote",
            Topic::Running => "running",
            Topic::Desk => "desk",
            Topic::Master => "master",
            Topic::WidgetValues => "widgetValues",
            Topic::Stats => "stats",
            Topic::TestPattern => "testPattern",
        }
    }

    /// Faz parte do projeto (e deixa alterações não salvas).
    pub fn is_project(self) -> bool {
        self.path().starts_with("project") && self != Topic::ProjectPath
    }

    pub fn data(self, s: &Snapshot) -> Value {
        let p = &s.project;
        match self {
            Topic::Project => json!(p),
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
            Topic::Library => json!(s.library),
            Topic::Interfaces => json!(s.interfaces),
            Topic::Remote => json!(s.remote),
            Topic::Running => json!(s.running),
            Topic::Desk => json!(s.desk),
            Topic::Master => json!(s.master),
            Topic::WidgetValues => json!(s.widget_values),
            Topic::Stats => json!(s.stats),
            Topic::TestPattern => json!(s.test_pattern),
        }
    }
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
            vec![
                T::Project,
                T::ProjectPath,
                T::Dirty,
                T::Running,
                T::WidgetValues,
                T::Desk,
            ]
        }
        Op::OpenProject { .. } | Op::SaveProject { .. } => {
            return Err("Abrir e salvar projetos chegam na F5.".into());
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
            vec![
                T::Fixtures,
                T::FixtureGroups,
                T::ChannelGroups,
                T::Functions,
                T::Console,
                T::Stage,
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
            vec![T::Functions, T::Console, T::Running]
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
                topics.extend([T::Desk, T::TestPattern]);
            }
            topics
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
            let in_project = p.fixture_defs.iter().any(|d| d.id == def.id);
            upsert(&mut s.library, def.clone(), |d| &d.id)?;
            if !in_project {
                return Ok((vec![T::Library], Value::Null));
            }
            upsert(&mut p.fixture_defs, def, |d| &d.id)?;
            vec![T::Library, T::FixtureDefs]
        }
        Op::ImportQxf { file_name, .. } => {
            // A conversão de verdade é da F4; até lá o resultado é simulado, como no protótipo
            let mut def = s
                .library
                .first()
                .cloned()
                .ok_or("A biblioteca está vazia.")?;
            def.id = format!("importado-{}", new_id());
            def.manufacturer = "Importado do QLC+".into();
            def.model = file_name.trim_end_matches(".qxf").to_string();
            let report = ImportReport {
                def_id: def.id.clone(),
                channels: def.channels.len() as u16,
                modes: def.modes.len() as u16,
                warnings: vec![
                    "Capability \"Strobe aleatório\" sem equivalente: guardada como texto.".into(),
                    "Canal \"Função especial\" sem grupo conhecido: marcado como \"outro\".".into(),
                ],
            };
            s.library.push(def);
            return Ok((vec![T::Library], json!(report)));
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

/// Canais de intensidade de cada universo, na ordem do projeto: o Grand Master e o blackout só
/// agem neles. Canal sem fixture também conta, para o blackout apagar a luz ligada só pela Mesa simples.
pub fn intensity_patch(p: &Project) -> Vec<(Id, Box<[bool; 512]>)> {
    p.universes
        .iter()
        .map(|u| {
            let mut intensity = Box::new([true; 512]);
            for f in p.fixtures.iter().filter(|f| f.universe == u.id) {
                for (i, channel) in mode_channels(p, f).enumerate() {
                    let index = (usize::from(f.address) + i).checked_sub(1);
                    if let Some(slot) = index.and_then(|c| intensity.get_mut(c)) {
                        *slot = is_intensity(channel.attribute);
                    }
                }
            }
            (u.id.clone(), intensity)
        })
        .collect()
}

fn is_intensity(attribute: Attribute) -> bool {
    use Attribute::*;
    matches!(
        attribute,
        Dimmer | Red | Green | Blue | White | Amber | Uv | Cyan | Magenta | Yellow
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

/// Endereços (universo, canal DMX) dos canais de um grupo de canais.
fn group_addresses(p: &Project, group: &str) -> Vec<(Id, u16)> {
    let Some(group) = p.channel_groups.iter().find(|g| g.id == group) else {
        return vec![];
    };
    group
        .channels
        .iter()
        .filter_map(|c| {
            let f = p.fixtures.iter().find(|f| f.id == c.fixture)?;
            let channel = f.address + c.channel;
            (channel <= 512).then(|| (f.universe.clone(), channel))
        })
        .collect()
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
            Op::SaveProject { path: None },
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
    fn intensity_patch_marks_dimmer_and_colors_but_not_position() {
        let s = example();
        let patch = intensity_patch(&s.project);
        let (id, palco) = &patch[0];
        assert_eq!(id, "u1");
        // Moving head no 101: o primeiro canal dele é pan
        let mh = s.project.fixtures.iter().find(|f| f.id == "mh1").unwrap();
        let first = mode_channels(&s.project, mh).next().unwrap();
        assert_eq!(palco[100], is_intensity(first.attribute));
        assert!(!palco.iter().all(|i| *i));
        // Canal sem fixture conta como intensidade
        assert!(palco[511]);
    }
}
