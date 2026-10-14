#!/usr/bin/env bash
# Render the Scoop manifest for a release.
# Usage: scripts/scoop-manifest.sh <version> <dist-dir-with-sha256-files>
set -euo pipefail

version=$1
dist=$2
target="x86_64-pc-windows-msvc"
hash=$(cat "$dist/prog-$target.zip.sha256")

cat <<MANIFEST
{
    "version": "${version}",
    "description": "Manage git repositories by path rules and find them by keyword",
    "homepage": "https://github.com/bytemain/prog",
    "architecture": {
        "64bit": {
            "url": "https://github.com/bytemain/prog/releases/download/v${version}/prog-${target}.zip",
            "hash": "${hash}",
            "extract_dir": "prog-${target}"
        }
    },
    "bin": "prog.exe",
    "checkver": "github",
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/bytemain/prog/releases/download/v\$version/prog-${target}.zip",
                "hash": {
                    "url": "\$url.sha256"
                },
                "extract_dir": "prog-${target}"
            }
        }
    }
}
MANIFEST
