// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Mapping snapshots and the apply service shared by the tray and the CLI.

use anyhow::Result;

use crate::buttons::{Action, Button, ButtonMapping};
use crate::config::Config;

const SAVED_PROFILE_WARNING: &str =
    "Saved rivalcfg profile reflects the last write, not the live mouse state.";
const TRAY_CONFIG_WARNING: &str =
    "No saved button mapping was found. These values come from this tray.";
const JUST_WRITTEN_WARNING: &str = "These values were just written by this tray.";
const DEFAULTS_WARNING: &str =
    "No saved button mapping was found. Showing the device factory defaults.";

/// Where the current mapping came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingSource {
    SavedProfile,
    TrayConfig,
    Defaults,
}

/// One button row in the snapshot.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ButtonState {
    pub id: String,
    pub label: String,
    pub location: String,
    pub action: String,
    pub action_label: String,
}

/// One selectable action in the snapshot.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ActionOption {
    pub id: String,
    pub label: String,
    pub description: String,
}

/// Self-describing view of the current mapping.
#[derive(Clone, Debug, serde::Serialize)]
pub struct MappingSnapshot {
    #[serde(skip)]
    pub mapping: ButtonMapping,
    pub buttons: Vec<ButtonState>,
    pub actions: Vec<ActionOption>,
    pub layout: String,
    pub source: MappingSource,
    pub profile: Option<String>,
    pub raw: String,
    pub warning: Option<String>,
}

/// Resolve the mapping from the saved profile, then the tray config.
pub fn from_sources(settings: Option<&serde_json::Value>, config: Option<&str>) -> MappingSnapshot {
    let mut saved_warning = None;

    if let Some(settings) = settings {
        if let Some((profile, raw)) = profile_mapping_string(settings) {
            match ButtonMapping::parse(&raw) {
                Ok(mapping) => {
                    return build(
                        mapping,
                        MappingSource::SavedProfile,
                        Some(profile),
                        Some(SAVED_PROFILE_WARNING.to_string()),
                    );
                }
                Err(error) => {
                    saved_warning = Some(format!(
                        "Saved rivalcfg profile \"{profile}\" could not be parsed: {error:#}"
                    ));
                }
            }
        }
    }

    if let Some(raw) = config {
        match ButtonMapping::parse(raw) {
            Ok(mapping) => {
                return build(
                    mapping,
                    MappingSource::TrayConfig,
                    None,
                    Some(join_warnings(saved_warning, TRAY_CONFIG_WARNING)),
                );
            }
            Err(error) => {
                let defaults =
                    format!("{DEFAULTS_WARNING} The tray mapping could not be parsed: {error:#}");
                return build(
                    ButtonMapping::default(),
                    MappingSource::Defaults,
                    None,
                    Some(join_warnings(saved_warning, &defaults)),
                );
            }
        }
    }

    build(
        ButtonMapping::default(),
        MappingSource::Defaults,
        None,
        Some(join_warnings(saved_warning, DEFAULTS_WARNING)),
    )
}

/// Read the saved profile and resolve the current mapping.
pub async fn load(config: &Config) -> MappingSnapshot {
    let settings = match crate::rivalcfg::get_saved_settings().await {
        Ok(settings) => settings,
        Err(error) => {
            tracing::warn!("Could not read the saved Rivalcfg profile: {error:#}");
            None
        }
    };
    from_sources(settings.as_ref(), config.buttons_mapping.as_deref())
}

/// Apply one assignment and return the resulting snapshot.
///
/// This writes to the mouse. Do not call it from unit tests.
pub async fn apply(config: &mut Config, button: Button, action: Action) -> Result<MappingSnapshot> {
    let current = load(config).await;
    let mapping = ButtonMapping::merge(Some(&current.raw), button, action);
    let raw = mapping.to_rivalcfg();
    crate::rivalcfg::run_rivalcfg(&["-b", &raw]).await?;
    config.buttons_mapping = Some(raw.clone());
    config.save().await?;
    Ok(snapshot_from_raw(&raw))
}

/// Snapshot for a mapping this process has just written.
pub fn snapshot_from_raw(raw: &str) -> MappingSnapshot {
    match ButtonMapping::parse(raw) {
        Ok(mapping) => build(
            mapping,
            MappingSource::TrayConfig,
            None,
            Some(JUST_WRITTEN_WARNING.to_string()),
        ),
        Err(error) => {
            tracing::warn!("Could not parse the mapping that was just written: {error:#}");
            build(
                ButtonMapping::default(),
                MappingSource::Defaults,
                None,
                Some(join_warnings(
                    None,
                    &format!("The mapping that was just written could not be parsed: {error:#}"),
                )),
            )
        }
    }
}

