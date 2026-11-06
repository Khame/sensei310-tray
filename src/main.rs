//! sensei310-tray - KDE tray controller for SteelSeries Sensei 310
//!
//! Provides a StatusNotifierItem (SNI) tray icon with menu for DPI, polling rate,
//! RGB colors, and gradients. Uses `rivalcfg` CLI as backend.

mod tray;
mod rivalcfg;
mod config;
mod kde_dialogs;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("info".parse().unwrap()))
        .init();

    info!("Starting sensei310-tray v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = config::Config::load().await.unwrap_or_default();
    let config = Arc::new(Mutex::new(config));

    // Verify rivalcfg is available
    let rivalcfg_ok = match rivalcfg::check_rivalcfg().await {
        Ok(()) => {
            info!("rivalcfg found and working");
            true
        }
        Err(e) => {
            warn!("rivalcfg not available: {}", e);
            false
        }
    };

    // Verify kdialog is available (for color dialogs)
    let kdialog_ok = kde_dialogs::check_kdialog().await;
    if !kdialog_ok {
        warn!("kdialog not found - color dialogs will not work");
    }

    // If rivalcfg is missing, show error and exit
    if !rivalcfg_ok {
        let msg = "rivalcfg not found in PATH. Please install rivalcfg (pipx install rivalcfg) to configure the mouse.";
        error!("{}", msg);
        if kdialog_ok {
            kde_dialogs::show_error("sensei310-tray", msg).await.ok();
        }
        std::process::exit(1);
    }

    // Create tray icon and menu
    let tray = tray::TrayController::new(config.clone()).await?;

    // Run the event loop
    tray.run().await?;

    Ok(())
}