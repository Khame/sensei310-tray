// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! KDE-native dialogs via kdialog (no GUI crate dependencies)

use std::fmt::Write as _;

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::{debug, error, warn};

use crate::buttons::{Action, Button, ButtonMapping};

/// Window icon for every dialog, so the title bar shows the mouse instead of
/// kdialog's own icon.
const DIALOG_ICON: &str = "input-mouse";

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Turn a non-success kdialog exit into an error. `name` is the command as logged.
fn kdialog_error(output: &std::process::Output, name: &str) -> anyhow::Error {
    match output.status.code() {
        Some(code) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("{name} failed with exit code {code}: {stderr}");
            anyhow::anyhow!("{name} failed with exit code {code}: {stderr}")
        }
        None => {
            error!("kdialog terminated by signal");
            anyhow::anyhow!("kdialog terminated by signal")
        }
    }
}

/// Pick a color using kdialog --getcolor
/// Returns None if user cancelled, Some(color) if picked, Err if kdialog failed
pub async fn pick_color(title: &str) -> Result<Option<RgbColor>> {
    debug!("Opening color dialog via kdialog: {}", title);

    let output = Command::new("kdialog")
        .args(["--getcolor", "--title", title, "--icon", DIALOG_ICON])
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
        _ => Err(kdialog_error(&output, "kdialog")),
    }
}

/// Show error message via kdialog --error
pub async fn show_error(title: &str, message: &str) -> Result<()> {
    debug!("Showing error dialog: {} - {}", title, message);
    let output = Command::new("kdialog")
        .args(["--error", message, "--title", title, "--icon", DIALOG_ICON])
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
        .args(["--msgbox", message, "--title", title, "--icon", DIALOG_ICON])
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

    if s.len() != 6 || !s.is_ascii() {
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

/// One row in the kdialog action list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionListEntry {
    pub tag: String,
    pub label: String,
    pub checked: bool,
}

/// Build the action list for one button with the current action checked.
pub fn action_list_entries(button: Button, mapping: &ButtonMapping) -> Vec<ActionListEntry> {
    let current = mapping.action(button);
    mapping
        .actions()
        .into_iter()
        .map(|action| ActionListEntry {
            tag: action.id().to_string(),
            label: format!("{} ({})", action.label(), action.description()),
            checked: &action == current,
        })
        .collect()
}

/// Ask the user for a new action with kdialog --radiolist.
///
/// Returns `Ok(None)` when the user cancels.
pub async fn pick_action(button: Button, mapping: &ButtonMapping) -> Result<Option<Action>> {
    let entries = action_list_entries(button, mapping);
    let mut args: Vec<String> = vec![
        String::from("--title"),
        format!("Assign {}", button.label()),
        String::from("--icon"),
        String::from(DIALOG_ICON),
        String::from("--radiolist"),
        format!(
            "Choose an action for {} ({}).",
            button.label(),
            button.location()
        ),
    ];
    for entry in entries {
        args.push(entry.tag);
        args.push(entry.label);
        args.push(String::from(if entry.checked { "on" } else { "off" }));
    }

    debug!("Opening action picker for {}", button.label());
    let output = Command::new("kdialog")
        .args(&args)
        .output()
        .await
        .context("Failed to execute kdialog. Is kdialog installed?")?;

    match output.status.code() {
        Some(0) => {
            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if tag.is_empty() {
                debug!("Action picker returned an empty tag");
                return Ok(None);
            }
            debug!("Action picked for {}: {}", button.label(), tag);
            Ok(Some(Action::parse(&tag)))
        }
        Some(1) => {
            debug!("Action picker cancelled by user");
            Ok(None)
        }
        _ => Err(kdialog_error(&output, "kdialog --radiolist")),
    }
}

/// Ask for one line of text with kdialog --inputbox.
///
/// Returns `Ok(None)` when the user cancels or leaves the field empty.
pub async fn ask_text(title: &str, prompt: &str, initial: &str) -> Result<Option<String>> {
    debug!("Opening text dialog via kdialog: {}", title);
    let output = Command::new("kdialog")
        .args([
            "--title",
            title,
            "--icon",
            DIALOG_ICON,
            "--inputbox",
            prompt,
            initial,
        ])
        .output()
        .await
        .context("Failed to execute kdialog. Is kdialog installed?")?;

    match output.status.code() {
        Some(0) => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() {
                debug!("Text dialog returned an empty value");
                Ok(None)
            } else {
                Ok(Some(text))
            }
        }
        Some(1) => {
            debug!("Text dialog cancelled by user");
            Ok(None)
        }
        _ => Err(kdialog_error(&output, "kdialog --inputbox")),
    }
}

