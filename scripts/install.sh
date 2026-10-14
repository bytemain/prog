#!/bin/sh
# Install the latest prebuilt prog binary from GitHub Releases.
#   curl -fsSL https://raw.githubusercontent.com/bytemain/prog/main/scripts/install.sh | sh
# Environment:
#   PROG_VERSION      release tag to install (default: latest), e.g. v0.1.0
#   PROG_INSTALL_DIR  install directory (default: ~/.local/bin)
set -eu

repo="bytemain/prog"
install_dir="${PROG_INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)" in
  Darwin) os="apple-darwin" ;;
  Linux) os="unknown-linux-musl" ;;
  *) echo "Unsupported OS: $(uname -s). Download a release manually from https://github.com/$repo/releases" >&2; exit 1 ;;
esac

case "$(uname -m)" in
  x86_64 | amd64) arch="x86_64" ;;
  arm64 | aarch64) arch="aarch64" ;;
  *) echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

target="$arch-$os"
if [ -n "${PROG_VERSION:-}" ]; then
  url="https://github.com/$repo/releases/download/$PROG_VERSION/prog-$target.tar.gz"
else
  url="https://github.com/$repo/releases/latest/download/prog-$target.tar.gz"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $url"
curl -fsSL "$url" -o "$tmp/prog.tar.gz"
tar -xzf "$tmp/prog.tar.gz" -C "$tmp"

mkdir -p "$install_dir"
mv "$tmp/prog-$target/prog" "$install_dir/prog"
chmod +x "$install_dir/prog"
echo "Installed $("$install_dir/prog" --version) to $install_dir/prog"

case ":$PATH:" in
  *":$install_dir:"*) ;;
  *) echo "Note: $install_dir is not in your PATH. Add it to your shell profile." ;;
esac
