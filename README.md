# neo-re

AlphaSmart NEO reverse-engineering notes and working tools.

The core validated result is direct USB access: the NEO can be switched from HID
keyboard mode `081e:bd04` to direct USB mode `081e:bd01`, where AlphaWord files
and SmartApplet metadata can be read through bulk endpoints. The repo also
contains a validated `Alpha USB` SmartApplet path for Android backups without
root, proxy hardware, or typewriter fallback.

## Implemented And Working

- Desktop USB detection, HID-to-direct switching, and direct endpoint probing.
- Read-only AlphaWord slot listing and file download.
- SmartApplet listing, dumping, installing, clearing, and stock restore tooling.
- NEO OS image flashing/recovery tooling used successfully on physical hardware.
- Desktop TUI backup app and desktop GUI backup app.
- Android GUI backup path using the `Alpha USB` SmartApplet to enter direct USB.
- Native Rust SmartApplet SDK/packer targeting `m68k-unknown-none-elf`.
- Betawise-derived C SmartApplet workflow for reliable NEO applets, including
  `Basic Writer`, `Forth Mini`, and `WriteOrDie`.
- Native Rust SmartApplet: validated `Alpha USB`.
- Desktop/headless SmartApplet emulator for stock firmware plus repo applets.

## Main Components

- `real-check/`: live USB command tool for probing, switching to direct mode,
  listing/downloading AlphaWord files, SmartApplet operations, and recovery
  commands. Start with [real-check/README.md](real-check/README.md).
- `alpha-cli/`: backup TUI and GUI for desktop and Android. Build/run details
  are in [alpha-cli/README.md](alpha-cli/README.md).
- `aplha-rust-native/`: native Rust SmartApplet SDK and packer targeting
  `m68k-unknown-none-elf`. See [aplha-rust-native/README.md](aplha-rust-native/README.md).
- `smartapplets/`: current validated Betawise-derived C applet workflow and
  reusable NEO applet SDK helpers. See [smartapplets/README.md](smartapplets/README.md).
- `alpha-emu/`: desktop/headless SmartApplet emulator using `m68000` plus
  emulated NEO OS services and applet validators. See
  [alpha-emu/README.md](alpha-emu/README.md).
- `poc/neotools/`: protocol parsers, packet builders, validators, and offline
  reverse-engineering helpers.
- `docs/`: protocol notes and device findings. The most important recovery note
  is [docs/2026-04-18-neo-recovery-runbook.md](docs/2026-04-18-neo-recovery-runbook.md).
