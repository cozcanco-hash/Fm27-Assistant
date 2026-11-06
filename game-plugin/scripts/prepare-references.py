#!/usr/bin/env python3
"""Download two compile-time assemblies from an official, checksum-pinned release.
This does not install or run a loader. Use the target game's loader for runtime tests.
"""
import hashlib
import pathlib
import sys
import urllib.request
import zipfile

def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as source:
        while chunk := source.read(1024 * 1024):
            value.update(chunk)
    return value.hexdigest()


url = 'https://github.com/BepInEx/BepInEx/releases/download/v6.0.0-pre.2/BepInEx-Unity.IL2CPP-win-x64-6.0.0-pre.2.zip'
expected = '616ec7eb06cf11b2a0000e8fcef04d1b12bb58e84a2e0bdac9523234fc193ceb'
destination = pathlib.Path(sys.argv[1]).resolve()
destination.mkdir(parents=True, exist_ok=True)
archive = destination / 'reference.zip'
if not archive.exists() or digest(archive) != expected:
    partial = destination / 'reference.download'
    total = 0
    try:
        with urllib.request.urlopen(url, timeout=60) as response, partial.open('wb') as output:
            while chunk := response.read(1024 * 1024):
                total += len(chunk)
                if total > 100 * 1024 * 1024:
                    raise ValueError('Reference download exceeded 100 MiB')
                output.write(chunk)
        if digest(partial) != expected:
            raise ValueError('BepInEx reference checksum did not match')
        partial.replace(archive)
    finally:
        partial.unlink(missing_ok=True)
with zipfile.ZipFile(archive) as bundle:
    for name in ['BepInEx.Core.dll', 'BepInEx.Unity.IL2CPP.dll']:
        entry = bundle.getinfo('BepInEx/core/' + name)
        if entry.file_size > 2 * 1024 * 1024:
            raise ValueError('Unexpected reference assembly size')
        (destination / name).write_bytes(bundle.read(entry))
print(destination)
