//! Os scripts de teste são curtos e originais; os do QLC+ só entram como dados (`real_scripts_*`).

use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use rlc_core::PropertyKind;
use rlc_script::{CALL_LIMIT, Reply, Session, inspect};

/// Um script API 3 completo: o corpo de `rgbMap` e as propriedades são o que muda em cada teste.
fn script(api: u8, extra: &str, rgb_map: &str) -> String {
    format!(
        r#"var testAlgo;
(function () {{
  var algo = new Object;
  algo.apiVersion = {api};
  algo.name = "Teste";
  algo.author = "Autor";
  algo.properties = new Array();
  {extra}
  algo.rgbMap = function (width, height, rgb, step) {{ {rgb_map} }};
  algo.rgbMapStepCount = algo.rgbMapStepCount || function (width, height) {{ return width; }};
  testAlgo = algo;
  return algo;
}})()"#
    )
}

/// Um mapa de `height` linhas em que toda célula vale `value` (uma expressão JS).
fn filled(value: &str) -> String {
    format!(
        "var m = []; for (var y = 0; y < height; y++) {{ m[y] = []; \
         for (var x = 0; x < width; x++) m[y][x] = {value}; }} return m;"
    )
}

fn wait(session: &Session) -> Reply {
    let start = Instant::now();
    loop {
        if let Some(reply) = session.poll() {
            return reply;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "a sessão não respondeu"
        );
        thread::sleep(Duration::from_millis(2));
    }
}

fn open(source: &str, properties: &[(&str, &str)]) -> Session {
    let properties = properties
        .iter()
        .map(|(n, v)| (n.to_string(), v.to_string()))
        .collect();
    Session::open(Arc::from(source), properties, 4, 3)
}

fn map_of(reply: Reply) -> Vec<Vec<u32>> {
    match reply {
        Reply::Map { map, .. } => map,
        Reply::Opened { .. } => panic!("esperava um mapa, veio Opened"),
        Reply::Failed(message) => panic!("esperava um mapa, veio Failed: {message}"),
    }
}

fn failure(reply: Reply) -> String {
    match reply {
        Reply::Failed(message) => message,
        _ => panic!("esperava Failed"),
    }
}

#[test]
fn inspect_reads_api1_script() {
    let meta = inspect(&script(1, "", &filled("rgb"))).unwrap();
    assert_eq!(
        (meta.name.as_str(), meta.author.as_str(), meta.api),
        ("Teste", "Autor", 1)
    );
    // Sem `acceptColors`, o QLC+ assume as duas cores
    assert_eq!(meta.accepts, 2);
    assert!(meta.properties.is_empty());
}

#[test]
fn inspect_reads_api2_properties_with_defaults() {
    let extra = r#"
      algo.acceptColors = 1;
      algo.mode = 1;
      algo.properties.push("name:mode|type:list|display:Modo  de   cor|values:Frio,Quente|write:setMode|read:getMode");
      algo.setMode = function (v) { algo.mode = (v === "Quente") ? 1 : 0; };
      algo.getMode = function () { return algo.mode === 1 ? "Quente" : "Frio"; };
      algo.size = 5;
      algo.properties.push("name:size|type:range|display:Tamanho|values:1,20|write:setSize|read:getSize");
      algo.setSize = function (v) { algo.size = parseInt(v, 10); };
      algo.getSize = function () { return algo.size; };
      algo.gain = 0.5;
      algo.properties.push("name:gain|type:float|display:Ganho|write:setGain|read:getGain");
      algo.setGain = function (v) { algo.gain = parseFloat(v); };
      algo.getGain = function () { return algo.gain; };
      algo.label = "oi";
      algo.properties.push("name:label|type:string|display:Rótulo|write:setLabel|read:getLabel");
      algo.setLabel = function (v) { algo.label = v; };
      algo.getLabel = function () { return algo.label; };
      algo.properties.push("name:|type:string|display:Sem nome|write:x|read:y");
      algo.properties.push("name:semtipo|display:Sem tipo|write:x|read:y");
    "#;
    let meta = inspect(&script(2, extra, &filled("rgb"))).unwrap();
    assert_eq!((meta.api, meta.accepts), (2, 1));
    // As duas últimas (sem nome e sem tipo) o QLC+ também descarta
    let names: Vec<_> = meta.properties.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["mode", "size", "gain", "label"]);

    let mode = &meta.properties[0];
    assert_eq!(mode.display, "Modo de cor");
    assert_eq!(
        mode.kind,
        PropertyKind::List {
            values: vec!["Frio".into(), "Quente".into()]
        }
    );
    assert_eq!(
        (
            mode.write.as_str(),
            mode.read.as_str(),
            mode.default.as_str()
        ),
        ("setMode", "getMode", "Quente")
    );
    assert_eq!(
        meta.properties[1].kind,
        PropertyKind::Range { min: 1, max: 20 }
    );
    assert_eq!(meta.properties[1].default, "5");
    assert_eq!(meta.properties[2].kind, PropertyKind::Float);
    assert_eq!(meta.properties[2].default, "0.5");
    assert_eq!(meta.properties[3].kind, PropertyKind::Text);
    assert_eq!(meta.properties[3].default, "oi");
}

