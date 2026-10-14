#!/usr/bin/env bash
# Render the Homebrew formula for a release.
# Usage: scripts/homebrew-formula.sh <version> <dist-dir-with-sha256-files>
set -euo pipefail

version=$1
dist=$2
base="https://github.com/bytemain/prog/releases/download/v${version}"

sha() {
  cat "$dist/prog-$1.tar.gz.sha256"
}

cat <<FORMULA
class Prog < Formula
  desc "Manage git repositories by path rules and find them by keyword"
  homepage "https://github.com/bytemain/prog"
  version "${version}"

  on_macos do
    on_arm do
      url "${base}/prog-aarch64-apple-darwin.tar.gz"
      sha256 "$(sha aarch64-apple-darwin)"
    end
    on_intel do
      url "${base}/prog-x86_64-apple-darwin.tar.gz"
      sha256 "$(sha x86_64-apple-darwin)"
    end
  end

  on_linux do
    on_arm do
      url "${base}/prog-aarch64-unknown-linux-musl.tar.gz"
      sha256 "$(sha aarch64-unknown-linux-musl)"
    end
    on_intel do
      url "${base}/prog-x86_64-unknown-linux-musl.tar.gz"
      sha256 "$(sha x86_64-unknown-linux-musl)"
    end
  end

  def install
    bin.install "prog"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/prog --version")
  end
end
FORMULA
