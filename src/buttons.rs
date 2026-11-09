// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Button and action vocabulary for the Sensei 310.
//!
//! This module is independent of any user interface. It owns the rivalcfg
//! wire format, the factory defaults, and the labels shown to users.

use anyhow::{Context, Result};
use std::fmt::Write as _;

/// One of the eight physical buttons the Sensei 310 exposes to rivalcfg.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Button1 = 1,
    Button2 = 2,
    Button3 = 3,
    Button4 = 4,
    Button5 = 5,
    Button6 = 6,
    Button7 = 7,
    Button8 = 8,
}

impl Button {
    /// All buttons in their physical order.
    pub const ALL: [Button; 8] = [
        Button::Button1,
        Button::Button2,
        Button::Button3,
        Button::Button4,
        Button::Button5,
        Button::Button6,
        Button::Button7,
        Button::Button8,
    ];

    /// Canonical rivalcfg identifier, for example `button4`.
    pub fn id(self) -> &'static str {
        match self {
            Button::Button1 => "button1",
            Button::Button2 => "button2",
            Button::Button3 => "button3",
            Button::Button4 => "button4",
            Button::Button5 => "button5",
            Button::Button6 => "button6",
            Button::Button7 => "button7",
            Button::Button8 => "button8",
        }
    }

    /// User-facing name, for example `Button 4`.
    pub fn label(self) -> String {
        format!("Button {}", self.number())
    }

    /// Physical position on the mouse.
    ///
    /// Positions follow the rivalcfg Sensei 310 button diagram. The left side
    /// is the thumb side for right-handed use.
    pub fn location(self) -> &'static str {
        match self {
            Button::Button1 => "left main button",
            Button::Button2 => "right main button",
            Button::Button3 => "scroll wheel click",
            Button::Button4 => "left side, rear",
            Button::Button5 => "left side, front",
            Button::Button6 => "right side, rear",
            Button::Button7 => "right side, front",
            Button::Button8 => "CPI button behind the scroll wheel",
        }
    }

    /// Parse `buttonN` case-insensitively. Returns `None` outside 1 through 8.
    pub fn parse(value: &str) -> Option<Button> {
        let value = value.trim().to_ascii_lowercase();
        let number = value.strip_prefix("button")?.parse::<u8>().ok()?;
        Button::ALL
            .get(usize::from(number).checked_sub(1)?)
            .copied()
    }

    fn number(self) -> u8 {
        self as u8
    }

    fn index(self) -> usize {
        usize::from(self.number() - 1)
    }
}

/// What a button can do. `Other` preserves actions this build does not know.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Button(Button),
    Dpi,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Disabled,
    Other(String),
}

impl Action {
    /// Every known action in menu order. `Other` is not included.
    pub fn known() -> Vec<Action> {
        let mut actions: Vec<Action> = Button::ALL
            .iter()
            .map(|button| Action::Button(*button))
            .collect();
        actions.extend([
            Action::Dpi,
            Action::ScrollUp,
            Action::ScrollDown,
            Action::PageUp,
            Action::PageDown,
            Action::Disabled,
        ]);
        actions
    }

    /// Canonical rivalcfg value, for example `disabled` or `PageUp`.
    pub fn id(&self) -> &str {
        match self {
            Action::Button(button) => button.id(),
            Action::Dpi => "dpi",
            Action::ScrollUp => "scrollup",
            Action::ScrollDown => "scrolldown",
            Action::PageUp => "PageUp",
            Action::PageDown => "PageDown",
            Action::Disabled => "disabled",
            Action::Other(raw) => raw,
        }
    }

    /// User-facing name, for example `Nothing` or `Button 3 (middle click)`.
    pub fn label(&self) -> String {
        match self {
            Action::Button(Button::Button3) => String::from("Button 3 (middle click)"),
            Action::Button(button) => button.label(),
            Action::Dpi => String::from("DPI switch"),
            Action::ScrollUp => String::from("Scroll up"),
            Action::ScrollDown => String::from("Scroll down"),
            Action::PageUp => String::from("Page Up"),
            Action::PageDown => String::from("Page Down"),
            Action::Disabled => String::from("Nothing"),
            Action::Other(raw) => capitalize(raw),
        }
    }

    /// One short sentence that explains the action in the picker.
    pub fn description(&self) -> String {
        match self {
            Action::Button(Button::Button1) => String::from("left click"),
            Action::Button(Button::Button2) => String::from("right click"),
            Action::Button(Button::Button3) => String::from("middle click"),
            Action::Button(button) => format!("mouse button {}", button.number()),
            Action::Dpi => String::from("switch between DPI preset 1 and DPI preset 2"),
            Action::ScrollUp => String::from("scroll wheel up"),
            Action::ScrollDown => String::from("scroll wheel down"),
            Action::PageUp => String::from("Page Up key"),
            Action::PageDown => String::from("Page Down key"),
            Action::Disabled => String::from("the button does nothing"),
            Action::Other(_) => String::from("unknown action from the saved profile"),
        }
    }

