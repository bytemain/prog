#!/bin/sh
# Install a prebuilt prog binary from GitHub Releases.
#   curl -fsSL https://raw.githubusercontent.com/bytemain/prog/main/scripts/install.sh | sh
# Env: PROG_VERSION (e.g. v0.1.0, default latest), PROG_INSTALL_DIR (default ~/.local/bin)
set -eu

repo="bytemain/prog"
install_dir="${PROG_INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)" in
  Darwin) os="apple-darwin" ;;
  Linux) os="unknown-linux-musl" ;;
  *) echo "Unsupported OS: $(uname -s). On Windows, download the zip from https://github.com/$repo/releases" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  arm64|aarch64) arch="aarch64" ;;
  x86_64|amd64) arch="x86_64" ;;
  *) echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac
target="$arch-$os"

if [ -n "${PROG_VERSION:-}" ]; then
  base="https://github.com/$repo/releases/download/$PROG_VERSION"
else
  base="https://github.com/$repo/releases/latest/download"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading prog for $target..."
curl -fsSL "$base/prog-$target.tar.gz" -o "$tmp/prog.tar.gz"
tar -xzf "$tmp/prog.tar.gz" -C "$tmp"
mkdir -p "$install_dir"
install -m 755 "$tmp/prog-$target/prog" "$install_dir/prog"
echo "Installed prog to $install_dir/prog"

case ":$PATH:" in
  *":$install_dir:"*) ;;
  *) echo "Note: $install_dir is not on your PATH." ;;
esac
