# Changelog

## 1.1.1 - 2026-11-14

- Start the tray when the mouse is not connected, instead of failing once and
  blocking on an error dialog.
- Write the active profile whenever the Sensei 310 appears on the USB bus, so a
  late enumeration at login or a reconnect during a session restores the
  settings.
- Grey out every menu action that needs the mouse while it is disconnected, and
  update the menu when it is plugged in or unplugged. The header says
  `SteelSeries Sensei 310 (not connected)` instead of letting an action fail
  with a rivalcfg error.
- Show `Device Info…` as a table headed by the active tray profile, with the
  firmware and the last settings Rivalcfg saved, and give every dialog the mouse
  window icon instead of kdialog's default one.
- Include rivalcfg's stdout in error messages. Some failures, a missing device
  among them, print there only and used to produce an empty message.

## 1.1.0 - 2026-11-13

- Store mouse settings as named profiles in `settings.json`, with a `Profiles`
  menu for switching, creating a profile from the current settings, and
  deleting one.
- Apply the active profile when the tray starts, so the mouse returns to the
  last selected settings after a login.
- Migrate the flat `settings.json` from 1.0.4 and older into the `default`
  profile.

## 1.0.4 - 2026-11-10

- Share tray menu construction, the DPI preset submenu, rivalcfg command
  execution, and kdialog exit handling.
- Parse buttons and actions through one code path and simplify the
  configuration setters.
- Fix the release install instructions and document key import and fingerprint
  verification.

## 1.0.3 - 2026-11-08

- Drop the keyboard layout line from the Device Info dialog.

## 1.0.2 - 2026-11-07

- Add icons to every tray menu item and submenu entry.
- Add a trademark notice to the README, About dialog, and `--version`.
- Include the changelog in the source tarball.

## 1.0.1 - 2026-11-07

- Add an About dialog and `--version` output.
- Add SPDX license headers and package metadata.
- Document the release tarball and package signing steps.

## 1.0.0 - 2026-11-07

- Initial release with DPI presets, polling rate, RGB colors, gradients, and button mapping.
- Add Void Linux packaging, a signed XBPS repository, and release assets.
