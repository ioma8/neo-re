# neo-re

AlphaSmart NEO reverse-engineering notes and tools.

The core validated result is direct USB access: the NEO can be switched from HID
keyboard mode `081e:bd04` to direct USB mode `081e:bd01`, where AlphaWord files
and SmartApplet metadata can be read through bulk endpoints. The repo also
contains a validated `Alpha USB` SmartApplet path for Android backups without
root, proxy hardware, or typewriter fallback.

## What Is Here

- `real-check/`: live USB command tool for probing, switching to direct mode,
  listing/downloading AlphaWord files, SmartApplet operations, and recovery
  commands. Start with [real-check/README.md](real-check/README.md).
- `alpha-cli/`: backup TUI and GUI for desktop and Android. Build/run details
  are in [alpha-cli/README.md](alpha-cli/README.md).
- `aplha-rust-native/`: native Rust SmartApplet SDK and packer targeting
  `m68k-unknown-none-elf`. See [aplha-rust-native/README.md](aplha-rust-native/README.md).
- `poc/neotools/`: protocol parsers, packet builders, validators, and offline
  reverse-engineering helpers.
- `docs/`: protocol notes and device findings. The most important recovery note
  is [docs/2026-04-18-neo-recovery-runbook.md](docs/2026-04-18-neo-recovery-runbook.md).

## Current Status

- Desktop HID-to-direct switching is physically validated.
- Read-only AlphaWord listing and file download are physically validated.
- Android backup works when the NEO runs the `Alpha USB` SmartApplet before USB
  connection.
- Native Rust SmartApplet builds are structurally validated; treat new applets as
  hardware-facing experiments and validate carefully.
- OS/app flashing and clearing commands exist, but they are recovery tools and
  can rewrite persistent device state.

## Public Repo Hygiene

Local reverse-engineering artifacts are intentionally ignored:

- `analysis/`
- `exports/`
- `NEOManager3_9_3USPC/`

The Git history has been purged of proprietary NEO Manager installer payloads.
Keep those paths ignored.
