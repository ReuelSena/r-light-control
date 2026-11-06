//! Projetos em arquivo (A5): `.rlc` numa pasta só, salvos e abertos pelo nome, e a cópia de
//! recuperação, que guarda também o arquivo de onde o projeto veio.

use std::path::{Path, PathBuf};

use rlc_core::Project;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::write_atomic;

const FORMAT: &str = "r-light-control/project";
const SCHEMA_VERSION: u32 = 12;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFile {
    format: String,
    schema_version: u32,
    /// Só na recuperação: o arquivo do projeto, se ele já foi salvo.
    origin: Option<String>,
    project: Value,
}

/// Caminho do projeto na pasta. Recusa nomes que o Windows não aceita.
pub fn path(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    if name.is_empty()
        || name.starts_with('.')
        || name
            .chars()
            .any(|c| c.is_control() || r#"<>:"/\|?*"#.contains(c))
    {
        return Err(format!("Nome de projeto inválido: \"{name}\"."));
    }
    Ok(dir.join(format!("{name}.rlc")))
}

/// Nome do projeto pelo arquivo (sem a pasta e o `.rlc`).
pub fn name(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Projetos da pasta, do mais recente para o mais antigo.
pub fn list(dir: &Path) -> Vec<String> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "rlc"))
        .map(|e| (e.metadata().and_then(|m| m.modified()).ok(), e.path()))
        .collect();
    files.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    files.iter().map(|(_, path)| name(path)).collect()
}

/// Lê um projeto (ou a recuperação). Recusa o que não é projeto e o que veio de uma versão mais nova.
pub fn read(path: &Path) -> Result<(Project, Option<String>), String> {
    let invalid = |e: serde_json::Error| format!("{} não é um projeto válido ({e}).", name(path));
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Não foi possível ler {}: {e}", path.display()))?;
    let mut file: ProjectFile = serde_json::from_str(&text).map_err(invalid)?;
    if file.format != FORMAT {
        return Err(format!(
            "{} não é um projeto do R Light Control.",
            name(path)
        ));
    }
    if file.schema_version > SCHEMA_VERSION {
        return Err(format!(
            "{} foi salvo numa versão mais nova do app.",
            name(path)
        ));
    }
    migrate(&mut file.project, file.schema_version);
    let project = serde_json::from_value(file.project).map_err(invalid)?;
    Ok((project, file.origin))
}

