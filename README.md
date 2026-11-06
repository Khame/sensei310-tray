# sensei310-tray

Lightweight KDE tray controller for **SteelSeries Sensei 310** — configure DPI, RGB lighting, and polling rate from the system tray.

![StatusNotifierItem](https://img.shields.io/badge/StatusNotifierItem-KDE%20Plasma-blue)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![License](https://img.shields.io/badge/License-MIT-green)

## Features

- 🖱 **DPI Presets**: 400–12000 DPI (8 presets)
- ⚡ **Polling Rate**: 125 / 250 / 500 / 1000 Hz
- 🌈 **RGB Control**: Logo + scroll wheel colors (KDE native color picker)
- 🎨 **Gradient Presets**: Rainbow, Fire, Ocean, Sunset, Matrix
- 🔄 **Reset to Defaults**: One-click factory reset
- 💾 **Persistent Settings**: Remembers last used values
- 🚀 **Lightweight**: ~2.5 MB binary, zero Qt/GTK dependencies
- 🔔 **Native KDE Tray**: Uses StatusNotifierItem (SNI) via DBus

## Screenshots

*Tray menu in KDE Plasma:*
```
SteelSeries Sensei 310
─────────────────────
▶ DPI Presets
  400  800  1600  2400  3200  6400  12000
▶ Polling Rate
  125 Hz  250 Hz  500 Hz  1000 Hz
▶ RGB Colors
  Logo Color…
  Wheel Color…
  ──────────
  Rainbow  Fire  Ocean  Sunset  Matrix
─────────────────────
Reset to Defaults
Refresh Status
─────────────────────
Quit
```

## Requirements

- **Rust** 1.70+ (for building)
- **rivalcfg** (runtime dependency) — `pipx install rivalcfg`
- **KDE Plasma** with `xdg-desktop-portal-kde` (for native color dialog)
- **DBus** (for StatusNotifierItem)

## Installation

### From Void Linux repository (recommended)

```bash
# Build and install via xbps-src
cd ~/workspace/projects/software/void/void-packages
./xbps-src pkg sensei310-tray
xi sensei310-tray
```

### From source (Cargo)

```bash
# Install runtime dependency first
pipx install rivalcfg
sudo rivalcfg --update-udev

# Build
git clone https://github.com/khame/sensei310-tray
cd sensei310-tray
cargo build --release

# Install manually
sudo cp target/release/sensei310-tray /usr/local/bin/
cp packaging/sensei310-tray.desktop ~/.config/autostart/
```

## Usage

1. **Start manually**: Run `sensei310-tray` from terminal or KRunner (Alt+Space)
2. **Auto-start**: Enabled by default via `.desktop` file in `~/.config/autostart/`
3. **Tray icon**: Appears in KDE system tray (right panel, near clock)
4. **Configure**: Right-click tray icon → select DPI, polling rate, colors, gradients

## Configuration

Settings stored in `~/.config/sensei310-tray/settings.json`:

```json
{
  "dpi": 1600,
  "polling_rate": 1000,
  "logo_color": "#FF6600",
  "wheel_color": "#0066FF",
  "window_geometry": null
}
```

## Building for Void Linux

### Template location

```
~/workspace/projects/software/void/void-packages/srcpkgs/sensei310-tray/
├── template
└── files/
    └── usr/share/applications/sensei310-tray.desktop
```

### Build command

```bash
cd ~/workspace/projects/software/void/void-packages
./xbps-src pkg sensei310-tray
```

### Template content

```bash
pkgname=sensei310-tray
version=1.0.0
revision=1
build_style=cargo
hostmakedepends="pkg-config rust cargo"
makedepends="dbus-devel libxkbcommon-devel wayland-devel libxcb-devel"
short_desc="KDE tray controller for SteelSeries Sensei 310 (DPI, RGB, polling)"
maintainer="Khame <khame@example.com>"
license="MIT"
homepage="https://github.com/khame/sensei310-tray"
distfiles=""
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      sensei310-tray                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  tray-icon   │  │     rfd      │  │     rivalcfg     │  │
│  │  (SNI/DBus)  │◄─┤ (KDE portal) │  │     (CLI)        │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│         ▲                ▲                   ▲              │
│         │                │                   │              │
│  KDE Plasma          xdg-desktop-        rivalcfg          │
│  System Tray         portal-kde          (Python)          │
└─────────────────────────────────────────────────────────────┘
```

## Dependencies (Cargo)

| Crate | Purpose | Features |
|-------|---------|----------|
| `tray-icon` | StatusNotifierItem tray | `status-notifier`, `image` |
| `rfd` | Native dialogs | `kde` (uses xdg-desktop-portal-kde) |
| `tokio` | Async runtime | `rt`, `process`, `sync` |
| `serde` | Config serialization | `derive` |
| `dirs` | Config directory | - |

## License

MIT License — see [LICENSE](LICENSE) for details.

## Contributing

1. Fork the repository
2. Create feature branch
3. Commit changes
4. Open Pull Request

## Related Projects

- [rivalcfg](https://github.com/flozz/rivalcfg) — CLI backend
- [OpenRGB](https://openrgb.org/) — Alternative RGB control
- [RivalTune](https://github.com/polydezcom/RivalTune) — GTK4 GUI frontend