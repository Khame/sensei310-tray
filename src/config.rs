// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Configuration persistence: named profiles of mouse settings.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

/// Profile that always exists and is used when nothing else is selected.
pub const DEFAULT_PROFILE: &str = "default";

/// Longest accepted profile name, in characters.
const MAX_NAME_LENGTH: usize = 32;

/// Returned by [`Config::active_profile`] when the active profile is missing.
const EMPTY_PROFILE: Profile = Profile {
    dpi1: None,
    dpi2: None,
    polling_rate: None,
    logo_color: None,
    wheel_color: None,
    buttons_mapping: None,
};

/// Mouse settings that belong to one profile.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// Last used DPI preset 1
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dpi1: Option<u32>,
    /// Last used DPI preset 2
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dpi2: Option<u32>,
    /// Last used polling rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polling_rate: Option<u32>,
    /// Last used logo color (hex)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_color: Option<String>,
    /// Last used wheel color (hex)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wheel_color: Option<String>,
    /// Last used button mapping (rivalcfg format)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buttons_mapping: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Profile loaded at startup and written by the menu.
    #[serde(default = "default_profile_name")]
    pub active: String,
    /// Settings per profile name, sorted by name.
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_geometry: Option<WindowGeometry>,
}

fn default_profile_name() -> String {
    DEFAULT_PROFILE.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            active: DEFAULT_PROFILE.to_string(),
            profiles: BTreeMap::from([(DEFAULT_PROFILE.to_string(), Profile::default())]),
            window_geometry: None,
        }
    }
}

/// The flat format written by sensei310-tray 1.0.4 and older.
#[derive(Debug, Deserialize)]
struct LegacyConfig {
    #[serde(default)]
    dpi1: Option<u32>,
    #[serde(default)]
    dpi2: Option<u32>,
    #[serde(default)]
    polling_rate: Option<u32>,
    #[serde(default)]
    logo_color: Option<String>,
    #[serde(default)]
    wheel_color: Option<String>,
    #[serde(default)]
    buttons_mapping: Option<String>,
    #[serde(default)]
    window_geometry: Option<WindowGeometry>,
}

impl LegacyConfig {
    /// Move the flat settings into the `default` profile.
    fn into_config(self) -> Config {
        let profile = Profile {
            dpi1: self.dpi1,
            dpi2: self.dpi2,
            polling_rate: self.polling_rate,
            logo_color: self.logo_color,
            wheel_color: self.wheel_color,
            buttons_mapping: self.buttons_mapping,
        };
        Config {
            active: DEFAULT_PROFILE.to_string(),
            profiles: BTreeMap::from([(DEFAULT_PROFILE.to_string(), profile)]),
            window_geometry: self.window_geometry,
        }
    }
}

