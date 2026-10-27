//! A prévia dos RGB Matrix de script: cada criação tem uma sessão do script só para a tela, que anda em
//! ordem como o motor (os scripts guardam estado). A resposta é esperada aqui, mas com prazo: um script
//! que falha ou demora responde com o erro, e a sessão só recomeça quando o script ou as propriedades
//! mudam.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rlc_core::{Id, MatrixPreview, Project};
use rlc_engine::{MatrixProgram, MatrixSource};
use rlc_script::{Reply, Session};

/// Quanto a prévia espera por uma resposta do script (o limite de cada chamada dele é menor).
const WAIT: Duration = Duration::from_millis(600);

#[derive(Default)]
pub struct Previews(HashMap<Id, Entry>);

struct Entry {
    /// O que a sessão abriu: o texto, as propriedades e a grade.
    key: (Arc<str>, Vec<(String, String)>, u16, u16),
    session: Session,
    steps: Option<usize>,
    failure: Option<String>,
}

impl Previews {
    /// O mapa do passo `step` (entre os passos do script) da criação `id`. `matrix` é o RGB Matrix com o
    /// script dele.
    pub fn preview(
        &mut self,
        project: &Project,
        id: &Id,
        matrix: &MatrixProgram,
        step: u32,
    ) -> Result<MatrixPreview, String> {
        // As sessões de criações que saíram do projeto acabam
        self.0
            .retain(|id, _| project.functions.iter().any(|f| f.id == *id));
        let MatrixSource::Script { source, properties } = &matrix.source else {
            return Err("Não é um script.".into());
        };
        let key = (
            source.clone(),
            properties.clone(),
            matrix.width,
            matrix.height,
        );
        if self.0.get(id).is_none_or(|e| e.key != key) {
            let session = Session::open(source.clone(), properties.clone(), key.2, key.3);
            let entry = Entry {
                key,
                session,
                steps: None,
                failure: None,
            };
            self.0.insert(id.clone(), entry);
        }
        let entry = self.0.get_mut(id).expect("a sessão acabou de entrar");
        if let Some(message) = &entry.failure {
            return Err(message.clone());
        }
        let result = entry.ask(matrix, step);
        if let Err(message) = &result {
            entry.failure = Some(message.clone());
        }
        result
    }
}

impl Entry {
    fn ask(&mut self, matrix: &MatrixProgram, step: u32) -> Result<MatrixPreview, String> {
        let steps = match self.steps {
            Some(steps) => steps,
            None => match self.wait()? {
                Reply::Opened { steps } => *self.steps.insert(steps.max(1)),
                _ => return Err("O script respondeu fora de ordem.".into()),
            },
        };
        let step = step as usize % steps;
        let pack = |[r, g, b]: [u8; 3]| u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b);
        let colors = matrix.colors.iter().copied().map(pack).collect();
        self.session
            .request(step, pack(matrix.color(step, steps)), colors);
        match self.wait()? {
            Reply::Map { map, .. } => Ok(MatrixPreview {
                steps: steps as u32,
                map,
            }),
            _ => Err("O script respondeu fora de ordem.".into()),
        }
    }

    /// Espera a próxima resposta; um erro do script ou a demora viram o erro da prévia.
    fn wait(&self) -> Result<Reply, String> {
        let until = Instant::now() + WAIT;
        loop {
            match self.session.poll() {
                Some(Reply::Failed(message)) => return Err(message),
                Some(reply) => return Ok(reply),
                None if Instant::now() >= until => return Err("O script demorou demais.".into()),
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        }
    }
}

/// Um script de 3 passos, original, para os testes: o passo `n` pinta a grade de verde
/// `(n + 1) × 20 × nível`, e o nível é uma propriedade de 1 a 5.
#[cfg(test)]
pub(crate) const TEST_SCRIPT: &str = r#"
(function () {
  var algo = new Object;
  algo.apiVersion = 2;
  algo.name = "Teste";
  algo.author = "teste";
  var level = 1;
  algo.properties = ["name:level|type:range|display:Nível|values:1,5|write:setLevel|read:getLevel"];
  algo.setLevel = function (v) { level = parseInt(v); };
  algo.getLevel = function () { return level; };
  algo.rgbMap = function (width, height, rgb, step) {
    var map = new Array(height);
    for (var y = 0; y < height; y++) {
      map[y] = new Array(width);
      for (var x = 0; x < width; x++) map[y][x] = ((step + 1) * 20 * level) << 8;
    }
    return map;
  };
  algo.rgbMapStepCount = function (width, height) { return 3; };
  return algo;
})()
"#;