#[test]
fn inspect_reads_api3_script() {
    let extra = "algo.acceptColors = 3; algo.rgbMapSetColors = function (colors) {};";
    let meta = inspect(&script(3, extra, &filled("rgb"))).unwrap();
    assert_eq!((meta.api, meta.accepts), (3, 3));
}

#[test]
fn inspect_rejects_invalid_api_version() {
    for api in ["0", "4", "undefined", "\"x\""] {
        let error =
            inspect(&script(1, "", &filled("rgb")).replace("= 1;", &format!("= {api};"))).err();
        assert_eq!(
            error.as_deref(),
            Some("apiVersion inválida"),
            "apiVersion = {api}"
        );
    }
}

#[test]
fn inspect_rejects_empty_name_syntax_error_and_missing_functions() {
    let nameless = script(1, "", &filled("rgb")).replace("\"Teste\"", "\"\"");
    assert!(inspect(&nameless).unwrap_err().contains("nome"));
    assert!(
        inspect("(function () {")
            .unwrap_err()
            .contains("SyntaxError")
    );
    assert!(
        inspect("throw new Error('boom')")
            .unwrap_err()
            .contains("boom")
    );
    assert!(inspect("42").unwrap_err().contains("objeto do algoritmo"));
    let no_set_colors = script(3, "", &filled("rgb"));
    assert!(
        inspect(&no_set_colors)
            .unwrap_err()
            .contains("rgbMapSetColors")
    );
    let no_properties = script(2, "algo.properties = 7;", &filled("rgb"));
    assert!(inspect(&no_properties).unwrap_err().contains("properties"));
}

#[test]
fn scripts_run_in_sloppy_mode_like_qlcplus() {
    // Variável sem `var`: erro no modo estrito, aceita pelo QLC+
    let source = script(1, "undeclared = 7;", &filled("rgb"));
    assert!(inspect(&source).is_ok());
}

#[test]
fn session_returns_maps_in_order_with_rgb_masked() {
    let session = open(&script(1, "", &filled("(x == step) ? rgb : 0")), &[]);
    assert!(matches!(wait(&session), Reply::Opened { steps: 4 }));
    session.request(0, 0xFF0000, vec![]);
    session.request(2, 0x00FF00, vec![]);
    let first = match wait(&session) {
        Reply::Map { step, map } => (step, map),
        _ => panic!("esperava um mapa"),
    };
    assert_eq!(first.0, 0);
    assert_eq!(first.1.len(), 3);
    assert_eq!(first.1[0], [0xFF0000, 0, 0, 0]);
    let second = match wait(&session) {
        Reply::Map { step, map } => (step, map),
        _ => panic!("esperava um mapa"),
    };
    assert_eq!((second.0, second.1[2][2]), (2, 0x00FF00));
    assert!(session.poll().is_none());
}

#[test]
fn session_converts_float_cells_and_drops_alpha() {
    let session = open(
        &script(
            1,
            "",
            &filled("x == 0 ? 255.4 : (x == 1 ? 0xFF123456 : 16711680.5)"),
        ),
        &[],
    );
    wait(&session);
    session.request(0, 0, vec![]);
    let map = map_of(wait(&session));
    assert_eq!(map[0], [255, 0x123456, 0xFF0001, 0xFF0001]);
}