impl Config {
    /// Get config file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("sensei310-tray");
        Ok(config_dir.join("settings.json"))
    }

    /// Load configuration from disk, migrating the flat format if needed.
    pub async fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).await?;
        Self::parse(&content)
    }

    /// Parse a settings file, accepting both the profile format and the old
    /// flat one.
    pub fn parse(raw: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(raw)?;
        let mut config = if value.get("profiles").is_some() {
            serde_json::from_value::<Config>(value)?
        } else {
            serde_json::from_value::<LegacyConfig>(value)?.into_config()
        };
        config.normalize();
        Ok(config)
    }

    /// Save configuration to disk
    pub async fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut config = self.clone();
        config.normalize();
        let content = serde_json::to_string_pretty(&config)?;
        // Replace the file atomically so interrupted writes cannot truncate it.
        let temporary_path = path.with_extension("json.tmp");
        fs::write(&temporary_path, content).await?;
        fs::rename(&temporary_path, &path).await?;
        Ok(())
    }

    /// Make sure the default profile exists and the active name resolves.
    fn normalize(&mut self) {
        self.profiles
            .entry(DEFAULT_PROFILE.to_string())
            .or_default();
        if !self.profiles.contains_key(&self.active) {
            self.active = DEFAULT_PROFILE.to_string();
        }
    }

    /// Settings of the active profile.
    pub fn active_profile(&self) -> &Profile {
        self.profiles.get(&self.active).unwrap_or(&EMPTY_PROFILE)
    }

    /// Settings of the active profile, created empty when it is missing.
    pub fn active_profile_mut(&mut self) -> &mut Profile {
        let active = self.active.clone();
        self.profiles.entry(active).or_default()
    }

    /// Profile names in menu order.
    pub fn names(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }

    /// Select an existing profile. Returns false when the name is unknown.
    pub fn select(&mut self, name: &str) -> bool {
        if self.profiles.contains_key(name) {
            self.active = name.to_string();
            true
        } else {
            false
        }
    }

    /// Create a profile as a copy of the active one and select it.
    pub fn create(&mut self, name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            bail!("Profile name cannot be empty.");
        }
        if name.chars().count() > MAX_NAME_LENGTH {
            bail!("Profile name cannot be longer than {MAX_NAME_LENGTH} characters.");
        }
        if name.chars().any(char::is_control) {
            bail!("Profile name cannot contain control characters.");
        }
        if self
            .profiles
            .keys()
            .any(|existing| existing.eq_ignore_ascii_case(name))
        {
            bail!("Profile \"{name}\" already exists.");
        }

        self.profiles
            .insert(name.to_string(), self.active_profile().clone());
        self.active = name.to_string();
        Ok(())
    }

    /// Delete an inactive profile.
    pub fn delete(&mut self, name: &str) -> Result<()> {
        if name == DEFAULT_PROFILE {
            bail!("The default profile cannot be deleted.");
        }
        if !self.profiles.contains_key(name) {
            bail!("Profile \"{name}\" does not exist.");
        }
        if name == self.active {
            bail!("Switch to another profile before deleting \"{name}\".");
        }
        if self.profiles.len() <= 1 {
            bail!("At least one profile must remain.");
        }

        self.profiles.remove(name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_json() -> &'static str {
        r##"{
            "dpi1": 800,
            "dpi2": 1600,
            "polling_rate": 1000,
            "logo_color": "#ff0000",
            "wheel_color": "#00ff00",
            "buttons_mapping": "buttons(button1=button1; layout=qwerty)"
        }"##
    }

    #[test]
    fn default_config_has_one_default_profile() {
        let config = Config::default();
        assert_eq!(config.active, DEFAULT_PROFILE);
        assert_eq!(config.names(), vec![DEFAULT_PROFILE.to_string()]);
        assert_eq!(config.active_profile(), &Profile::default());
    }

    #[test]
    fn parses_the_legacy_flat_format_into_the_default_profile() {
        let config = Config::parse(legacy_json()).unwrap();
        assert_eq!(config.active, DEFAULT_PROFILE);
        let profile = config.active_profile();
        assert_eq!(profile.dpi1, Some(800));
        assert_eq!(profile.polling_rate, Some(1000));
        assert_eq!(profile.logo_color.as_deref(), Some("#ff0000"));
        assert_eq!(
            profile.buttons_mapping.as_deref(),
            Some("buttons(button1=button1; layout=qwerty)")
        );
    }

    #[test]
    fn parses_named_profiles() {
        let raw = r#"{
            "active": "gaming",
            "profiles": {
                "default": {"dpi1": 800},
                "gaming": {"dpi1": 3200, "polling_rate": 1000}
            }
        }"#;
        let config = Config::parse(raw).unwrap();
        assert_eq!(config.active, "gaming");
        assert_eq!(config.active_profile().dpi1, Some(3200));
        assert_eq!(
            config.names(),
            vec!["default".to_string(), "gaming".to_string()]
        );
    }

    #[test]
    fn adds_the_default_profile_when_it_is_missing() {
        let raw = r#"{"active": "gaming", "profiles": {"gaming": {"dpi1": 3200}}}"#;
        let config = Config::parse(raw).unwrap();
        assert!(config.profiles.contains_key(DEFAULT_PROFILE));
    }

    #[test]
    fn falls_back_to_default_when_active_is_missing() {
        let raw = r#"{"active": "gone", "profiles": {"default": {"dpi1": 800}}}"#;
        let config = Config::parse(raw).unwrap();
        assert_eq!(config.active, DEFAULT_PROFILE);
    }

    #[test]
    fn create_copies_the_active_profile() {
        let mut config = Config::default();
        config.active_profile_mut().dpi1 = Some(3200);
        config.create("gaming").unwrap();
        assert_eq!(config.active, "gaming");
        assert_eq!(config.active_profile().dpi1, Some(3200));
        assert_eq!(config.profiles["default"].dpi1, Some(3200));
    }

    #[test]
    fn create_rejects_duplicate_names_ignoring_case() {
        let mut config = Config::default();
        let error = config.create("Default").unwrap_err();
        assert!(error.to_string().contains("already exists"));
    }

    #[test]
    fn create_rejects_blank_and_overlong_names() {
        let mut config = Config::default();
        assert!(config.create("   ").is_err());
        assert!(config.create(&"x".repeat(MAX_NAME_LENGTH + 1)).is_err());
    }

    #[test]
    fn delete_refuses_default_and_active_profiles() {
        let mut config = Config::default();
        config.create("gaming").unwrap();
        assert!(config.delete(DEFAULT_PROFILE).is_err());
        assert!(config.delete("gaming").is_err());
        assert!(config.delete("missing").is_err());
    }

    #[test]
    fn delete_removes_an_inactive_profile() {
        let mut config = Config::default();
        config.create("gaming").unwrap();
        config.select(DEFAULT_PROFILE);
        config.delete("gaming").unwrap();
        assert_eq!(config.names(), vec![DEFAULT_PROFILE.to_string()]);
    }

    #[test]
    fn select_ignores_unknown_names() {
        let mut config = Config::default();
        assert!(!config.select("gone"));
        assert_eq!(config.active, DEFAULT_PROFILE);
    }

    #[test]
    fn serializes_named_profiles_and_omits_unset_values() {
        let mut config = Config::default();
        config.create("gaming").unwrap();
        config.active_profile_mut().dpi1 = Some(3200);

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"active\":\"gaming\""));
        assert!(json.contains("\"profiles\""));
        assert!(json.contains("\"default\""));
        assert!(!json.contains("polling_rate"));

        let parsed = Config::parse(&json).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn resets_only_the_active_profile() {
        let mut config = Config::default();
        config.active_profile_mut().dpi1 = Some(800);
        config.create("gaming").unwrap();
        *config.active_profile_mut() = Profile::default();
        assert_eq!(config.active_profile(), &Profile::default());
        assert_eq!(config.profiles[DEFAULT_PROFILE].dpi1, Some(800));
    }
}
