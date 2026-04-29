# AlphaGUI OS And Recovery Operations Design

Date: 2026-04-29

## Goal

Extend AlphaGUI's OS Operations tab so it exposes the system and recovery
operations that have been validated in this repository, without surfacing
speculative or known-broken repair flows.

The GUI must remain safer than the raw `real-check` CLI. Normal operations
should be easy to find. Recovery operations should be available, but visually
separated, strongly warned, and limited to flows proven by local scripts,
tests, and recovery documentation.

## Evidence Base

Validated or implemented sources:

- `real-check/src/real_check/__init__.py`
- `real-check/src/real_check/client.py`
- `real-check/tests/test_client.py`
- `real-check/tests/test_cli.py`
- `docs/2026-04-18-neo-recovery-runbook.md`
- `docs/superpowers/specs/2026-04-25-alpha-gui-tabbed-manager-design.md`
- `alpha-core/src/neo_client.rs`
- `alpha-gui-tauri/src-tauri/src/commands.rs`

Already present in the Rust/Tauri backend:

- direct USB detection and HID-to-direct switching
- file listing and backup
- full backup of files plus installed SmartApplets
- SmartApplet install and applet-area clear/reinstall
- bundled OS image flash through the updater/Small ROM protocol
- device restart after OS flash

Validated in `real-check` but not fully exposed in the GUI:

- `debug-applets`: read raw SmartApplet records
- `debug-attributes`: read raw AlphaWord file attribute records
- `restart-device`: restart through updater command `0x08`
- incremental stock applet restoration after applet-area clear

Known-bad or intentionally excluded:

- individual applet removal by index
- broad `restore-stock-applets --restart`
- SmartApplet operations while in Small ROM
- validator-disabled recovery OS flashing
- fake firmware flashing without a separate validated firmware image
- speculative Small ROM diagnostics, NVRAM clear, or reset routines

## Product Structure

The OS Operations tab should contain two areas:

1. Normal OS Operations
2. Advanced Recovery

Normal operations are shown by default. Advanced Recovery is collapsed or
visually separated and uses stronger warning copy. Both areas are available on
desktop and mobile when direct USB communication is available.

## Normal OS Operations

### Backup Everything

Keep the existing `Backup Everything` action.

Behavior:

- backs up all AlphaWord files
- backs up all installed SmartApplets
- writes to the configured backup directory
- shows structured progress for file and applet phases

This should remain the first action in the OS Operations tab and should be
recommended before every destructive operation.

### Reflash Bundled OS

Expose the existing bundled OS flash as a real enabled operation.

Behavior:

- uses the bundled stock OS image from `BundledCatalog`
- validates the OS image before flashing
- enters updater/Small ROM protocol through the proven sequence
- erases and writes OS segments
- finalizes the update
- restarts the device
- marks the device disconnected and asks the user to reconnect after reboot

Confirmation text must state:

- this can brick the device if interrupted
- USB must remain connected
- backup is recommended first
- the operation flashes the bundled stock OS image only

Do not expose the validator-disabled recovery OS.

### Restart Device

Add a normal `Restart Device` action.

Behavior:

- calls the existing updater restart command through Rust core
- confirms first because USB will disconnect temporarily
- after success, marks the GUI disconnected and returns to the connection
  screen

Copy:

> Restart the connected device. USB will disconnect while it reboots.

## Advanced Recovery

Advanced Recovery should be visually separated from routine operations. It may
be collapsed by default or rendered as a clearly marked warning section.

Intro copy:

> These tools are for repairing devices with damaged applet/file catalogs or
> for collecting diagnostics before a destructive repair. Use only with a
> current backup.

### Read Diagnostics

Expose read-only diagnostics equivalent to:

- `real-check debug-applets`
- `real-check debug-attributes`

Behavior:

- does not write to flash
- reads SmartApplet catalog records
- reads AlphaWord file attribute records
- shows the result in a copyable log panel
- can later add save-to-backup-folder, but the first implementation only needs
  a reliable copyable log

The output does not need to be beginner-friendly. It should preserve enough
technical detail to support future recovery/debugging work:

- page offset
- response status
- payload length/checksum
- applet row, id, name, and raw record bytes
- file slot attribute response status
- file name, file length, reserved length where parseable

### Restore Original Stock Applets

Expose a destructive recovery action that restores only the original bundled
stock SmartApplets.

This must not install `Alpha USB` by default or as part of this operation.
Alpha USB remains managed by the SmartApplets tab.

Behavior:

1. Require direct USB mode.
2. Offer or strongly recommend `Backup Everything` first.
3. Show a destructive confirmation.
4. Clear the SmartApplet area with the proven clear command.
5. Install bundled original stock applets one at a time.
6. After each install, list SmartApplets to verify the table remains readable.
7. Stop immediately on the first failed install or failed verification.
8. Refresh inventory after success.

The restore set comes from the bundled original stock applets already embedded
through `alpha-core/src/bundled_assets.rs`. The operation should exclude:

- `Alpha USB`
- user-added applets
- any validator-disabled recovery components
- individual applet removal

