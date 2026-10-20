// Sem janela de console no executável de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicUsize, Ordering};

use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};

fn main() -> anyhow::Result<()> {
    let _log_guard = rlc_server::init_logging()?;
    let settings = rlc_server::Settings::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Uma segunda execução só traz a janela existente para frente.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(move |app| {
            // O servidor sobe antes da janela, porque é ele que entrega a UI (E2).
            let port = tauri::async_runtime::block_on(rlc_server::start(&settings))?;
            // Em dev a UI vem do servidor do Nuxt (hot reload); em produção, do servidor Rust.
            let url = match app.config().build.dev_url.clone() {
                Some(dev_url) if tauri::is_dev() => dev_url,
                _ => format!("http://127.0.0.1:{port}").parse()?,
            };
            let handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("R Light Control")
                .inner_size(1280.0, 800.0)
                .min_inner_size(800.0, 600.0)
                // Sem o tratador de arrastar e soltar do Tauri, que no Windows engole o arrastar do
                // navegador: o dock (E13) precisa dele para separar as abas.
                .disable_drag_drop_handler()
                .on_document_title_changed(|window, title| {
                    let _ = window.set_title(&title);
                })
                // "Abrir em janela" na UI chama window.open, que vira uma janela nativa e pode ir
                // para outro monitor (E13). Endereços de fora do app são ignorados.
                .on_new_window(move |url, _| {
                    if matches!(url.host_str(), Some("127.0.0.1" | "localhost")) {
                        open_window(handle.clone(), url);
                    }
                    NewWindowResponse::Deny
                })
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}

/// Janela externa: a mesma UI, conectada ao motor como mais um cliente (E13).
fn open_window(app: AppHandle, url: Url) {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // Fora do callback do WebView2: criar uma janela dentro dele trava no Windows.
    std::thread::spawn(move || {
        let label = format!("janela-{}", COUNT.fetch_add(1, Ordering::Relaxed));
        let built = WebviewWindowBuilder::new(&app, label, WebviewUrl::External(url))
            .inner_size(1100.0, 750.0)
            .disable_drag_drop_handler()
            .on_document_title_changed(|window, title| {
                let _ = window.set_title(&title);
            })
            .build();
        if let Err(e) = built {
            tracing::error!("não foi possível abrir a janela externa: {e}");
        }
    });
}
