# Forth Mini Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild `Forth Mini` as a firmware-driven four-line SmartApplet that evaluates input on `Enter` and does not freeze after the first keypress.

**Architecture:** Keep the applet adapter thin and let the firmware own lifecycle and key delivery. Put all REPL state and evaluation logic in a pure Rust module that renders four fixed 28-byte lines, and validate primarily through real firmware-launched emulator interaction rather than synthetic direct-entry paths.

**Tech Stack:** Rust 2024, `no_std`, native `m68k-unknown-none-elf` applet build, existing `alpha-neo-sdk`, existing `alpha-emu`, Cargo tests, emulator integration tests.

---

## File Map

- Modify: `aplha-rust-native/applets/forth_mini/src/main.rs`
  - Replace the current focus-loop/polling applet adapter with a firmware-driven callback adapter.
- Modify: `aplha-rust-native/applets/forth_mini/src/forth.rs`
  - Replace the current REPL implementation with a smaller pure state machine and screen model.
- Modify: `alpha-emu/src/firmware_session.rs`
  - Replace the old direct-message-heavy Forth Mini tests with firmware-launched interaction tests.
- Optional modify: `alpha-emu/src/main.rs`
  - Only if `--validate-forth-mini` still assumes synthetic direct entry and needs to be aligned with the new launch path.
- Optional modify: `aplha-rust-native/README.md`
  - Update the Forth Mini description after the redesign is stable.

## Task 1: Lock In the Failing Firmware-Launched Behavior

**Files:**
- Modify: `alpha-emu/src/firmware_session.rs`
- Test: `alpha-emu/src/firmware_session.rs`

- [ ] **Step 1: Write a failing emulator test for the real launch path**

Add one focused test that:

- boots the full system image
- opens `Forth Mini` through the normal firmware path already used by the emulator
- sends one printable key through the matrix event path
- asserts that the LCD changes and no exception/freeze is recorded

Target shape:

```rust
#[test]
fn forth_mini_accepts_first_printable_key_after_real_launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = boot_full_system_for_forth_validation()?;
    // TODO: launch through the normal firmware applet-list path
    let before = session.lcd_snapshot();
    session.tap_matrix_code_long(0x38);
    session.run_steps(300_000);
    let after = session.lcd_snapshot();
    assert_ne!(before.pixels, after.pixels);
    assert!(session.snapshot().last_exception.is_none());
    Ok(())
}
```

- [ ] **Step 2: Run the new test to verify it fails**

Run:

```bash
cargo test --manifest-path alpha-emu/Cargo.toml forth_mini_accepts_first_printable_key_after_real_launch -- --nocapture
```

Expected: FAIL because the current applet still freezes or does not update the LCD after the first key.

- [ ] **Step 3: Add one narrow regression test for evaluation on Enter**

Add a second focused test that:

- launches `Forth Mini` normally
- enters `1`, `Enter`, `2`, `Enter`, `+`, `Enter`
- asserts the output line shows `3`

Keep it firmware-launched and matrix-key-driven.

- [ ] **Step 4: Run both tests and record the failing behavior**

Run:

```bash
cargo test --manifest-path alpha-emu/Cargo.toml forth_mini_ -- --nocapture
```

Expected: at least one FAIL that reproduces the current broken launch/input behavior.

- [ ] **Step 5: Commit**

```bash
git add alpha-emu/src/firmware_session.rs
git commit -m "test: capture real Forth Mini launch failure"
```

## Task 2: Replace the Pure REPL Core with a Smaller State Model

**Files:**
- Modify: `aplha-rust-native/applets/forth_mini/src/forth.rs`
- Test: `aplha-rust-native/applets/forth_mini/src/forth.rs`

- [ ] **Step 1: Write failing unit tests for the target REPL behavior**

Add or rewrite tests so they describe the final reduced behavior exactly:

- fresh empty prompt line
- append printable bytes
- backspace removes one byte
- `Enter` evaluates the full input line
- `Enter` on `1`, `2`, `+` produces output `3`
- stack words work
- output/error line formatting is fixed-width and plain text

Example target shape:

```rust
#[test]
fn evaluates_line_only_after_enter() {
    let mut repl = Repl::new();
    repl.accept_byte(b'1');
    assert_eq!(repl.output_line(), blank_line());
    repl.accept_key(KeyAction::Enter);
    assert_eq!(repl.stack_depth(), 1);
}
```

- [ ] **Step 2: Run the Forth Mini unit tests to verify the new expectations fail**

Run:

```bash
cargo +nightly test --manifest-path aplha-rust-native/Cargo.toml -p forth-mini-applet
```

Expected: FAIL because the current `forth.rs` API and behavior do not match the smaller redesign.

- [ ] **Step 3: Rewrite `forth.rs` to the minimal pure model**

Implement:

- a small `ReplState`/`Repl` struct
- bounded stack
- bounded input buffer
- one visible output line
- methods such as:

```rust
pub fn accept_printable(&mut self, byte: u8)
pub fn backspace(&mut self)
pub fn enter(&mut self)
pub fn line_title(&self) -> [u8; 28]
pub fn line_stack(&self) -> [u8; 28]
pub fn line_output(&self) -> [u8; 28]
pub fn line_prompt(&self) -> [u8; 28]
```

Keep this file pure. No SDK calls, no trap assumptions, no applet callback code.

- [ ] **Step 4: Run the Forth Mini unit tests to verify they pass**

Run:

```bash
cargo +nightly test --manifest-path aplha-rust-native/Cargo.toml -p forth-mini-applet
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add aplha-rust-native/applets/forth_mini/src/forth.rs
git commit -m "refactor: rebuild Forth Mini REPL core"
```

## Task 3: Replace the Applet Adapter with Firmware-Driven Callbacks

