//! Tray icon and menu management using ksni (StatusNotifierItem)

use anyhow::Result;
use ksni::{Handle, MenuItem, Tray, TrayMethods};
use ksni::menu::{StandardItem, SubMenu, Separator};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::kde_dialogs;
use crate::rivalcfg;

/// Tray implementation for ksni
struct TrayImpl {
    config: Arc<Mutex<Config>>,
}

impl Tray for TrayImpl {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        "SteelSeries Sensei 310".into()
    }

    fn icon_name(&self) -> String {
        "input-mouse".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        use ksni::menu::*;
        let mut items: Vec<MenuItem<Self>> = vec![];

        // Device info header (disabled StandardItem)
        items.push(StandardItem {
            label: "SteelSeries Sensei 310".into(),
            enabled: false,
            ..Default::default()
        }.into());

        items.push(MenuItem::Separator);

        // DPI Presets submenu
        let dpi_items = vec![
            ("400", 400),
            ("800", 800),
            ("1600", 1600),
            ("2400", 2400),
            ("3200", 3200),
            ("6400", 6400),
            ("12000", 12000),
        ];
        let mut dpi_submenu = vec![];
        for (label, dpi) in dpi_items {
            let cfg = self.config.clone();
            dpi_submenu.push(StandardItem {
                label: label.into(),
                enabled: true,
                activate: Box::new(move |_: &mut TrayImpl| {
                    let cfg = cfg.clone();
                    let dpi = dpi;
                    tokio::spawn(async move {
                        if let Err(e) = rivalcfg::run_rivalcfg(&["-s", &dpi.to_string(), "-S", &dpi.to_string()]).await {
                            error!("Failed to set DPI: {}", e);
                        }
                        cfg.lock().await.set_dpi(dpi).await.ok();
                    });
                }),
                ..Default::default()
            }.into());
        }
        items.push(SubMenu {
            label: "DPI Presets".into(),
            enabled: true,
            submenu: dpi_submenu,
            ..Default::default()
        }.into());

        // Polling Rate submenu
        let poll_items = vec![
            ("125 Hz", 125),
            ("250 Hz", 250),
            ("500 Hz", 500),
            ("1000 Hz", 1000),
        ];
        let mut poll_submenu = vec![];
        for (label, rate) in poll_items {
            let cfg = self.config.clone();
            poll_submenu.push(StandardItem {
                label: label.into(),
                enabled: true,
                activate: Box::new(move |_: &mut TrayImpl| {
                    let cfg = cfg.clone();
                    let rate = rate;
                    tokio::spawn(async move {
                        if let Err(e) = rivalcfg::run_rivalcfg(&["-p", &rate.to_string()]).await {
                            error!("Failed to set polling rate: {}", e);
                        }
                        cfg.lock().await.set_polling_rate(rate).await.ok();
                    });
                }),
                ..Default::default()
            }.into());
        }
        items.push(SubMenu {
            label: "Polling Rate".into(),
            enabled: true,
            submenu: poll_submenu,
            ..Default::default()
        }.into());

        items.push(MenuItem::Separator);

        // RGB Colors
        let mut rgb_submenu = vec![];

        let cfg = self.config.clone();
        rgb_submenu.push(StandardItem {
            label: "Logo Color…".into(),
            enabled: true,
            activate: Box::new(move |_: &mut TrayImpl| {
                let cfg = cfg.clone();
                tokio::spawn(async move {
                    if let Ok(Some(color)) = kde_dialogs::pick_color("Logo Color").await {
                        let hex = color.to_hex();
                        if let Err(e) = rivalcfg::run_rivalcfg(&["-c", &hex]).await {
                            error!("Failed to set logo color: {}", e);
                        }
                        cfg.lock().await.set_logo_color(hex).await.ok();
                    }
                });
            }),
            ..Default::default()
        }.into());

        let cfg = self.config.clone();
        rgb_submenu.push(StandardItem {
            label: "Wheel Color…".into(),
            enabled: true,
            activate: Box::new(move |_: &mut TrayImpl| {
                let cfg = cfg.clone();
                tokio::spawn(async move {
                    if let Ok(Some(color)) = kde_dialogs::pick_color("Wheel Color").await {
                        let hex = color.to_hex();
                        if let Err(e) = rivalcfg::run_rivalcfg(&["-C", &hex]).await {
                            error!("Failed to set wheel color: {}", e);
                        }
                        cfg.lock().await.set_wheel_color(hex).await.ok();
                    }
                });
            }),
            ..Default::default()
        }.into());

        rgb_submenu.push(MenuItem::Separator);

        // Gradient presets
        let gradients = vec![
            ("Rainbow", "rgbgradient(duration=5000; colors=0%:red,33%:lime,66%:blue)"),
            ("Fire", "rgbgradient(duration=3000; colors=0%:red,50%:yellow,100%:white)"),
            ("Ocean", "rgbgradient(duration=4000; colors=0%:blue,50%:cyan,100%:white)"),
            ("Sunset", "rgbgradient(duration=6000; colors=0%:purple,50%:orange,100%:pink)"),
            ("Matrix", "rgbgradient(duration=2000; colors=0%:green,100%:lime)"),
        ];
        for (label, gradient) in gradients {
            let g = gradient.to_string();
            rgb_submenu.push(StandardItem {
                label: label.into(),
                enabled: true,
                activate: Box::new(move |_: &mut TrayImpl| {
                    let g = g.clone();
                    tokio::spawn(async move {
                        if let Err(e) = rivalcfg::run_rivalcfg(&["-c", &g, "-C", &g]).await {
                            error!("Failed to set gradient: {}", e);
                        }
                    });
                }),
                ..Default::default()
            }.into());
        }

        items.push(SubMenu {
            label: "RGB Colors".into(),
            enabled: true,
            submenu: rgb_submenu,
            ..Default::default()
        }.into());

        items.push(MenuItem::Separator);

        // Reset to Defaults
        let cfg = self.config.clone();
        items.push(StandardItem {
            label: "Reset to Defaults".into(),
            enabled: true,
            activate: Box::new(move |_: &mut TrayImpl| {
                let cfg = cfg.clone();
                tokio::spawn(async move {
                    if let Err(e) = rivalcfg::run_rivalcfg(&["-r"]).await {
                        error!("Failed to reset: {}", e);
                    }
                    cfg.lock().await.reset();
                });
            }),
            ..Default::default()
        }.into());

        items.push(MenuItem::Separator);

        // Quit
        items.push(StandardItem {
            label: "Quit".into(),
            enabled: true,
            icon_name: "application-exit".into(),
            activate: Box::new(|_: &mut TrayImpl| {
                std::process::exit(0);
            }),
            ..Default::default()
        }.into());

        items
    }
}

/// Controller wrapper
pub struct TrayController {
    handle: Handle<TrayImpl>,
}

impl TrayController {
    pub async fn new(config: Arc<Mutex<Config>>) -> Result<Self> {
        let tray_impl = TrayImpl { config };
        let handle = tray_impl.spawn().await?;

        info!("Tray icon created successfully via ksni");
        Ok(Self { handle })
    }

    pub async fn run(self) -> Result<()> {
        tokio::signal::ctrl_c().await?;
        info!("Shutdown signal received");
        Ok(())
    }
}