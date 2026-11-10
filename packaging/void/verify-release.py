#!/usr/bin/env python3
"""Check that the version and the source tarball agree across the release files.

Run before tagging a release and after regenerating the tarball:

    python3 packaging/void/verify-release.py

Optionally point at a void-packages checkout to compare the copied template and
the cached source tarball:

    python3 packaging/void/verify-release.py --void-packages ~/path/to/void-packages
"""
import argparse
import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PKG = 'sensei310-tray'


def read(path):
    try:
        return path.read_text()
    except OSError as error:
        fail(f'cannot read {path}: {error}')


def fail(message):
    print(f'FAIL: {message}')
    sys.exit(1)


def cargo_version():
    match = re.search(r'^version\s*=\s*"([^"]+)"', read(ROOT / 'Cargo.toml'), re.MULTILINE)
    if not match:
        fail('Cargo.toml has no package version')
    return match.group(1)


def lock_version():
    text = read(ROOT / 'Cargo.lock')
    match = re.search(r'\[\[package\]\]\nname = "' + PKG + r'"\nversion = "([^"]+)"', text)
    if not match:
        fail(f'Cargo.lock has no package entry for {PKG}')
    return match.group(1)


def template_fields(path):
    text = read(path)
    fields = {}
    for key in ('version', 'revision', 'checksum', 'distfiles'):
        match = re.search(rf'^{key}=(.+)$', text, re.MULTILINE)
        if not match:
            fail(f'{path} has no {key}')
        fields[key] = match.group(1).strip().strip('"')
    return fields


def sha256(path):
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for block in iter(lambda: handle.read(1 << 20), b''):
            digest.update(block)
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--void-packages', type=Path,
                        help='optional void-packages checkout to compare against')
    args = parser.parse_args()

    version = cargo_version()
    locked = lock_version()
    template = template_fields(ROOT / 'packaging/void/template')

    if version != locked:
        fail(f'Cargo.toml is {version} but Cargo.lock is {locked}')
    print(f'ok: Cargo.toml and Cargo.lock agree on {version}')

    if template['version'] != version:
        fail(f'template version is {template["version"]} but Cargo.toml is {version}')
    print(f'ok: template version is {version}')

    tarball = ROOT / f'{PKG}-{version}.tar.gz'
    if not tarball.is_file():
        fail(f'missing tarball {tarball.name}, run prepare-local.py')
    actual = sha256(tarball)
    if actual != template['checksum']:
        fail(f'template checksum is {template["checksum"]} but {tarball.name} is {actual}')
    print(f'ok: template checksum matches {tarball.name}')

    expected_url = (f'https://github.com/Khame/{PKG}/releases/download/'
                    f'v${{version}}/{PKG}-${{version}}.tar.gz')
    if template['distfiles'] != expected_url:
        fail(f'distfiles is {template["distfiles"]} but expected {expected_url}')
    print('ok: distfiles points at the release tarball')

    if args.void_packages:
        copied = args.void_packages / 'srcpkgs' / PKG / 'template'
        if not copied.is_file():
            fail(f'missing {copied}')
        if copied.read_bytes() != (ROOT / 'packaging/void/template').read_bytes():
            fail(f'{copied} differs from packaging/void/template')
        print('ok: void-packages template is identical')

        cached = (args.void_packages / 'hostdir' / 'sources' / f'{PKG}-{version}'
                  / f'{PKG}-{version}.tar.gz')
        if cached.is_file():
            if sha256(cached) != template['checksum']:
                fail(f'cached source {cached} does not match the template checksum')
            print('ok: cached source tarball matches the template checksum')
        else:
            print('note: no cached source tarball to compare')

    print(f'PASS: {PKG} {version}_{template["revision"]} is consistent')


if __name__ == '__main__':
    main()