The final applet order should follow the proven recovery runbook order where
possible, rather than alphabetical display order:

1. AlphaWord Plus
2. Neo Font - Small
3. Neo Font - Medium
4. Neo Font - Large
5. Neo Font - Very Large
6. Neo Font - Extra Large
7. KeyWords
8. Control Panel
9. Beamer
10. AlphaQuiz
11. Calculator
12. Text2Speech Updater
13. SpellCheck Large USA

Do not use the old broad `restore-stock-applets --restart` flow. The recovery
runbook records that it cleared the applet area but did not complete safely on
a fragile device. The GUI flow must use the safer incremental install-and-verify
sequence.

### Small ROM Recovery Instructions

Add a recovery panel that explains how to use the bundled OS flash when the
device is already in updater/Small ROM state.

The panel should explain:

- Small ROM is used when normal boot is broken but the device can still expose
  direct USB/updater mode.
- Connect the device over USB after entering the updater/Small ROM state.
- AlphaGUI can use the same bundled OS flashing protocol once direct USB is
  available.
- SmartApplet operations are not available in Small ROM; local recovery notes
  observed status `0x92` for those commands.
- The validated physical entry sequence is to hold `Right Shift` + `,` + `.`
  + `/` while powering on, then enter password `ernie` when prompted. Firmware
  evidence: Small ROM entry gate at `0x00401378` checks encoded keys `0x6e`,
  `0x60`, `0x62`, and `0x73`; the password path at `0x004013c0` checks bytes
  `3a 3d 7f 30 3a` for `ernie`.

The panel should provide a `Reflash Bundled OS from Small ROM` button. It calls
the same backend as `Reflash Bundled OS`, with copy tailored to Small ROM. It
does not use a different OS image.

The GUI copy should say:

> Hold Right Shift + comma + period + slash while powering on, then enter the
> password "ernie" when prompted. Connect USB after the Small ROM Updater
> appears. SmartApplet operations are not available in Small ROM; use this only
> to reflash the bundled OS.

## Backend Changes

Add Rust/Tauri commands for operations not yet exposed:

- `restart_device`
- `read_recovery_diagnostics`
- `restore_original_stock_applets`

The implementation must stay in Rust. Do not call Python helpers from desktop
or mobile builds.

### `restart_device`

Use the existing `NeoClient::open_and_init()` path, then call
`client.restart_device()`.

After success the frontend should set connection state to disconnected.

### `read_recovery_diagnostics`

Add read-only methods in `alpha-core` equivalent to the Python client debug
helpers. Prefer structured data plus raw log lines so the GUI can display a
plain text diagnostic report without parsing in TypeScript.

The command should emit progress:

- entering updater mode
- reading applet records
- reading file attributes
- formatting diagnostics

### `restore_original_stock_applets`

Use the embedded stock applet bytes from the bundled catalog.

The command should:

- emit progress before clearing applet area
- clear applet area
- install each stock applet with chunk progress
- list applets after each install
- verify that the installed applet id appears in the refreshed list
- stop on any mismatch

Progress phases should include the current applet name and `N/M` counts.

## Frontend Changes

Update `OsOperations.tsx` so it has:

- normal operation cards:
  - Backup Everything
  - Reflash Bundled OS
  - Restart Device
- advanced recovery section:
  - Read Diagnostics
  - Restore Original Stock Applets
  - Small ROM Recovery Instructions and Reflash Bundled OS from Small ROM

Use existing progress and confirmation dialogs.

Add a diagnostic log modal/panel with:

- monospace text
- copy button

Saving the diagnostic log to the backup folder is a future enhancement. Do not
block the first implementation on it.

## Error Handling

Every destructive action must use confirmation.

Restore Original Stock Applets must stop on the first failure and show:

- failed phase
- current applet name, if any
- backend error
- instruction to avoid further writes until diagnostics are saved

Reflash Bundled OS must show a reconnect instruction after success or after the
device disappears during reboot.

Diagnostics should tolerate partial reads where possible and include the last
successful step in the report. Protocol parse failures should be visible in the
log rather than hidden.

## Testing

Add unit tests before implementation for:

- bundled stock restore order excludes Alpha USB
- stock restore plan contains only original stock applets
- diagnostics formatter includes applet and file-attribute sections
- restart command is exposed through Tauri command registration
- restore command stops if post-install verification fails
- Small ROM copy does not claim SmartApplet operations work in Small ROM

Run after each implementation step:

- `cargo test --manifest-path alpha-core/Cargo.toml`
- `cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml`
- `npm --prefix alpha-gui-tauri run typecheck`
- `npm --prefix alpha-gui-tauri run build`

For Android-sensitive changes, also run:

- `npm --prefix alpha-gui-tauri run tauri -- android build --debug`

## Non-Goals

- No validator-disabled recovery OS.
- No individual applet removal.
- No broad restore-stock-applets command.
- No firmware flash card unless a separate firmware image and protocol are
  validated.
- No speculative Small ROM operations.
- No Python helper calls from the app.
