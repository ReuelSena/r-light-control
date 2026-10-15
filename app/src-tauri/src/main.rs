// Sem janela de console no executável de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

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
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("R Light Control")
                .inner_size(1280.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