fn build(
    mapping: ButtonMapping,
    source: MappingSource,
    profile: Option<String>,
    warning: Option<String>,
) -> MappingSnapshot {
    let buttons = Button::ALL
        .iter()
        .map(|button| {
            let action = mapping.action(*button);
            ButtonState {
                id: button.id().to_string(),
                label: button.label(),
                location: button.location().to_string(),
                action: action.id().to_string(),
                action_label: action.label(),
            }
        })
        .collect();

    let actions = mapping
        .actions()
        .into_iter()
        .map(|action| ActionOption {
            id: action.id().to_string(),
            label: action.label(),
            description: action.description(),
        })
        .collect();

    MappingSnapshot {
        buttons,
        actions,
        layout: mapping.layout().to_string(),
        raw: mapping.to_rivalcfg(),
        source,
        profile,
        warning,
        mapping,
    }
}

/// Profile name and raw mapping, preferring the `default` profile.
fn profile_mapping_string(settings: &serde_json::Value) -> Option<(String, String)> {
    let profiles = settings.as_object()?;

    if let Some(raw) = profiles
        .get("default")
        .and_then(|profile| profile.get("buttons_mapping"))
        .and_then(|value| value.as_str())
    {
        return Some((String::from("default"), raw.to_string()));
    }

    profiles.iter().find_map(|(name, profile)| {
        let raw = profile.get("buttons_mapping")?.as_str()?;
        Some((name.clone(), raw.to_string()))
    })
}

fn join_warnings(first: Option<String>, second: &str) -> String {
    match first {
        Some(first) => format!("{first} {second}"),
        None => second.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn settings_with_default() -> serde_json::Value {
        json!({
            "default": {
                "buttons_mapping": "buttons(button1=button2; button2=button2; button3=button3; button4=button4; button5=button5; button6=PageDown; button7=PageUp; button8=dpi; layout=qwerty)"
            },
            "other": {
                "buttons_mapping": "buttons(button1=disabled; layout=qwerty)"
            }
        })
    }

    #[test]
    fn prefers_the_default_profile() {
        let snapshot = from_sources(Some(&settings_with_default()), None);
        assert_eq!(snapshot.source, MappingSource::SavedProfile);
        assert_eq!(snapshot.profile.as_deref(), Some("default"));
        assert_eq!(snapshot.buttons[0].action, "button2");
    }

    #[test]
    fn falls_back_to_the_first_profile_with_a_mapping() {
        let settings = json!({
            "other": {"buttons_mapping": "buttons(button1=disabled; layout=qwerty)"}
        });
        let snapshot = from_sources(Some(&settings), None);
        assert_eq!(snapshot.profile.as_deref(), Some("other"));
        assert_eq!(snapshot.buttons[0].action, "disabled");
    }

    #[test]
    fn snapshot_uses_saved_profile_first() {
        let settings = settings_with_default();
        let snapshot = from_sources(
            Some(&settings),
            Some("buttons(button1=disabled; layout=qwerty)"),
        );
        assert_eq!(snapshot.source, MappingSource::SavedProfile);
        assert_eq!(snapshot.profile.as_deref(), Some("default"));
        assert_eq!(snapshot.buttons.len(), 8);
        assert_eq!(snapshot.actions[0].id, "button1");
    }

    #[test]
    fn snapshot_falls_back_to_tray_config() {
        let snapshot = from_sources(None, Some("buttons(button1=disabled; layout=qwerty)"));
        assert_eq!(snapshot.source, MappingSource::TrayConfig);
        assert_eq!(snapshot.buttons[0].action, "disabled");
    }

    #[test]
    fn snapshot_falls_back_to_defaults() {
        let snapshot = from_sources(None, None);
        assert_eq!(snapshot.source, MappingSource::Defaults);
        assert_eq!(snapshot.buttons[7].action, "dpi");
    }

    #[test]
    fn malformed_saved_mapping_falls_back_with_a_warning() {
        let settings = json!({"default": {"buttons_mapping": "not-a-mapping"}});
        let snapshot = from_sources(
            Some(&settings),
            Some("buttons(button1=disabled; layout=qwerty)"),
        );
        assert_eq!(snapshot.source, MappingSource::TrayConfig);
        assert!(snapshot.warning.is_some());
    }

    #[test]
    fn snapshot_serializes_to_stable_json() {
        let snapshot = from_sources(None, None);
        let value = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(value["source"], "defaults");
        assert_eq!(value["buttons"][0]["id"], "button1");
        assert_eq!(value["buttons"][0]["location"], "left main button");
        assert_eq!(value["actions"][0]["id"], "button1");
        assert!(value["actions"][0]["description"].as_str().is_some());
        assert!(value.get("mapping").is_none());
    }

    #[test]
    fn applied_snapshot_uses_the_new_mapping() {
        let raw = "buttons(button1=disabled; button2=button2; button3=button3; button4=button4; button5=button5; button6=PageDown; button7=PageUp; button8=dpi; layout=qwerty)";
        let snapshot = snapshot_from_raw(raw);
        assert_eq!(snapshot.source, MappingSource::TrayConfig);
        assert_eq!(snapshot.raw, raw);
        assert_eq!(snapshot.buttons[0].action, "disabled");
        assert_eq!(
            snapshot.warning.as_deref(),
            Some("These values were just written by this tray.")
        );
    }
}
