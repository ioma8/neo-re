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

## Build Applets

```sh
cd aplha-rust-native
./build.sh alpha_usb
./build.sh forth_mini
./build.sh basic_writer
```

Outputs:

```text
../exports/applets/alpha-usb-native.os3kapp
../exports/applets/forth-mini.os3kapp
../exports/applets/basic-writer.os3kapp
```

The wrapper runs:

1. `cargo +nightly build -p <applet-package> --target m68k-unknown-none-elf -Z build-std=core,panic_abort --release`
2. `cargo +nightly run -p alpha-neo-pack -- <applet-name> <linked-elf> <output.os3kapp>`

Release mode is intentional. The tested 2026-03-25 nightly segfaulted while compiling `compiler_builtins` for this target in debug mode, but the release build succeeds.

## Structure

- `crates/alpha-neo-sdk`: `no_std` applet-side SDK with message dispatch, display traps, USB helpers, and the shared m68k entry shell.
- `crates/alpha-neo-pack`: host-side ELF-to-OS3KApp packer.
- `applets/alpha_usb`: native Cargo-built Alpha USB applet authored as Rust callbacks.
- `applets/forth_mini`: experimental minimal Forth REPL applet.
- `applets/basic_writer`: minimal AlphaWord-style text editor with 8 file keys.

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
  applets menu. This is the safe place to initialize applet state and draw the
  first screen. For ordinary text-entry applets, return `Status::OK` and handle
  input through `on_char`/`on_key`.
