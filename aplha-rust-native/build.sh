#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPLET="${1:-}"

if [[ "${APPLET}" != "alpha_usb" ]]; then
  echo "usage: ./build.sh alpha_usb" >&2
  exit 2
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found" >&2
  exit 1
fi

if ! command -v m68k-linux-gnu-ld >/dev/null 2>&1; then
  echo "m68k-linux-gnu-ld not found; install an m68k GNU binutils toolchain" >&2
  exit 1
fi

cd "${ROOT}"
cargo +nightly build -p alpha-usb-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort
cargo +nightly run -p alpha-neo-pack -- \
  alpha-usb \
  target/m68k-unknown-none-elf/debug/libalpha_usb_applet.a \
  ../exports/applets/alpha-usb-native.os3kapp