**Files:**
- Modify: `aplha-rust-native/applets/forth_mini/src/main.rs`
- Test: `alpha-emu/src/firmware_session.rs`

- [ ] **Step 1: Write a failing adapter-focused emulator test if one is still missing**

Add one test that proves the applet responds correctly to:

- `on_focus` launch
- first printable character
- `Backspace`
- `Enter`
- exit key

Only add the missing case. Do not duplicate existing tests.

- [ ] **Step 2: Run the focused emulator test to verify it fails**

Run:

```bash
cargo test --manifest-path alpha-emu/Cargo.toml forth_mini_ -- --nocapture
```

Expected: FAIL while the old `main.rs` still uses the wrong architecture.

- [ ] **Step 3: Rewrite `main.rs` as a thin callback adapter**

Implement this shape:

```rust
impl Applet for ForthMini {
    fn on_focus(ctx: &mut Context) -> Status { /* reset state + redraw */ }
    fn on_char(ctx: &mut Context) -> Status { /* printable byte + redraw */ }
    fn on_key(ctx: &mut Context) -> Status { /* Enter/Backspace/Exit + redraw */ }
}
```

Rules:

- no polling loop
- no `pump_events`
- no `yield_once` loop
- no custom per-character drawing helper
- redraw the full four lines every time
- use only stable SDK helpers already validated in other applets

If session state must live in applet memory, keep the memory block explicit and
minimal. Do not reintroduce hidden architecture tricks.

- [ ] **Step 4: Run the applet unit tests and emulator tests**

Run:

```bash
cargo +nightly test --manifest-path aplha-rust-native/Cargo.toml -p forth-mini-applet
cargo test --manifest-path alpha-emu/Cargo.toml forth_mini_ -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add aplha-rust-native/applets/forth_mini/src/main.rs alpha-emu/src/firmware_session.rs
git commit -m "feat: switch Forth Mini to firmware-driven callbacks"
```

## Task 4: Rebuild and Validate the Native Applet Package

**Files:**
- Modify: `aplha-rust-native/applets/forth_mini/src/main.rs`
- Modify: `aplha-rust-native/applets/forth_mini/src/forth.rs`
- Output: `exports/applets/forth-mini.os3kapp`

- [ ] **Step 1: Build the native applet package**

Run:

```bash
cd aplha-rust-native && ./build.sh forth_mini
```

Expected: PASS and writes `../exports/applets/forth-mini.os3kapp`.

- [ ] **Step 2: Run the emulator validation command against the rebuilt applet**

Run:

```bash
cargo run --manifest-path alpha-emu/Cargo.toml -- ../analysis/cab/os3kneorom.os3kos --validate-forth-mini
```

Expected: PASS, or fail with a concrete new mismatch rather than the old freeze.

- [ ] **Step 3: If the validator still uses synthetic direct-entry assumptions, align it with the real launch path**

If needed, modify:

- `alpha-emu/src/main.rs`

so `--validate-forth-mini` launches through the normal firmware applet path
before typing keys.

- [ ] **Step 4: Re-run package and validation commands**

Run:

```bash
cd aplha-rust-native && ./build.sh forth_mini
cargo run --manifest-path alpha-emu/Cargo.toml -- ../analysis/cab/os3kneorom.os3kos --validate-forth-mini
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add aplha-rust-native/applets/forth_mini/src/main.rs aplha-rust-native/applets/forth_mini/src/forth.rs alpha-emu/src/main.rs exports/applets/forth-mini.os3kapp
git commit -m "build: validate redesigned Forth Mini applet"
```

## Task 5: Clean Up Old Assumptions and Update Docs

**Files:**
- Modify: `aplha-rust-native/README.md`
- Modify: `hypotheses.tsv`

- [ ] **Step 1: Update README to match the new architecture**

Replace outdated statements that describe Forth Mini as:

- event-driven with mixed `on_char`/`on_key` assumptions not matching the new code
- using obsolete prompt-rendering behavior
- relying on the old broken lifecycle

- [ ] **Step 2: Record the validated root cause and replacement strategy in `hypotheses.tsv`**

Add or update entries for:

- the old freeze-after-first-input hypothesis
- the validated replacement architecture

- [ ] **Step 3: Run repository checks for touched Rust code**

Run:

```bash
cargo test --manifest-path alpha-emu/Cargo.toml
cargo +nightly test --manifest-path aplha-rust-native/Cargo.toml -p alpha-neo-sdk
cargo +nightly test --manifest-path aplha-rust-native/Cargo.toml -p forth-mini-applet
git diff --check
```

Expected: all PASS.

- [ ] **Step 4: Commit**

```bash
git add aplha-rust-native/README.md hypotheses.tsv
git commit -m "docs: update Forth Mini redesign notes"
```

## Task 6: Final Manual Verification

**Files:**
- No source changes required unless verification finds a bug

- [ ] **Step 1: Launch the GUI emulator with the full system image**

Run:

```bash
cargo run --manifest-path alpha-emu/Cargo.toml -- ../analysis/cab/os3kneorom.os3kos
```

- [ ] **Step 2: Open Forth Mini through the SmartApplets menu and verify these cases manually**

Check:

- first printable key appears immediately
- no freeze after the first key
- `Backspace` works
- `1`, `Enter`, `2`, `Enter`, `+`, `Enter` shows `3`
- `.s` shows a stack summary
- exit key returns to the firmware menu

- [ ] **Step 3: If manual verification finds a bug, go back to the smallest failing automated test first**

Do not patch by inspection. Add or tighten a failing test, then repeat the
red-green cycle.

- [ ] **Step 4: Commit final fixes, if any**

```bash
git add <exact files changed>
git commit -m "fix: complete Forth Mini redesign verification"
```
