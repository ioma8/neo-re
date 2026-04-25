# Alpha GUI Tabbed Manager Redesign

Date: 2026-04-25

## Goal

Rework `alpha-cli` into a connection-gated AlphaSmart NEO manager with a shared desktop/mobile flow. The app should hide raw file paths for stock operations, bundle known-good applets and firmware images, and present clear progress for backup, applet flashing, OS flashing, and connection switching.

The implementation remains in the current Rust/egui app. Do not replatform to Tauri for this iteration.

## Scope

In scope:

- Connection-first screen with no tabs until direct USB is available.
- Desktop auto-switch from HID keyboard mode to direct USB when possible.
- Mobile HID guidance that tells the user to install and run Alpha USB from the desktop app first.
- Shared tab model for desktop and mobile: `Dashboard`, `SmartApplets`, `OS Operations`, `About`.
- Bundled original applets from the repo stock applet backup/export source plus bundled `Alpha USB`.
- Bundled firmware/system images for validated OS operations.
- Applet checklist flow that can install only newly selected applets, or clear and reinstall the checked set when removal/replacement is required.
- Structured progress for all long-running operations.
- Confirmation dialogs for destructive applet and OS operations.

Out of scope for this first redesign:

- Switching to Tauri.
- Supporting unvalidated AlphaSmart models beyond NEO.
- Inventing unvalidated Small ROM operations.
- Exposing raw stock applet or stock firmware paths as normal user workflow.

## Product Flow

### Connection Screen

The app opens to a single connection screen with no tabs. It polls device state and shows status, progress, and any last error.

States:

- `Missing`: prompt the user to connect the AlphaSmart NEO over USB.
- `HID` on desktop: show that keyboard mode was detected, then automatically attempt the existing HID-to-direct switch.
- `HID` on mobile: explain that mobile cannot perform this switch. The user must use the desktop app to flash `Alpha USB`, run that SmartApplet on the device, then reconnect to the phone.
- `Direct`: load inventory and reveal the main tabs.
- `Unavailable/Error`: show the failed step and a retry action.

Desktop auto-switch progress should show: HID detected, switching to direct, waiting for re-enumeration, connected.

### Navigation

Once direct USB is available, the app shows four tabs:

- `Dashboard`
- `SmartApplets`
- `OS Operations`
- `About`

Desktop uses a left navigation rail similar to the supplied HTML mockups. Mobile uses a bottom navigation bar. The content and workflow state should be shared; platform differences should be capability-based, not duplicated screens.

## Tabs

### Dashboard

Dashboard lists all AlphaWord files in a minimal list/table:

- slot
- file name
- file size
- per-file `Backup` action

The top-level action is `Backup All Files`.

Progress:

- listing files
- downloading slot `N`
- writing backup file
- completed count out of total

If no files are loaded, the dashboard should show an empty state and a refresh action.

### SmartApplets

The top action is:

`Flash Alpha USB SmartApplet for smartphone connection`

This flashes the bundled Alpha USB applet through the proven applet install path.

Below it is a unified applet checklist. The list is built from:

- bundled stock/original applets from the repo stock applet backup/export source
- bundled `Alpha USB`
- applets currently installed on the device
- any applets added from file during the current session

Initial checkbox state:

- installed applets are checked
- bundled applets not installed are unchecked
- added-from-file applets are checked after selection

When the checkbox set changes, `Flash to device` becomes enabled. The app classifies the change:

- Add-only: every currently installed applet remains checked, and only new applets are selected. Use direct applet install without clearing existing applets.
- Reconcile/reflash: any installed applet is unchecked, or the selected set requires replacing the applet area. Confirm, clear applet area, then install all checked applets.

The app should prompt the user before any clear/reinstall flow and state exactly how many applets will be installed and whether the applet area will be cleared.

`Add new applet from file` remains available, primarily for desktop. It adds the chosen file to the session checklist. Stock applets and Alpha USB do not require file picking.

Progress:

- classifying changes
- clearing applet area when needed
- installing applet `N` of `M`
- applet name currently being flashed
- final inventory refresh

### OS Operations

Top action:

`Backup Everything`

This backs up all AlphaWord files, all applets, and a manifest/settings snapshot where available.

