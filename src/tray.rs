// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Tray icon and menu management using ksni (StatusNotifierItem)

use anyhow::Result;
use ksni::menu::{RadioGroup, RadioItem, StandardItem, SubMenu};
use ksni::{Handle, MenuItem, Tray, TrayMethods};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::buttons::{Action, Button, ButtonMapping};
use crate::config::{Config, Profile, DEFAULT_PROFILE};
use crate::kde_dialogs;
use crate::mapping;
use crate::profiles;
use crate::rivalcfg;

const ICON_DEVICE: &str = "input-mouse";
const ICON_INFO: &str = "dialog-information";
const ICON_PROFILE: &str = "user-identity";
const ICON_PROFILE_NEW: &str = "list-add";
const ICON_PROFILE_DELETE: &str = "list-remove";
const ICON_DPI: &str = "speedometer";
const ICON_POLLING: &str = "chronometer";
const ICON_RGB: &str = "color-management";
const ICON_BUTTONS: &str = "input-gaming";
const ICON_RESET: &str = "edit-reset";
const ICON_ABOUT: &str = "help-about";
const ICON_QUIT: &str = "application-exit";

/// Tray implementation for ksni
struct TrayImpl {
    config: Arc<Mutex<Config>>,
    mapping: ButtonMapping,
    /// Profile names in menu order.
    profiles: Vec<String>,
    /// Profile the menu marks as selected.
    active_profile: String,
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
        ICON_DEVICE.into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items: Vec<MenuItem<Self>> = vec![];

