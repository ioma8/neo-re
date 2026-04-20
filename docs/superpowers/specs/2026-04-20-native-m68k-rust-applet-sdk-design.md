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

`applets/alpha_usb` is a `#![no_std]` applet crate. Applet authors must write normal Rust callbacks, not per-applet assembly. A shared SDK runtime may contain the m68k ABI shell and OS trap glue, but the applet-specific behavior is compiled by Cargo/rustc for `m68k-unknown-none-elf`.

The authoring surface follows the Betawise split: the SDK owns the header/entry/ABI/syscall details, while each applet implements message callbacks. Unlike Betawise's C `ProcessMessage`, our callback body is Rust and can use ordinary Rust control flow, local variables, helper functions, and match/if/loop logic.

## Native ABI Shape

The applet exports one m68k entrypoint. The entrypoint receives the OS message and status pointer using the calling convention observed in the working generated applet:

- command/message at `0x04(a7)`
- parameter at `0x08(a7)`
- status output pointer at `0x0c(a7)`

The SDK provides a small dispatch layer that calls applet hooks for focus, USB plug, identity, and default messages. Known OS entrypoints used by the existing Alpha USB applet are exposed as narrowly named SDK functions. Runtime-critical applets must also be checked for position-safe code: local control flow should use PC-relative branches/calls, and string/data access must not depend on unrelocated low absolute addresses.

The first Rust callback API is:

```rust
use alpha_neo_sdk::prelude::*;

struct AlphaUsb;

impl Applet for AlphaUsb {
    fn on_focus(ctx: &mut Context) -> Status {
        ctx.screen().clear();
        screen_line!(ctx, 2, b"Now connect the NEO");
        screen_line!(ctx, 3, b"to your computer or");
        screen_line!(ctx, 4, b"smartphone via USB.");
        ctx.system().idle_forever()
    }

    fn on_usb_plug(ctx: &mut Context) -> Status {
        if ctx.usb().is_keyboard_connection() {
            ctx.usb().switch_to_direct();
            Status::UsbHandled
        } else {
            Status::Unhandled
        }
    }
}

alpha_neo_sdk::export_applet!(AlphaUsb);
```

`export_applet!` emits the shared SDK entry binding and connects the applet type to the Rust dispatch function. It must not require an applet-local `entry.s` file.

## Alpha USB Behavior

The native Alpha USB applet should:

- use applet id `0xA130`
- use name `Alpha USB`
- use version `1.20`
- use the known working Alpha USB header flags and memory values
- draw the same device-side instruction text on focus
- handle the USB plug path by completing HID-to-direct mode and marking direct USB connected
- return the known USB handled status
- avoid low absolute intra-applet `jsr` calls and unrelocated absolute string pointers
- use the validated SmartApplet trap opcodes and trap-block call shape

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