Bundled OS operations:

- `Reflash Firmware`
- `Reflash System`

Both use bundled validated images and require one extra confirmation. The confirmation must state brick risk and require a stable USB connection.

Small ROM operations:

- Show only operations backed by repo-proven `real-check` helpers.
- Do not present speculative diagnostics or reset actions that have not been validated.

Progress:

- backup files phase
- backup applets phase
- manifest/settings phase
- OS image validation
- erase/write phases when exposed by the backend
- command/log progress when detailed flashing progress is not available

### About

About includes:

- `AlphaGUI` project description
- GitHub repository link
- documentation link if available
- app version from Cargo package metadata
- NEO-only validation note
- disclaimer that flashing can brick the device and is used at the user's own risk

## Architecture

Keep the current `alpha-cli` Rust/egui codebase. Split the large GUI surface into focused modules while preserving existing backend helpers.

Suggested modules:

- `gui.rs`: app state, task handling, routing, shared shell
- `gui_connection.rs`: connection screen and device gating
- `gui_dashboard.rs`: file listing and file backup UI
- `gui_applets.rs`: applet checklist, diff classification, applet flashing UI
- `gui_os.rs`: backup-everything, bundled OS flashing, Small ROM operation UI
- `gui_about.rs`: project information and warnings
- `bundled_assets.rs`: bundled applet/OS manifest and source lookup
- `operation_progress.rs`: progress model and shared progress UI

Shared state should model:

- selected tab
- device mode
- loaded file inventory
- loaded applet inventory
- bundled applet catalog
- bundled OS image catalog
- applet checklist state
- current confirmation dialog
- current operation progress

Platform differences should be represented as capabilities:

- desktop can auto-switch HID to direct
- mobile cannot auto-switch HID and must show Alpha USB instructions
- file picking for custom applets may be desktop-first
- direct communication is required before tabs are shown on all platforms

## Bundled Assets

Canonical source for stock/original applets:

- repo stock applet backup/export directory, plus `exports/applets/alpha-usb-native.os3kapp`

The bundled asset layer should support two modes:

- development mode: load from repo paths for fast iteration
- packaged mode: use embedded bytes or packaged resources so users do not choose paths for stock workflows

Bundled OS images should follow the same pattern. Stock firmware/system operations should not require the user to browse for an image.

Each bundled item should expose:

- display name
- applet id or image id
- version where known
- size
- source kind: bundled stock, bundled Alpha USB, added from file, installed-only
- bytes/path resolver for flashing

## Progress And Confirmation

Replace task feedback that is only spinner/log based with structured progress.

Progress event fields:

- title
- phase
- optional item label
- completed count
- total count
- indeterminate flag
- log line

Use determinate progress when counts are known, such as files or applets. Use indeterminate progress for backend commands that do not expose granular byte/segment progress yet.

The connection screen shows progress inline. Main tabs use a shared operation overlay/dialog. After completion, the last operation summary remains visible.

Confirmations:

- applet clear/reinstall confirmation
- OS firmware reflash confirmation
- OS system reflash confirmation
- any validated Small ROM operation that can alter device state

## Testing Strategy

Start with pure workflow tests before broad UI edits.

Tests to add:

- connection gate hides tabs until direct USB
- desktop HID detection chooses auto-switch
- mobile HID detection chooses instruction state
- applet diff classification detects add-only vs clear/reinstall
- installed applets initialize checked
- bundled but missing applets initialize unchecked
- added-from-file applets are included in the checklist
- OS operations resolve bundled images without user paths
- progress events represent known-count backup and applet flashing phases

Validation commands during implementation:

- `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui` after each small UI/code change
- `cargo test --manifest-path alpha-cli/Cargo.toml` after workflow/model changes
- `cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-linux-android --bin alpha-gui` after mobile-relevant changes

## Open Implementation Notes

- The exact repo directory for original stock applets should be discovered during implementation and wrapped in `bundled_assets.rs`.
- If `real-check` only reports OS flashing progress after completion, the GUI should show honest phase-level progress and live logs rather than fake byte-level precision.
- The first implementation pass should prioritize working flows and tests over perfect visual parity with the HTML mockups.