- `on_char`: called for message `0x20`; `ctx.param() & 0xff` is the typed byte.
- `on_key`: called for message `0x21`; use this for non-character keys or decode
  logical key values with `alpha_neo_sdk::keyboard::logical_key_to_byte`.
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
ctx.keyboard().is_ready();           // non-blocking keyboard poll
ctx.keyboard().read_key();           // read one pending firmware logical key code
ctx.keyboard().read_byte();          // read and decode one pending printable/control byte
ctx.keyboard().drain();              // clear stale queued keys before a modal session
ctx.screen().set_cursor(1, 1, 28);   // place the firmware textbox cursor
```

`A094` / `read_key()` returns the NEO firmware logical key code, not ASCII.
For example, physical `1` returns logical `0x38`; treating that as ASCII draws
`8`. Interactive applets that want text input should use `read_byte()` or
`alpha_neo_sdk::keyboard::logical_key_to_byte()` before feeding a parser.

Known statuses are named:

```rust
Status::OK
Status::UNHANDLED
Status::APPLET_EXIT
Status::USB_HANDLED
Status::raw(0x1234) // escape hatch for newly discovered statuses
```

Prefer the named statuses in applet code. Use `Status::raw` only when a new
status has been identified and is not yet represented by the SDK.

### Validated Text Editing Path

For AlphaWord-like text entry, the validated path is firmware-owned editing via
`A084` / `alpha_neo_sdk::keyboard::text_box`, not a custom polling loop copied
from `forth_mini`.

The reusable SDK pieces for that flow are:

```rust
let exit_keys = alpha_neo_sdk::keyboard::textbox_exit_keys();
ctx.keyboard().drain();
ctx.screen().set_cursor(row, col, width);
let exit = alpha_neo_sdk::keyboard::normalize_textbox_exit(
    alpha_neo_sdk::keyboard::text_box(
        &mut buffer,
        &mut len,
        max_len,
        &exit_keys,
        false,
    ),
);
if let Some(slot) = alpha_neo_sdk::keyboard::file_slot_for_exit_key(exit) {
    // File 1..8 switch
}
```

Validated behavior behind those helpers:

- `text_box` owns printable typing, deletion, and basic in-line editing.
- the exit key list must include arrows, file keys, and `Esc` to match the
  observed AlphaWord-style flow.
- the raw textbox return value can carry modifier bits such as Caps Lock; strip
  them with `normalize_textbox_exit` before comparing key codes.
- stale chooser/menu keys should be drained before entering a textbox session.
- use stack-local arrays or SDK helpers that return arrays by value for static
  key tables; borrowed global slices can reintroduce forbidden `.got` sections
  into packaged applets.

### Text And Data Rules

Use `screen_line!(ctx, row, b"...")` for display text. The byte-string literal
is intentional: it lets the compiler inline the bytes into PC-relative code
without creating GOT-backed data pointers.

Until the SDK adds a validated static-data story, avoid normal `&str`, global
tables, heap allocation, and `std`. They may compile but can produce output that
is unsafe on the NEO because applet binaries are relocated by the OS3K applet
loader, not by a normal ELF loader.

The m68k target config also disables jump-table lowering. Keep that setting:
PIC jump tables create `.got`/`.got.plt` sections, and the packer rejects those
sections because the NEO applet loader does not behave like a normal ELF dynamic
loader.

Betawise builds C applets with `-ffixed-a5 -ffixed-d7`. That reserves those
registers from ordinary C compiler allocation; it is not a rule that a final
SmartApplet binary can never mention them. The stock Calculator applet uses
both registers extensively, with `a5` acting like a runtime/local-state base in
many paths and `d7` used by applet/runtime code.

Rust does not currently expose a proven equivalent to GCC's fixed-register
flags for this m68k target. Use `scripts/audit-m68k-registers.sh` when changing
entry or callback code to see where `a5`/`d7` appear, but do not treat every
appearance as a packaging failure.

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

## Included Applets

### Alpha USB

`Alpha USB` is the validated Android/desktop bridge applet. Open it on the NEO,
connect USB, and it switches the device from HID keyboard mode into direct USB
mode without using typewriter fallback.

Build:

```sh
./build.sh alpha_usb
```

### Forth Mini

`Forth Mini` is a small interactive Forth-style REPL. It currently supports:

```text
+ - * / mod dup drop swap over . .s clear
```

It accepts signed decimal integers and keeps a 16-cell stack. This is an
experimental applet intended to exercise OS-dispatched text input, display updates, and
small stateful Rust applet code.

Its lifecycle is event-driven: `on_focus` initializes and draws the REPL,
`on_char` processes printable input, and `on_key` handles only control keys.
Printable `Key` messages must not feed the REPL because the full OS also sends
`Char` messages for text; accepting both causes duplicate or nonsensical prompt
characters. A Calculator-style modal key loop was tested and rejected for this
applet because it can fight the full OS debounce path and make later typed input
stop reaching the REPL.

The REPL stores state at `A5 + 0x300` inside its declared base-memory block.
Keeping the low A5 area free avoids clobbering firmware/app-runtime scratch
state; early versions that wrote the REPL at `A5 + 0` produced display/input
corruption. The focus screen is drawn after a full screen clear because leftover
menu highlight state can otherwise remain visible.

Current status: the Rust source compiles, packages, passes host tests, and
passes the full-System headless emulator validation for `1 Enter`, `2 Enter`,
`+ Enter` producing stack value `3`. It has not yet been validated on the
physical device. Treat it as experimental until the REPL has been installed and
exercised with a recovery path available.

Build:

```sh
./build.sh forth_mini
```

### Basic Writer

`Basic Writer` is the current reference applet for AlphaWord-style text entry.
It uses the firmware textbox control for continuous typing, local Rust editor
state for cursor/view restoration, direct file-key switching across 8 slots,
and the shared SDK entry macro.

The important constraint is architectural: use `Forth Mini` only as a small
event-driven callback example. Do not use its interactive input flow as the
reference for text-editor applets. The validated editor path is the firmware
textbox flow captured in `Basic Writer`.

Build:

```sh
./build.sh basic_writer
```

## Adding Another Applet

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
cargo +nightly clippy -p forth-mini-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort --release -- -D warnings -W clippy::pedantic -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic
./build.sh alpha_usb
./build.sh forth_mini
```

Then inspect the linked ELF:

```sh
m68k-elf-readelf -r target/m68k-unknown-none-elf/release/alpha-usb-applet
m68k-elf-readelf -S target/m68k-unknown-none-elf/release/alpha-usb-applet
```

Expected properties:

- no relocation entries
- no `.got` or `.got.plt`
- `scripts/audit-m68k-registers.sh <linked-elf>` reviewed when changing ABI or
  callback dispatch code
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
