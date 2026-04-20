# aplha-rust-native

Native Rust SDK for AlphaSmart NEO SmartApplets.

This workspace builds applet code with Cargo for `m68k-unknown-none-elf`, then
packages the linked ELF into an AlphaSmart `.os3kapp` image.

Applet behavior is authored as ordinary Rust callbacks. The SDK owns the NEO
entry ABI, message dispatch, trap wrappers, USB helpers, and package validation,
so individual applets should not need applet-local assembly or raw OS addresses.

## Requirements

- Rust nightly with `rust-src`
- `m68k-elf-binutils` or another m68k GNU linker

On macOS with Homebrew:

```sh
brew install m68k-elf-binutils
```

## Build Alpha USB

```sh
cd aplha-rust-native
./build.sh alpha_usb
```

Output:

```text
../exports/applets/alpha-usb-native.os3kapp
```

The wrapper runs:

1. `cargo +nightly build -p alpha-usb-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort --release`
2. `cargo +nightly run -p alpha-neo-pack -- alpha-usb <linked-elf> <output.os3kapp>`

Release mode is intentional. The tested 2026-03-25 nightly segfaulted while compiling `compiler_builtins` for this target in debug mode, but the release build succeeds.

## Structure

- `crates/alpha-neo-sdk`: `no_std` applet-side SDK with message dispatch, display traps, USB helpers, and the shared m68k entry shell.
- `crates/alpha-neo-pack`: host-side ELF-to-OS3KApp packer.
- `applets/alpha_usb`: native Cargo-built Alpha USB applet authored as Rust callbacks.

## Applet Authoring

Applet crates are normal `no_std` Rust binaries for the m68k target. The applet
source should be simple: define a type, implement `Applet`, then export it with
`export_applet!`.

Minimal complete applet:

```rust
#![no_std]
#![no_main]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

use core::panic::PanicInfo;
use alpha_neo_sdk::prelude::*;

struct MyApplet;

impl Applet for MyApplet {
    const ID: u16 = 0xA131;

    fn on_focus(ctx: &mut Context) -> Status {
        ctx.screen().clear();
        screen_line!(ctx, 2, b"Hello from Rust");
        ctx.system().idle_forever()
    }

    fn on_usb_plug(ctx: &mut Context) -> Status {
        if ctx.usb().is_keyboard_connection() {
            ctx.usb().switch_to_direct();
            Status::USB_HANDLED
        } else {
            Status::UNHANDLED
        }
    }
}

export_applet!(MyApplet);

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
```

Normal Rust control flow is allowed inside callbacks. Use `if`, `match`,
locals, helper functions, and small applet-local structs as needed. Device-facing
operations should go through SDK contexts so the build can keep the final image
relocatable and reject risky output.

### Supported Callbacks

Implement only the callbacks your applet needs:

- `const ID: u16`: the SmartApplet id returned to the NEO.
- `on_focus`: called for message `0x19` when the applet is opened from the
  applets menu. This is the safe place to draw instructions and enter the applet
  UI loop.
- `on_usb_plug`: called for message `0x30001` on USB attach. Keep this
  non-blocking. Do not draw, flush, wait for keys, or idle here.
- `on_usb_mac_init`: called for message `0x10001`.
- `on_usb_pc_init`: called for message `0x20001`.
- `on_identity`: called for message `0x26`. The default returns `Self::ID`.

Unknown messages return `Status::UNHANDLED` by default.

### SDK Context API

The high-level SDK API currently exposed to applets:

```rust
ctx.param();                         // raw message parameter
ctx.screen().clear();                // clear the applet display
screen_line!(ctx, 2, b"Text");       // write one immediate byte-literal line
ctx.system().idle_forever();         // stay in the applet screen
ctx.usb().is_keyboard_connection();  // current Alpha USB applet gate
ctx.usb().switch_to_direct();        // run the validated direct-USB OS path
```

Known statuses are named:

```rust
Status::OK
Status::UNHANDLED
Status::USB_HANDLED
Status::raw(0x1234) // escape hatch for newly discovered statuses
```

Prefer the named statuses in applet code. Use `Status::raw` only when a new
status has been identified and is not yet represented by the SDK.

### Text And Data Rules

Use `screen_line!(ctx, row, b"...")` for display text. The byte-string literal
is intentional: it lets the compiler inline the bytes into PC-relative code
without creating GOT-backed data pointers.

Until the SDK adds a validated static-data story, avoid normal `&str`, global
tables, heap allocation, and `std`. They may compile but can produce output that
is unsafe on the NEO because applet binaries are relocated by the OS3K applet
loader, not by a normal ELF loader.

### USB Callback Safety

The physical-device failure that produced `File 127/128 MaxSize overflow`
errors came after an experimental Alpha USB version tried to own the screen and
idle from inside the USB attach callback. Treat USB callbacks as dispatcher
callbacks:

- Do the minimum required device action.
- Return a status immediately.
- Draw user-facing text only in `on_focus`.

The stable Alpha USB behavior is:

```rust
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
        Status::USB_HANDLED
    } else {
        Status::UNHANDLED
    }
}
```

## Adding Another Applet

The current native workspace has one packaged applet, `applets/alpha_usb`.
Adding a second native applet currently requires:

1. Create `applets/<name>/Cargo.toml`.
2. Create `applets/<name>/src/main.rs` using the SDK pattern above.
3. Add the crate to the workspace `members` in `Cargo.toml`.
4. Add a manifest entry to `crates/alpha-neo-pack` for its output name, applet
   name, id, version, and memory/header metadata.
5. Extend `build.sh` with the new package/output mapping.

Keep new applets conservative until their binary output has been inspected and
the physical device has a recovery path.

## Validation Before Flashing

Run these from `aplha-rust-native` after changing SDK or applet code:

```sh
cargo +nightly fmt --all -- --check
cargo +nightly test
cargo +nightly clippy --all-targets -- -D warnings -W clippy::pedantic -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic
cargo +nightly clippy -p alpha-usb-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort --release -- -D warnings -W clippy::pedantic -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic
./build.sh alpha_usb
```

Then inspect the linked ELF:

```sh
m68k-elf-readelf -r target/m68k-unknown-none-elf/release/alpha-usb-applet
m68k-elf-readelf -S target/m68k-unknown-none-elf/release/alpha-usb-applet
```

Expected properties:

- no relocation entries
- no `.got` or `.got.plt`
- applet-local control flow is PC-relative

Validate the packaged `.os3kapp` structure:

```sh
uv run --project ../poc/neotools python - <<'PY'
from pathlib import Path
from neotools.os3kapp import parse_os3kapp_image

path = Path("../exports/applets/alpha-usb-native.os3kapp")
image = parse_os3kapp_image(path.read_bytes())
print(f"id=0x{image.header.applet_id:04x} name={image.header.name} bytes={path.stat().st_size}")
PY
```

For Alpha USB specifically, disassembly should contain the validated direct USB
OS sequence:

```text
jsr 0x0041f9a0
jsr 0x00424780
move.b #1,0x00013cf9
jsr 0x0044044e
jsr 0x00424780
jsr 0x0044047c
jsr 0x00410b26
```

`exports/applets/alpha-usb-native.os3kapp` is structurally valid and implements
the same direct-USB behavior as the physically validated Alpha USB applet, but
it is not byte-exact with the older Python-generated package.

## Static Safety Checks

The generated native applet is checked for:

- no low absolute `jsr 0x0000....` calls for intra-applet control flow
- no nonempty `.got` or `.got.plt` sections
- no ELF relocation sections
- validated trap opcodes `A000`, `A004`, `A010`, `A098`, and `A25C`
- validated direct USB OS call addresses
- valid OS3KApp header, info table, and add-applet fields via `poc/neotools`
