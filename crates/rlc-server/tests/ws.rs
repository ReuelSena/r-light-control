//! Ponta a ponta: clientes de teste no servidor de verdade, com o motor e a saída Dummy rodando.

use std::net::Ipv4Addr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn server() -> u16 {
    let settings = rlc_server::Settings {
        port: 0,
        bind: Ipv4Addr::LOCALHOST.into(),
        ..Default::default()
    };
    rlc_server::start(&settings).await.unwrap()
}

async fn connect(port: u16) -> Socket {
    connect_async(format!("ws://127.0.0.1:{port}/ws"))
        .await
        .unwrap()
        .0
}

/// Próxima mensagem de texto. Ficam de fora os quadros binários e os números do Monitoramento,
/// que chegam a cada segundo e podem cair no meio de qualquer sequência.
async fn next_json(ws: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(3), ws.next())
            .await
            .expect("sem mensagem em 3 s")
            .unwrap()
            .unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if !matches!(value["topic"].as_str(), Some("stats" | "interfaces")) {
                return value;
            }
        }
    }
}

/// Espera um quadro do universo em que `ok` vale (os avisos de texto no meio são ignorados).
async fn frame_until(ws: &mut Socket, universe: &str, ok: impl Fn(&[u8]) -> bool) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        let message = tokio::time::timeout_at(deadline, ws.next())
            .await
            .expect("o quadro esperado não chegou em 3 s")
            .unwrap()
            .unwrap();
        if let Message::Binary(bytes) = message {
            assert_eq!(bytes[0], 1);
            let len = usize::from(bytes[1]);
            assert_eq!(bytes.len(), 2 + len + 512);
            if &bytes[2..2 + len] == universe.as_bytes() && ok(&bytes[2 + len..]) {
                return;
            }
        }
    }
}

async fn cmd(ws: &mut Socket, id: u64, op: Value) {
    let message = json!({ "t": "cmd", "id": id, "data": op }).to_string();
    ws.send(Message::Text(message.into())).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn clients_get_state_events_and_the_engine_output() {
    let port = server().await;
    let mut a = connect(port).await;
    let hello = next_json(&mut a).await;
    assert_eq!(
        (hello["t"].as_str(), hello["v"].as_u64()),
        (Some("hello"), Some(1))
    );
    let id = hello["data"]["clientId"].clone();
    let snapshot = next_json(&mut a).await;
    assert_eq!(snapshot["t"], "snapshot");
    assert_eq!(snapshot["data"]["project"]["universes"][0]["id"], "u1");
    assert!(
        snapshot["data"]["remote"]["clients"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == id)
    );

    // Segundo aparelho: entra na lista e recebe o que o primeiro muda
    let mut b = connect(port).await;
    assert_eq!(next_json(&mut b).await["t"], "hello");
    assert_eq!(next_json(&mut b).await["t"], "snapshot");
    assert_eq!(next_json(&mut a).await["topic"], "remote");

    // Mesa simples: canal 1 (dimmer do PAR 1) e canal 101 (pan do moving head 1).
    // O aviso chega antes da resposta, para quem espera a resposta já ter o estado novo.
    cmd(
        &mut a,
        1,
        json!({ "op": "setDeskLevel", "universe": "u1", "channel": 1, "value": 255 }),
    )
    .await;
    let event = next_json(&mut a).await;
    assert_eq!(
        (event["t"].as_str(), event["topic"].as_str()),
        (Some("evt"), Some("desk"))
    );
    assert_eq!(
        next_json(&mut a).await,
        json!({ "t": "ack", "id": 1, "data": null })
    );
    assert_eq!(next_json(&mut b).await["data"]["levels"][0]["value"], 255);
    cmd(
        &mut a,
        2,
        json!({ "op": "setDeskLevel", "universe": "u1", "channel": 101, "value": 200 }),
    )
    .await;
    frame_until(&mut a, "u1", |f| f[0] == 255 && f[100] == 200).await;

    // Grand Master a 50% reduz só a intensidade; o blackout zera a intensidade e mantém o pan
    cmd(&mut a, 3, json!({ "op": "setGrandMaster", "value": 128 })).await;
    frame_until(&mut a, "u1", |f| f[0] == 128 && f[100] == 200).await;
    cmd(&mut a, 4, json!({ "op": "setBlackout", "on": true })).await;
    frame_until(&mut b, "u1", |f| f[0] == 0 && f[100] == 200).await;

    // Recusa com o motivo, sem aviso
    cmd(
        &mut a,
        5,
        json!({ "op": "setDeskLevel", "universe": "u1", "channel": 600, "value": 1 }),
    )
    .await;
    loop {
        let reply = next_json(&mut a).await;
        if reply["id"] == 5 {
            assert_eq!(reply["t"], "err");
            break;
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn scenes_fade_in_take_live_edits_and_panic_cuts_them() {
    let port = server().await;
    let mut ws = connect(port).await;
    assert_eq!(next_json(&mut ws).await["t"], "hello");
    let snapshot = next_json(&mut ws).await;
    let functions = snapshot["data"]["project"]["functions"].as_array().unwrap();
    let mut scene = functions.iter().find(|f| f["id"] == "s1").unwrap().clone();

    // "Frente âmbar" (fade in de 1 s) no PAR 1: dimmer, vermelho e verde sobem juntos
    let run = json!({ "op": "setFunctionRunning", "id": "s1", "running": true });
    cmd(&mut ws, 1, run.clone()).await;
    frame_until(&mut ws, "u1", |f| f[0] > 0 && f[0] < 255).await;
    frame_until(&mut ws, "u1", |f| f[..3] == [255, 255, 140]).await;

    // Editada rodando, vale na hora
    scene["values"][2]["value"] = json!(60);
    cmd(
        &mut ws,
        2,
        json!({ "op": "saveFunction", "function": scene }),
    )
    .await;
    frame_until(&mut ws, "u1", |f| f[2] == 60).await;

    // O pânico corta na hora: do aceso direto para o apagado, sem fade out
    cmd(&mut ws, 3, json!({ "op": "panic", "all": false })).await;
    frame_until(&mut ws, "u1", |f| {
        assert!(f[0] == 255 || f[0] == 0, "fade out no pânico: {}", f[0]);
        f[0] == 0
    })
    .await;

    // O elemento "Tempos" do console (VC-3) troca os fades gravados: com o fade in em 0, a cena
    // entra direto…
    let dial = |fade_out: u32| json!({ "op": "setWidgetValue", "widget": "sd1", "value": [0, fade_out, 0] });
    cmd(&mut ws, 4, dial(2000)).await;
    cmd(&mut ws, 5, run).await;
    frame_until(&mut ws, "u1", |f| {
        assert!(f[0] == 0 || f[0] == 255, "fade in com o Tempos: {}", f[0]);
        f[0] == 255
    })
    .await;
    // …e o fade out mexido com a cena rodando vale ao parar
    cmd(&mut ws, 6, dial(0)).await;
    let stop = json!({ "op": "setFunctionRunning", "id": "s1", "running": false });
    cmd(&mut ws, 7, stop).await;
    frame_until(&mut ws, "u1", |f| {
        assert!(f[0] == 255 || f[0] == 0, "fade out com o Tempos: {}", f[0]);
        f[0] == 0
    })
    .await;
}
