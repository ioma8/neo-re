# Forth Mini Native Applet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a native Rust `Forth Mini` SmartApplet with a minimal interactive Forth-like REPL.

**Architecture:** Keep the interpreter pure and host-testable inside the applet crate, then wire it to the NEO through a small SDK keyboard/display extension. The applet owns state on the `on_focus` stack and runs a calculator-style key polling loop; it does not use USB callbacks or persistent device storage.

**Tech Stack:** Rust 2024, `no_std`, `m68k-unknown-none-elf`, existing `alpha-neo-sdk` and `alpha-neo-pack`.

---

### Task 1: Host-Testable Forth Core

**Files:**
- Create: `aplha-rust-native/applets/forth_mini/Cargo.toml`
- Create: `aplha-rust-native/applets/forth_mini/memory.x`
- Create: `aplha-rust-native/applets/forth_mini/src/main.rs`
- Modify: `aplha-rust-native/Cargo.toml`

- [x] Add a new applet crate with a `Repl` struct that supports signed decimal integers and words `+ - * / mod dup drop swap over . .s clear`.
- [x] Add unit tests for arithmetic, stack manipulation, underflow, divide-by-zero, printing, and unknown words.
- [x] Run `cargo +nightly test -p forth-mini-applet` and confirm tests pass.

### Task 2: SDK Display And Keyboard Helpers

**Files:**
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/lib.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/context.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/display.rs`
- Create: `aplha-rust-native/crates/alpha-neo-sdk/src/keyboard.rs`

- [x] Add dynamic display helpers for stack-local byte slices and fixed-width row clearing.
- [x] Add keyboard polling wrappers for traps `A094`, `A09C`, and `A0A4`.
- [x] Run `cargo +nightly check` after the SDK edit.

### Task 3: Applet UI Loop

**Files:**
- Modify: `aplha-rust-native/applets/forth_mini/src/main.rs`

- [x] Implement `Applet for ForthMini` with id `0xA131`.
- [x] In `on_focus`, draw `Forth Mini`, then enter a key polling loop.
- [x] Printable ASCII appends to the prompt, enter evaluates the line, and backspace edits input.
- [x] Keep screen output to six NEO text rows.
- [x] Run `cargo +nightly check -p forth-mini-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort --release`.

### Task 4: Packaging And Validation

**Files:**
- Modify: `aplha-rust-native/crates/alpha-neo-pack/src/main.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-pack/src/os3kapp.rs`
- Modify: `aplha-rust-native/build.sh`
- Modify: `aplha-rust-native/README.md`

- [x] Add `forth-mini` manifest packaging to `alpha-neo-pack`.
- [x] Add `./build.sh forth_mini` support that writes `../exports/applets/forth-mini.os3kapp`.
- [x] Add packer shape tests for Forth Mini.
- [x] Run `./build.sh forth_mini`.
- [x] Verify no ELF relocations and no `.got`/`.got.plt`.
- [x] Validate the `.os3kapp` with `poc/neotools`.