#[test]
fn session_keeps_state_between_calls() {
    // O contador do plasma e do fogo: cada chamada vê o que a anterior deixou
    let extra = "var calls = 0;";
    let source = script(
        1,
        extra,
        &format!(
            "calls++; var m = {}; m[0][0] = calls; return m;",
            "[[0,0,0,0],[0,0,0,0],[0,0,0,0]]"
        ),
    );
    let session = open(&source, &[]);
    wait(&session);
    for expected in 1..=3 {
        session.request(0, 0, vec![]);
        assert_eq!(map_of(wait(&session))[0][0], expected);
    }
}

#[test]
fn session_writes_properties_before_the_first_map() {
    let extra = r#"
      algo.level = 1;
      algo.properties.push("name:level|type:range|display:Nível|values:0,9|write:setLevel|read:getLevel");
      algo.setLevel = function (v) { algo.level = parseInt(v, 10); };
      algo.getLevel = function () { return algo.level; };
      algo.rgbMapStepCount = function (width, height) { return algo.level * 10; };
    "#;
    let source = script(2, extra, &filled("algo.level"));
    let session = open(&source, &[("level", "7"), ("inexistente", "1")]);
    // O `rgbMapStepCount` já vê o valor escrito
    assert!(matches!(wait(&session), Reply::Opened { steps: 70 }));
    session.request(0, 0, vec![]);
    assert_eq!(map_of(wait(&session))[1][1], 7);
}

#[test]
fn session_ignores_properties_of_api1() {
    let session = open(&script(1, "", &filled("rgb")), &[("level", "7")]);
    assert!(matches!(wait(&session), Reply::Opened { .. }));
}

#[test]
fn session_sends_colors_to_set_colors_only_when_they_change() {
    let extra = r#"
      algo.acceptColors = 2;
      var received = [];
      var sets = 0;
      algo.rgbMapSetColors = function (colors) { sets++; received = colors; };
    "#;
    let body = "var m = [[0,0,0,0],[0,0,0,0],[0,0,0,0]]; \
                m[0][0] = received.length; m[0][1] = received[0]; m[0][2] = received[1] || 0; m[0][3] = sets; return m;";
    let session = open(&script(3, extra, body), &[]);
    wait(&session);
    session.request(0, 0, vec![0x112233, 0x445566, 0x778899]);
    // A lista tem o tamanho de `acceptColors`: a terceira cor não cabe
    assert_eq!(map_of(wait(&session))[0], [2, 0x112233, 0x445566, 1]);
    session.request(1, 0, vec![0x112233, 0x445566, 0x778899]);
    assert_eq!(map_of(wait(&session))[0][3], 1);
    session.request(2, 0, vec![0xABCDEF]);
    assert_eq!(map_of(wait(&session))[0], [2, 0xABCDEF, 0, 2]);
}

#[test]
fn session_exception_becomes_failed_with_the_message() {
    let session = open(&script(1, "", "throw new Error('boom');"), &[]);
    wait(&session);
    session.request(0, 0, vec![]);
    assert!(failure(wait(&session)).contains("boom"));
    // A sessão acabou: nenhuma resposta mais virá
    session.request(1, 0, vec![]);
    thread::sleep(Duration::from_millis(50));
    assert!(session.poll().is_none());
}

#[test]
fn session_invalid_maps_become_failed() {
    for body in [
        "return 1;",
        "return [1, 2, 3];",
        "return [[0,0,0,0],[0,0,0,0]];",
        "return [[0,0,0,0],[0,0,0,0],[0,0,0]];",
        "return [[0,0,0,0],[0,0,0,0],['a','b','c','d']];",
        "return [[0,0,0,0],[0,0,0,0],[0,0,0,undefined]];",
    ] {
        let session = open(&script(1, "", body), &[]);
        wait(&session);
        session.request(0, 0, vec![]);
        assert!(failure(wait(&session)).contains("rgbMap"), "{body}");
    }
}

#[test]
fn session_step_count_must_be_a_number() {
    let source = script(
        1,
        "algo.rgbMapStepCount = function () { return 'x'; };",
        &filled("0"),
    );
    assert!(failure(wait(&open(&source, &[]))).contains("rgbMapStepCount"));
}

