//! Um cliente em `/ws` (A3, `docs/protocolo.md`): recebe o `hello` e o estado completo, depois os
//! avisos de mudança e, a 25 Hz, os valores dos universos que mudaram; manda operações e recebe a
//! resposta de cada uma.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use rlc_core::{Id, Op};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::broadcast::{self, error::RecvError, error::TryRecvError};

use crate::{AppState, PROTOCOL_VERSION};

/// Tipo do frame binário com os 512 valores de um universo.
const UNIVERSE_FRAME: u8 = 1;

#[derive(Deserialize)]
struct Cmd {
    id: u64,
    data: Value,
}

pub async fn client(
    mut socket: WebSocket,
    state: AppState,
    address: SocketAddr,
    user_agent: String,
) {
    let (id, snapshot, mut events) = {
        let mut hub = state.lock();
        let id = hub.join(address, &user_agent);
        let (snapshot, events) = hub.snapshot();
        (id, snapshot, events)
    };
    let hello = json!({ "t": "hello", "v": PROTOCOL_VERSION, "data": {
        "appVersion": env!("CARGO_PKG_VERSION"),
        "clientId": id,
    }});
    if send(&mut socket, hello.to_string().into()).await && send(&mut socket, snapshot).await {
        run(&mut socket, &state, address, &mut events).await;
    }
    state.lock().leave(&id);
}

async fn run(
    socket: &mut WebSocket,
    state: &AppState,
    address: SocketAddr,
    events: &mut broadcast::Receiver<Utf8Bytes>,
) {
    let mut sent: HashMap<Id, [u8; 512]> = HashMap::new();
    let mut frames = tokio::time::interval(Duration::from_millis(40));
    let mut allow_lan = state.allow_lan.clone();
    let local = address.ip().is_loopback();
    loop {
        tokio::select! {
            message = socket.recv() => match message {
                Some(Ok(Message::Text(text))) => {
                    let reply = command(state, &text);
                    // Os avisos da operação saem antes da resposta: quem espera a resposta já
                    // encontra o estado novo
                    if !forward(socket, state, events).await || !send(socket, reply).await {
                        return;
                    }
                }
                Some(Ok(_)) => {}
                _ => return,
            },
            event = events.recv() => match event {
                Ok(event) => if !send(socket, event).await { return },
                Err(RecvError::Lagged(_)) => if !resync(socket, state, events).await { return },
                Err(RecvError::Closed) => return,
            },
            _ = frames.tick() => if !send_frames(socket, state, &mut sent).await { return },
            // Acesso pela rede local desligado: os aparelhos de fora caem na hora
            _ = allow_lan.changed(), if !local => if !*allow_lan.borrow() { return },
        }
    }
}

fn command(state: &AppState, text: &str) -> Utf8Bytes {
    let reply = match serde_json::from_str::<Cmd>(text) {
        Err(e) => json!({ "t": "err", "id": null, "data": format!("Comando inválido: {e}") }),
        Ok(cmd) => match serde_json::from_value::<Op>(cmd.data) {
            Err(e) => {
                json!({ "t": "err", "id": cmd.id, "data": format!("Operação inválida: {e}") })
            }
            Ok(op) => match state.lock().command(op) {
                Ok(data) => json!({ "t": "ack", "id": cmd.id, "data": data }),
                Err(e) => json!({ "t": "err", "id": cmd.id, "data": e }),
            },
        },
    };
    reply.to_string().into()
}

/// Repassa os avisos que já estão na fila, sem esperar.
async fn forward(
    socket: &mut WebSocket,
    state: &AppState,
    events: &mut broadcast::Receiver<Utf8Bytes>,
) -> bool {
    loop {
        match events.try_recv() {
            Ok(event) => {
                if !send(socket, event).await {
                    return false;
                }
            }
            Err(TryRecvError::Empty) => return true,
            Err(TryRecvError::Lagged(_)) => return resync(socket, state, events).await,
            Err(TryRecvError::Closed) => return false,
        }
    }
}

/// Cliente lento perdeu avisos: recebe o estado completo de novo e uma fila nova.
async fn resync(
    socket: &mut WebSocket,
    state: &AppState,
    events: &mut broadcast::Receiver<Utf8Bytes>,
) -> bool {
    let (snapshot, fresh) = state.lock().snapshot();
    *events = fresh;
    send(socket, snapshot).await
}

/// Valores dos universos que mudaram desde o último envio: `[1, tamanho do id, id, 512 valores]`.
async fn send_frames(
    socket: &mut WebSocket,
    state: &AppState,
    sent: &mut HashMap<Id, [u8; 512]>,
) -> bool {
    let frames = state.frames.load();
    for (id, frame) in &frames.universes {
        if sent.get(id) == Some(frame) {
            continue;
        }
        let id_bytes = &id.as_bytes()[..id.len().min(255)];
        let mut message = Vec::with_capacity(2 + id_bytes.len() + 512);
        message.extend([UNIVERSE_FRAME, id_bytes.len() as u8]);
        message.extend_from_slice(id_bytes);
        message.extend_from_slice(frame);
        if socket.send(Message::Binary(message.into())).await.is_err() {
            return false;
        }
        sent.insert(id.clone(), *frame);
    }
    true
}

async fn send(socket: &mut WebSocket, text: Utf8Bytes) -> bool {
    socket.send(Message::Text(text)).await.is_ok()
}
