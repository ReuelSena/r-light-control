//! Executa os scripts de RGB Matrix do QLC+ (JavaScript ES5, API 1 a 3) no QuickJS.
//!
//! O QLC+ roda os scripts num interpretador só, sem limite nenhum: um laço infinito trava o app.
//! Aqui cada RGB Matrix rodando tem o seu interpretador numa thread própria (os scripts guardam
//! estado entre as chamadas), e cada chamada ao script tem um prazo e um teto de memória: o que
//! passa disso vira um erro da sessão, sem derrubar o motor.

use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use rlc_core::{PropertyKind, ScriptProperty};
use rquickjs::{
    Array, CatchResultExt, CaughtError, Context, Ctx, Exception, Function, Object, Runtime, Value,
    context::EvalOptions, convert::Coerced,
};

/// Limites do QLC+ não existem; aqui: tempo de cada chamada ao script e memória do interpretador.
pub const CALL_LIMIT: Duration = Duration::from_millis(250);
pub const MEMORY_LIMIT: usize = 32 << 20;

/// Pilha do QuickJS. A thread tem o dobro, para sobrar folga às funções nativas.
const STACK_LIMIT: usize = 512 << 10;

/// Metadados de um script, lidos ao importá-lo.
#[derive(Debug)]
pub struct Meta {
    pub name: String,
    pub author: String,
    pub api: u8,
    pub accepts: u8,
    pub properties: Vec<ScriptProperty>,
}

/// O que o script respondeu.
#[derive(Debug)]
pub enum Reply {
    /// O script abriu: o `rgbMapStepCount(width, height)` (chamado uma vez, depois de aplicar as
    /// propriedades).
    Opened { steps: usize },
    /// O mapa de um passo: `map[y][x]` em 0xRRGGBB.
    Map { step: usize, map: Vec<Vec<u32>> },
    /// Erro ou limite estourado: a sessão acabou e nenhuma resposta mais virá.
    Failed(String),
}

/// Prazo da chamada em curso, que o interpretador consulta de tempos em tempos.
struct Limits {
    deadline: Rc<Cell<Instant>>,
    expired: Rc<Cell<bool>>,
}

impl Limits {
    /// Uma chamada ao script: rearma o prazo e traduz a exceção, o tempo e a memória esgotados em
    /// texto.
    fn call<'js, T>(
        &self,
        ctx: &Ctx<'js>,
        f: impl FnOnce() -> rquickjs::Result<T>,
    ) -> Result<T, String> {
        self.deadline.set(Instant::now() + CALL_LIMIT);
        self.expired.set(false);
        f().catch(ctx).map_err(|error| {
            if self.expired.get() {
                return format!(
                    "tempo esgotado: o script passou de {} ms numa chamada (laço infinito?)",
                    CALL_LIMIT.as_millis()
                );
            }
            let thrown = match error {
                CaughtError::Exception(exception) => exception.into_value(),
                CaughtError::Value(value) => value,
                CaughtError::Error(error) => return error.to_string(),
            };
            // `Error.prototype.toString`: "ReferenceError: x is not defined"
            let text = thrown
                .get::<Coerced<String>>()
                .map(|text| text.0)
                .unwrap_or_default();
            if text.contains("out of memory") {
                format!(
                    "memória esgotada: o script passou de {} MiB",
                    MEMORY_LIMIT >> 20
                )
            } else {
                text
            }
        })
    }
}

/// Cria um interpretador novo, com os limites, e roda `f` nele.
fn interpret<R>(
    f: impl for<'js> FnOnce(&Limits, &Ctx<'js>) -> Result<R, String>,
) -> Result<R, String> {
    let unavailable = |error: rquickjs::Error| format!("interpretador indisponível: {error}");
    let runtime = Runtime::new().map_err(unavailable)?;
    runtime.set_memory_limit(MEMORY_LIMIT);
    runtime.set_max_stack_size(STACK_LIMIT);
    let limits = Limits {
        deadline: Rc::new(Cell::new(Instant::now() + CALL_LIMIT)),
        expired: Rc::new(Cell::new(false)),
    };
    let (deadline, expired) = (limits.deadline.clone(), limits.expired.clone());
    // Devolver `true` aborta o script com uma exceção que ele não consegue capturar
    runtime.set_interrupt_handler(Some(Box::new(move || {
        let over = Instant::now() >= deadline.get();
        if over {
            expired.set(true);
        }
        over
    })));
    // O QJSEngine do QLC+ também só tem os objetos padrão do ECMAScript: sem `console` nem `print`
    let context = Context::full(&runtime).map_err(unavailable)?;
    context.with(|ctx| f(&limits, &ctx))
}

