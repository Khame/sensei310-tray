// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Keep the active profile on the mouse as it comes and goes.
//!
//! The tray is started at login, often before the mouse has finished
//! enumerating or while a different mouse is connected. Instead of failing once
//! and blocking the tray behind an error dialog, the profile is written every
//! time the Sensei 310 appears on the USB bus.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::kde_dialogs;
use crate::profiles;
use crate::rivalcfg;

/// How often the USB bus is checked for the mouse.
const POLL_INTERVAL: Duration = Duration::from_secs(3);

/// How many times one appearance of the mouse is tried before giving up.
const MAX_ATTEMPTS: u32 = 3;

/// What happened to the profile that was written to the mouse.
enum Loaded {
    /// Settings were sent to the mouse.
    Applied(String),
    /// The profile exists but stores no settings.
    Empty(String),
}

/// Watch the USB bus in the background and load the active profile whenever
/// the mouse appears.
pub fn spawn(config: Arc<Mutex<Config>>) {
    tokio::spawn(async move { watch(config).await });
}

async fn watch(config: Arc<Mutex<Config>>) {
    let mut loaded = false;
    let mut attempts = 0;
    let mut missing_reported = false;

    loop {
        if !rivalcfg::sensei_connected() {
            if !missing_reported {
                info!("Sensei 310 is not connected; waiting for it to appear");
                missing_reported = true;
            }
            loaded = false;
            attempts = 0;
        } else if !loaded && attempts < MAX_ATTEMPTS {
            missing_reported = false;
            attempts += 1;
            match apply_active(&config).await {
                Ok(Loaded::Applied(name)) => {
                    loaded = true;
                    info!("Loaded profile \"{name}\"");
                }
                Ok(Loaded::Empty(name)) => {
                    loaded = true;
                    info!("Profile \"{name}\" has no stored settings; nothing to load");
                }
                Err(error) => {
                    if attempts < MAX_ATTEMPTS {
                        warn!("{error:#}; retrying");
                    } else {
                        error!("{error:#}");
                        report(&error);
                    }
                }
            }
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Write the active profile to the mouse. The configuration lock is held while
/// rivalcfg runs, so menu actions and this task never talk to the device at the
/// same time.
async fn apply_active(config: &Arc<Mutex<Config>>) -> Result<Loaded> {
    let config = config.lock().await;
    let name = config.active.clone();
    let profile = config.active_profile().clone();

    if profiles::arguments(&profile).is_empty() {
        return Ok(Loaded::Empty(name));
    }

    profiles::apply(&profile)
        .await
        .with_context(|| format!("Could not load profile \"{name}\""))?;
    Ok(Loaded::Applied(name))
}

/// Show the failure without blocking the watcher or the tray.
fn report(error: &anyhow::Error) {
    let message = format!("{error:#}");
    tokio::spawn(async move {
        kde_dialogs::show_error("sensei310-tray", &message)
            .await
            .ok();
    });
}
