#!/usr/bin/env python3
"""Stage the current sources for a local xbps-src build without host paths."""
import argparse
from pathlib import Path
import shutil
import tarfile
import tempfile

def prepare(source, tree):
    destination = tree / 'srcpkgs/sensei310-tray'
    if not (tree / 'xbps-src').is_file():
        raise ValueError(f'Not a void-packages checkout: {tree}')
    files = destination / 'files'
    files.mkdir(parents=True, exist_ok=True)
    archive = files / 'sensei310-tray-source.tar.gz'
    names = ['Cargo.toml', 'LICENSE', 'README.md', 'src',
             'packaging/sensei310-tray.desktop']
    if (source / 'Cargo.lock').is_file():
        names.append('Cargo.lock')
    with tempfile.NamedTemporaryFile(dir=files, suffix='.tar.gz', delete=False) as temporary:
        temporary_path = Path(temporary.name)
    try:
        with tarfile.open(temporary_path, 'w:gz') as output:
            for name in names:
                output.add(source / name, arcname=name, recursive=True)
        temporary_path.replace(archive)
        archive.chmod(0o644)
    finally:
        temporary_path.unlink(missing_ok=True)
    shutil.copy2(source / 'packaging/void/template', destination / 'template')
    print(f'Staged sources: {archive}')
    if 'Cargo.lock' not in names:
        print('Cargo.lock is missing. Run cargo generate-lockfile and repeat preparation before building (--locked).')

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--void-tree', type=Path,
                        default=Path.home() / 'workspace/projects/software/void/void-packages')
    args = parser.parse_args()
    prepare(Path(__file__).resolve().parents[2], args.void_tree.resolve())
