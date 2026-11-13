# sensei310-tray

KDE tray controller for the SteelSeries Sensei 310. It configures DPI presets
from 400 to 12000, polling rate from 125 to 1000 Hz, logo and scroll wheel
colors, gradient presets, and all eight buttons. Hardware access goes through
the `rivalcfg` CLI.

## Why this exists

I use a Sensei 310 on Void Linux with KDE Plasma. `rivalcfg` handles the
hardware, but it is a command line tool and the mouse has eight buttons whose
physical positions are easy to mix up. I wanted a tray menu for the common
settings and a picker that names each button by where it sits on the mouse.

It is a personal project, not an official Void package, and there is no
support promise behind it. Bug reports and patches are welcome anyway.

## Install

### Signed XBPS repository

Packages are published for x86_64 at
`https://khame.github.io/void-packages/`.

```bash
sudo sh -c 'echo "repository=https://khame.github.io/void-packages/" > /etc/xbps.d/10-khame.conf'
sudo xbps-install -S      # answer Y to import the repository key
sudo xbps-install sensei310-tray
```

`rivalcfg` is installed as a dependency. The repository is signed, and unsigned
installation is not supported.

On the first sync xbps asks to import the signing key. Compare its fingerprint
before you answer yes:

```
35:2b:06:7c:52:04:76:b1:4a:de:a0:8b:49:9c:e2:66
```

You can compute it yourself from the published key:

```sh
curl -fsSO https://khame.github.io/void-packages/Khame-void-repo.pub
ssh-keygen -E md5 -lf <(ssh-keygen -i -m PKCS8 -f Khame-void-repo.pub)
```

The result should match the fingerprint above. It shows the key and the
fingerprint belong together, though both come from the same site.

### Release asset

A manual, one-time install from the GitHub release. The release holds the
package, its `.sig2` signature, and the public key. `xbps-install` 0.60.7 does
not accept a package URL, so index the file in a local directory first.

```bash
mkdir -p ~/sensei310-tray-repo && cd ~/sensei310-tray-repo
curl -LO https://github.com/Khame/sensei310-tray/releases/download/v1.1.0/sensei310-tray-1.1.0_1.x86_64.xbps
curl -LO https://github.com/Khame/sensei310-tray/releases/download/v1.1.0/sensei310-tray-1.1.0_1.x86_64.xbps.sig2
curl -LO https://github.com/Khame/sensei310-tray/releases/download/v1.1.0/35.2b.06.7c.52.04.76.b1.4a.de.a0.8b.49.9c.e2.66.plist
sudo install -Dm644 35.2b.06.7c.52.04.76.b1.4a.de.a0.8b.49.9c.e2.66.plist /var/db/xbps/keys/35:2b:06:7c:52:04:76:b1:4a:de:a0:8b:49:9c:e2:66.plist
xbps-rindex -a sensei310-tray-1.1.0_1.x86_64.xbps
sudo xbps-install -R ~/sensei310-tray-repo -f sensei310-tray
```

GitHub replaces the colons in the key name with dots, so the `install` step
restores the fingerprint name that xbps uses. Adding the repository above
imports the same key, so the copy is only needed for this route. The name in
that path is the fingerprint you can compute with the command above.

Updates are manual with this route. The release asset contains only the tray,
so install `rivalcfg` from the signed repository first.

### Local XBPS package

To build from this checkout, copy the template into a void-packages tree. The
template downloads the source tarball from the GitHub release, so run
`prepare-local.py` and upload a new tarball only when you change the sources.

```bash
cp packaging/void/template ~/workspace/projects/software/void/void-packages/srcpkgs/sensei310-tray/template

cd ~/workspace/projects/software/void/void-packages
./xbps-src clean sensei310-tray
./xbps-src check sensei310-tray
./xbps-src pkg sensei310-tray
sudo xbps-install -R hostdir/binpkgs sensei310-tray
```

Bump `revision` in `packaging/void/template` for every rebuilt package. Run
`xbps-src` as a regular user.

### From source with Cargo

Install `rivalcfg` first, preferably from the signed repository so the system
udev rules are present.

```bash
git clone https://github.com/Khame/sensei310-tray.git
cd sensei310-tray
cargo build --release
sudo install -Dm755 target/release/sensei310-tray /usr/local/bin/sensei310-tray
install -Dm644 packaging/sensei310-tray.desktop ~/.config/autostart/sensei310-tray.desktop
```

## Start the tray

Run `sensei310-tray` from a terminal or KRunner. The package also installs an
autostart file in `/etc/xdg/autostart`, so the tray starts with the KDE session.
A source install uses the file in `~/.config/autostart` instead.

## What gets installed

- `/usr/bin/sensei310-tray`
- `/usr/share/applications/sensei310-tray.desktop`
- `/etc/xdg/autostart/sensei310-tray.desktop`
- `/usr/share/licenses/sensei310-tray/LICENSE`

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
About sensei310-tray
─────────────────────
Quit
```

`About sensei310-tray` shows the version, author, license, repository, and
trademark notice.

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

- Void Linux x86_64 with glibc, runit, and elogind
- KDE Plasma with DBus for StatusNotifierItem
- `kdialog`
- `rivalcfg` in `PATH`
- Read and write access to the mouse through the rivalcfg udev rules
- A Rust 2021 toolchain when building from source

The local rivalcfg package installs system rules in
`/usr/lib/udev/rules.d/`. After installing it, reload the rules and make sure
your user is in the `plugdev` group:

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo usermod -aG plugdev "$USER"
```

