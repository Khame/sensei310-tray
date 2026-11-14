// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Wrapper for rivalcfg CLI commands

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::{debug, error};

use crate::buttons::{capitalize, Button, ButtonMapping};

/// USB identifiers of the SteelSeries Sensei 310.
const SENSEI_VENDOR_ID: &str = "1038";
const SENSEI_PRODUCT_ID: &str = "1722";

/// Where the kernel publishes the USB devices that are currently connected.
const USB_DEVICES: &str = "/sys/bus/usb/devices";

/// Run rivalcfg and return trimmed stdout. `name` appears in error messages.
async fn run(args: &[&str], name: &str) -> Result<String> {
    let output = Command::new("rivalcfg")
        .args(args)
        .output()
        .await
        .with_context(|| format!("Failed to execute {name}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "{name} failed: {}",
            failure_detail(&stderr, &stdout, &output.status)
        )
    }
}

/// rivalcfg reports some failures on stdout only ("E: No supported device
/// found."), so an error that looks only at stderr can come out empty.
fn failure_detail(stderr: &str, stdout: &str, status: &std::process::ExitStatus) -> String {
    match (stderr.trim(), stdout.trim()) {
        ("", "") => status.to_string(),
        ("", stdout) => stdout.to_string(),
        (stderr, "") => stderr.to_string(),
        (stderr, stdout) => format!("{stderr}\n{stdout}"),
    }
}

/// Check if rivalcfg is available and working
pub async fn check_rivalcfg() -> Result<()> {
    let version = run(&["--version"], "rivalcfg").await?;
    debug!("rivalcfg version: {}", version);
    Ok(())
}

/// Run rivalcfg with given arguments
pub async fn run_rivalcfg(args: &[&str]) -> Result<()> {
    debug!("Running: rivalcfg {}", args.join(" "));
    match run(args, "rivalcfg").await {
        Ok(stdout) => {
            debug!("rivalcfg stdout: {}", stdout);
            Ok(())
        }
        Err(error) => {
            error!("{error:#}");
            Err(error)
        }
    }
}

/// Read the last saved Rivalcfg profile, not live settings from the mouse.
pub async fn get_saved_settings() -> Result<Option<serde_json::Value>> {
    let config_dir =
        dirs::config_dir().context("Could not determine the configuration directory")?;
    let path = config_dir.join("rivalcfg").join("1038_1722.device.json");
    let contents = match tokio::fs::read_to_string(&path).await {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| {
                format!("Could not read Rivalcfg settings from {}", path.display())
            });
        }
    };
    let settings: serde_json::Value = serde_json::from_str(&contents)
        .with_context(|| format!("Could not parse Rivalcfg settings from {}", path.display()))?;
    anyhow::ensure!(
        settings.is_object(),
        "Rivalcfg settings must be a JSON object"
    );
    Ok(Some(settings))
}

/// Turn `SteelSeries Sensei 310 (firmware v36.1)` into `v36.1`.
pub fn firmware_label(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(rest) = raw.split("firmware ").nth(1) {
        let version = rest
            .trim_end_matches(|c: char| c == ')' || c.is_whitespace())
            .trim();
        if !version.is_empty() {
            return version.to_string();
        }
    }
    raw.to_string()
}

/// Format the saved Rivalcfg profile for humans instead of dumping raw JSON.
pub fn format_saved_profile(settings: &serde_json::Value) -> String {
    let mut text = String::new();
    let Some(profiles) = settings.as_object() else {
        text.push_str("Saved Rivalcfg data is not a profile object:\n");
        let _ = writeln!(
            text,
            "{}",
            serde_json::to_string_pretty(settings).unwrap_or_default()
        );
        return text.trim_end().to_string();
    };

    if profiles.is_empty() {
        return String::from("No saved Rivalcfg profile.");
    }

    for (index, (name, profile)) in profiles.iter().enumerate() {
        if index > 0 {
            text.push('\n');
        }
        let _ = writeln!(text, "Saved Rivalcfg profile \"{name}\"");
        text.push_str("Not read from the mouse; may differ from its current state.\n");
        match profile.as_object() {
            Some(fields) => write_profile_fields(&mut text, fields),
            None => {
                let _ = writeln!(text, "\n{}", scalar(profile));
            }
        }
    }

    text.trim_end().to_string()
}

