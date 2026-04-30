#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPLET="${1:-}"
RUN_GUI=0
VALIDATE=1

usage() {
  cat >&2 <<'EOF'
usage: ./scripts/build-smartapplet.sh <applet> [--gui] [--no-validate]

Examples:
  ./scripts/build-smartapplet.sh basic_writer_bw
  ./scripts/build-smartapplet.sh basic_writer_bw --gui
EOF
  exit 2
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required tool: $1" >&2
    if [[ "$1" == m68k-elf-gcc || "$1" == m68k-elf-ld || "$1" == m68k-elf-objcopy ]]; then
      echo "install with: brew install m68k-elf-binutils m68k-elf-gcc" >&2
    fi
    exit 1
  fi
}

[[ -n "${APPLET}" ]] || usage
shift

while [[ $# -gt 0 ]]; do
  case "$1" in
    --gui)
      RUN_GUI=1
      ;;
    --no-validate)
      VALIDATE=0
      ;;
    *)
      usage
      ;;
  esac
  shift
done

APPLET_DIR="${ROOT}/smartapplets/${APPLET}"
APPLET_ENV="${APPLET_DIR}/applet.env"
[[ -d "${APPLET_DIR}" && -f "${APPLET_ENV}" ]] || {
  echo "unknown smartapplet: ${APPLET}" >&2
  exit 1
}

require_cmd cargo
require_cmd m68k-elf-gcc
require_cmd m68k-elf-ld
require_cmd m68k-elf-objcopy
require_cmd make

# shellcheck disable=SC1090
source "${APPLET_ENV}"

make -C "${APPLET_DIR}" clean all

cargo run \
  --manifest-path "${ROOT}/aplha-rust-native/Cargo.toml" \
  -p alpha-neo-pack -- \
  "${PACKER_NAME}" \
  "${APPLET_DIR}/${ELF_NAME}" \
  "${ROOT}/${OUTPUT_PATH}"

if [[ "${VALIDATE}" -eq 1 && -n "${VALIDATE_FLAG:-}" ]]; then
  cargo run \
    --manifest-path "${ROOT}/alpha-emu/Cargo.toml" -- \
    --headless \
    "${VALIDATE_FLAG}" \
    --lcd-ocr \
    "${ROOT}/analysis/cab/os3kneorom.os3kos"
fi

if [[ "${RUN_GUI}" -eq 1 ]]; then
  cargo run \
    --manifest-path "${ROOT}/alpha-emu/Cargo.toml" -- \
    "${ROOT}/analysis/cab/os3kneorom.os3kos"
fi

