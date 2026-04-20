# neo-re

Reverse-engineering notes and tooling for AlphaSmart NEO USB, AlphaWord file
backup, SmartApplet handling, and custom SmartApplet experiments.

The repo is centered on one validated finding: a NEO can be switched from HID
keyboard mode `081e:bd04` into direct USB mode `081e:bd01`. Direct mode exposes
bulk endpoints that can list and read AlphaWord files, inspect SmartApplets, and
install or recover applets/OS images through the reverse-engineered protocol.

## Current State

- Direct USB switching from a desktop host is validated with the HID output
  report sequence used by NEO Manager.
- Read-only AlphaWord file listing and file download are validated against a
  physical NEO.
- The `Alpha USB` SmartApplet path is validated as the no-root Android route:
  launch the applet on the NEO, connect to Android, and the device re-enumerates
  in direct USB mode for backups.
- A native Rust SmartApplet SDK exists and can build the Alpha USB applet for
  `m68k-unknown-none-elf`.
- OS/app recovery commands exist and were used successfully, but they are
  destructive and should be treated as recovery tooling.

## Main Directories

- `real-check/`: live USB probe and command tool. This is the main place for
  device operations such as `watch`, `switch-to-direct`, `list`, `get`,
  `install-applet`, `clear-applet-area`, and `install-os-image`.
- `alpha-cli/`: end-user backup application. It includes a TUI and GUI for
  listing NEO files and saving validated text backups.
- `aplha-rust-native/`: native Rust SmartApplet SDK and packer. Applets are
  authored as Rust callbacks and built with Cargo for the m68k target.
- `poc/neotools/`: protocol parsing, packet builders, validators, and offline
  reverse-engineering helpers.
- `docs/`: findings, protocol notes, SmartApplet dataflow, direct USB notes,
  and the physical-device recovery runbook.

## Local Artifact Directories

The following are intentionally ignored and should not be pushed:

- `analysis/`: local extracted binaries, dumps, and reverse-engineering scratch
  data.
- `exports/`: generated applets, document backups, comparison outputs, and other
  local generated files.
- `NEOManager3_9_3USPC/`: local proprietary NEO Manager installer extraction.

The Git history has been purged of the proprietary installer payloads. Keep this
repo public-safe by leaving those paths ignored.

## Common Commands

Check USB mode:

```bash
uv run --project real-check real-check watch --timeout 5
```

Switch a desktop-connected NEO to direct USB:

```bash
uv run --project real-check real-check switch-to-direct
uv run --project real-check real-check probe
```

List AlphaWord files:

```bash
uv run --project real-check real-check list
```

Build the native Rust Alpha USB applet:

```bash
cd aplha-rust-native
./build.sh alpha_usb
```

Run the desktop backup TUI:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml
```

## Building The GUI

Desktop GUI development run:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml --bin alpha-gui
```

Desktop GUI release build:

```bash
cargo build --manifest-path alpha-cli/Cargo.toml --bin alpha-gui --release
```

The desktop binary is written under:

```text
alpha-cli/target/release/alpha-gui
```

Validate desktop GUI targets:

```bash
cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-apple-darwin --bin alpha-gui
```

Android GUI target check:

```bash
cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-linux-android --bin alpha-gui
```

Build the Android debug APK:

```bash
ANDROID_NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358" cargo apk build --manifest-path alpha-cli/Cargo.toml --lib
```

APK output:

```text
alpha-cli/target/debug/apk/alpha-gui.apk
```

Install the debug APK over ADB:

```bash
adb install -r alpha-cli/target/debug/apk/alpha-gui.apk
```

Android direct USB access requires launching the `Alpha USB` SmartApplet on the
NEO before connecting it to the phone. A plain HID-keyboard NEO is blocked from
normal apps by Android's USB host stack.

Read the detailed command safety guide before writing to the device:

```text
real-check/README.md
```

## Safety Notes

Read-only commands such as `watch`, `probe`, `list`, `applets`, `verify-get`,
and `get --output` do not modify device flash. SmartApplet and OS commands such
as `install-applet`, `clear-applet-area`, `restore-stock-applets`, and
`install-os-image` do write persistent device state.

Before any write command, back up what you can and keep the recovery runbook
nearby:

```text
docs/2026-04-18-neo-recovery-runbook.md
```
