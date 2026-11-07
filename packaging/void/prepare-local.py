#!/usr/bin/env python3
"""Create the source tarball that the Void template downloads."""
import argparse
import hashlib
from pathlib import Path
import tarfile


def package_version(source):
    for line in (source / 'Cargo.toml').read_text().splitlines():
        if line.startswith('version = '):
            return line.split('"')[1]
    raise ValueError('Cargo.toml has no version')


def prepare(source, output):
    version = package_version(source)
    prefix = f'{source.name}-{version}'
    names = ['Cargo.toml', 'CHANGELOG.md', 'LICENSE', 'README.md', 'src',
             'packaging/sensei310-tray.desktop']
    if (source / 'Cargo.lock').is_file():
        names.append('Cargo.lock')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(output, 'w:gz') as archive:
        for name in names:
            archive.add(source / name, arcname=f'{prefix}/{name}', recursive=True)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    print(f'Created {output}')
    print(f'sha256: {digest}')
    if 'Cargo.lock' not in names:
        print('Cargo.lock is missing. Run cargo generate-lockfile and repeat before building (--locked).')


if __name__ == '__main__':
    source = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path,
                        default=Path(f'{source.name}-{package_version(source)}.tar.gz'))
    args = parser.parse_args()
    prepare(source, args.output.resolve())
