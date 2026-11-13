// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Turning a stored profile into rivalcfg arguments and writing it to the mouse.

use anyhow::Result;

use crate::config::Profile;
use crate::rivalcfg;

/// rivalcfg arguments that restore one profile, in a stable order.
pub fn arguments(profile: &Profile) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    if let Some(dpi) = profile.dpi1 {
        args.push("-s".into());
        args.push(dpi.to_string());
    }
    if let Some(dpi) = profile.dpi2 {
        args.push("-S".into());
        args.push(dpi.to_string());
    }
    if let Some(rate) = profile.polling_rate {
        args.push("-p".into());
        args.push(rate.to_string());
    }
    if let Some(color) = &profile.logo_color {
        args.push("-c".into());
        args.push(color.clone());
    }
    if let Some(color) = &profile.wheel_color {
        args.push("-C".into());
        args.push(color.clone());
    }
    if let Some(mapping) = &profile.buttons_mapping {
        args.push("-b".into());
        args.push(mapping.clone());
    }
    args
}

/// Write one profile to the mouse. An empty profile changes nothing.
pub async fn apply(profile: &Profile) -> Result<()> {
    let args = arguments(profile);
    if args.is_empty() {
        return Ok(());
    }

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    rivalcfg::run_rivalcfg(&refs).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_arguments_for_a_complete_profile() {
        let profile = Profile {
            dpi1: Some(800),
            dpi2: Some(1600),
            polling_rate: Some(1000),
            logo_color: Some("#ff0000".into()),
            wheel_color: Some("#00ff00".into()),
            buttons_mapping: Some("buttons(button1=button1; layout=qwerty)".into()),
        };

        assert_eq!(
            arguments(&profile),
            vec![
                "-s",
                "800",
                "-S",
                "1600",
                "-p",
                "1000",
                "-c",
                "#ff0000",
                "-C",
                "#00ff00",
                "-b",
                "buttons(button1=button1; layout=qwerty)",
            ]
        );
    }

    #[test]
    fn skips_settings_the_profile_does_not_set() {
        let profile = Profile {
            dpi1: Some(400),
            ..Profile::default()
        };

        assert_eq!(arguments(&profile), vec!["-s", "400"]);
    }

    #[test]
    fn empty_profile_produces_no_arguments() {
        assert!(arguments(&Profile::default()).is_empty());
    }
}
