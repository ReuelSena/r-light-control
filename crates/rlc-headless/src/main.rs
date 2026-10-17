//! Servidor sem janela: o mesmo do app desktop, para testes (e, no futuro, um Raspberry Pi).
//! Uso: `rlc-headless [--port N]`. Sem `--port`, vale a porta do `settings.json`.

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _log_guard = rlc_server::init_logging()?;
    let mut settings = rlc_server::Settings::load();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--port" {
            settings.port = args
                .next()
                .context("--port precisa de um número")?
                .parse()?;
        }
    }
    rlc_server::start(&settings).await?;
    tokio::signal::ctrl_c().await?;
    Ok(())
}
