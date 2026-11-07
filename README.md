# sensei310-tray

Lightweight KDE tray controller for the SteelSeries Sensei 310. Configure DPI,
RGB lighting, polling rate, and button mapping from the system tray.

![StatusNotifierItem](https://img.shields.io/badge/StatusNotifierItem-KDE%20Plasma-blue)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![License](https://img.shields.io/badge/License-MIT-green)

## Features

- DPI presets 1 and 2: 400, 800, 1600, 2400, 3200, 6400, and 12000
- Polling rate: 125, 250, 500, and 1000 Hz
- RGB control for the logo and the scroll wheel through the KDE color picker
- Gradient presets: Rainbow, Fire, Ocean, Sunset, and Matrix
- Guided button mapping with a `kdialog` list and short descriptions
- Device info from the firmware and the saved rivalcfg profile
- Persistent settings for the last used values
- Native StatusNotifierItem tray with no Qt or GTK dependency
- Small Rust binary that calls the `rivalcfg` CLI for hardware access

## Tray menu

```text
SteelSeries Sensei 310
─────────────────────
Device Info…
─────────────────────
DPI Preset 1
  400  800  1600  2400  3200  6400  12000
DPI Preset 2
  400  800  1600  2400  3200  6400  12000
Polling Rate
  125 Hz  250 Hz  500 Hz  1000 Hz
─────────────────────
RGB Colors
  Logo Color…
  Wheel Color…
  ──────────
  Rainbow  Fire  Ocean  Sunset  Matrix
─────────────────────
Button Mapping
  Button Layout…
  ──────────
  Button 1 (left main button): Button 1…
  Button 2 (right main button): Button 2…
  Button 3 (scroll wheel click): Button 3 (middle click)…
  Button 4 (left side, rear): Button 4…
  Button 5 (left side, front): Button 5…
  Button 6 (right side, rear): Page Down…
  Button 7 (right side, front): Page Up…
  Button 8 (CPI button behind the scroll wheel): DPI switch…
─────────────────────
Reset to Defaults
─────────────────────
Quit
```

Each button row shows the physical position and the current action. The
`Button Layout…` item lists all eight positions. Clicking a row opens a
`kdialog --radiolist` titled `Assign Button 4` with every action, a short
description, and the current action checked. The dialog text repeats the
physical position. Cancelling writes nothing. A successful choice writes one
`rivalcfg -b` call and shows a passive popup.

The positions follow the rivalcfg Sensei 310 button diagram: Button 1 and
Button 2 are the main clicks, Button 3 is the scroll wheel click, Button 4 and
Button 5 are on the left side, Button 6 and Button 7 are on the right side, and
Button 8 is the CPI button behind the scroll wheel. On the side, "rear" is
closer to the palm and "front" is closer to the cable.

## Requirements

- `rivalcfg` in `PATH`
- `kdialog`
- KDE Plasma with DBus for StatusNotifierItem
- A Rust 2021 toolchain when building from source
- Read and write access to the mouse through the rivalcfg udev rules

The local rivalcfg package installs system rules in
`/usr/lib/udev/rules.d/`. After installing it, reload the rules and make sure
your user is in the `plugdev` group:

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo usermod -aG plugdev "$USER"
```

Log out and back in after changing the group.

## Installation

### Local Void Linux package

The package builds from a staged local tarball, not a remote distfile:

```bash
cd ~/workspace/projects/software/sensei310-tray
python3 packaging/void/prepare-local.py
cd ~/workspace/projects/software/void/void-packages
./xbps-src pkg sensei310-tray
sudo xbps-install -R hostdir/binpkgs sensei310-tray
```

Run `prepare-local.py` after every source change. Bump `revision` in
`packaging/void/template` for every rebuilt package.

### From source with Cargo

```bash
pipx install rivalcfg
sudo rivalcfg --update-udev

git clone https://github.com/khame/sensei310-tray
cd sensei310-tray
cargo build --release
sudo cp target/release/sensei310-tray /usr/local/bin/
cp packaging/sensei310-tray.desktop ~/.config/autostart/
```

## Usage

1. Start the tray from a terminal or KRunner with `sensei310-tray`.
2. Autostart is provided by the desktop file in `/etc/xdg/autostart`.
3. Right-click the tray icon to change DPI, polling rate, colors, or buttons.

The tray does not read live mouse state. It shows the last saved rivalcfg
profile and the tray settings, and it warns when those differ from a live read.

## Button mapping from the CLI

The tray and future frontends share one JSON contract:

```bash
sensei310-tray --buttons-json
```

The command prints a snapshot to stdout and keeps logging on stderr:

```json
{
  "buttons": [
    {
      "id": "button1",
      "label": "Button 1",
      "location": "left main button",
      "action": "button1",
      "action_label": "Button 1"
    }
  ],
  "actions": [
    {
      "id": "button1",
      "label": "Button 1",
      "description": "left click"
    }
  ],
  "layout": "qwerty",
  "source": "saved_profile",
  "profile": "default",
  "raw": "buttons(button1=button1; ...; layout=qwerty)",
  "warning": "Saved rivalcfg profile reflects the last write, not the live mouse state."
}
```

`source` is `saved_profile`, `tray_config`, or `defaults`. The `actions` array
is the full vocabulary with descriptions, so a frontend does not have to know
rivalcfg strings.

Apply one assignment:

```bash
sensei310-tray --set-button button4=disabled
```

The command writes the device, saves the tray config, and prints the resulting
snapshot. Errors go to stderr with exit status 1. This changes the mouse. The
snapshot after a write carries the warning `These values were just written by
this tray.` because it comes from the string that was sent, not from a profile
read.

## Configuration

Settings are stored in `~/.config/sensei310-tray/settings.json`:

```json
{
  "dpi1": 1600,
  "dpi2": 3200,
  "polling_rate": 1000,
  "logo_color": "#FF6600",
  "wheel_color": "#0066FF",
  "buttons_mapping": "buttons(button1=button1; button2=button2; button3=button3; button4=button4; button5=button5; button6=PageDown; button7=PageUp; button8=dpi; layout=qwerty)",
  "window_geometry": null
}
```

These values are the last values this tray wrote. They are not a live read from
the mouse.

## Building for Void Linux

The local package lives in `srcpkgs/sensei310-tray` inside the void-packages
checkout:

```text
srcpkgs/sensei310-tray/
├── template
└── files/
    ├── sensei310-tray-source.tar.gz
    └── sensei310-tray.desktop
```

`packaging/void/prepare-local.py` rebuilds the tarball from the project sources
and copies `packaging/void/template` into the package directory. The template
extracts the tarball from `FILESDIR` and uses `build_style=cargo`.

After a source change:

```bash
python3 packaging/void/prepare-local.py
cd ../void/void-packages
./xbps-src clean sensei310-tray
./xbps-src check sensei310-tray
./xbps-src pkg sensei310-tray
xlint sensei310-tray
```

`xbps-src clean` is required because a previous `wrksrc` is reused and the
staged tarball is not extracted again.

## Architecture

```text
KDE tray (ksni StatusNotifierItem)
        │
        ├── kdialog: color picker, action picker, popups
        │
        └── rivalcfg CLI: one write per change
                │
                ▼
        SteelSeries Sensei 310
```

Source modules:

- `src/buttons.rs`: button and action vocabulary, rivalcfg wire format, merge
- `src/mapping.rs`: snapshot, source resolution, and the apply service
- `src/cli.rs`: `--buttons-json` and `--set-button`
- `src/tray.rs`: tray menu and the guided picker
- `src/kde_dialogs.rs`: kdialog wrappers
- `src/rivalcfg.rs`: rivalcfg process calls and saved profile reading
- `src/config.rs`: `settings.json` persistence

The tray and the CLI both call the same mapping service. The menu renders a
snapshot and never parses rivalcfg strings itself.

## Dependencies (Cargo)

| Crate | Purpose |
|-------|---------|
| `ksni` | StatusNotifierItem tray |
| `tokio` | Async runtime and subprocesses |
| `serde` | Config and snapshot serialization |
| `serde_json` | Saved profile and JSON output |
| `dirs` | Config directory |
| `anyhow` | Error context |
| `tracing` | Diagnostics |
| `tracing-subscriber` | Log filtering |

## Tests

Tests live in `#[cfg(test)]` modules next to the code. They do not call
`rivalcfg` and do not change the mouse. Run them through the package:

```bash
./xbps-src clean sensei310-tray
./xbps-src check sensei310-tray
```

Manual checks in KDE cover the picker, cancellation, the confirmation popup,
and the refreshed menu label.

## License

MIT License. See [LICENSE](LICENSE) for details.

## Related Projects

- [rivalcfg](https://github.com/flozz/rivalcfg): CLI backend
- [OpenRGB](https://openrgb.org/): alternative RGB control
- [RivalTune](https://github.com/polydezcom/RivalTune): GTK4 GUI frontend
