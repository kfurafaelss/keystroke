mod app;
mod config;
mod input;
mod tray;
mod ui;

use anyhow::Result;
use config::Config;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, warn};
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

    let (tray_tx, tray_rx) = mpsc::channel();

    thread::spawn(move || match tray::start_tray() {
        Ok((action_receiver, handle)) => {
            debug!("Tray started successfully");

            loop {
                match action_receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(action) => {
                        debug!("Tray action received: {:?}", action);
                        if let Err(e) = tray_tx.send(action.clone()) {
                            warn!("Failed to forward tray action: {}", e);
                        }

                        if matches!(action, tray::TrayAction::Quit) {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        debug!("Tray receiver disconnected");
                        break;
                    }
                }
            }

            drop(handle);
        }
        Err(e) => {
            warn!("Failed to start system tray: {}", e);
        }
    });

    let app = app::App::new(config)?;

    glib::timeout_add_local(Duration::from_millis(100), move || {
        while let Ok(action) = tray_rx.try_recv() {
            match action {
                tray::TrayAction::Quit => {
                    debug!("Quit action received from tray");

                    std::process::exit(0);
                }
                tray::TrayAction::ShowLauncher => {
                    debug!("Show launcher action from tray");
                }
                tray::TrayAction::KeystrokeMode => {
                    debug!("Keystroke mode action from tray");
                }
                tray::TrayAction::BubbleMode => {
                    debug!("Bubble mode action from tray");
                }
                tray::TrayAction::OpenSettings => {
                    debug!("Settings action from tray");
                }
                tray::TrayAction::TogglePause => {
                    debug!("Toggle pause action from tray");
                }
            }
        }
        glib::ControlFlow::Continue
    });

    let exit_code = app.run()?;

    std::process::exit(exit_code);
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,keystroke=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}