/// Ask a yes/no question with kdialog --yesno. A cancel counts as no.
pub async fn confirm(title: &str, message: &str) -> Result<bool> {
    debug!("Opening confirmation dialog: {}", title);
    let output = Command::new("kdialog")
        .args(["--title", title, "--icon", DIALOG_ICON, "--yesno", message])
        .output()
        .await
        .context("Failed to execute kdialog. Is kdialog installed?")?;

    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(kdialog_error(&output, "kdialog --yesno")),
    }
}

/// Pick one profile name with kdialog --radiolist. The first entry is
/// preselected; the list must not be empty.
///
/// Returns `Ok(None)` when the user cancels.
pub async fn pick_profile(title: &str, prompt: &str, names: &[String]) -> Result<Option<String>> {
    let mut args: Vec<String> = vec![
        String::from("--title"),
        title.to_string(),
        String::from("--icon"),
        String::from(DIALOG_ICON),
        String::from("--radiolist"),
        prompt.to_string(),
    ];
    for (index, name) in names.iter().enumerate() {
        args.push(name.clone());
        args.push(name.clone());
        args.push(String::from(if index == 0 { "on" } else { "off" }));
    }

    debug!("Opening profile picker with {} entries", names.len());
    let output = Command::new("kdialog")
        .args(&args)
        .output()
        .await
        .context("Failed to execute kdialog. Is kdialog installed?")?;

    match output.status.code() {
        Some(0) => {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name))
            }
        }
        Some(1) => {
            debug!("Profile picker cancelled by user");
            Ok(None)
        }
        _ => Err(kdialog_error(&output, "kdialog --radiolist")),
    }
}

/// Show a short passive popup after a successful assignment.
pub async fn notify_applied(button: Button, action: &Action) -> Result<()> {
    let message = format!("{}: {}", button.label(), action.label());
    debug!("Showing confirmation: {}", message);
    let output = Command::new("kdialog")
        .args([
            "--title",
            "Sensei 310 Tray",
            "--icon",
            DIALOG_ICON,
            "--passivepopup",
            message.as_str(),
            "3",
        ])
        .output()
        .await
        .context("Failed to execute kdialog --passivepopup")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("kdialog --passivepopup failed: {}", stderr);
    }
    Ok(())
}

/// Text for the dialog that explains where each physical button is.
pub fn button_layout_text() -> String {
    let mut text = String::from("Physical buttons on the Sensei 310:\n\n");
    for button in Button::ALL {
        let _ = writeln!(text, "{}: {}", button.label(), button.location());
    }
    text.push_str("\nThe left side is the thumb side for right-handed use.");
    text
}

/// Show where each physical button is.
pub async fn show_button_layout() -> Result<()> {
    show_info("Button Layout", &button_layout_text()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors() {
        assert_eq!(
            parse_kdialog_color(" #00aBff\n").unwrap(),
            RgbColor {
                r: 0,
                g: 171,
                b: 255
            }
        );
        assert_eq!(parse_kdialog_color("123456").unwrap().to_hex(), "#123456");
    }

    #[test]
    fn rejects_invalid_colors_without_panicking() {
        for input in ["", "#123", "#GG0000", "aé123", "ééé", "#1234567"] {
            assert!(parse_kdialog_color(input).is_err(), "accepted {input:?}");
        }
    }

    #[test]
    fn action_entries_show_descriptions_and_current_choice() {
        let mapping = ButtonMapping::default();
        let entries = action_list_entries(Button::Button1, &mapping);
        let dpi = entries.iter().find(|entry| entry.tag == "dpi").unwrap();
        assert!(dpi.label.contains("DPI"));
        assert!(dpi.label.contains("preset"));
        assert!(!dpi.checked);
        let left = entries.iter().find(|entry| entry.tag == "button1").unwrap();
        assert!(left.checked);
    }

    #[test]
    fn action_entries_include_an_unknown_current_action() {
        let mapping = ButtonMapping::parse(
            "buttons(button1=macro1; button2=button2; button3=button3; button4=button4; button5=button5; button6=PageDown; button7=PageUp; button8=dpi; layout=qwerty)",
        )
        .unwrap();
        let entries = action_list_entries(Button::Button1, &mapping);
        let unknown = entries.iter().find(|entry| entry.tag == "macro1").unwrap();
        assert!(unknown.checked);
        assert!(unknown.label.contains("unknown"));
    }

    #[test]
    fn layout_text_lists_every_button_and_location() {
        let text = button_layout_text();
        for button in Button::ALL {
            let label = button.label();
            assert!(text.contains(label.as_str()), "missing {label}");
            assert!(
                text.contains(button.location()),
                "missing {}",
                button.location()
            );
        }
    }
}
