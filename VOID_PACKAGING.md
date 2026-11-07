# Void Linux packaging: sensei310-tray maintainer notes

This repository holds the application, the desktop entry, and the Void template.
The package directory in a void-packages tree is generated from these files.

Users should read [README.md](README.md) instead.

## Where the files live

| Path | What it is |
|---|---|
| `packaging/void/template` | the `xbps-src` template: version, revision, dependencies, `do_extract`, `post_install` |
| `packaging/void/prepare-local.py` | rebuilds the source tarball and copies the template into a void-packages tree |
| `packaging/sensei310-tray.desktop` | desktop entry, also installed as an autostart file |
| `src/` | application sources |
| `Cargo.toml`, `Cargo.lock` | Rust manifest and lockfile |

`prepare-local.py` writes these files into `srcpkgs/sensei310-tray` in the
void-packages tree:

```text
srcpkgs/sensei310-tray/
├── template
└── files/
    ├── sensei310-tray-source.tar.gz
    └── sensei310-tray.desktop
```

The tarball contains `Cargo.toml`, `Cargo.lock`, `LICENSE`, `README.md`,
`src/`, and `packaging/sensei310-tray.desktop`. It does not contain this file.

## About the template

`build_style=cargo`. `hostmakedepends="pkg-config tar"` and
`depends="kdialog rivalcfg"`.

There is no remote distfile. `distfiles` and `checksum` are empty, and
`do_extract` unpacks `${FILESDIR}/sensei310-tray-source.tar.gz` into `${wrksrc}`.
`xlint` reports both empty variables as warnings. That is expected for a local
package.

`post_install` installs the desktop entry in `/usr/share/applications` and
`/etc/xdg/autostart`, and installs `LICENSE` with `vlicense`. No systemd units
are shipped. The tray is a session process, not a runit service.

The package carries no patches. The `rivalcfg` package it depends on carries
two, documented in `srcpkgs/rivalcfg/PATCHES.md` in the void-packages tree.

## Building locally

From the project checkout:

```sh
python3 packaging/void/prepare-local.py
cd ../void/void-packages
./xbps-src clean sensei310-tray
./xbps-src check sensei310-tray
./xbps-src pkg sensei310-tray
sudo xbps-install -R hostdir/binpkgs sensei310-tray
```

Run `xbps-src` as a regular user. Bump `revision` in `packaging/void/template`
for every rebuilt package. The package ends up in `hostdir/binpkgs/`.

`./xbps-src clean` is required after a source change. The build directory is
reused and the staged tarball is not extracted again, so a build without clean
can test stale sources.

After `./xbps-src pkg`, the cargo autodep is removed from the masterdir. Run
`./xbps-src check` again before using the masterdir toolchain for `cargo fmt` or
`cargo clippy`.

## Things that bit me

- `xbps-src clean` is not optional after restaging. The old `wrksrc` wins.
- Empty `distfiles` and `checksum` are correct here, but `xlint` still warns.
- The tarball includes `README.md`, so a README change needs a restage and a
  revision bump to keep the committed tarball in step with the repository.
- The masterdir toolchain disappears after `pkg` because cargo is an autodep.

## Publishing

Nothing is published. There is no signed XBPS repository and no release asset.

If this ever changes, publish only to `Khame/void-packages` and only after an
explicit request. Never push to `void-linux/void-packages` or to an upstream
project.

## Author and license

Khame ([@Khame](https://github.com/Khame)),
[khame.dev@protonmail.com](mailto:khame.dev@protonmail.com)

MIT for the material written here.
