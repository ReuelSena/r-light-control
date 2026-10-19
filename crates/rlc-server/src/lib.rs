//! Servidor HTTP/WebSocket: entrega a UI, guarda o estado (a autoridade, A2) e conversa com os
//! clientes. A janela do desktop e os navegadores na rede local usam o mesmo caminho (E2).

mod apply;
mod hub;
mod library;
mod project;
mod ws;

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use anyhow::Context;
use axum::Router;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use rlc_engine::Published;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub const PROTOCOL_VERSION: u32 = 1;
const APP_DIR: &str = "r-light-control";

/// Configurações do usuário, em `%APPDATA%\r-light-control\settings.json`.
#[derive(Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub port: u16,
    /// `0.0.0.0` abre a porta para a rede local; `127.0.0.1` restringe ao próprio PC.
    pub bind: IpAddr,
    /// Aceita aparelhos da rede local (o "Permitir aparelhos da rede local" do ⚙).
    pub allow_lan: bool,
    /// PIN de acesso, exigido a partir da F11.
    pub pin: Option<String>,
    /// Tipo escolhido à mão para cada interface USB (pelo id, `ftdi-` e o número de série).
    pub interface_kinds: BTreeMap<String, rlc_core::InterfaceKind>,
    /// Último projeto aberto ou salvo (pelo nome), que o app abre ao iniciar.
    pub last_project: Option<String>,
    /// Pasta do app (`%APPDATA%\r-light-control`), com a biblioteca de fixtures e a cópia de
    /// recuperação. O `load` põe as pastas do usuário; sem elas (nos testes), tudo fica na memória e o
    /// servidor abre o show de exemplo.
    #[serde(skip)]
    pub app_dir: Option<PathBuf>,
    /// Pasta dos projetos (`Documentos\R Light Control`).
    #[serde(skip)]
    pub projects_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port: 8090,
            bind: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            allow_lan: true,
            pin: None,
            interface_kinds: BTreeMap::new(),
            last_project: None,
            app_dir: None,
            projects_dir: None,
        }
    }
}

impl Settings {
    /// Pasta do app nas configurações do usuário (`%APPDATA%\r-light-control`).
    fn dir() -> PathBuf {
        dirs::config_dir().unwrap_or_default().join(APP_DIR)
    }

    /// Lê o arquivo. Se ele não existir, cria com os valores padrão para o usuário poder editar.
    pub fn load() -> Self {
        let path = Self::dir().join("settings.json");
        let mut settings = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
                tracing::warn!(
                    "{} inválido ({e}); usando os valores padrão",
                    path.display()
                );
                Self::default()
            }),
            Err(_) => {
                let settings = Self::default();
                settings.save();
                settings
            }
        };
        settings.app_dir = Some(Self::dir());
        settings.projects_dir = Some(
            dirs::document_dir()
                .unwrap_or_default()
                .join("R Light Control"),
        );
        settings
    }

    pub fn save(&self) {
        let path = Self::dir().join("settings.json");
        let saved = std::fs::create_dir_all(path.parent().unwrap_or(&path)).and_then(|_| {
            std::fs::write(
                &path,
                serde_json::to_string_pretty(self).unwrap_or_default(),
            )
        });
        if let Err(e) = saved {
            tracing::warn!("não foi possível gravar {}: {e}", path.display());
        }
    }
}

/// Escrita atômica (A5): um arquivo temporário que depois toma o lugar do antigo. Cria a pasta se
/// faltar.
pub(crate) fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let temp = path.with_extension("tmp");
    std::fs::create_dir_all(path.parent().unwrap_or(path))?;
    std::fs::write(&temp, text)?;
    std::fs::rename(&temp, path)
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

/// O que cada requisição recebe: o estado (atrás da trava, A2), os quadros do motor (sem trava)
/// e se a rede local está liberada.
#[derive(Clone)]
pub(crate) struct AppState {
    hub: Arc<Mutex<hub::Hub>>,
    frames: Published,
    allow_lan: watch::Receiver<bool>,
}

impl AppState {
    fn lock(&self) -> MutexGuard<'_, hub::Hub> {
        self.hub.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Este computador sempre entra; os outros, só com a rede local liberada.
    fn allowed(&self, address: SocketAddr) -> bool {
        address.ip().is_loopback() || *self.allow_lan.borrow()
    }
}

/// Abre a porta, inicia o motor e as saídas e passa a servir em segundo plano. Retorna a porta aberta.
/// Precisa ser chamada dentro de um runtime Tokio.
pub async fn start(settings: &Settings) -> anyhow::Result<u16> {
    let addr = SocketAddr::new(settings.bind, settings.port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("não foi possível abrir a porta {addr}"))?;
    let port = listener.local_addr()?.port();
    let hub = hub::Hub::new(settings, port);
    let state = AppState {
        frames: hub.frames(),
        allow_lan: hub.allow_lan.subscribe(),
        hub: Arc::new(Mutex::new(hub)),
    };
    let stats = state.clone();
    tokio::spawn(async move {
        let mut every = tokio::time::interval(Duration::from_secs(1));
        loop {
            every.tick().await;
            // A lista do USB pode levar alguns ms: fora da trava do estado
            let devices = tokio::task::spawn_blocking(rlc_dmx::ftdi::devices)
                .await
                .unwrap_or_default();
            stats.lock().refresh(&devices);
        }
    });
    // O passo dos chasers e as criações que terminaram sozinhas; a trava do estado só quando mudam
    let follow = state.clone();
    tokio::spawn(async move {
        let mut every = tokio::time::interval(Duration::from_millis(40));
        let mut seen = vec![];
        loop {
            every.tick().await;
            let frames = follow.frames.load();
            if frames.chasers != seen || !frames.done.is_empty() {
                seen = frames.chasers.clone();
                follow.lock().follow_engine(&frames);
            }
        }
    });
    let app = Router::new()
        .route("/ws", get(ws_upgrade))
        .fallback(static_file)
        .with_state(state);
    tokio::spawn(async move {
        let service = app.into_make_service_with_connect_info::<SocketAddr>();
        if let Err(e) = axum::serve(listener, service).await {
            tracing::error!("o servidor parou: {e}");
        }
    });
    tracing::info!("servidor ouvindo em {}:{port}", settings.bind);
    Ok(port)
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if !state.allowed(address) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    ws.on_upgrade(move |socket| ws::client(socket, state, address, user_agent))
}

#[derive(RustEmbed)]
#[folder = "../../ui/.output/public"]
#[allow_missing = true]
struct UiFiles;

/// Arquivos da UI embutidos no binário. Rotas desconhecidas devolvem o `index.html` (SPA).
async fn static_file(
    State(state): State<AppState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    uri: Uri,
) -> Response {
    if !state.allowed(address) {
        return (
            StatusCode::FORBIDDEN,
            "Acesso pela rede local desligado no ⚙ do app.",
        )
            .into_response();
    }
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
