//! Projetos em arquivo (A5): `.rlc` numa pasta só, salvos e abertos pelo nome, e a cópia de
//! recuperação, que guarda também o arquivo de onde o projeto veio.

use std::path::{Path, PathBuf};

use rlc_core::Project;
use serde::Deserialize;
use serde_json::json;

use crate::write_atomic;

const FORMAT: &str = "r-light-control/project";
const SCHEMA_VERSION: u32 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFile {
    format: String,
    schema_version: u32,
    /// Só na recuperação: o arquivo do projeto, se ele já foi salvo.
    origin: Option<String>,
    project: Project,
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
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Não foi possível ler {}: {e}", path.display()))?;
    let file: ProjectFile = serde_json::from_str(&text)
        .map_err(|e| format!("{} não é um projeto válido ({e}).", name(path)))?;
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
    Ok((file.project, file.origin))
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
}
