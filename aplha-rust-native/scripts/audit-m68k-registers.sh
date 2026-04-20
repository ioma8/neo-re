#!/usr/bin/env bash
set -euo pipefail

ELF="${1:-}"
if [[ -z "${ELF}" ]]; then
  echo "usage: scripts/audit-m68k-registers.sh <linked-m68k-elf>" >&2
  exit 2
fi

if ! command -v m68k-elf-objdump >/dev/null 2>&1; then
  echo "m68k-elf-objdump not found" >&2
  exit 1
fi

if ! command -v rg >/dev/null 2>&1; then
  echo "rg not found" >&2
  exit 1
fi

matches="$(m68k-elf-objdump -dr "${ELF}" | rg -n '%a5|%d7|a5@|d7' || true)"
if [[ -z "${matches}" ]]; then
  echo "register audit: no a5/d7 references found"
  exit 0
fi

count="$(printf '%s\n' "${matches}" | wc -l | tr -d ' ')"
echo "register audit: found ${count} a5/d7 references"
echo
printf '%s\n' "${matches}" | head -80
echo
echo "Note: this is an audit, not a failure. Official Calculator also uses a5/d7."
echo "Betawise's -ffixed-a5 -ffixed-d7 reserves them from C compiler allocation;"
echo "it does not mean a valid final applet binary can never mention those registers."
