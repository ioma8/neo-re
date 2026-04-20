# Native m68k Rust Applet SDK Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `aplha-rust-native`, a native Cargo-based m68k Rust SmartApplet SDK and packer with an Alpha USB applet.

**Architecture:** The workspace separates applet-side `no_std` SDK code, a native m68k applet crate, and a host-side packer CLI. `build.sh` orchestrates Cargo’s m68k build and host packaging into `.os3kapp`. The Alpha USB applet links a small binutils-assembled entry stub to avoid unrelocated absolute local references from rustc output.

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

### Task 3: no_std SDK

**Files:**
- Create: `aplha-rust-native/crates/alpha-neo-sdk/Cargo.toml`
- Create: `aplha-rust-native/crates/alpha-neo-sdk/src/lib.rs`
- Create: `aplha-rust-native/crates/alpha-neo-sdk/src/abi.rs`
- Create: `aplha-rust-native/crates/alpha-neo-sdk/src/display.rs`
- Create: `aplha-rust-native/crates/alpha-neo-sdk/src/usb.rs`

- [ ] Define message IDs, status values, applet manifest types, and OS call wrappers.
- [ ] Provide a `dispatch` helper for applets to route messages to handlers.
- [ ] Keep unsafe calls small and isolated.
- [ ] Run host-side `cargo check --workspace`.

### Task 4: Native Alpha USB Applet

**Files:**
- Create: `aplha-rust-native/applets/alpha_usb/Cargo.toml`
- Create: `aplha-rust-native/applets/alpha_usb/src/lib.rs`
- Create: `aplha-rust-native/applets/alpha_usb/memory.x`

- [ ] Implement the Alpha USB manifest and handlers using the SDK.
- [ ] Export the m68k entrypoint and panic handler.
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