Log out and back in after changing the group. The package installs no systemd
units. musl is untested.

## Tested on

ASUS ROG Strix G513QM: Ryzen 9 5900HX, Radeon Vega plus RTX 3060 Mobile, 16 GB
RAM. Void x86_64 with glibc 2.41, kernel 7.3.1, XBPS 0.60.7, runit, Plasma 6.7.5
on Wayland, and kdialog 26.08.1. The mouse is a SteelSeries Sensei 310
(`1038:1722`).

The package builds, the 37 automated tests pass, and `--buttons-json` returns
the expected snapshot. The tray starts under KDE. The location rows, the
`Button Layout…` dialog, and the picker text were checked in KDE. A no-op
`--set-button` run wrote the same mapping back and updated both settings files.

A visible action change from the picker was not recorded here. DPI, polling
rate, and RGB writes were not exercised during this work.

## Version

```bash
sensei310-tray --version
```

The output names the version, author, license, repository, and trademark
notice. See [CHANGELOG.md](CHANGELOG.md) for release notes.

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

Settings live in `~/.config/sensei310-tray/settings.json` as named profiles:

```json
{
  "active": "gaming",
  "profiles": {
    "default": {
      "dpi1": 1600,
      "dpi2": 3200,
      "polling_rate": 1000,
      "logo_color": "#FF6600",
      "wheel_color": "#0066FF",
      "buttons_mapping": "buttons(button1=button1; button2=button2; button3=button3; button4=button4; button5=button5; button6=PageDown; button7=PageUp; button8=dpi; layout=qwerty)"
    },
    "gaming": {
      "dpi1": 3200,
      "polling_rate": 1000
    }
  }
}
```

The `Profiles` menu lists every profile and marks the active one. Picking a
profile writes its stored values to the mouse with one `rivalcfg` call and makes
it the active profile. Settings changed from the menu are saved into the active
profile. `New Profile…` asks for a name and starts from a copy of the current
settings. `Delete Profile…` removes an inactive profile; the `default` profile
and the active one cannot be deleted, and at least one profile always remains.

At login the tray applies the active profile, so the mouse comes back with the
last settings that were selected. A profile that does not store a value leaves
that setting alone, and an empty profile changes nothing.

These values are the last values this tray wrote. They are not a live read from
the mouse. The tray warns that the saved rivalcfg profile reflects the last
write, not the live mouse state.

Files written by sensei310-tray 1.0.4 and older have no `profiles` object. They
are read as the `default` profile and rewritten in the new format on the next
save.

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
- `src/profiles.rs`: profile arguments and the startup apply
- `src/config.rs`: `settings.json` persistence

The tray and the CLI both call the same mapping service. The menu renders a
snapshot and never parses rivalcfg strings itself.

## Patches

`sensei310-tray` carries no patches. The local `rivalcfg` package carries two:

- `relax-flit-core-version.patch`: drops the `flit_core <4` pin so the package
  builds with the flit_core version Void ships.
- `udev-system-rules.patch`: teaches `rivalcfg` about
  `/usr/lib/udev/rules.d`, and writes `MODE="0660"`, `GROUP="plugdev"`, and
  `TAG+="uaccess"` rules. It touches `rivalcfg/udev.py`,
  `rivalcfg/__main__.py`, and `rivalcfg/debug.py`.

Nothing has been sent upstream. Full notes, verification steps, and drop
conditions live in `srcpkgs/rivalcfg/PATCHES.md` in the void-packages checkout.
The package also ships `INSTALL.msg` with the udev reload and `plugdev` steps.

## Repository layout

- `README.md`: this file
- `CHANGELOG.md`: release notes
- `VOID_PACKAGING.md`: how the template is put together and how to build it
- `Cargo.toml`, `Cargo.lock`: Rust manifest and lockfile
- `src/`: application sources
- `packaging/sensei310-tray.desktop`: desktop and autostart entry
- `packaging/void/template`: local Void template
- `packaging/void/prepare-local.py`: creates the source tarball for a release
- `LICENSE`: MIT for this project

`prepare-local.py` creates the source tarball from this checkout and prints its
SHA-256. The tarball is attached to the GitHub release and is not edited by
hand.

## Upstream

Project, install documentation and release notes:
[Khame/sensei310-tray](https://github.com/Khame/sensei310-tray)

Hardware backend: [flozz/rivalcfg](https://github.com/flozz/rivalcfg), packaged
locally as `rivalcfg-4.17.0_6`.

Packaged version: `1.1.0_1`

License: MIT for this project. `rivalcfg` is WTFPL.

## Author

Khame ([@Khame](https://github.com/Khame)),
[khame.dev@protonmail.com](mailto:khame.dev@protonmail.com)

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

## Trademarks

SteelSeries and Sensei are trademarks or registered trademarks of SteelSeries
ApS or its affiliates. This project is not affiliated with, endorsed by, or
sponsored by SteelSeries.

## Related Projects

- [rivalcfg](https://github.com/flozz/rivalcfg): CLI backend
- [OpenRGB](https://openrgb.org/): alternative RGB control
- [RivalTune](https://github.com/polydezcom/RivalTune): GTK4 GUI frontend
