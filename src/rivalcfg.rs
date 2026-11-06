//! Wrapper for rivalcfg CLI commands

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::{debug, error};

/// Check if rivalcfg is available and working
pub async fn check_rivalcfg() -> Result<()> {
    let output = Command::new("rivalcfg")
        .arg("--version")
        .output()
        .await
        .context("Failed to execute rivalcfg")?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        debug!("rivalcfg version: {}", version.trim());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("rivalcfg failed: {}", stderr);
    }
}

/// Run rivalcfg with given arguments
pub async fn run_rivalcfg(args: &[&str]) -> Result<()> {
    debug!("Running: rivalcfg {}", args.join(" "));

    let output = Command::new("rivalcfg")
        .args(args)
        .output()
        .await
        .context("Failed to execute rivalcfg")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("rivalcfg stdout: {}", stdout.trim());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("rivalcfg stderr: {}", stderr);
        anyhow::bail!("rivalcfg failed: {}", stderr);
    }
}

/// Get current device settings from rivalcfg
pub async fn get_device_info() -> Result<serde_json::Value> {
    let output = Command::new("rivalcfg")
        .args(["--print-debug"])
        .output()
        .await
        .context("Failed to execute rivalcfg --print-debug")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse the debug output (key=value format)
        let mut map = serde_json::Map::new();
        for line in stdout.lines() {
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), serde_json::Value::String(v.trim().to_string()));
            }
        }
        Ok(serde_json::Value::Object(map))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("rivalcfg --print-debug failed: {}", stderr);
    }
}

/// Get firmware version
pub async fn get_firmware_version() -> Result<String> {
    let output = Command::new("rivalcfg")
        .arg("--firmware-version")
        .output()
        .await
        .context("Failed to execute rivalcfg --firmware-version")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("rivalcfg --firmware-version failed: {}", stderr);
    }
}