    /// Parse a rivalcfg action. Unknown values become `Other`.
    pub fn parse(value: &str) -> Action {
        let value = value.trim();
        let lower = value.to_ascii_lowercase();
        match lower.as_str() {
            "button1" => Action::Button(Button::Button1),
            "button2" => Action::Button(Button::Button2),
            "button3" => Action::Button(Button::Button3),
            "button4" => Action::Button(Button::Button4),
            "button5" => Action::Button(Button::Button5),
            "button6" => Action::Button(Button::Button6),
            "button7" => Action::Button(Button::Button7),
            "button8" => Action::Button(Button::Button8),
            "dpi" => Action::Dpi,
            "scrollup" => Action::ScrollUp,
            "scrolldown" => Action::ScrollDown,
            "pageup" => Action::PageUp,
            "pagedown" => Action::PageDown,
            "disabled" | "none" => Action::Disabled,
            "middle click" | "middleclick" | "middle" => Action::Button(Button::Button3),
            _ => Action::Other(value.to_string()),
        }
    }
}

/// A complete `buttons(...)` value: eight assignments, extra keys, and layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonMapping {
    assignments: [Action; 8],
    extra: Vec<(String, String)>,
    layout: String,
}

impl Default for ButtonMapping {
    fn default() -> Self {
        Self {
            assignments: [
                Action::Button(Button::Button1),
                Action::Button(Button::Button2),
                Action::Button(Button::Button3),
                Action::Button(Button::Button4),
                Action::Button(Button::Button5),
                Action::PageDown,
                Action::PageUp,
                Action::Dpi,
            ],
            extra: Vec::new(),
            layout: String::from("qwerty"),
        }
    }
}

impl ButtonMapping {
    /// Parse `buttons(button1=button1; ...; layout=qwerty)`.
    ///
    /// Missing buttons keep the factory defaults. Unknown keys and unknown
    /// actions are preserved so a newer rivalcfg value roundtrips unchanged.
    pub fn parse(raw: &str) -> Result<Self> {
        let inner = raw
            .trim()
            .strip_prefix("buttons(")
            .context("Expected a value that starts with buttons(")?
            .strip_suffix(')')
            .context("Expected a value that ends with )")?;

        let mut mapping = ButtonMapping::default();
        for part in inner.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let (key, value) = part
                .split_once('=')
                .with_context(|| format!("Expected key=value in {part:?}"))?;
            let key = key.trim();
            let value = value.trim();
            anyhow::ensure!(!key.is_empty(), "Empty key in {part:?}");
            anyhow::ensure!(!value.is_empty(), "Empty value for {key:?}");

            if key.eq_ignore_ascii_case("layout") {
                mapping.layout = value.to_string();
            } else if let Some(button) = Button::parse(key) {
                mapping.assignments[button.index()] = Action::parse(value);
            } else {
                mapping.extra.push((key.to_string(), value.to_string()));
            }
        }
        Ok(mapping)
    }

    /// Serialize back to the rivalcfg wire format.
    pub fn to_rivalcfg(&self) -> String {
        let mut text = String::from("buttons(");
        for button in Button::ALL {
            let _ = write!(text, "{}={}; ", button.id(), self.action(button).id());
        }
        for (key, value) in &self.extra {
            let _ = write!(text, "{key}={value}; ");
        }
        let _ = write!(text, "layout={})", self.layout);
        text
    }

    /// Current action for one button.
    pub fn action(&self, button: Button) -> &Action {
        &self.assignments[button.index()]
    }

    /// Assign one action without touching the other buttons.
    pub fn set_action(&mut self, button: Button, action: Action) {
        self.assignments[button.index()] = action;
    }

    /// Layout name, for example `qwerty`.
    pub fn layout(&self) -> &str {
        &self.layout
    }

    /// Known actions followed by any unknown action currently assigned.
    pub fn actions(&self) -> Vec<Action> {
        let mut actions = Action::known();
        for action in &self.assignments {
            if matches!(action, Action::Other(_))
                && !actions.iter().any(|known| known.id() == action.id())
            {
                actions.push(action.clone());
            }
        }
        actions
    }

    /// Start from the saved mapping when it parses, otherwise from defaults.
    pub fn merge(current: Option<&str>, button: Button, action: Action) -> Self {
        let mut mapping = current
            .and_then(|raw| ButtonMapping::parse(raw).ok())
            .unwrap_or_default();
        mapping.set_action(button, action);
        mapping
    }
}

