//! KDE-native dialogs via kdialog (no GUI crate dependencies)

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::{debug, error, warn};

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Pick a color using kdialog --getcolor
/// Returns None if user cancelled, Some(color) if picked, Err if kdialog failed
pub async fn pick_color(title: &str) -> Result<Option<RgbColor>> {
    debug!("Opening color dialog via kdialog: {}", title);

    let output = Command::new("kdialog")
        .args(["--getcolor", "--title", title])
        .output()
        .await
        .context("Failed to execute kdialog. Is kdialog installed?")?;

    match output.status.code() {
        Some(0) => {
            // Success - parse color output (format: #RRGGBB)
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.is_empty() {
                debug!("kdialog returned empty output");
                return Ok(None);
            }
            let color = parse_kdialog_color(&stdout)?;
            debug!("Color picked via kdialog: {}", color.to_hex());
            Ok(Some(color))
        }
        Some(1) => {
            // User cancelled
            debug!("Color dialog cancelled by user");
            Ok(None)
        }
        Some(code) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("kdialog failed with exit code {}: {}", code, stderr);
            anyhow::bail!("kdialog failed with exit code {}: {}", code, stderr);
        }
        None => {
            error!("kdialog terminated by signal");
            anyhow::bail!("kdialog terminated by signal");
        }
    }
}

/// Show error message via kdialog --error
pub async fn show_error(title: &str, message: &str) -> Result<()> {
    debug!("Showing error dialog: {} - {}", title, message);
    let output = Command::new("kdialog")
        .args(["--error", message, "--title", title])
        .output()
        .await
        .context("Failed to execute kdialog --error")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("kdialog --error failed: {}", stderr);
    }
    Ok(())
}

/// Show info message via kdialog --msgbox
pub async fn show_info(title: &str, message: &str) -> Result<()> {
    debug!("Showing info dialog: {} - {}", title, message);
    let output = Command::new("kdialog")
        .args(["--msgbox", message, "--title", title])
        .output()
        .await
        .context("Failed to execute kdialog --msgbox")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("kdialog --msgbox failed: {}", stderr);
    }
    Ok(())
}

/// Parse kdialog --getcolor output (#RRGGBB format)
fn parse_kdialog_color(s: &str) -> Result<RgbColor> {
    let s = s.trim();
    let s = s.strip_prefix('#').unwrap_or(s);

    if s.len() != 6 {
        anyhow::bail!("Invalid color format: expected #RRGGBB, got {}", s);
    }

    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;

    Ok(RgbColor { r, g, b })
}

/// Check if kdialog is available
pub async fn check_kdialog() -> bool {
    Command::new("kdialog")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}