# Native m68k Rust Applet SDK Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `aplha-rust-native`, a native Cargo-based m68k Rust SmartApplet SDK and packer with an Alpha USB applet authored as real Rust callbacks.

**Architecture:** The workspace separates applet-side `no_std` SDK code, a native m68k applet crate, and a host-side packer CLI. `build.sh` orchestrates Cargo’s m68k build and host packaging into `.os3kapp`. The SDK owns the shared m68k ABI shell and trap wrappers; applet crates implement Rust `Applet` callbacks and do not contain per-applet assembly.

**Tech Stack:** Rust 2024, nightly `build-std`, `m68k-unknown-none-elf`, `m68k-elf-ld`, `goblin` for ELF parsing, shell wrapper for build orchestration.

---

### Task 1: Workspace Skeleton

**Files:**
- Create: `aplha-rust-native/Cargo.toml`
- Create: `aplha-rust-native/rust-toolchain.toml`
- Create: `aplha-rust-native/.cargo/config.toml`
- Create: `aplha-rust-native/build.sh`

- [ ] Create a Rust workspace with members `crates/alpha-neo-sdk`, `crates/alpha-neo-pack`, and `applets/alpha_usb`.
- [ ] Add a wrapper script that validates the applet name and delegates build/package steps.
- [ ] Run `cargo check --workspace` from `aplha-rust-native`.

### Task 2: Host Packer

**Files:**
- Create: `aplha-rust-native/crates/alpha-neo-pack/Cargo.toml`
- Create: `aplha-rust-native/crates/alpha-neo-pack/src/main.rs`
- Create: `aplha-rust-native/crates/alpha-neo-pack/src/os3kapp.rs`
- Create: `aplha-rust-native/crates/alpha-neo-pack/src/elf.rs`

- [ ] Port the validated OS3KApp header builder and AlphaWord metadata table.
- [ ] Parse ELF bytes and extract the `.text` section.
- [ ] Add tests for header fields, metadata sentinel, and ELF extraction failure cases.
- [ ] Run `cargo test -p alpha-neo-pack`.

### Task 3: no_std SDK Runtime and Callback API

**Files:**
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/Cargo.toml`
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/lib.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/abi.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/display.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-sdk/src/usb.rs`
- Create: `aplha-rust-native/crates/alpha-neo-sdk/src/context.rs`
- Create: `aplha-rust-native/crates/alpha-neo-sdk/src/runtime.rs`

- [ ] Define `Applet`, `Context`, `Screen`, `Usb`, `System`, and `Identity` high-level APIs.
- [ ] Move the NEO stack ABI entry shell into the shared SDK runtime.
- [ ] Add `export_applet!(Type)` so applet crates expose Rust callbacks without applet-local assembly.
- [ ] Keep unsafe calls small and isolated.
- [ ] Run host-side `cargo check --workspace`.

### Task 4: Native Alpha USB Applet

**Files:**
- Modify: `aplha-rust-native/applets/alpha_usb/Cargo.toml`
- Modify: `aplha-rust-native/applets/alpha_usb/src/main.rs`
- Delete: `aplha-rust-native/applets/alpha_usb/src/entry.s`
- Delete: `aplha-rust-native/applets/alpha_usb/build.rs`
- Modify: `aplha-rust-native/applets/alpha_usb/memory.x`

- [ ] Implement the Alpha USB handlers using Rust `Applet` callbacks.
- [ ] Export the m68k entrypoint through `alpha_neo_sdk::export_applet!(AlphaUsb)`.
- [ ] Build with `cargo +nightly build -p alpha-usb-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort`.

### Task 5: End-to-End Build

**Files:**
- Modify: `aplha-rust-native/build.sh`
- Create: `aplha-rust-native/README.md`

- [ ] Make `./build.sh alpha_usb` build the native ELF and package it.
- [ ] Write output to `exports/applets/alpha-usb-native.os3kapp`.
- [ ] Validate package structure with the packer.
- [ ] Document required tools and exact commands.
- [ ] Run final `cargo fmt`, `cargo test --workspace`, and `./build.sh alpha_usb`.
