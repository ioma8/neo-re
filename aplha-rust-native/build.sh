#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPLET="${1:-}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found" >&2
  exit 1
fi

cd "${ROOT}"
case "${APPLET}" in
  alpha_usb)
    PACKAGE="alpha-usb-applet"
    PACKER_NAME="alpha-usb"
    OUTPUT="../exports/applets/alpha-usb-native.os3kapp"
    ;;
  forth_mini)
    PACKAGE="forth-mini-applet"
    PACKER_NAME="forth-mini"
    OUTPUT="../exports/applets/forth-mini.os3kapp"
    ;;
  *)
    echo "usage: ./build.sh alpha_usb|forth_mini" >&2
    exit 2
    ;;
esac

cargo +nightly build -p "${PACKAGE}" --target m68k-unknown-none-elf -Z build-std=core,panic_abort --release
cargo +nightly run -p alpha-neo-pack -- \
  "${PACKER_NAME}" \
  "target/m68k-unknown-none-elf/release/${PACKAGE}" \
  "${OUTPUT}"
