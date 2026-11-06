//! Configuration persistence for sensei310-tray

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Last used DPI preset
    pub dpi: Option<u32>,
    /// Last used polling rate
    pub polling_rate: Option<u32>,
    /// Last used logo color (hex)
    pub logo_color: Option<String>,
    /// Last used wheel color (hex)
    pub wheel_color: Option<String>,
    /// Window geometry for dialogs (optional)
    pub window_geometry: Option<WindowGeometry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Config {
    /// Get config file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("sensei310-tray");
        Ok(config_dir.join("settings.json"))
    }

    /// Load configuration from disk
    pub async fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).await?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to disk
    pub async fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content).await?;
        Ok(())
    }

    /// Reset to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Update DPI setting
    pub async fn set_dpi(&mut self, dpi: u32) -> Result<()> {
        self.dpi = Some(dpi);
        self.save().await
    }

    /// Update polling rate
    pub async fn set_polling_rate(&mut self, rate: u32) -> Result<()> {
        self.polling_rate = Some(rate);
        self.save().await
    }

    /// Update logo color
    pub async fn set_logo_color(&mut self, color: String) -> Result<()> {
        self.logo_color = Some(color);
        self.save().await
    }

    /// Update wheel color
    pub async fn set_wheel_color(&mut self, color: String) -> Result<()> {
        self.wheel_color = Some(color);
        self.save().await
    }
}