/// Cadeia de migrações (A5): leva um projeto de uma versão antiga até a atual, uma versão por vez.
fn migrate(project: &mut Value, version: u32) {
    // 1 → 2 (F7): o fim do Single Shot virou opção dos chasers e das sequências; os antigos apagam
    if version < 2 {
        let functions = project.get_mut("functions").and_then(Value::as_array_mut);
        for f in functions
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
        {
            if matches!(
                f.get("type").and_then(Value::as_str),
                Some("chaser" | "sequence")
            ) {
                f.insert("keepLast".into(), false.into());
            }
        }
    }
    // 2 → 3 (F7): o elemento "Tempos" ganhou "Seguir o BPM", ligada também nos que já existiam
    if version < 3 {
        let widgets = project
            .pointer_mut("/console/widgets")
            .and_then(Value::as_array_mut);
        for w in widgets
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
        {
            if w.get("type").and_then(Value::as_str) == Some("speedDial") {
                w.insert("beats".into(), true.into());
            }
        }
    }
    // 3 → 4 (F8): o EFX ficou sem a ordem aleatória, e a largura e a altura vão até 127, como no QLC+
    if version < 4 {
        let functions = project.get_mut("functions").and_then(Value::as_array_mut);
        for f in functions
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
        {
            if f.get("type").and_then(Value::as_str) != Some("efx") {
                continue;
            }
            if f.get("runOrder").and_then(Value::as_str) == Some("random") {
                f.insert("runOrder".into(), "loop".into());
            }
            for key in ["width", "height"] {
                if f.get(key).and_then(Value::as_u64).is_some_and(|v| v > 127) {
                    f.insert(key.into(), 127.into());
                }
            }
        }
    }
    // 4 → 5 e 5 → 6: a grade do console ganhou o dobro de colunas (células pela metade), duas vezes; as
    // posições e os tamanhos dos elementos dobram a cada uma, e eles continuam do mesmo jeito na tela
    for _ in version.max(4)..6 {
        let widgets = project
            .pointer_mut("/console/widgets")
            .and_then(Value::as_array_mut);
        for layouts in widgets
            .into_iter()
            .flatten()
            .filter_map(|w| w.get_mut("layouts").and_then(Value::as_object_mut))
        {
            for rect in layouts.values_mut().filter_map(Value::as_object_mut) {
                for cells in rect.values_mut() {
                    if let Some(n) = cells.as_u64() {
                        *cells = (n * 2).into();
                    }
                }
            }
        }
    }
    // 6 → 7 (F10): o "flash" de antes, que sobe e desce com os fades, virou o Temp (o Flash novo
    // acende na hora); o slider ganhou a lista de criações do modo Velocidade
    if version < 7 {
        let widgets = project
            .pointer_mut("/console/widgets")
            .and_then(Value::as_array_mut);
        for w in widgets
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
        {
            match w.get("type").and_then(Value::as_str) {
                Some("button") if w.get("mode").and_then(Value::as_str) == Some("flash") => {
                    w.insert("mode".into(), "temp".into());
                }
                Some("slider") => {
                    w.entry("functions").or_insert(json!([]));
                }
                _ => {}
            }
        }
    }
    // 7 → 8 (F10b): a cena ganhou os grupos de fixtures, e o grupo de canais pode acompanhar um
    // grupo de fixtures. O que já existe continua por fixture (o EFX não muda: o item antigo é uma
    // fixture)
    if version < 8 {
        let functions = project.get_mut("functions").and_then(Value::as_array_mut);
        for f in functions
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
        {
            if f.get("type").and_then(Value::as_str) == Some("scene") {
                f.entry("groups").or_insert(json!([]));
            }
        }
        let groups = project
            .get_mut("channelGroups")
            .and_then(Value::as_array_mut);
        for g in groups
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
        {
            g.entry("fixtureGroup").or_insert(Value::Null);
            g.entry("attributes").or_insert(json!([]));
        }
    }
    // 8 → 9 (F10c): a prioridade Normal (a mistura de antes), os delays zerados e a curva linear nas
    // cenas e nos tempos dos chasers, e nenhum tipo de atributo com tempos próprios. Na cue list, o
    // "próximo" virou o Go (o principal)
    if version < 9 {
        let delays = |t: &mut serde_json::Map<String, Value>| {
            t.entry("delayInMs").or_insert(0.into());
            t.entry("delayOutMs").or_insert(0.into());
            t.entry("curve").or_insert("linear".into());
        };
        let functions = project.get_mut("functions").and_then(Value::as_array_mut);
        for f in functions
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
        {
            f.entry("priority").or_insert("normal".into());
            match f.get("type").and_then(Value::as_str) {
                Some("scene") => {
                    delays(f);
                    f.entry("attributeTimes").or_insert(json!([]));
                }
                Some("chaser" | "sequence") => {
                    if let Some(common) = f.get_mut("common").and_then(Value::as_object_mut) {
                        delays(common);
                    }
                    let steps = f.get_mut("steps").and_then(Value::as_array_mut);
                    for timing in steps
                        .into_iter()
                        .flatten()
                        .filter_map(|s| s.get_mut("timing").and_then(Value::as_object_mut))
                    {
                        delays(timing);
                    }
                }
                _ => {}
            }
        }
        let widgets = project
            .pointer_mut("/console/widgets")
            .and_then(Value::as_array_mut);
        for w in widgets.into_iter().flatten() {
            if w["type"] != "cueList" {
                continue;
            }
            let Some(inputs) = w.get_mut("inputs").and_then(Value::as_array_mut) else {
                continue;
            };
            let main = inputs.iter().any(|i| i["control"] == "main");
            inputs.retain(|i| !(main && i["control"] == "next"));
            for i in inputs.iter_mut().filter(|i| i["control"] == "next") {
                i["control"] = "main".into();
            }
        }
    }
    // 9 → 10 (F10d): o EFX sem desenho à mão e com a velocidade constante; os itens sem desenho
    // próprio, sem espelho e sem trocar os eixos
    if version < 10 {
        let functions = project.get_mut("functions").and_then(Value::as_array_mut);
        for f in functions
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
            .filter(|f| f.get("type").and_then(Value::as_str) == Some("efx"))
        {
            f.entry("path").or_insert(json!([]));
            f.entry("speed").or_insert("linear".into());
            f.entry("speedPoints").or_insert(Value::Null);
            let items = f.get_mut("fixtures").and_then(Value::as_array_mut);
            for x in items.into_iter().flatten().filter_map(Value::as_object_mut) {
                x.entry("shape").or_insert(Value::Null);
                for key in ["invertPan", "invertTilt", "swapAxes"] {
                    x.entry(key).or_insert(false.into());
                }
            }
        }
    }
    // 10 → 11 (F9): o algoritmo do RGB Matrix virou um tipo (os de amostra de antes abrem como Plain),
    // ele controla o RGB das cabeças e não tem a ordem Aleatória; o projeto guarda os scripts importados
    if version < 11 {
        if let Some(p) = project.as_object_mut() {
            p.entry("scripts").or_insert(json!([]));
        }
        let functions = project.get_mut("functions").and_then(Value::as_array_mut);
        for f in functions
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
            .filter(|f| f.get("type").and_then(Value::as_str) == Some("rgbMatrix"))
        {
            // O nome de um algoritmo de amostra de antes (um texto) abre como Plain
            if f.get("algorithm").is_some_and(Value::is_string) {
                f.insert("algorithm".into(), json!({ "type": "plain" }));
            }
            f.entry("control").or_insert("rgb".into());
            if f.get("runOrder").and_then(Value::as_str) == Some("random") {
                f.insert("runOrder".into(), "loop".into());
            }
        }
    }
    // 11 → 12 (F9b): o Phaser é um tipo novo de criação, e os projetos antigos não o têm: nada muda
}