/// Uma thread com pilha de sobra: o limite de pilha do QuickJS conta a partir de onde o
/// interpretador nasce, e quem chama pode estar com a pilha quase cheia.
fn spawn<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<thread::JoinHandle<T>, String> {
    thread::Builder::new()
        .name("rlc-script".into())
        .stack_size(2 * STACK_LIMIT)
        .spawn(f)
        .map_err(|error| format!("não foi possível iniciar a thread do interpretador: {error}"))
}

/// O script avaliado, com as funções que o QLC+ exige de cada versão da API.
struct Script<'js> {
    algo: Object<'js>,
    api: u8,
    rgb_map: Function<'js>,
    step_count: Function<'js>,
    /// Só no API 3.
    set_colors: Option<Function<'js>>,
    /// Só no API 2 em diante.
    properties: Vec<ScriptProperty>,
}

/// Avalia o texto do script (o valor da última expressão é o objeto do algoritmo) e confere o que
/// o QLC+ confere em `RGBScript::evaluate`.
fn load<'js>(limits: &Limits, ctx: &Ctx<'js>, source: &str) -> Result<Script<'js>, String> {
    // Como no QLC+, os scripts rodam sem o modo estrito: vários dependem disso
    let mut options = EvalOptions::default();
    options.strict = false;
    let value: Value = limits.call(ctx, || ctx.eval_with_options(source, options))?;
    let algo = value
        .into_object()
        .ok_or("o script não devolveu o objeto do algoritmo")?;

    let api: Coerced<i32> = limits.call(ctx, || algo.get("apiVersion"))?;
    let api = u8::try_from(api.0)
        .ok()
        .filter(|api| (1..=3).contains(api))
        .ok_or("apiVersion inválida")?;

    let function = |name: &str| -> Result<Function<'js>, String> {
        let value: Value = limits.call(ctx, || algo.get(name))?;
        value
            .into_function()
            .ok_or_else(|| format!("o script não tem a função {name}()"))
    };
    let rgb_map = function("rgbMap")?;
    let step_count = function("rgbMapStepCount")?;
    let set_colors = if api >= 3 {
        Some(function("rgbMapSetColors")?)
    } else {
        None
    };
    let properties = if api >= 2 {
        read_properties(limits, ctx, &algo)?
    } else {
        Vec::new()
    };
    Ok(Script {
        algo,
        api,
        rgb_map,
        step_count,
        set_colors,
        properties,
    })
}

/// As propriedades de `algo.properties`, como o `RGBScript::loadProperties` do QLC+. Cada uma é um
/// texto `name:x|type:list|display:..|values:a,b|write:fn|read:fn`; o `values` depende do `type`
/// já ter vindo antes, e só vale o que fica entre o primeiro e o segundo ":" de cada campo.
fn read_properties<'js>(
    limits: &Limits,
    ctx: &Ctx<'js>,
    algo: &Object<'js>,
) -> Result<Vec<ScriptProperty>, String> {
    let list: Value = limits.call(ctx, || algo.get("properties"))?;
    let list = list.into_array().ok_or("properties não é uma lista")?;
    let entries: Vec<String> = limits.call(ctx, || {
        list.iter::<Coerced<String>>()
            .map(|entry| entry.map(|entry| entry.0))
            .collect()
    })?;

    let mut properties = Vec::new();
    for entry in entries {
        let (mut name, mut display, mut write, mut read) = Default::default();
        let mut kind = None;
        for field in entry.split('|') {
            let mut parts = field.split(':');
            let (Some(key), Some(value)) = (parts.next(), parts.next()) else {
                continue;
            };
            match key.trim() {
                "name" => name = value.to_string(),
                "type" => match value {
                    "list" => kind = Some(PropertyKind::List { values: Vec::new() }),
                    "range" => kind = Some(PropertyKind::Range { min: 0, max: 0 }),
                    "float" => kind = Some(PropertyKind::Float),
                    "string" => kind = Some(PropertyKind::Text),
                    _ => {}
                },
                "display" => display = value.split_whitespace().collect::<Vec<_>>().join(" "),
                "values" => match &mut kind {
                    Some(PropertyKind::List { values }) => {
                        *values = value.split(',').map(str::to_string).collect();
                    }
                    Some(PropertyKind::Range { min, max }) => {
                        let mut bounds = value.split(',').map(|n| n.trim().parse().unwrap_or(0));
                        if let (Some(low), Some(high)) = (bounds.next(), bounds.next()) {
                            (*min, *max) = (low, high);
                        }
                    }
                    _ => {}
                },
                "write" => write = value.trim().to_string(),
                "read" => read = value.trim().to_string(),
                _ => {}
            }
        }
        let Some(kind) = kind.filter(|_| !name.is_empty()) else {
            continue;
        };
        // O padrão é o que `read` devolve agora, antes de qualquer `write`
        let default = limits.call(ctx, || {
            let Some(read) = algo.get::<_, Value>(read.as_str())?.into_function() else {
                return Ok(String::new());
            };
            let value: Option<Coerced<String>> = read.call(())?;
            Ok(value.map(|text| text.0).unwrap_or_default())
        })?;
        properties.push(ScriptProperty {
            name,
            display,
            kind,
            write,
            read,
            default,
        });
    }
    Ok(properties)
}

