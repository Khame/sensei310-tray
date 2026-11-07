//! Command-line entry points for the tray and the JSON contract.

use anyhow::Result;

use crate::buttons::{parse_assignment, Action, Button};

/// What the binary should do for this invocation.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Tray,
    ButtonsJson,
    SetButton { button: Button, action: Action },
}

/// Parse arguments after the program name.
pub fn parse<I>(args: I) -> Result<Command>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    match args.as_slice() {
        [] => Ok(Command::Tray),
        [flag] if flag == "--buttons-json" => Ok(Command::ButtonsJson),
        [flag, value] if flag == "--set-button" => {
            let (button, action) = parse_assignment(value)?;
            Ok(Command::SetButton { button, action })
        }
        _ => anyhow::bail!("usage: sensei310-tray [--buttons-json | --set-button buttonN=action]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buttons::{Action, Button};

    fn args(values: &[&str]) -> std::vec::IntoIter<String> {
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn no_arguments_starts_the_tray() {
        assert_eq!(parse(args(&[])).unwrap(), Command::Tray);
    }

    #[test]
    fn buttons_json_is_a_command() {
        assert_eq!(
            parse(args(&["--buttons-json"])).unwrap(),
            Command::ButtonsJson
        );
    }

    #[test]
    fn set_button_parses_the_assignment() {
        assert_eq!(
            parse(args(&["--set-button", "button4=disabled"])).unwrap(),
            Command::SetButton {
                button: Button::Button4,
                action: Action::Disabled,
            }
        );
    }

    #[test]
    fn unknown_arguments_are_rejected() {
        assert!(parse(args(&["--wat"])).is_err());
        assert!(parse(args(&["--set-button"])).is_err());
        assert!(parse(args(&["--set-button", "button9=disabled"])).is_err());
    }
}
