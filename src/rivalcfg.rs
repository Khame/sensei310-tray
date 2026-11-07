//! Wrapper for rivalcfg CLI commands

use std::fmt::Write as _;

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::{debug, error};

use crate::buttons::{Button, ButtonMapping};

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

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
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
        let rendered: Vec<String> = colors.iter().map(|color| color.clone()).collect();
        text.push_str(&rendered.join(" · "));
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
    let _ = writeln!(
        text,
        "Button layout: {}",
        mapping.layout().to_ascii_uppercase()
    );
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
