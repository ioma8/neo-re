# Basic Writer SmartApplet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust `Basic Writer` SmartApplet with eight retrievable owned files, AlphaWord-like typing, arrow navigation, 4-line scrolling, file-key switching, visible cursor, and autosave.

**Architecture:** Add a new `basic_writer` applet crate with a host-testable pure editor model, thin applet callback glue, and a storage module that gates persistence on proven SmartApplet-owned file calls. Extend the packer to declare eight owned files and write metadata. Extend the emulator only as needed to discover and validate `Basic Writer`.

**Tech Stack:** Rust 2024, `no_std` m68k applet target, existing `alpha-neo-sdk`, `alpha-neo-pack`, `alpha-emu`, radare2 for bounded AlphaWord trap analysis, Cargo unit tests, headless emulator validation.

---

## File Map

- Create `aplha-rust-native/applets/basic_writer/Cargo.toml`: applet package declaration.
- Create `aplha-rust-native/applets/basic_writer/memory.x`: linker script copied from current applets.
- Create `aplha-rust-native/applets/basic_writer/src/editor.rs`: pure editor model and tests.
- Create `aplha-rust-native/applets/basic_writer/src/storage.rs`: slot serialization plus firmware storage wrappers.
- Create `aplha-rust-native/applets/basic_writer/src/main.rs`: `Applet` implementation and drawing/input glue.
- Modify `aplha-rust-native/Cargo.toml`: add applet workspace member.
- Modify `aplha-rust-native/build.sh`: add `basic_writer`.
- Modify `aplha-rust-native/crates/alpha-neo-pack/src/os3kapp.rs`: support `file_count` and owned file metadata.
- Modify `aplha-rust-native/crates/alpha-neo-pack/src/main.rs`: add `basic-writer` manifest.
- Modify `aplha-rust-native/crates/alpha-neo-sdk/src/keyboard.rs`: add logical file/arrow key helpers if missing.
- Modify `aplha-rust-native/crates/alpha-neo-sdk/src/display.rs` and `context.rs`: add cursor/storage trap wrappers only if proven.
- Modify `alpha-emu/src/memory.rs` and `alpha-emu/src/main.rs`: discover/launch/validate `Basic Writer` if needed.

## Task 1: Packer Declares Eight Owned Files

**Files:**
- Modify: `aplha-rust-native/crates/alpha-neo-pack/src/os3kapp.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-pack/src/main.rs`

- [ ] Write failing packer test: `packages_basic_writer_shape` expects id `0xA132`, name `Basic Writer`, `file_count = 8`, footer, and write metadata records for `0x0105/0x100B` and `0xC001/0x8011..0x8018`.
- [ ] Run: `cargo test -p alpha-neo-pack packages_basic_writer_shape` from `aplha-rust-native`; expect failure because `file_count`/manifest support is absent.
- [ ] Add `file_count: u8` and `owned_file_write_metadata: bool` to `AppletManifest`; write header byte `0x17` from `file_count`; keep existing applets at `0`.
- [ ] Add `AppletName::BasicWriter` and manifest with `id: 0xA132`, `base_memory_size: 0x7000`, `extra_memory_size: 0x2000`, `file_count: 8`, `owned_file_write_metadata: true`.
- [ ] Run: `cargo test -p alpha-neo-pack packages_basic_writer_shape`; expect pass.
- [ ] Run: `cargo check -p alpha-neo-pack`; expect pass.

## Task 2: Pure Editor Core

**Files:**
- Create: `aplha-rust-native/applets/basic_writer/src/editor.rs`
- Create: `aplha-rust-native/applets/basic_writer/src/main.rs`
- Create: `aplha-rust-native/applets/basic_writer/Cargo.toml`
- Create: `aplha-rust-native/applets/basic_writer/memory.x`
- Modify: `aplha-rust-native/Cargo.toml`

- [ ] Add the new crate with minimal `main.rs` exporting `BasicWriter`, no behavior yet.
- [ ] Write failing editor tests for insert, newline, backspace, left/right, up/down across wrapped rows, viewport scrolling, 4096-byte limit, unsupported-byte dropping, per-slot RAM cursor/viewport restoration, and fresh applet restart semantics.
- [ ] Run: `cargo test -p basic-writer-applet editor`; expect failing compile/test because editor is not implemented.
- [ ] Implement minimal `Editor`, `Document`, `SlotState`, and constants: `FILE_COUNT = 8`, `MAX_FILE_BYTES = 4096`, `SCREEN_ROWS = 4`, `SCREEN_COLS = 28`.
- [ ] Implement byte insertion/deletion, wrapping coordinate calculation, cursor movement, viewport clamp, slot switching, and fresh session initialization that resets active slot to 1 while placing cursor at EOF for loaded text with no RAM navigation state.
- [ ] Run: `cargo test -p basic-writer-applet editor`; expect pass.
- [ ] Run: `cargo check -p basic-writer-applet`; expect pass.