/// `algo.acceptColors`: sem ele, o script aceita as duas cores (início e fim), como no QLC+. O
/// script pode mudá-lo ao receber uma propriedade (o plasma), então é lido a cada uso.
fn accept_colors<'js>(limits: &Limits, ctx: &Ctx<'js>, algo: &Object<'js>) -> Result<u8, String> {
    let accepts: Option<Coerced<i32>> = limits.call(ctx, || algo.get("acceptColors"))?;
    Ok(accepts.map_or(2, |accepts| accepts.0.clamp(0, 5) as u8))
}

/// Avalia o script num interpretador novo (com os limites) e lê o nome, o autor, a versão da API,
/// quantas cores ele aceita e as propriedades, com o valor padrão de cada uma.
pub fn inspect(source: &str) -> Result<Meta, String> {
    let source = source.to_owned();
    spawn(move || {
        interpret(|limits, ctx| {
            let script = load(limits, ctx, &source)?;
            let text = |key: &str| -> Result<String, String> {
                let text: Option<Coerced<String>> = limits.call(ctx, || script.algo.get(key))?;
                Ok(text.map(|text| text.0).unwrap_or_default())
            };
            let name = text("name")?;
            if name.trim().is_empty() {
                return Err("o script não tem nome".into());
            }
            Ok(Meta {
                name,
                author: text("author")?,
                api: script.api,
                accepts: accept_colors(limits, ctx, &script.algo)?,
                properties: script.properties,
            })
        })
    })?
    .join()
    .unwrap_or_else(|_| Err("erro interno no interpretador".into()))
}

struct Request {
    step: usize,
    rgb: u32,
    colors: Vec<u32>,
}

/// Uma instância de um script rodando numa thread própria (o interpretador não sai dela): cada
/// RGB Matrix rodando tem a sua, porque os scripts guardam estado (o plasma e o fogo avançam o
/// tempo a cada chamada; vários reiniciam listas no passo 0). Largar a `Session` encerra a thread.
pub struct Session {
    requests: Sender<Request>,
    replies: Receiver<Reply>,
}

impl Session {
    /// Abre sem esperar: a thread avalia o script, aplica `properties` (nome e valor em texto, que
    /// vai ao método `write` da propriedade; o que o script não declara é ignorado) e responde
    /// `Reply::Opened`.
    pub fn open(
        source: Arc<str>,
        properties: Vec<(String, String)>,
        width: u16,
        height: u16,
    ) -> Session {
        let (requests, request_rx) = mpsc::channel();
        let (reply_tx, replies) = mpsc::channel();
        let spawn_failed = reply_tx.clone();
        let spawned = spawn(move || {
            // O script não derruba o app nem por um pânico do interpretador
            let served = catch_unwind(AssertUnwindSafe(|| {
                serve(&source, &properties, width, height, &request_rx, &reply_tx)
            }));
            if served.is_err() {
                let _ = reply_tx.send(Reply::Failed("erro interno no interpretador".into()));
            }
        });
        if let Err(error) = spawned {
            let _ = spawn_failed.send(Reply::Failed(error));
        }
        Session { requests, replies }
    }