        // Device info header (disabled StandardItem)
        items.push(
            StandardItem {
                label: "SteelSeries Sensei 310".into(),
                enabled: false,
                icon_name: ICON_DEVICE.into(),
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
                icon_name: ICON_INFO.into(),
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

        // Profiles: pick one, create one from the current settings, delete one.
        items.push(profile_submenu(&self.profiles, &self.active_profile));
        items.push(MenuItem::Separator);

        // DPI Preset submenus: same choices, different rivalcfg flag and setting.
        items.push(dpi_submenu(
            &self.config,
            "DPI Preset 1",
            "-s",
            Setting::Dpi1,
        ));
        items.push(dpi_submenu(
            &self.config,
            "DPI Preset 2",
            "-S",
            Setting::Dpi2,
        ));

        // Polling Rate submenu
        let mut poll_submenu = vec![];
        for rate in [125, 250, 500, 1000] {
            poll_submenu.push(setting_item(
                &self.config,
                format!("{rate} Hz"),
                ICON_POLLING,
                vec!["-p".into(), rate.to_string()],
                Setting::PollingRate(rate),
            ));
        }
        items.push(
            SubMenu {
                label: "Polling Rate".into(),
                enabled: true,
                icon_name: ICON_POLLING.into(),
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
                icon_name: ICON_RGB.into(),
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
                icon_name: ICON_RGB.into(),
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
            rgb_submenu.push(setting_item(
                &self.config,
                label.into(),
                ICON_RGB,
                vec!["-c".into(), g.clone(), "-C".into(), g.clone()],
                Setting::Gradient(g),
            ));
        }

        items.push(
            SubMenu {
                label: "RGB Colors".into(),
                enabled: true,
                icon_name: ICON_RGB.into(),
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
                icon_name: ICON_BUTTONS.into(),
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
                    icon_name: ICON_BUTTONS.into(),
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
                icon_name: ICON_BUTTONS.into(),
                submenu: button_mapping_submenu,
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // Reset to Defaults
        items.push(setting_item(
            &self.config,
            "Reset to Defaults".into(),
            ICON_RESET,
            vec!["-r".into()],
            Setting::Reset,
        ));

        items.push(MenuItem::Separator);

        // About
        items.push(
            StandardItem {
                label: "About sensei310-tray".into(),
                enabled: true,
                icon_name: ICON_ABOUT.into(),
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
                icon_name: ICON_QUIT.into(),
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

/// Submenu with one radio row per profile plus the create and delete actions.
fn profile_submenu(names: &[String], active: &str) -> MenuItem<TrayImpl> {
    let selected = names.iter().position(|name| name == active).unwrap_or(0);
    let options = names
        .iter()
        .map(|name| RadioItem {
            label: name.clone(),
            enabled: true,
            icon_name: ICON_PROFILE.into(),
            ..Default::default()
        })
        .collect();

    let submenu: Vec<MenuItem<TrayImpl>> = vec![
        RadioGroup {
            selected,
            select: Box::new(|tray: &mut TrayImpl, index: usize| {
                let Some(name) = tray.profiles.get(index).cloned() else {
                    return;
                };
                let config = tray.config.clone();
                let handle = tray.handle.clone();
                tokio::spawn(async move {
                    switch_profile(config, handle, name).await;
                });
            }),
            options,
        }
        .into(),
        MenuItem::Separator,
        StandardItem {
            label: "New Profile…".into(),
            enabled: true,
            icon_name: ICON_PROFILE_NEW.into(),
            activate: Box::new(|tray: &mut TrayImpl| {
                let config = tray.config.clone();
                let handle = tray.handle.clone();
                tokio::spawn(async move {
                    create_profile(config, handle).await;
                });
            }),
            ..Default::default()
        }
        .into(),
        StandardItem {
            label: "Delete Profile…".into(),
            enabled: true,
            icon_name: ICON_PROFILE_DELETE.into(),
            activate: Box::new(|tray: &mut TrayImpl| {
                let config = tray.config.clone();
                let handle = tray.handle.clone();
                tokio::spawn(async move {
                    delete_profile(config, handle).await;
                });
            }),
            ..Default::default()
        }
        .into(),
    ];

    SubMenu {
        label: "Profiles".into(),
        enabled: true,
        icon_name: ICON_PROFILE.into(),
        submenu,
        ..Default::default()
    }
    .into()
}

/// Build a menu row that runs rivalcfg and then saves the matching setting.
fn setting_item(
    config: &Arc<Mutex<Config>>,
    label: String,
    icon_name: &'static str,
    args: Vec<String>,
    setting: Setting,
) -> MenuItem<TrayImpl> {
    let config = config.clone();
    StandardItem {
        label,
        enabled: true,
        icon_name: icon_name.into(),
        activate: Box::new(move |_: &mut TrayImpl| {
            let config = config.clone();
            let args = args.clone();
            let setting = setting.clone();
            tokio::spawn(async move {
                apply_setting(config, args, setting).await;
            });
        }),
        ..Default::default()
    }
    .into()
}

/// Both DPI presets offer the same choices, differing only in flag and setting.
fn dpi_submenu(
    config: &Arc<Mutex<Config>>,
    label: &str,
    flag: &'static str,
    setting: fn(u32) -> Setting,
) -> MenuItem<TrayImpl> {
    let submenu = [400, 800, 1600, 2400, 3200, 6400, 12000]
        .into_iter()
        .map(|dpi| {
            setting_item(
                config,
                dpi.to_string(),
                ICON_DPI,
                vec![flag.into(), dpi.to_string()],
                setting(dpi),
            )
        })
        .collect();
    SubMenu {
        label: label.into(),
        enabled: true,
        icon_name: ICON_DPI.into(),
        submenu,
        ..Default::default()
    }
    .into()
}

/// Controller wrapper
pub struct TrayController {
    _handle: Handle<TrayImpl>,
}

impl TrayController {
    pub async fn new(config: Arc<Mutex<Config>>) -> Result<Self> {
        let (snapshot, profiles, active_profile) = {
            let config = config.lock().await;
            let snapshot = mapping::load(&config).await;
            (snapshot, config.names(), config.active.clone())
        };

        let tray_impl = TrayImpl {
            config,
            mapping: snapshot.mapping,
            profiles,
            active_profile,
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
#[derive(Clone)]
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
        let profile = config.active_profile_mut();
        match setting {
            Setting::Dpi1(value) => profile.dpi1 = Some(value),
            Setting::Dpi2(value) => profile.dpi2 = Some(value),
            Setting::PollingRate(value) => profile.polling_rate = Some(value),
            Setting::LogoColor(value) => profile.logo_color = Some(value),
            Setting::WheelColor(value) => profile.wheel_color = Some(value),
            Setting::Gradient(value) => {
                profile.logo_color = Some(value.clone());
                profile.wheel_color = Some(value);
            }
            Setting::Reset => *profile = Profile::default(),
        }
        config.save().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;
    if let Err(error) = result {
        report_error(error).await;
    }
}

/// Write a profile picked in the menu to the mouse and make it active.
async fn switch_profile(
    config: Arc<Mutex<Config>>,
    handle: Option<Handle<TrayImpl>>,
    name: String,
) {
    let result = async {
        let mut config = config.lock().await;
        let previous = config.active.clone();
        if !config.select(&name) {
            anyhow::bail!("Profile \"{name}\" does not exist.");
        }
        let profile = config.active_profile().clone();
        if let Err(error) = profiles::apply(&profile).await {
            config.select(&previous);
            return Err(error);
        }
        config.save().await?;
        Ok::<ButtonMapping, anyhow::Error>(mapping::load(&config).await.mapping)
    }
    .await;

    match result {
        Ok(mapping) => {
            if let Some(handle) = handle {
                let _ = handle
                    .update(move |tray: &mut TrayImpl| {
                        tray.active_profile = name;
                        tray.mapping = mapping;
                    })
                    .await;
            }
        }
        Err(error) => report_error(error).await,
    }
}

/// Create a profile as a copy of the current settings after asking for a name.
async fn create_profile(config: Arc<Mutex<Config>>, handle: Option<Handle<TrayImpl>>) {
    let prompt = "Name for the new profile. It starts as a copy of the active profile.";
    let name = match kde_dialogs::ask_text("New Profile", prompt, "").await {
        Ok(Some(name)) => name,
        Ok(None) => return,
        Err(error) => return report_error(error).await,
    };

    let result = async {
        let mut config = config.lock().await;
        config.create(&name)?;
        config.save().await?;
        Ok::<(Vec<String>, String), anyhow::Error>((config.names(), config.active.clone()))
    }
    .await;

    match result {
        Ok((names, active)) => set_profile_list(handle, names, active).await,
        Err(error) => report_error(error).await,
    }
}

/// Delete an inactive profile after picking it and confirming.
async fn delete_profile(config: Arc<Mutex<Config>>, handle: Option<Handle<TrayImpl>>) {
    let (names, active) = {
        let config = config.lock().await;
        (config.names(), config.active.clone())
    };
    let deletable: Vec<String> = names
        .into_iter()
        .filter(|name| name != DEFAULT_PROFILE && name != &active)
        .collect();

    if deletable.is_empty() {
        let message = "There is no profile to delete. The default and the active profile cannot be removed; create another profile and switch to it first.";
        if let Err(error) = kde_dialogs::show_info("Delete Profile", message).await {
            error!("Could not show the delete hint: {error:#}");
        }
        return;
    }

    let name = match kde_dialogs::pick_profile(
        "Delete Profile",
        "Choose the profile to delete.",
        &deletable,
    )
    .await
    {
        Ok(Some(name)) => name,
        Ok(None) => return,
        Err(error) => return report_error(error).await,
    };

    let question = format!("Delete profile \"{name}\" and its saved settings?");
    match kde_dialogs::confirm("Delete Profile", &question).await {
        Ok(true) => {}
        Ok(false) => return,
        Err(error) => return report_error(error).await,
    }

    let result = async {
        let mut config = config.lock().await;
        config.delete(&name)?;
        config.save().await?;
        Ok::<(Vec<String>, String), anyhow::Error>((config.names(), config.active.clone()))
    }
    .await;

    match result {
        Ok((names, active)) => set_profile_list(handle, names, active).await,
        Err(error) => report_error(error).await,
    }
}

/// Refresh the profile list and the selected row after a change.
async fn set_profile_list(handle: Option<Handle<TrayImpl>>, profiles: Vec<String>, active: String) {
    if let Some(handle) = handle {
        let _ = handle
            .update(move |tray: &mut TrayImpl| {
                tray.profiles = profiles;
                tray.active_profile = active;
            })
            .await;
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
        "sensei310-tray {}\n\nKDE tray controller for the SteelSeries Sensei 310.\n\nCopyright (c) 2026 Khame\nMIT License\nhttps://github.com/Khame/sensei310-tray\n\n{}",
        env!("CARGO_PKG_VERSION"),
        crate::TRADEMARK_NOTICE
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

    #[test]
    fn about_text_includes_the_trademark_notice() {
        assert!(about_text().contains(crate::TRADEMARK_NOTICE));
    }

    #[test]
    fn profile_menu_marks_the_active_profile() {
        let names = vec![DEFAULT_PROFILE.to_string(), "gaming".to_string()];
        let item = profile_submenu(&names, "gaming");
        let MenuItem::SubMenu(submenu) = item else {
            panic!("expected a submenu");
        };
        assert_eq!(submenu.label, "Profiles");
        let MenuItem::RadioGroup(group) = &submenu.submenu[0] else {
            panic!("expected a radio group");
        };
        assert_eq!(group.selected, 1);
        assert_eq!(group.options.len(), 2);
        assert_eq!(group.options[1].label, "gaming");
    }

    #[test]
    fn every_menu_item_has_an_icon() {
        let tray = TrayImpl {
            config: Arc::new(Mutex::new(Config::default())),
            mapping: ButtonMapping::default(),
            profiles: vec![DEFAULT_PROFILE.to_string()],
            active_profile: DEFAULT_PROFILE.to_string(),
            handle: None,
        };

        assert_menu_icons(&tray.menu());
    }

    fn assert_menu_icons<T>(items: &[MenuItem<T>]) {
        for item in items {
            match item {
                MenuItem::Standard(item) => {
                    assert!(
                        !item.icon_name.is_empty(),
                        "missing icon for {}",
                        item.label
                    );
                }
                MenuItem::SubMenu(item) => {
                    assert!(
                        !item.icon_name.is_empty(),
                        "missing icon for {}",
                        item.label
                    );
                    assert_menu_icons(&item.submenu);
                }
                MenuItem::RadioGroup(group) => {
                    for option in &group.options {
                        assert!(
                            !option.icon_name.is_empty(),
                            "missing icon for {}",
                            option.label
                        );
                    }
                }
                MenuItem::Separator => {}
                _ => {}
            }
        }
    }
}
