// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Tray icon and menu management using ksni (StatusNotifierItem)

use anyhow::Result;
use ksni::menu::{StandardItem, SubMenu};
use ksni::{Handle, MenuItem, Tray, TrayMethods};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::buttons::{Action, Button, ButtonMapping};
use crate::config::Config;
use crate::kde_dialogs;
use crate::mapping;
use crate::rivalcfg;

/// Tray implementation for ksni
struct TrayImpl {
    config: Arc<Mutex<Config>>,
    mapping: ButtonMapping,
    handle: Option<Handle<TrayImpl>>,
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
        let mut items: Vec<MenuItem<Self>> = vec![];

        // Device info header (disabled StandardItem)
        items.push(
            StandardItem {
                label: "SteelSeries Sensei 310".into(),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // Device information
        items.push(
            StandardItem {
                label: "Device Info…".into(),
                enabled: true,
                activate: Box::new(|_: &mut TrayImpl| {
                    tokio::spawn(async {
                        show_device_info().await;
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // DPI Preset 1 submenu
        let dpi_items = vec![
            ("400", 400),
            ("800", 800),
            ("1600", 1600),
            ("2400", 2400),
            ("3200", 3200),
            ("6400", 6400),
            ("12000", 12000),
        ];
        let mut dpi1_submenu = vec![];
        for (label, dpi) in dpi_items {
            let label = label.to_string();
            let cfg = self.config.clone();
            dpi1_submenu.push(
                StandardItem {
                    label,
                    enabled: true,
                    activate: Box::new(move |_: &mut TrayImpl| {
                        let cfg = cfg.clone();
                        tokio::spawn(async move {
                            apply_setting(
                                cfg,
                                vec!["-s".into(), dpi.to_string()],
                                Setting::Dpi1(dpi),
                            )
                            .await;
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }
        items.push(
            SubMenu {
                label: "DPI Preset 1".into(),
                enabled: true,
                submenu: dpi1_submenu,
                ..Default::default()
            }
            .into(),
        );

        // DPI Preset 2 submenu
        let dpi_items_2 = vec![
            ("400", 400),
            ("800", 800),
            ("1600", 1600),
            ("2400", 2400),
            ("3200", 3200),
            ("6400", 6400),
            ("12000", 12000),
        ];
        let mut dpi2_submenu = vec![];
        for (label, dpi) in dpi_items_2 {
            let label = label.to_string();
            let cfg = self.config.clone();
            dpi2_submenu.push(
                StandardItem {
                    label,
                    enabled: true,
                    activate: Box::new(move |_: &mut TrayImpl| {
                        let cfg = cfg.clone();
                        tokio::spawn(async move {
                            apply_setting(
                                cfg,
                                vec!["-S".into(), dpi.to_string()],
                                Setting::Dpi2(dpi),
                            )
                            .await;
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }
        items.push(
            SubMenu {
                label: "DPI Preset 2".into(),
                enabled: true,
                submenu: dpi2_submenu,
                ..Default::default()
            }
            .into(),
        );

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
            poll_submenu.push(
                StandardItem {
                    label: label.into(),
                    enabled: true,
                    activate: Box::new(move |_: &mut TrayImpl| {
                        let cfg = cfg.clone();
                        tokio::spawn(async move {
                            apply_setting(
                                cfg,
                                vec!["-p".into(), rate.to_string()],
                                Setting::PollingRate(rate),
                            )
                            .await;
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }
        items.push(
            SubMenu {
                label: "Polling Rate".into(),
                enabled: true,
                submenu: poll_submenu,
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // RGB Colors
        let mut rgb_submenu = vec![];

        let cfg = self.config.clone();
        rgb_submenu.push(
            StandardItem {
                label: "Logo Color…".into(),
                enabled: true,
                activate: Box::new(move |_: &mut TrayImpl| {
                    let cfg = cfg.clone();
                    tokio::spawn(async move {
                        match kde_dialogs::pick_color("Logo Color").await {
                            Ok(Some(color)) => {
                                let hex = color.to_hex();
                                apply_setting(
                                    cfg,
                                    vec!["-c".into(), hex.clone()],
                                    Setting::LogoColor(hex),
                                )
                                .await;
                            }
                            Ok(None) => {}
                            Err(error) => report_error(error).await,
                        }
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        let cfg = self.config.clone();
        rgb_submenu.push(
            StandardItem {
                label: "Wheel Color…".into(),
                enabled: true,
                activate: Box::new(move |_: &mut TrayImpl| {
                    let cfg = cfg.clone();
                    tokio::spawn(async move {
                        match kde_dialogs::pick_color("Wheel Color").await {
                            Ok(Some(color)) => {
                                let hex = color.to_hex();
                                apply_setting(
                                    cfg,
                                    vec!["-C".into(), hex.clone()],
                                    Setting::WheelColor(hex),
                                )
                                .await;
                            }
                            Ok(None) => {}
                            Err(error) => report_error(error).await,
                        }
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        rgb_submenu.push(MenuItem::Separator);

        // Gradient presets
        let gradients = vec![
            (
                "Rainbow",
                "rgbgradient(duration=5000; colors=0%:red,33%:lime,66%:blue)",
            ),
            (
                "Fire",
                "rgbgradient(duration=3000; colors=0%:red,50%:yellow,100%:white)",
            ),
            (
                "Ocean",
                "rgbgradient(duration=4000; colors=0%:blue,50%:#00FFFF,100%:white)",
            ),
            (
                "Sunset",
                "rgbgradient(duration=6000; colors=0%:purple,50%:#FFA500,100%:#FFC0CB)",
            ),
            (
                "Matrix",
                "rgbgradient(duration=2000; colors=0%:green,100%:lime)",
            ),
        ];
        for (label, gradient) in gradients {
            let g = gradient.to_string();
            let cfg = self.config.clone();
            rgb_submenu.push(
                StandardItem {
                    label: label.into(),
                    enabled: true,
                    activate: Box::new(move |_: &mut TrayImpl| {
                        let g = g.clone();
                        let cfg = cfg.clone();
                        tokio::spawn(async move {
                            apply_setting(
                                cfg,
                                vec!["-c".into(), g.clone(), "-C".into(), g.clone()],
                                Setting::Gradient(g),
                            )
                            .await;
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        items.push(
            SubMenu {
                label: "RGB Colors".into(),
                enabled: true,
                submenu: rgb_submenu,
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // Button Mapping submenu: layout help, then one row per button.
        let mut button_mapping_submenu = vec![
            StandardItem {
                label: "Button Layout…".into(),
                enabled: true,
                activate: Box::new(|_: &mut TrayImpl| {
                    tokio::spawn(async {
                        if let Err(error) = kde_dialogs::show_button_layout().await {
                            report_error(error).await;
                        }
                    });
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
        ];
        for button in Button::ALL {
            let label = button_menu_label(button, self.mapping.action(button));
            button_mapping_submenu.push(
                StandardItem {
                    label,
                    enabled: true,
                    activate: Box::new(move |tray: &mut TrayImpl| {
                        let cfg = tray.config.clone();
                        let handle = tray.handle.clone();
                        let mapping = tray.mapping.clone();
                        tokio::spawn(async move {
                            pick_and_apply(cfg, handle, mapping, button).await;
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }
        items.push(
            SubMenu {
                label: "Button Mapping".into(),
                enabled: true,
                submenu: button_mapping_submenu,
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // Reset to Defaults
        let cfg = self.config.clone();
        items.push(
            StandardItem {
                label: "Reset to Defaults".into(),
                enabled: true,
                activate: Box::new(move |_: &mut TrayImpl| {
                    let cfg = cfg.clone();
                    tokio::spawn(async move {
                        apply_setting(cfg, vec!["-r".into()], Setting::Reset).await;
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // About
        items.push(
            StandardItem {
                label: "About sensei310-tray".into(),
                enabled: true,
                icon_name: "help-about".into(),
                activate: Box::new(|_: &mut TrayImpl| {
                    tokio::spawn(async move {
                        if let Err(error) =
                            kde_dialogs::show_info("About sensei310-tray", &about_text()).await
                        {
                            tracing::warn!("Could not show the About dialog: {error:#}");
                        }
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // Quit
        items.push(
            StandardItem {
                label: "Quit".into(),
                enabled: true,
                icon_name: "application-exit".into(),
                activate: Box::new(|_: &mut TrayImpl| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

/// Controller wrapper
pub struct TrayController {
    _handle: Handle<TrayImpl>,
}

impl TrayController {
    pub async fn new(config: Arc<Mutex<Config>>) -> Result<Self> {
        let snapshot = {
            let config = config.lock().await;
            mapping::load(&config).await
        };

        let tray_impl = TrayImpl {
            config,
            mapping: snapshot.mapping,
            handle: None,
        };
        let handle = tray_impl.spawn().await?;

        let handle_clone = handle.clone();
        let _ = handle
            .update(move |tray| tray.handle = Some(handle_clone))
            .await;

        info!("Tray icon created successfully via ksni");
        Ok(Self { _handle: handle })
    }

    pub async fn run(self) -> Result<()> {
        tokio::signal::ctrl_c().await?;
        info!("Shutdown signal received");
        Ok(())
    }
}
/// Keep device commands and persisted settings in the same order.
enum Setting {
    Dpi1(u32),
    Dpi2(u32),
    PollingRate(u32),
    LogoColor(String),
    WheelColor(String),
    Gradient(String),
    Reset,
}

async fn apply_setting(config: Arc<Mutex<Config>>, args: Vec<String>, setting: Setting) {
    let result = async {
        let mut config = config.lock().await;
        let arguments: Vec<&str> = args.iter().map(String::as_str).collect();
        rivalcfg::run_rivalcfg(&arguments).await?;
        match setting {
            Setting::Dpi1(value) => config.set_dpi1(value).await?,
            Setting::Dpi2(value) => config.set_dpi2(value).await?,
            Setting::PollingRate(value) => config.set_polling_rate(value).await?,
            Setting::LogoColor(value) => config.set_logo_color(value).await?,
            Setting::WheelColor(value) => config.set_wheel_color(value).await?,
            Setting::Gradient(value) => {
                config.logo_color = Some(value.clone());
                config.wheel_color = Some(value);
                config.save().await?;
            }
            Setting::Reset => {
                config.reset();
                config.save().await?;
            }
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;
    if let Err(error) = result {
        report_error(error).await;
    }
}

/// Label for one button row, for example `Button 4 (left side, rear): Nothing…`.
fn button_menu_label(button: Button, action: &Action) -> String {
    format!(
        "{} ({}): {}…",
        button.label(),
        button.location(),
        action.label()
    )
}

/// Ask for a new action, apply it, refresh the menu, and confirm.
async fn pick_and_apply(
    config: Arc<Mutex<Config>>,
    handle: Option<Handle<TrayImpl>>,
    mapping: ButtonMapping,
    button: Button,
) {
    match kde_dialogs::pick_action(button, &mapping).await {
        Ok(Some(action)) => {
            let result = async {
                let mut config = config.lock().await;
                mapping::apply(&mut config, button, action.clone()).await
            }
            .await;

            match result {
                Ok(snapshot) => {
                    if let Some(handle) = handle {
                        let _ = handle
                            .update(|tray: &mut TrayImpl| tray.mapping = snapshot.mapping)
                            .await;
                    }
                    if let Err(error) = kde_dialogs::notify_applied(button, &action).await {
                        tracing::warn!("Could not show confirmation: {error:#}");
                    }
                }
                Err(error) => report_error(error).await,
            }
        }
        Ok(None) => {}
        Err(error) => report_error(error).await,
    }
}

async fn report_error(error: anyhow::Error) {
    error!("Mouse operation failed: {error:#}");
    if let Err(dialog_error) =
        kde_dialogs::show_error("Sensei 310 Tray", &format!("Operation failed: {error:#}")).await
    {
        error!("Could not display error dialog: {dialog_error:#}");
    }
}

async fn show_device_info() {
    let result = async {
        let firmware_raw = rivalcfg::get_firmware_version().await?;
        let firmware_version = rivalcfg::firmware_label(&firmware_raw);
        let settings_text = match rivalcfg::get_saved_settings().await? {
            Some(settings) => rivalcfg::format_saved_profile(&settings),
            None => String::from(
                "No saved Rivalcfg profile was found.\n\nRivalcfg cannot read the current settings from the mouse.",
            ),
        };
        Ok::<String, anyhow::Error>(format!(
            "Firmware version: {firmware_version}\n\n{settings_text}"
        ))
    }
    .await;

    match result {
        Ok(message) => {
            if let Err(error) = kde_dialogs::show_info("SteelSeries Sensei 310", &message).await {
                report_error(error).await;
            }
        }
        Err(error) => report_error(error).await,
    }
}

fn about_text() -> String {
    format!(
        "sensei310-tray {}\n\nKDE tray controller for the SteelSeries Sensei 310.\n\nCopyright (c) 2026 Khame\nMIT License\nhttps://github.com/Khame/sensei310-tray",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buttons::{Action, Button};

    #[test]
    fn button_menu_labels_show_current_assignment() {
        assert_eq!(
            button_menu_label(Button::Button4, &Action::Disabled),
            "Button 4 (left side, rear): Nothing…"
        );
    }

    #[test]
    fn about_text_names_the_author_and_license() {
        let text = about_text();
        assert!(text.contains("Khame"));
        assert!(text.contains("MIT"));
        assert!(text.contains(env!("CARGO_PKG_VERSION")));
    }
}
