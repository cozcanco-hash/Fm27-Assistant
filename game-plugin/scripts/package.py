#!/usr/bin/env python3
import pathlib
import zipfile
root = pathlib.Path(__file__).resolve().parents[1]
output = root / 'artifacts'
output.mkdir(exist_ok=True)
files = {
    'FM27.Assistant.Plugin.dll': root / 'src/FM27.Assistant.Plugin/bin/Release/net6.0/FM27.Assistant.Plugin.dll',
    'FM27.Assistant.Client.dll': root / 'src/FM27.Assistant.Client/bin/Release/net6.0/FM27.Assistant.Client.dll',
    'READ_FIRST.md': root / 'README.md',
}
with zipfile.ZipFile(output / 'FM27-Assistant-in-game-experimental.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
    for name, path in files.items():
        archive.write(path, 'FM27Assistant/' + name)
print(output / 'FM27-Assistant-in-game-experimental.zip')
