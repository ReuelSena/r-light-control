//! Projetos em arquivo (A5): `.rlc` numa pasta só, salvos e abertos pelo nome, e a cópia de
//! recuperação, que guarda também o arquivo de onde o projeto veio.

use std::path::{Path, PathBuf};

use rlc_core::Project;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::write_atomic;

const FORMAT: &str = "r-light-control/project";
const SCHEMA_VERSION: u32 = 6;

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
    for _ in version.max(4)..SCHEMA_VERSION {
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
        let project = crate::hub::example_snapshot(None).project;
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
        for f in &mut project.functions {
            if let rlc_core::FunctionKind::Efx(efx) = &mut f.kind {
                efx.width = 127;
            }
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
