// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! sensei310-tray - KDE tray controller for SteelSeries Sensei 310
//!
//! Provides a StatusNotifierItem (SNI) tray icon with menu for DPI, polling rate,
//! RGB colors, and button mapping. Uses `rivalcfg` CLI as backend.

mod buttons;
mod cli;
mod config;
mod kde_dialogs;
mod mapping;
mod rivalcfg;
mod tray;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub(crate) const TRADEMARK_NOTICE: &str = "SteelSeries and Sensei are trademarks or registered trademarks of SteelSeries ApS or its affiliates. This project is not affiliated with, endorsed by, or sponsored by SteelSeries.";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging on stderr so JSON output stays clean.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .init();

    info!("Starting sensei310-tray v{}", env!("CARGO_PKG_VERSION"));

    match cli::parse(std::env::args().skip(1))? {
        cli::Command::Tray => run_tray().await,
        cli::Command::Version => {
            println!("sensei310-tray {}", env!("CARGO_PKG_VERSION"));
            println!("KDE tray controller for the SteelSeries Sensei 310");
            println!("Copyright (c) 2026 Khame");
            println!("MIT License");
            println!("https://github.com/Khame/sensei310-tray");
            println!();
            println!("{TRADEMARK_NOTICE}");
            Ok(())
        }
        cli::Command::ButtonsJson => {
            let config = load_config().await;
            let snapshot = mapping::load(&config).await;
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
            Ok(())
        }
        cli::Command::SetButton { button, action } => {
            let mut config = load_config().await;
            match mapping::apply(&mut config, button, action).await {
                Ok(snapshot) => {
                    println!("{}", serde_json::to_string_pretty(&snapshot)?);
                    Ok(())
                }
                Err(error) => {
                    eprintln!("error: {error:#}");
                    std::process::exit(1);
                }
            }
        }
    }
}

async fn load_config() -> config::Config {
    match config::Config::load().await {
        Ok(config) => config,
        Err(error) => {
            warn!("Could not load settings; using defaults: {error:#}");
            config::Config::default()
        }
    }
}

async fn run_tray() -> Result<()> {
    let config = Arc::new(Mutex::new(load_config().await));

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

    // Verify kdialog is available (for color dialogs and the button picker)
    let kdialog_ok = kde_dialogs::check_kdialog().await;
    if !kdialog_ok {
        warn!("kdialog not found - dialogs will not work");
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