/// Parse a CLI assignment such as `button4=disabled`.
pub fn parse_assignment(value: &str) -> Result<(Button, Action)> {
    let (key, raw) = value
        .split_once('=')
        .with_context(|| format!("Expected button=action, got {value:?}"))?;
    let button = Button::parse(key).with_context(|| format!("Unknown button {:?}", key.trim()))?;
    Ok((button, Action::parse(raw)))
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVED: &str = "buttons(button1=button2; button2=button2; button3=button3; button4=disabled; button5=button5; button6=PageDown; button7=PageUp; button8=dpi; layout=qwerty)";

    #[test]
    fn button_ids_and_labels_are_stable() {
        assert_eq!(Button::Button1.id(), "button1");
        assert_eq!(Button::Button8.label(), "Button 8");
        assert_eq!(Button::ALL.len(), 8);
    }

    #[test]
    fn button_parse_is_case_insensitive() {
        assert_eq!(Button::parse("Button4"), Some(Button::Button4));
        assert_eq!(Button::parse("button9"), None);
    }

    #[test]
    fn button_locations_describe_the_physical_layout() {
        assert_eq!(Button::Button1.location(), "left main button");
        assert_eq!(Button::Button4.location(), "left side, rear");
        assert_eq!(Button::Button5.location(), "left side, front");
        assert_eq!(
            Button::Button8.location(),
            "CPI button behind the scroll wheel"
        );
    }

    #[test]
    fn action_labels_are_human_readable() {
        assert_eq!(Action::Button(Button::Button4).label(), "Button 4");
        assert_eq!(
            Action::Button(Button::Button3).label(),
            "Button 3 (middle click)"
        );
        assert_eq!(Action::Dpi.label(), "DPI switch");
        assert_eq!(Action::Disabled.label(), "Nothing");
    }

    #[test]
    fn action_descriptions_explain_the_choice() {
        assert!(Action::Dpi.description().contains("DPI preset"));
        assert!(Action::Disabled.description().contains("does nothing"));
        assert!(Action::Button(Button::Button1)
            .description()
            .contains("left click"));
    }

    #[test]
    fn unknown_actions_are_preserved() {
        assert_eq!(Action::parse("macro1"), Action::Other("macro1".to_string()));
    }

    #[test]
    fn known_actions_have_no_duplicate_ids() {
        let mut ids: Vec<String> = Action::known()
            .iter()
            .map(Action::id)
            .map(str::to_string)
            .collect();
        let count = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }

    #[test]
    fn parses_saved_mapping() {
        let mapping = ButtonMapping::parse(SAVED).unwrap();
        assert_eq!(
            mapping.action(Button::Button1),
            &Action::Button(Button::Button2)
        );
        assert_eq!(mapping.action(Button::Button4), &Action::Disabled);
        assert_eq!(mapping.layout(), "qwerty");
    }

    #[test]
    fn missing_buttons_keep_factory_defaults() {
        let mapping = ButtonMapping::parse("buttons(button1=disabled; layout=azerty)").unwrap();
        assert_eq!(mapping.action(Button::Button1), &Action::Disabled);
        assert_eq!(mapping.action(Button::Button6), &Action::PageDown);
        assert_eq!(mapping.layout(), "azerty");
    }

    #[test]
    fn rejects_a_non_buttons_value() {
        assert!(ButtonMapping::parse("rgbgradient(duration=1000)").is_err());
    }

    #[test]
    fn roundtrips_the_wire_format() {
        let mapping = ButtonMapping::parse(SAVED).unwrap();
        assert_eq!(mapping.to_rivalcfg(), SAVED);
    }

    #[test]
    fn merge_preserves_other_assignments_and_layout() {
        let mapping = ButtonMapping::merge(
            Some(SAVED),
            Button::Button4,
            Action::Button(Button::Button1),
        );
        assert_eq!(
            mapping.action(Button::Button1),
            &Action::Button(Button::Button2)
        );
        assert_eq!(
            mapping.action(Button::Button4),
            &Action::Button(Button::Button1)
        );
        assert_eq!(mapping.layout(), "qwerty");
    }

    #[test]
    fn merge_without_current_uses_defaults() {
        let mapping = ButtonMapping::merge(None, Button::Button8, Action::Disabled);
        assert_eq!(mapping.action(Button::Button8), &Action::Disabled);
        assert_eq!(mapping.action(Button::Button6), &Action::PageDown);
    }

    #[test]
    fn preserves_unknown_buttons_and_actions() {
        let raw = "buttons(button1=macro1; button9=button9; layout=colemak)";
        let mapping = ButtonMapping::parse(raw).unwrap();
        assert_eq!(
            mapping.action(Button::Button1),
            &Action::Other("macro1".into())
        );
        let written = mapping.to_rivalcfg();
        assert!(written.contains("button1=macro1"));
        assert!(written.contains("button9=button9"));
        assert!(written.ends_with("layout=colemak)"));
    }

    #[test]
    fn parses_an_assignment_for_the_cli() {
        assert_eq!(
            parse_assignment("button4=disabled").unwrap(),
            (Button::Button4, Action::Disabled)
        );
        assert!(parse_assignment("button9=disabled").is_err());
        assert!(parse_assignment("button4").is_err());
    }
}
