//! Biblioteca de fixtures do usuário gravada no PC (E3): um arquivo por definição, com o nome do id
//! dela, em `{format, schemaVersion, fixture}` (A5).

use std::path::Path;

use rlc_core::FixtureDef;
use serde::Deserialize;
use serde_json::json;

const FORMAT: &str = "r-light-control/fixture";
const SCHEMA_VERSION: u32 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureFile {
    format: String,
    schema_version: u32,
    fixture: FixtureDef,
}

/// Lê a biblioteca. Na primeira vez (sem a pasta), ela começa com `seed` (as genéricas do exemplo).
/// Um arquivo que não dá para ler fica de fora, com um aviso no log.
pub fn load(dir: &Path, seed: Vec<FixtureDef>) -> Vec<FixtureDef> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        for def in &seed {
            save(dir, def);
        }
        return seed;
    };
    let mut paths: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    let mut library = Vec::new();
    for path in paths {
        let file = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| serde_json::from_str::<FixtureFile>(&text).map_err(|e| e.to_string()));
        match file {
            Ok(file) if file.format == FORMAT && file.schema_version <= SCHEMA_VERSION => {
                library.push(file.fixture)
            }
            Ok(file) => tracing::warn!(
                "{}: formato {} versão {} desconhecido",
                path.display(),
                file.format,
                file.schema_version
            ),
            Err(e) => tracing::warn!("{} ignorado: {e}", path.display()),
        }
    }
    library
}

/// Grava a definição, com escrita atômica: um arquivo temporário que depois toma o lugar do antigo.
pub fn save(dir: &Path, def: &FixtureDef) {
    let path = dir.join(format!("{}.json", def.id));
    let temp = path.with_extension("json.tmp");
    let text = json!({ "format": FORMAT, "schemaVersion": SCHEMA_VERSION, "fixture": def });
    let saved = std::fs::create_dir_all(dir)
        .and_then(|_| {
            std::fs::write(
                &temp,
                serde_json::to_string_pretty(&text).unwrap_or_default(),
            )
        })
        .and_then(|_| std::fs::rename(&temp, &path));
    if let Err(e) = saved {
        tracing::warn!("não foi possível gravar {}: {e}", path.display());
    }
}

pub fn remove(dir: &Path, id: &str) {
    let path = dir.join(format!("{id}.json"));
    if let Err(e) = std::fs::remove_file(&path) {
        tracing::warn!("não foi possível apagar {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_is_seeded_saved_and_read_back() {
        let dir = std::env::temp_dir().join(format!("rlc-library-{}", crate::apply::new_id()));
        let seed = crate::hub::example_snapshot(None).library;
        assert_eq!(load(&dir, seed.clone()), seed);
        // Na segunda vez, vem dos arquivos (em ordem de id)
        let mut sorted = seed.clone();
        sorted.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(load(&dir, vec![]), sorted);
        remove(&dir, &seed[0].id);
        std::fs::write(dir.join("estragado.json"), "{").unwrap();
        assert_eq!(load(&dir, vec![]).len(), seed.len() - 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