## Task 3: Keyboard Mapping And Visible Drawing

**Files:**
- Modify: `aplha-rust-native/applets/basic_writer/src/main.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/keyboard.rs` if needed
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/display.rs` if needed

- [ ] Write failing tests for raw-key classification: arrows map to navigation actions, file keys map to slots `1..8`, printable decoded keys map to text input, unknown keys are ignored.
- [ ] Run: `cargo test -p basic-writer-applet key`; expect failure.
- [ ] Use bounded emulator/source evidence to identify existing raw codes for arrows/file keys; add helpers in applet or SDK without changing existing behavior.
- [ ] Implement applet input action classification and `draw_rows` that renders four 28-byte rows and a visible cursor marker.
- [ ] Run: `cargo test -p basic-writer-applet key`; expect pass.
- [ ] Run: `cargo check -p basic-writer-applet`; expect pass.

## Task 4: Storage Proof And Autosave

**Files:**
- Create: `aplha-rust-native/applets/basic_writer/src/storage.rs`
- Modify: `aplha-rust-native/applets/basic_writer/src/main.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/context.rs` and new SDK module if proven file traps are needed

- [ ] Create or update `hypotheses.tsv` with the storage question: which firmware trap sequence binds, reads, writes, and commits SmartApplet-owned file slots.
- [ ] Run bounded radare2 probes against `analysis/cab/alphawordplus.os3kapp` around the known file trap stubs (`A1EC`, `A2BC`, `A2C0`, `A2DC`, `A2EC`, `A2FC`, `A2D8`) and record only concise evidence.
- [ ] Write failing storage tests for serialization: load drops unsupported bytes, save clamps at 4096 bytes, slot numbers are `1..8`, and cursor/viewport state is not embedded in saved file bytes.
- [ ] Run: `cargo test -p basic-writer-applet storage`; expect failure.
- [ ] Implement host-side serialization/deserialization.
- [ ] If safe firmware load, save, and commit paths are all proven, add narrow SDK wrappers and applet autosave/load calls. If any of load, save, or commit is not proven, keep storage returning a hard failure and stop implementation with hypothesis notes.
- [ ] Run: `cargo test -p basic-writer-applet storage`; expect pass for serialization and, if wrappers exist, host API shape.
- [ ] Run: `cargo check -p basic-writer-applet`; expect pass.

## Task 5: Build Script And Package

**Files:**
- Modify: `aplha-rust-native/build.sh`

- [ ] Write failing smoke command: `./build.sh basic_writer` from `aplha-rust-native`; expect usage failure.
- [ ] Add `basic_writer` case: package `basic-writer-applet`, packer name `basic-writer`, output `../exports/applets/basic-writer.os3kapp`.
- [ ] Run: `cargo check -p alpha-neo-pack`.
- [ ] Run: `cargo check -p basic-writer-applet`.
- [ ] Run: `./build.sh basic_writer` from `aplha-rust-native`; expect output file created.

## Task 6: Headless Emulator Validation

**Files:**
- Modify: `alpha-emu/src/memory.rs`
- Modify: `alpha-emu/src/main.rs`
- Modify: `alpha-emu/src/firmware_session.rs` tests if needed

- [ ] Write failing emulator test or CLI validation path that discovers `Basic Writer` in `exports/applets/basic-writer.os3kapp`.
- [ ] Run the focused emulator test; expect failure because discovery/launch helper lacks `Basic Writer`.
- [ ] Add discovery/launch/validation support for `Basic Writer`.
- [ ] Run emulator validation with scripted text, arrows, file switching, and LCD output. Expected: text appears, cursor visible, arrows affect insertion, viewport scrolls, slots are independent.
- [ ] If storage wrappers are implemented, retrieve slot contents through existing updater tooling or added validator and confirm exact contents for at least two switched files, for example slot 1 contains `one` and slot 2 contains `two`.
- [ ] Run: `cargo test --manifest-path alpha-emu/Cargo.toml`.
- [ ] Run: `cargo check --manifest-path alpha-emu/Cargo.toml`.

## Task 7: Final Verification

**Files:**
- All changed files.

- [ ] Run: `cargo test` from `aplha-rust-native`.
- [ ] Run: `cargo check` from `aplha-rust-native`.
- [ ] Run: `cargo test --manifest-path alpha-emu/Cargo.toml`.
- [ ] Run: `cargo check --manifest-path alpha-emu/Cargo.toml`.
- [ ] Run: `./build.sh basic_writer` from `aplha-rust-native`.
- [ ] Run bounded `r2` package check on `exports/applets/basic-writer.os3kapp` to verify header id, file count, entry, and footer.
- [ ] Summarize persistence status honestly: fully proven retrievable files, or blocked with exact failing hypothesis.