/// Salva o projeto. A versão anterior do arquivo fica em `.rlc.bak`.
pub fn save(path: &Path, project: &Project) -> std::io::Result<()> {
    if path.exists() {
        std::fs::copy(path, path.with_extension("rlc.bak"))?;
    }
    let file = json!({ "format": FORMAT, "schemaVersion": SCHEMA_VERSION, "project": project });
    write_atomic(
        path,
        &serde_json::to_string_pretty(&file).unwrap_or_default(),
    )
}

/// Grava a cópia de recuperação, com o arquivo de onde o projeto veio.
pub fn save_recovery(path: &Path, project: &Project, origin: Option<&str>) {
    let file = json!({ "format": FORMAT, "schemaVersion": SCHEMA_VERSION, "origin": origin, "project": project });
    if let Err(e) = write_atomic(path, &file.to_string()) {
        tracing::warn!("não foi possível gravar {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_project_reads_back_identical_with_backup() {
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        let mut project = crate::hub::example_snapshot(None).project;
        let file = path(&dir, " Culto de domingo ").unwrap();
        save(&file, &project).unwrap();
        assert_eq!(read(&file).unwrap(), (project.clone(), None));
        // Salvar de novo guarda a versão anterior no .bak
        project.name = "Mudado".into();
        save(&file, &project).unwrap();
        assert_eq!(read(&file).unwrap().0.name, "Mudado");
        let backup = file.with_extension("rlc.bak");
        assert_eq!(read(&backup).unwrap().0.name, "Show de exemplo");
        assert_eq!(list(&dir), ["Culto de domingo"]);
        // Recuperação: com o arquivo de origem
        let recovery = dir.join("recuperacao.rlc");
        save_recovery(&recovery, &project, Some("C:\\x.rlc"));
        assert_eq!(read(&recovery).unwrap().1.as_deref(), Some("C:\\x.rlc"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refuses_bad_names_and_files() {
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        assert!(
            path(&dir, "a/b").is_err() && path(&dir, "  ").is_err() && path(&dir, ".x").is_err()
        );
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("ruim.rlc");
        std::fs::write(&bad, "{").unwrap();
        assert!(read(&bad).is_err());
        let newer = json!({ "format": FORMAT, "schemaVersion": 99, "project": crate::apply::empty_project() });
        std::fs::write(&bad, newer.to_string()).unwrap();
        assert!(read(&bad).unwrap_err().contains("mais nova"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// O console como uma versão antiga o gravava: a grade com `divisor` vezes menos colunas (2 na
    /// versão 5, 4 até a 4).
    fn shrink_console_grid(old: &mut Value, divisor: u64) {
        for w in old["console"]["widgets"].as_array_mut().unwrap() {
            for rect in w["layouts"].as_object_mut().unwrap().values_mut() {
                for cells in rect
                    .as_object_mut()
                    .into_iter()
                    .flat_map(|r| r.values_mut())
                {
                    *cells = json!(cells.as_u64().unwrap() / divisor);
                }
            }
        }
    }

    #[test]
    fn version_1_opens_migrated() {
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = crate::hub::example_snapshot(None).project;
        for f in &mut project.functions {
            if let rlc_core::FunctionKind::Chaser(c) = &mut f.kind {
                c.keep_last = false;
            }
        }
        // O arquivo como a versão 1 gravava: os chasers sem o "keepLast" (abrem apagando no fim) e o
        // elemento "Tempos" sem o "beats" (abre seguindo o BPM)
        let mut old = serde_json::to_value(&project).unwrap();
        for f in old["functions"].as_array_mut().unwrap() {
            f.as_object_mut().unwrap().remove("keepLast");
        }
        for w in old["console"]["widgets"].as_array_mut().unwrap() {
            w.as_object_mut().unwrap().remove("beats");
        }
        shrink_console_grid(&mut old, 4);
        let file = dir.join("antigo.rlc");
        let v1 = json!({ "format": FORMAT, "schemaVersion": 1, "project": old });
        std::fs::write(&file, v1.to_string()).unwrap();
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_3_opens_the_efx_within_the_limits() {
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = crate::hub::example_snapshot(None).project;
        // O "Círculo nos MHs" gravado com a ordem aleatória e o desenho maior que o curso
        let mut old = serde_json::to_value(&project).unwrap();
        let efx = old["functions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|f| f["id"] == "e1")
            .unwrap();
        efx["runOrder"] = json!("random");
        efx["width"] = json!(200);
        shrink_console_grid(&mut old, 4);
        let file = dir.join("antigo.rlc");
        let v3 = json!({ "format": FORMAT, "schemaVersion": 3, "project": old });
        std::fs::write(&file, v3.to_string()).unwrap();
        let e1 = project.functions.iter_mut().find(|f| f.id == "e1");
        if let Some(rlc_core::FunctionKind::Efx(efx)) = e1.map(|f| &mut f.kind) {
            efx.shape.width = 127;
        }
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_4_opens_with_the_console_grid_doubled_twice() {
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let project = crate::hub::example_snapshot(None).project;
        let mut old = serde_json::to_value(&project).unwrap();
        shrink_console_grid(&mut old, 4);
        let file = dir.join("antigo.rlc");
        let v4 = json!({ "format": FORMAT, "schemaVersion": 4, "project": old });
        std::fs::write(&file, v4.to_string()).unwrap();
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_6_opens_with_the_old_flash_as_temp_and_the_slider_list() {
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let project = crate::hub::example_snapshot(None).project;
        let mut old = serde_json::to_value(&project).unwrap();
        for w in old["console"]["widgets"].as_array_mut().unwrap() {
            if w["mode"] == "temp" {
                w["mode"] = "flash".into();
            }
            if let Some(slider) = w.as_object_mut().filter(|w| w["type"] == "slider") {
                slider.remove("functions");
            }
        }
        let file = dir.join("antigo.rlc");
        let v6 = json!({ "format": FORMAT, "schemaVersion": 6, "project": old });
        std::fs::write(&file, v6.to_string()).unwrap();
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_7_opens_with_the_scenes_and_channel_groups_by_fixture() {
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = crate::hub::example_snapshot(None).project;
        for f in &mut project.functions {
            if let rlc_core::FunctionKind::Scene(scene) = &mut f.kind {
                scene.groups.clear();
            }
        }
        for g in &mut project.channel_groups {
            g.fixture_group = None;
            g.attributes.clear();
        }
        // A versão 7 não tinha os grupos na cena nem no grupo de canais
        let mut old = serde_json::to_value(&project).unwrap();
        for f in old["functions"].as_array_mut().unwrap() {
            f.as_object_mut().unwrap().remove("groups");
        }
        for g in old["channelGroups"].as_array_mut().unwrap() {
            let g = g.as_object_mut().unwrap();
            g.remove("fixtureGroup");
            g.remove("attributes");
        }
        let file = dir.join("antigo.rlc");
        let v7 = json!({ "format": FORMAT, "schemaVersion": 7, "project": old });
        std::fs::write(&file, v7.to_string()).unwrap();
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_8_opens_with_normal_priority_linear_fades_and_the_go_in_the_cue_list() {
        use rlc_core::{Curve, FunctionKind, Priority};
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = crate::hub::example_snapshot(None).project;
        for f in &mut project.functions {
            f.priority = Priority::Normal;
            match &mut f.kind {
                FunctionKind::Scene(scene) => {
                    (scene.delay_in_ms, scene.delay_out_ms) = (0, 0);
                    scene.curve = Curve::Linear;
                    scene.attribute_times.clear();
                }
                FunctionKind::Chaser(c) | FunctionKind::Sequence(c) => {
                    let steps = c.steps.iter_mut().map(|s| &mut s.timing);
                    for t in std::iter::once(&mut c.common).chain(steps) {
                        (t.delay_in_ms, t.delay_out_ms) = (0, 0);
                        t.curve = Curve::Linear;
                    }
                }
                _ => {}
            }
        }
        // A versão 8 não tinha nada disso, e o Go da cue list era o "próximo"
        fn strip(v: &mut Value) {
            match v {
                Value::Object(o) => {
                    for key in [
                        "priority",
                        "delayInMs",
                        "delayOutMs",
                        "curve",
                        "attributeTimes",
                    ] {
                        o.remove(key);
                    }
                    o.values_mut().for_each(strip);
                }
                Value::Array(a) => a.iter_mut().for_each(strip),
                _ => {}
            }
        }
        let mut old = serde_json::to_value(&project).unwrap();
        strip(&mut old["functions"]);
        let cue_list = old["console"]["widgets"].as_array_mut().unwrap();
        let cl1 = cue_list.iter_mut().find(|w| w["id"] == "cl1").unwrap();
        for input in cl1["inputs"].as_array_mut().unwrap() {
            if input["control"] == "main" {
                input["control"] = "next".into();
            }
        }
        let file = dir.join("antigo.rlc");
        let v8 = json!({ "format": FORMAT, "schemaVersion": 8, "project": old });
        std::fs::write(&file, v8.to_string()).unwrap();
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_9_opens_the_efx_with_the_general_shape_and_constant_speed() {
        use rlc_core::{Curve, FunctionKind};
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = crate::hub::example_snapshot(None).project;
        for f in &mut project.functions {
            if let FunctionKind::Efx(efx) = &mut f.kind {
                if efx.shape.algorithm == rlc_core::EfxAlgorithm::Freehand {
                    efx.shape.algorithm = rlc_core::EfxAlgorithm::Circle;
                }
                (efx.shape.path, efx.shape.speed, efx.shape.speed_points) =
                    (vec![], Curve::Linear, None);
                for x in &mut efx.fixtures {
                    x.shape = None;
                    (x.invert_pan, x.invert_tilt, x.swap_axes) = (false, false, false);
                }
            }
        }
        // A versão 9 não tinha nada disso
        let mut old = serde_json::to_value(&project).unwrap();
        for f in old["functions"].as_array_mut().unwrap() {
            if f["type"] != "efx" {
                continue;
            }
            let f = f.as_object_mut().unwrap();
            for key in ["path", "speed", "speedPoints"] {
                f.remove(key);
            }
            for x in f["fixtures"].as_array_mut().unwrap() {
                let x = x.as_object_mut().unwrap();
                for key in ["shape", "invertPan", "invertTilt", "swapAxes"] {
                    x.remove(key);
                }
            }
        }
        let file = dir.join("antigo.rlc");
        let v9 = json!({ "format": FORMAT, "schemaVersion": 9, "project": old });
        std::fs::write(&file, v9.to_string()).unwrap();
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_10_opens_the_rgb_matrix_as_plain_in_rgb_without_the_random_order() {
        use rlc_core::{FunctionKind, RunOrder};
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let project = crate::hub::example_snapshot(None).project;
        // A versão 10 tinha o nome de um algoritmo de amostra, e a ordem Aleatória
        let mut old = serde_json::to_value(&project).unwrap();
        let mut matrices = 0;
        for f in old["functions"].as_array_mut().unwrap() {
            if f["type"] == "rgbMatrix" {
                f["algorithm"] = "chase".into();
                f["runOrder"] = "random".into();
                f.as_object_mut().unwrap().remove("control");
                matrices += 1;
            }
        }
        assert!(matrices > 0);
        let file = dir.join("antigo.rlc");
        let v10 = json!({ "format": FORMAT, "schemaVersion": 10, "project": old });
        std::fs::write(&file, v10.to_string()).unwrap();
        let opened = read(&file).unwrap().0;
        for f in &opened.functions {
            if let FunctionKind::RgbMatrix(m) = &f.kind {
                assert_eq!(m.algorithm, rlc_core::RgbAlgorithm::Plain);
                assert_eq!(m.control, rlc_core::RgbControl::Rgb);
                assert_eq!(m.run_order, RunOrder::Loop);
            }
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_11_opens_as_it_was_since_the_phaser_is_a_new_type() {
        use rlc_core::{FunctionKind, FunctionType};
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = crate::hub::example_snapshot(None).project;
        project
            .functions
            .retain(|f| !matches!(f.kind, FunctionKind::Phaser(_)));
        project.folders.retain(|f| f.kind != FunctionType::Phaser);
        let file = dir.join("antigo.rlc");
        let v11 = json!({ "format": FORMAT, "schemaVersion": 11, "project": project });
        std::fs::write(&file, v11.to_string()).unwrap();
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_5_opens_with_the_console_grid_doubled_once() {
        let dir = std::env::temp_dir().join(format!("rlc-projetos-{}", crate::apply::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let project = crate::hub::example_snapshot(None).project;
        let mut old = serde_json::to_value(&project).unwrap();
        shrink_console_grid(&mut old, 2);
        let file = dir.join("antigo.rlc");
        let v5 = json!({ "format": FORMAT, "schemaVersion": 5, "project": old });
        std::fs::write(&file, v5.to_string()).unwrap();
        assert_eq!(read(&file).unwrap().0, project);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
