//! Servidor HTTP/WebSocket: entrega a UI e conversa com os clientes.
//! A janela do desktop e os navegadores na rede local usam o mesmo caminho (E2).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Context;
use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::watch;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub const PROTOCOL_VERSION: u32 = 1;
const APP_DIR: &str = "r-light-control";

/// Configurações do usuário, em `%APPDATA%\r-light-control\settings.json`.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub port: u16,
    /// `0.0.0.0` libera o acesso pela rede local; `127.0.0.1` restringe ao próprio PC.
    pub bind: IpAddr,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port: 8090,
            bind: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        }
    }
}

impl Settings {
    /// Lê o arquivo. Se ele não existir, cria com os valores padrão para o usuário poder editar.
    pub fn load() -> Self {
        let dir = dirs::config_dir().unwrap_or_default().join(APP_DIR);
        let path = dir.join("settings.json");
        if let Ok(text) = std::fs::read_to_string(&path) {
            return serde_json::from_str(&text).unwrap_or_else(|e| {
                tracing::warn!(
                    "{} inválido ({e}); usando os valores padrão",
                    path.display()
                );
                Self::default()
            });
        }
        let settings = Self::default();
        let saved = std::fs::create_dir_all(&dir).and_then(|_| {
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&settings).unwrap_or_default(),
            )
        });
        if let Err(e) = saved {
            tracing::warn!("não foi possível criar {}: {e}", path.display());
        }
        settings
    }
}

/// Logs no terminal e em arquivo diário (`%LOCALAPPDATA%\r-light-control\logs`, últimos 7 dias).
/// Guarde o retorno até o fim do programa: é ele que descarrega o arquivo ao sair.
pub fn init_logging() -> anyhow::Result<WorkerGuard> {
    let dir = dirs::data_local_dir()
        .unwrap_or_default()
        .join(APP_DIR)
        .join("logs");
    // Sem a pasta, a limpeza dos logs antigos falha na primeira execução.
    std::fs::create_dir_all(&dir)?;
    let file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("rlc")
        .filename_suffix("log")
        .max_log_files(7)
        .build(dir)?;
    let (file, guard) = tracing_appender::non_blocking(file);
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer())
        .with(fmt::layer().with_writer(file).with_ansi(false))
        .init();
    Ok(guard)
}

/// Abre a porta, inicia o tick e passa a servir em segundo plano. Retorna a porta aberta.
/// Precisa ser chamada dentro de um runtime Tokio.
pub async fn start(settings: &Settings) -> anyhow::Result<u16> {
    let addr = SocketAddr::new(settings.bind, settings.port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("não foi possível abrir a porta {addr}"))?;
    let port = listener.local_addr()?.port();
    let app = Router::new()
        .route("/ws", get(ws_upgrade))
        .fallback(static_file)
        .with_state(spawn_tick());
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("o servidor parou: {e}");
        }
    });
    tracing::info!("servidor ouvindo em {}:{port}", settings.bind);
    Ok(port)
}

/// Contador a 50 Hz. Provisório: na F3 vira a thread do motor (`rlc-engine`).
fn spawn_tick() -> watch::Receiver<u64> {
    let (tx, rx) = watch::channel(0);
    thread::spawn(move || {
        // Prazo absoluto: um atraso num tick é compensado no seguinte, sem deriva.
        let mut next = Instant::now();
        loop {
            next += Duration::from_millis(20);
            thread::sleep(next.saturating_duration_since(Instant::now()));
            tx.send_modify(|count| *count += 1);
        }
    });
    rx
}

async fn ws_upgrade(ws: WebSocketUpgrade, State(ticks): State<watch::Receiver<u64>>) -> Response {
    ws.on_upgrade(|socket| client(socket, ticks))
}

async fn client(mut socket: WebSocket, mut ticks: watch::Receiver<u64>) {
    let hello = json!({ "t": "hello", "v": PROTOCOL_VERSION, "data": {
        "appVersion": env!("CARGO_PKG_VERSION"),
        "lanIp": local_ip_address::local_ip().ok(),
    }});
    if socket
        .send(Message::Text(hello.to_string().into()))
        .await
        .is_err()
    {
        return;
    }
    loop {
        tokio::select! {
            changed = ticks.changed() => {
                if changed.is_err() {
                    break;
                }
                let count = *ticks.borrow_and_update();
                let msg = json!({ "t": "evt", "topic": "tick", "data": count });
                if socket.send(Message::Text(msg.to_string().into())).await.is_err() {
                    break;
                }
            }
            // Na F0 o cliente ainda não envia comandos; aqui só detectamos o fechamento.
            msg = socket.recv() => if !matches!(msg, Some(Ok(_))) {
                break;
            },
        }
    }
}

#[derive(RustEmbed)]
#[folder = "../../ui/.output/public"]
#[allow_missing = true]
struct UiFiles;

/// Arquivos da UI embutidos no binário. Rotas desconhecidas devolvem o `index.html` (SPA).
async fn static_file(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    match UiFiles::get(path).or_else(|| UiFiles::get("index.html")) {
        Some(file) => (
            [(header::CONTENT_TYPE, file.metadata.mimetype().to_owned())],
            file.data,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "UI não compilada: rode `pnpm --filter ui generate`",
        )
            .into_response(),
    }
}