/// Settings Rivalcfg stores per profile, in a stable display order.
const PROFILE_FIELDS: &[&str] = &[
    "sensitivity1",
    "sensitivity2",
    "polling_rate",
    "logo_color",
    "wheel_color",
    "buttons_mapping",
];

fn write_profile_fields(text: &mut String, fields: &serde_json::Map<String, serde_json::Value>) {
    text.push('\n');

    if let Some(value) = fields.get("sensitivity1") {
        let _ = writeln!(text, "DPI preset 1: {}", scalar(value));
    }
    if let Some(value) = fields.get("sensitivity2") {
        let _ = writeln!(text, "DPI preset 2: {}", scalar(value));
    }
    if let Some(value) = fields.get("polling_rate") {
        let _ = writeln!(text, "Polling rate: {} Hz", scalar(value));
    }
    if let Some(value) = fields.get("logo_color") {
        let _ = writeln!(text, "Logo: {}", describe_color(&scalar(value)));
    }
    if let Some(value) = fields.get("wheel_color") {
        let _ = writeln!(text, "Wheel: {}", describe_color(&scalar(value)));
    }
    if let Some(value) = fields.get("buttons_mapping") {
        write_buttons(text, &scalar(value));
    }

    let extra: Vec<_> = fields
        .iter()
        .filter(|(key, _)| !PROFILE_FIELDS.contains(&key.as_str()))
        .collect();
    if !extra.is_empty() {
        text.push_str("\nOther saved values:\n");
        for (key, value) in extra {
            let _ = writeln!(text, "  {}: {}", humanize_key(key), scalar(value));
        }
    }
}

fn scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn humanize_key(key: &str) -> String {
    capitalize(&key.replace('_', " "))
}

/// `#00AA00` becomes `green (#00AA00)`, unknown values keep their raw form.
fn describe_color(raw: &str) -> String {
    let raw = raw.trim();
    match describe_gradient(raw) {
        Some(gradient) => gradient,
        None => describe_solid_color(raw),
    }
}

fn describe_solid_color(raw: &str) -> String {
    let raw = raw.trim();
    let (shown, name) = normalized_color(raw);
    match name {
        Some(name) => format!("{name} ({shown})"),
        None => shown,
    }
}

fn normalized_color(raw: &str) -> (String, Option<&'static str>) {
    match raw.strip_prefix('#') {
        Some(hex) => {
            let hex = hex.to_ascii_uppercase();
            let name = color_name(&hex);
            (format!("#{hex}"), name)
        }
        None => (raw.to_string(), color_name(raw)),
    }
}

fn color_name(token: &str) -> Option<&'static str> {
    Some(match token.to_ascii_lowercase().as_str() {
        "ff0000" | "red" => "red",
        "00ff00" | "lime" => "lime",
        "008000" | "00aa00" | "green" => "green",
        "0000ff" | "blue" => "blue",
        "ffffff" | "white" => "white",
        "000000" | "black" => "black",
        "ffff00" | "yellow" => "yellow",
        "00ffff" | "cyan" | "aqua" => "cyan",
        "ff00ff" | "magenta" | "fuchsia" => "magenta",
        "ffa500" | "orange" => "orange",
        "ffc0cb" | "pink" => "pink",
        "800080" | "purple" => "purple",
        "808080" | "gray" | "grey" => "gray",
        _ => return None,
    })
}

/// `rgbgradient(duration=1000; colors=0%: #ff0000, 33%: #00ff00)` becomes
/// `gradient 1 s — #FF0000 · #00FF00`
fn describe_gradient(raw: &str) -> Option<String> {
    let inner = raw.strip_prefix("rgbgradient(")?.strip_suffix(')')?;

    let mut duration = None;
    let mut colors: Vec<String> = Vec::new();
    for part in inner.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("duration=") {
            duration = rest.trim().parse::<u64>().ok();
        } else if let Some(rest) = part.strip_prefix("colors=") {
            for stop in rest.split(',') {
                let stop = stop.trim();
                if stop.is_empty() {
                    continue;
                }
                // Extract color part (ignore position)
                let Some((_, color_part)) = stop.split_once(':') else {
                    continue;
                };
                let color_part = color_part.trim();

                // Get the hex color
                let (shown, _) = normalized_color(color_part);
                colors.push(shown);
            }
        }
    }

    let mut text = String::from("gradient");
    if let Some(ms) = duration {
        if ms > 0 && ms % 1000 == 0 {
            let _ = write!(text, " {} s", ms / 1000);
        } else {
            let _ = write!(text, " {ms} ms");
        }
    }
    if !colors.is_empty() {
        text.push_str(" — ");
        text.push_str(&colors.join(" · "));
    }
    Some(text)
}

