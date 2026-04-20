# Alpha Emulator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `alpha-emu`, a desktop Rust emulator that loads the real `Alpha USB` `.os3kapp` package and executes enough m68k/NEO behavior for focus and USB attach flows.

**Architecture:** Keep the emulator narrow and testable: package parsing, domain state, CPU/instruction stepping, NEO OS shims, host message dispatch, and GUI are separate modules. Start with the real `Alpha USB` package and implement only the reached instruction/OS surface.

**Tech Stack:** Rust 2024, `m68000` for m68k execution/disassembly support where practical, `eframe/egui` for desktop UI, `thiserror`/`anyhow` for errors, `tracing` for diagnostics.

---

### Task 1: Scaffold `alpha-emu`

**Files:**
- Create: `alpha-emu/Cargo.toml`
- Create: `alpha-emu/src/lib.rs`
- Create: `alpha-emu/src/main.rs`
- Create: `alpha-emu/src/domain.rs`

- [ ] Create the Rust 2024 binary project with GUI and error dependencies.
- [ ] Add domain types for LCD rows, USB mode, applet metadata, and emulator status.
- [ ] Run `cargo check` from `alpha-emu`.

### Task 2: Load `.os3kapp`

**Files:**
- Create: `alpha-emu/src/os3kapp.rs`
- Modify: `alpha-emu/src/lib.rs`

- [ ] Implement a small parser for the existing OS3KApp header and payload layout used by native applet exports.
- [ ] Add tests that load `../exports/applets/alpha-usb-native.os3kapp`.
- [ ] Run `cargo check` and `cargo test`.

### Task 3: Add Focus/USB Execution Core

**Files:**
- Create: `alpha-emu/src/cpu68k.rs`
- Create: `alpha-emu/src/neo_os.rs`
- Create: `alpha-emu/src/applet_host.rs`
- Modify: `alpha-emu/src/lib.rs`

- [ ] Build a bounded interpreter host for the `Alpha USB` applet.
- [ ] Use m68k disassembly/execution support where useful, but keep explicit shims for known Alpha USB OS calls.
- [ ] Implement `open_applet()` for message `0x19`.
- [ ] Implement `simulate_usb_attach()` for message `0x30001`.
- [ ] Add tests for expected LCD text and HID-to-direct mode transition.
- [ ] Run `cargo check` and `cargo test`.

### Task 4: Add Desktop GUI

**Files:**
- Create: `alpha-emu/src/gui.rs`
- Modify: `alpha-emu/src/main.rs`

- [ ] Add an `eframe/egui` window with LCD rendering, applet metadata, USB mode, status, and error panel.
- [ ] Add buttons: `Open applet`, `Simulate USB attach`, `Reset emulator`.
- [ ] Accept an optional `.os3kapp` path argument, defaulting to `../exports/applets/alpha-usb-native.os3kapp`.
- [ ] Run `cargo check`.

### Task 5: Final Validation

**Files:**
- Modify docs if needed after implementation.

- [ ] Run `cargo fmt --all`.
- [ ] Run `cargo check`.
- [ ] Run `cargo test`.
- [ ] Run a strict but practical `cargo clippy` pass and fix warnings.
- [ ] Smoke-run `cargo run -- ../exports/applets/alpha-usb-native.os3kapp`.

