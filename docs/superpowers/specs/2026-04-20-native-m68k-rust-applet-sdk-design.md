# Native m68k Rust SmartApplet SDK Design

## Goal

Create `aplha-rust-native`, a new Rust workspace for writing AlphaSmart NEO SmartApplets as native `no_std` Rust compiled by Cargo for `m68k-unknown-none-elf`.

The first applet is a functional native Rust equivalent of the known working Alpha USB SmartApplet. Byte-for-byte equivalence with the Python and previous Rust generated `.os3kapp` files is explicitly out of scope for this iteration.

## Build Model

The applet source builds with Cargo directly:

```sh
cargo +nightly build -p alpha-usb-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort
```

Because Cargo does not provide a reliable post-build hook, packaging is handled by a small wrapper:

```sh
./build.sh alpha_usb
```

`build.sh` runs the m68k Cargo build, then invokes a host-side Rust packer to convert the applet ELF into an `.os3kapp` image.

## Workspace Layout

```text
aplha-rust-native/
  build.sh
  Cargo.toml
  rust-toolchain.toml
  .cargo/config.toml
  crates/
    alpha-neo-sdk/
    alpha-neo-pack/
  applets/
    alpha_usb/
```

`alpha-neo-sdk` is `#![no_std]` and contains the high-level applet API, message IDs, status values, OS trap bindings, and helper contexts.

`alpha-neo-pack` is a host CLI. It parses the m68k ELF, extracts the applet entry bytes, wraps them in the known OS3KApp container, writes AlphaWord metadata where requested, and validates the result.

`applets/alpha_usb` is a `#![no_std]` applet crate. It defines the manifest and message handlers using the SDK.

## Native ABI Shape

The applet exports one m68k entrypoint. The entrypoint receives the OS message and status pointer using the calling convention observed in the working generated applet:

- command/message at `0x04(a7)`
- parameter at `0x08(a7)`
- status output pointer at `0x0c(a7)`

The SDK provides a small dispatch layer that calls applet hooks for focus, USB plug, identity, and default messages. Known OS entrypoints used by the existing Alpha USB applet are exposed as narrowly named SDK functions.

## Alpha USB Behavior

The native Alpha USB applet should:

- use applet id `0xA130`
- use name `Alpha USB`
- use version `1.20`
- use the known working Alpha USB header flags and memory values
- draw the same device-side instruction text on focus
- handle the USB plug path by completing HID-to-direct mode and marking direct USB connected
- return the known USB handled status

## Validation

After each small edit, run the relevant external validation:

- `cargo check --workspace` for host-side workspace health
- `cargo +nightly check -p alpha-usb-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort` for applet health
- `cargo test --workspace` for packer and header tests
- `./build.sh alpha_usb` for end-to-end package output

The produced `.os3kapp` must pass structural validation and preserve the Alpha USB manifest/header expectations. Functional install testing on the physical NEO remains a later manual validation step.

## Constraints

The Rust `m68k-unknown-none-elf` target is Tier 3. It requires nightly `build-std` support and an m68k GNU linker because `rust-lld` does not support m68k. The implementation uses Homebrew `m68k-elf-binutils` on macOS and links with `m68k-elf-ld`.

Release builds are required for the current validated toolchain. `rustc 1.96.0-nightly (2026-03-25)` segfaulted while compiling `compiler_builtins` for this target in debug mode, while the release build completed successfully.

The implementation must not flash or install anything to the device. It only builds and validates the applet file.
