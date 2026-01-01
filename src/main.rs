mod app;
mod config;
mod input;
mod tray;
mod ui;

use anyhow::Result;
use config::Config;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> Result<()> {
    init_logging();

    info!("Starting Keystroke v{}", env!("CARGO_PKG_VERSION"));

    let config = Config::load().unwrap_or_else(|e| {
        warn!("Failed to load config: {}, using defaults", e);
        Config::default()
    });

    config.validate()?;

    info!("Configuration loaded: {:?}", config.position);

    match tray::start_tray() {
        Ok((tray_rx, tray_handle)) => {
            info!("System tray started successfully");

            let app = app::App::new(config)?;
            let exit_code = app.run_with_tray(tray_rx, tray_handle)?;
            std::process::exit(exit_code);
        }
        Err(e) => {
            warn!("Failed to start system tray: {}, running without tray", e);

            let app = app::App::new(config)?;
            let exit_code = app.run()?;
            std::process::exit(exit_code);
        }
    }
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,keystroke=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}