    /// Pede o mapa do passo `step` (sem esperar). `rgb` é a cor do passo e `colors` são todas as
    /// cores do RGB Matrix (0xRRGGBB): no API 3, o script as recebe em `rgbMapSetColors` antes do
    /// `rgbMap`, quando mudam. Os pedidos são atendidos em ordem, uma resposta por pedido.
    pub fn request(&self, step: usize, rgb: u32, colors: Vec<u32>) {
        // Sem thread (a sessão falhou), não há quem responda: o `Failed` já foi dado
        let _ = self.requests.send(Request { step, rgb, colors });
    }

    /// Próxima resposta pronta, sem bloquear.
    pub fn poll(&self) -> Option<Reply> {
        self.replies.try_recv().ok()
    }
}

/// O corpo da thread da sessão: abre o script e atende os pedidos até a `Session` ser largada.
fn serve(
    source: &str,
    properties: &[(String, String)],
    width: u16,
    height: u16,
    requests: &Receiver<Request>,
    replies: &Sender<Reply>,
) {
    let served = interpret(|limits, ctx| {
        let script = load(limits, ctx, source)?;
        for (name, value) in properties {
            let Some(property) = script
                .properties
                .iter()
                .find(|property| property.name == *name)
            else {
                continue;
            };
            limits.call(ctx, || {
                // Sem o método `write`, o QLC+ também só ignora a propriedade
                if let Some(write) = script
                    .algo
                    .get::<_, Value>(property.write.as_str())?
                    .into_function()
                {
                    write.call::<_, Value>((value.as_str(),))?;
                }
                Ok(())
            })?;
        }

        let steps: Value = limits.call(ctx, || script.step_count.call((width, height)))?;
        let steps = steps
            .as_number()
            .filter(|steps| *steps >= 0.0)
            .ok_or("rgbMapStepCount não devolveu um número")?;
        if replies
            .send(Reply::Opened {
                steps: steps as usize,
            })
            .is_err()
        {
            return Ok(());
        }

        let (width, height) = (width as usize, height as usize);
        let mut colors: Option<Vec<u32>> = None;
        while let Ok(request) = requests.recv() {
            if let Some(set_colors) = &script.set_colors
                && colors.as_ref() != Some(&request.colors)
            {
                // Como no QLC+: a lista tem o tamanho de `acceptColors` e leva só as cores que couberem
                let accepts = accept_colors(limits, ctx, &script.algo)? as usize;
                limits.call(ctx, || {
                    let list = Array::new(ctx.clone())?;
                    for (i, color) in request.colors.iter().take(accepts).enumerate() {
                        list.set(i, *color)?;
                    }
                    list.as_object().set("length", accepts)?;
                    set_colors.call::<_, Value>((list,))
                })?;
                colors = Some(request.colors);
            }

            let map = limits.call(ctx, || {
                let result: Value = script.rgb_map.call((width, height, request.rgb, request.step))?;
                let invalid = || {
                    let message = format!(
                        "rgbMap não devolveu uma matriz de {height} linhas por {width} colunas de números"
                    );
                    Exception::throw_message(ctx, &message)
                };
                let Some(rows) = result.into_array().filter(|rows| rows.len() >= height) else {
                    return Err(invalid());
                };
                let mut map = Vec::with_capacity(height);
                for y in 0..height {
                    let row = rows.get::<Value>(y)?.into_array().filter(|row| row.len() >= width);
                    let Some(row) = row else { return Err(invalid()) };
                    let mut cells = Vec::with_capacity(width);
                    for x in 0..width {
                        let Some(cell) = row.get::<Value>(x)?.as_number() else {
                            return Err(invalid());
                        };
                        // O QLC+ arredonda (`toUInt` do QVariant) e só usa o RGB
                        cells.push((cell.round() as i64 & 0xFF_FFFF) as u32);
                    }
                    map.push(cells);
                }
                Ok(map)
            })?;
            if replies
                .send(Reply::Map {
                    step: request.step,
                    map,
                })
                .is_err()
            {
                break;
            }
        }
        Ok(())
    });
    if let Err(message) = served {
        let _ = replies.send(Reply::Failed(message));
    }
}