/// Um laço infinito em cada tipo de chamada: aborta em `CALL_LIMIT`, com folga, e vira `Failed`.
#[test]
fn infinite_loop_is_aborted_in_every_kind_of_call() {
    let hang = "while (true) {}";
    let on_eval = script(1, hang, &filled("rgb"));
    let on_write = script(
        2,
        &format!(
            "algo.properties.push('name:p|type:string|write:w|read:r'); algo.w = function () {{ {hang} }}; algo.r = function () {{ return 'x'; }};"
        ),
        &filled("rgb"),
    );
    let on_count = script(
        1,
        &format!("algo.rgbMapStepCount = function () {{ {hang} }};"),
        &filled("rgb"),
    );
    let on_map = script(1, "", hang);
    let on_read = script(
        2,
        &format!(
            "algo.properties.push('name:p|type:string|write:w|read:r'); algo.r = function () {{ {hang} }};"
        ),
        &filled("rgb"),
    );
    let limit = CALL_LIMIT * 4;

    for (kind, source, properties, request) in [
        ("avaliação", &on_eval, vec![], false),
        ("write", &on_write, vec![("p", "1")], false),
        ("rgbMapStepCount", &on_count, vec![], false),
        ("rgbMap", &on_map, vec![], true),
    ] {
        let start = Instant::now();
        let session = open(source, &properties);
        let reply = if request {
            assert!(matches!(wait(&session), Reply::Opened { .. }), "{kind}");
            session.request(0, 0, vec![]);
            wait(&session)
        } else {
            wait(&session)
        };
        assert!(failure(reply).contains("tempo esgotado"), "{kind}");
        assert!(
            start.elapsed() < limit,
            "{kind}: levou {:?}",
            start.elapsed()
        );
    }

    for (kind, source) in [
        ("inspect (avaliação)", &on_eval),
        ("inspect (read)", &on_read),
    ] {
        let start = Instant::now();
        assert!(
            inspect(source).unwrap_err().contains("tempo esgotado"),
            "{kind}"
        );
        assert!(
            start.elapsed() < limit,
            "{kind}: levou {:?}",
            start.elapsed()
        );
    }
}

#[test]
fn script_cannot_swallow_the_interrupt() {
    // O prazo aborta com uma exceção que o `try/catch` do script não pega
    let body = "while (true) { try { while (true) {} } catch (e) {} }";
    let session = open(&script(1, "", body), &[]);
    wait(&session);
    let start = Instant::now();
    session.request(0, 0, vec![]);
    assert!(failure(wait(&session)).contains("tempo esgotado"));
    assert!(start.elapsed() < CALL_LIMIT * 4);
}

#[test]
fn deadline_is_rearmed_on_every_call() {
    // Cada chamada leva cerca de um terço do prazo: juntas passam dele, e nenhuma é abortada
    let busy = "var end = Date.now() + 80; while (Date.now() < end) {}";
    let session = open(&script(1, "", &format!("{busy} {}", filled("rgb"))), &[]);
    wait(&session);
    for step in 0..6 {
        session.request(step, 1, vec![]);
        assert_eq!(map_of(wait(&session))[0][0], 1);
    }
}

#[test]
fn memory_exhaustion_becomes_failed() {
    for body in [
        // O exemplo do pedido: pode acabar pelo tempo ou pela memória, e as duas viram `Failed`
        "var a = []; while (true) a.push(new Array(1e6));",
        // Esta estoura a memória bem antes do prazo
        "var a = []; while (true) a.push(new Array(1e6).fill(1));",
        // Esta o QuickJS recusa por conta própria ("string too long")
        "var s = 'x'; while (true) s += s;",
    ] {
        let session = open(&script(1, "", body), &[]);
        wait(&session);
        session.request(0, 0, vec![]);
        let message = failure(wait(&session));
        println!("{body} -> {message}");
        assert!(!message.is_empty(), "{body}");
        if body.contains("fill(1)") {
            assert!(message.contains("memória"), "{message}");
        }
    }
}