fn write_buttons(text: &mut String, raw: &str) {
    let Ok(mapping) = ButtonMapping::parse(raw) else {
        let _ = writeln!(text, "Buttons: {raw}");
        return;
    };

    text.push_str("Buttons:\n");
    for button in Button::ALL {
        let _ = writeln!(
            text,
            "  {} → {}",
            button.label(),
            mapping.action(button).label()
        );
    }
}

/// Get firmware version
pub async fn get_firmware_version() -> Result<String> {
    run(&["--firmware-version"], "rivalcfg --firmware-version").await
}

/// True when the Sensei 310 is present on the USB bus.
///
/// Reads the kernel's sysfs view, so it works before any udev rule has been
/// applied and without spawning rivalcfg.
pub fn sensei_connected() -> bool {
    sensei_connected_in(Path::new(USB_DEVICES))
}

/// [`sensei_connected`] against an arbitrary sysfs-like directory.
fn sensei_connected_in(root: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let device = entry.path();
        read_trimmed(&device.join("idVendor")).as_deref() == Some(SENSEI_VENDOR_ID)
            && read_trimmed(&device.join("idProduct")).as_deref() == Some(SENSEI_PRODUCT_ID)
    })
}

/// sysfs values end with a newline; missing files mean "not this device".
fn read_trimmed(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A directory that removes itself, so tests leave nothing behind.
    struct TempTree(PathBuf);

    impl TempTree {
        fn new(label: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("sensei310-tray-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("create temporary tree");
            Self(path)
        }

        fn device(&self, name: &str, vendor: &str, product: &str) -> &Self {
            let device = self.0.join(name);
            std::fs::create_dir_all(&device).expect("create device directory");
            std::fs::write(device.join("idVendor"), format!("{vendor}\n")).expect("write idVendor");
            std::fs::write(device.join("idProduct"), format!("{product}\n"))
                .expect("write idProduct");
            self
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn failure_detail_uses_stderr_when_present() {
        let status = failing_status();
        assert_eq!(
            failure_detail("W: udev rules are not installed.\n", "", &status),
            "W: udev rules are not installed."
        );
    }

    #[test]
    fn failure_detail_falls_back_to_stdout() {
        let status = failing_status();
        assert_eq!(
            failure_detail("", "E: No supported device found.\n", &status),
            "E: No supported device found."
        );
    }

    #[test]
    fn failure_detail_keeps_both_streams() {
        let status = failing_status();
        assert_eq!(
            failure_detail(
                "W: stale udev rules\n",
                "E: No supported device found.\n",
                &status
            ),
            "W: stale udev rules\nE: No supported device found."
        );
    }

    #[test]
    fn failure_detail_reports_the_exit_status_when_silent() {
        let status = failing_status();
        assert!(failure_detail("", "  \n", &status).contains('1'));
    }

    #[test]
    fn finds_the_sensei_among_other_devices() {
        let tree = TempTree::new("sensei-present");
        tree.device("1-1", "0b05", "1866")
            .device("3-1.1.1", SENSEI_VENDOR_ID, SENSEI_PRODUCT_ID)
            .device("3-1.2", "1532", "022a");

        assert!(sensei_connected_in(&tree.0));
    }

    #[test]
    fn ignores_devices_from_other_vendors() {
        let tree = TempTree::new("sensei-absent");
        tree.device("3-1.2", "1532", "022a");

        assert!(!sensei_connected_in(&tree.0));
    }

    #[test]
    fn ignores_interfaces_without_usb_ids() {
        let tree = TempTree::new("sensei-interfaces");
        tree.device("3-1.1.1", SENSEI_VENDOR_ID, SENSEI_PRODUCT_ID);
        std::fs::create_dir_all(tree.0.join("3-1.1.1:1.0")).expect("create interface directory");

        assert!(sensei_connected_in(&tree.0));
    }

    #[test]
    fn missing_sysfs_is_not_an_error() {
        assert!(!sensei_connected_in(Path::new(
            "/nonexistent/sensei310-tray/usb"
        )));
    }

    fn failing_status() -> std::process::ExitStatus {
        std::process::Command::new("sh")
            .args(["-c", "exit 1"])
            .status()
            .expect("run a command that exits 1")
    }
}
