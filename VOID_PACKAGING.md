# Void Linux packaging: sensei310-tray maintainer notes

This repository holds the application, the desktop entry, and the Void template.
The package directory in a void-packages tree is copied from these files, and
the template downloads the source tarball from a GitHub release.

Users should read [README.md](README.md) instead.

## Where the files live

| Path | What it is |
|---|---|
| `packaging/void/template` | the `xbps-src` template: version, revision, dependencies, `distfiles`, `checksum`, `post_install` |
| `packaging/void/prepare-local.py` | creates the source tarball for a release and prints its SHA-256 |
| `packaging/void/verify-release.py` | checks the version and checksum across `Cargo.toml`, `Cargo.lock`, the template, the tarball, and a void-packages checkout |
| `packaging/sensei310-tray.desktop` | desktop entry, also installed as an autostart file |
| `src/` | application sources |
| `Cargo.toml`, `Cargo.lock` | Rust manifest and lockfile |

`prepare-local.py` writes the source tarball that the template downloads. The
tarball has one top-level directory, `sensei310-tray-<version>`, and contains
`Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `LICENSE`, `README.md`, `src/`, and
`packaging/sensei310-tray.desktop`. It does not contain this file, `docs/`, or
`packaging/void/`.

The tarball is attached to the `v<version>` release in
[Khame/sensei310-tray](https://github.com/Khame/sensei310-tray). The template
does not use `FILESDIR`.

## About the template

`build_style=cargo` and `depends="kdialog rivalcfg"`.

`distfiles` points to the source tarball in the `v${version}` release of
`Khame/sensei310-tray`, and `checksum` is its SHA-256. `xbps-src` downloads and
verifies the tarball. There is no `do_extract` and no `FILESDIR` archive.

`post_install` installs the desktop entry in `/usr/share/applications` and
`/etc/xdg/autostart`, and installs `LICENSE` with `vlicense`. No systemd units
are shipped. The tray is a session process, not a runit service.

The package carries no patches. The `rivalcfg` package it depends on carries
two, documented in `srcpkgs/rivalcfg/PATCHES.md` in the void-packages tree.

## Building locally

Copy the template into a void-packages tree. The template downloads the source
tarball from the GitHub release.

```sh
git clone --depth=1 https://github.com/void-linux/void-packages.git
cd void-packages
./xbps-src binary-bootstrap

git clone --depth=1 https://github.com/Khame/void-packages.git /tmp/khame-void-packages
cp -a /tmp/khame-void-packages/srcpkgs/sensei310-tray srcpkgs/

./xbps-src clean sensei310-tray
./xbps-src check sensei310-tray
./xbps-src pkg sensei310-tray
sudo xbps-install --repository=hostdir/binpkgs sensei310-tray
```

Run `xbps-src` as a regular user. Bump `revision` in `packaging/void/template`
for every rebuilt package. The package ends up in `hostdir/binpkgs/`.

`./xbps-src clean` is required after a source change. The build directory is
reused and the source tarball is not extracted again, so a build without clean
can test stale sources.

After `./xbps-src pkg`, the cargo autodep is removed from the masterdir. Run
`./xbps-src check` again before using the masterdir toolchain for `cargo fmt` or
`cargo clippy`.

## Things that bit me

- `xbps-src clean` is not optional after a source change. The old `wrksrc` wins.
- The source tarball includes `README.md` and `Cargo.toml`, so a change there
  needs a new release, a new checksum, and a revision bump.
- The masterdir toolchain disappears after `pkg` because cargo is an autodep.
- `xbps-rindex -S` does not replace an existing `.sig2`. After a rebuild it
  exits 0 without a word and leaves the old signature, which then fails to
  verify. Remove `<pkg>.xbps.sig2` before signing again.

## Release invariants

The version and the source checksum must agree in every place at the same time:
`Cargo.toml`, `Cargo.lock`, `packaging/void/template`, the tarball attached to
the release, and the copy in `srcpkgs/sensei310-tray/template`.

`python3 packaging/void/verify-release.py --void-packages <path>` checks all of
them and exits nonzero on the first mismatch. Run it before tagging and again
after the checksum update.

Version 1.0.3 drifted. The source changed after the tag, a new tarball was
generated without a version bump, and `srcpkgs/sensei310-tray/template` ended up
at revision 2 with a checksum that matched only the local cache. The published
asset did not match, so a clean build failed. Do not build from a cached
tarball that does not match the template, and do not tag before the tarball and
the checksum agree.

## Publishing

Packages are published for x86_64 in `Khame/void-packages` under `docs/`, served
at `https://khame.github.io/void-packages/`. The `rivalcfg` dependency is
published from the same repository. Install documentation and release notes
live in this repository.

To publish a new revision:

1. For a new version, set it in `Cargo.toml`, `Cargo.lock`, and
   `packaging/void/template`, and set `revision=1`. For a rebuild, bump
   `revision` only.
2. Create the source tarball with
   `python3 packaging/void/prepare-local.py` and note its SHA-256.
3. Create or update the `v<version>` release in `Khame/sensei310-tray` and
   upload the tarball.
4. Update `distfiles` and `checksum` in the template, then run
   `python3 packaging/void/verify-release.py --void-packages <path>`.
5. Copy the template into `srcpkgs/sensei310-tray` in `Khame/void-packages`.
6. Build with `./xbps-src clean sensei310-tray` and
   `./xbps-src pkg sensei310-tray`.
7. Sign the package:

   ```sh
   pkg=$(ls -t hostdir/binpkgs/sensei310-tray-*.xbps | head -1)
   xbps-rindex -S --privkey="$HOME/.config/Khame-void-repo-private.pem" \
     --signedby="Khame <khame.dev@protonmail.com>" "$pkg"
   ```

8. Remove the previous `sensei310-tray` package and signature from `docs/`.
9. Copy the new `.xbps` and `.sig2` into `docs/`.
10. Rebuild and sign the index:
    `xbps-rindex -c docs`,
    `xbps-rindex -a docs/*.xbps`,
    `xbps-rindex -s --privkey="$HOME/.config/Khame-void-repo-private.pem" --signedby="Khame <khame.dev@protonmail.com>" docs`.
11. Commit and push `Khame/void-packages`.
12. Attach the same `.xbps` and `.sig2` to the release in `Khame/sensei310-tray`,
    together with the public key. The key is the `.plist` file from
    `/var/db/xbps/keys/`, not `Khame-void-repo.pub`, which is the PEM copy served
    by the site. GitHub replaces the colons in the asset name with dots, so the
    `install` step in `README.md` renames the file back to the fingerprint.
13. Update the version numbers in `README.md` and `CHANGELOG.md`.
14. Verify the published result:
    `xbps-query -M --repository=https://khame.github.io/void-packages/ -p pkgver sensei310-tray`
    has to return the new version. Without `-M` the query can answer from a
    cached index and look stale. The Pages build for the pushed commit has to
    report `built` in `gh api repos/Khame/void-packages/pages/builds/latest`.

Only x86_64 is published. Never push to `void-linux/void-packages` or to an
upstream project.

## Author and license

Khame ([@Khame](https://github.com/Khame)),
[khame.dev@protonmail.com](mailto:khame.dev@protonmail.com)

MIT for the material written here.