#[test]
fn infinite_recursion_becomes_failed() {
    let session = open(
        &script(1, "function f() { return f() + 1; }", "return f();"),
        &[],
    );
    wait(&session);
    session.request(0, 0, vec![]);
    assert!(!failure(wait(&session)).is_empty());
}

#[test]
fn dropping_the_session_ends_the_thread() {
    let source: Arc<str> = Arc::from(script(1, "", &filled("rgb")));
    let session = Session::open(source.clone(), vec![], 4, 3);
    assert!(matches!(wait(&session), Reply::Opened { .. }));
    // A thread guarda uma cópia do texto enquanto vive
    assert_eq!(Arc::strong_count(&source), 2);
    drop(session);
    let start = Instant::now();
    while Arc::strong_count(&source) > 1 {
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "a thread não terminou"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn dropping_the_session_aborts_a_busy_thread_within_the_limit() {
    let source: Arc<str> = Arc::from(script(1, "", "while (true) {}"));
    let session = Session::open(source.clone(), vec![], 4, 3);
    wait(&session);
    session.request(0, 0, vec![]);
    thread::sleep(Duration::from_millis(30));
    drop(session);
    let start = Instant::now();
    while Arc::strong_count(&source) > 1 {
        assert!(start.elapsed() < CALL_LIMIT * 4, "a thread não terminou");
        thread::sleep(Duration::from_millis(5));
    }
}

fn reference_scripts() -> Option<Vec<(String, String)>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../reference/qlcplus/resources/rgbscripts");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
    files.retain(|path| {
        path.extension().is_some_and(|ext| ext == "js") && path.file_name().unwrap() != "empty.js"
    });
    files.sort();
    Some(
        files
            .iter()
            .map(|path| {
                (
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    std::fs::read_to_string(path).unwrap(),
                )
            })
            .collect(),
    )
}

/// Os scripts da biblioteca do QLC+ são dados: todos importam, e os passos 0 a 2 saem em 8x8.
#[test]
fn real_scripts_inspect_and_run() {
    let Some(scripts) = reference_scripts() else {
        println!("reference/qlcplus não existe: teste dos scripts reais pulado");
        return;
    };
    let mut failures = Vec::new();
    // Os scripts que acenderam alguma célula: garante que os mapas não saem todos zerados
    let mut lit = std::collections::HashSet::new();
    for (file, source) in &scripts {
        let meta = match inspect(source) {
            Ok(meta) => meta,
            Err(error) => {
                failures.push(format!("{file}: inspect: {error}"));
                continue;
            }
        };
        // A segunda rodada escreve o valor padrão de cada propriedade, o que a tela de importação faz
        let defaults: Vec<_> = meta
            .properties
            .iter()
            .map(|p| (p.name.clone(), p.default.clone()))
            .collect();
        for properties in [Vec::new(), defaults] {
            let session = Session::open(Arc::from(source.as_str()), properties, 8, 8);
            let steps = match wait(&session) {
                Reply::Opened { steps } => steps,
                Reply::Failed(error) => {
                    failures.push(format!("{file}: abrir: {error}"));
                    continue;
                }
                Reply::Map { .. } => unreachable!(),
            };
            for step in 0..steps.min(3) {
                session.request(step, 0xFF0000, vec![0xFF0000, 0x0000FF]);
                match wait(&session) {
                    Reply::Map { map, .. }
                        if map.len() == 8 && map.iter().all(|row| row.len() == 8) =>
                    {
                        if map.iter().flatten().any(|cell| *cell != 0) {
                            lit.insert(file);
                        }
                    }
                    Reply::Map { map, .. } => failures.push(format!(
                        "{file}: passo {step}: mapa {}x{}",
                        map.len(),
                        map[0].len()
                    )),
                    Reply::Failed(error) => {
                        failures.push(format!("{file}: passo {step}: {error}"));
                        break;
                    }
                    Reply::Opened { .. } => unreachable!(),
                }
            }
        }
    }
    println!(
        "scripts reais: {} testados, {} falhas, {} com células acesas",
        scripts.len(),
        failures.len(),
        lit.len()
    );
    for failure in &failures {
        println!("  {failure}");
    }
    assert!(failures.is_empty(), "{failures:#?}");
    assert!(
        lit.len() > scripts.len() / 2,
        "poucos scripts acenderam células"
    );
}
