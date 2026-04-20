# AlphaSmart NEO Applet Emulator Design

## Goal

Create a new root Rust project, `alpha-emu`, that can load a packaged
AlphaSmart NEO `.os3kapp` file and execute enough of the applet binary to run
the validated `Alpha USB` SmartApplet.

The first version is intentionally narrow. It should prove the real packaged
applet executes through an emulated NEO process-message path, display, and USB
attach event. It does not need to run stock Calculator yet.

## First Supported Applet

The initial target is:

```text
exports/applets/alpha-usb-native.os3kapp
```

Expected behavior:

1. Load the `.os3kapp` package and extract metadata, entry offset, and payload.
2. Send process message `0x19` to simulate opening the applet from the menu.
3. Render the applet instructions on an emulated LCD:

   ```text
   Now connect the NEO
   to your computer or
   smartphone via USB.
   ```

4. Provide a UI button, `Simulate USB attach`, that sends process message
   `0x30001`.
5. Emulate the validated Alpha USB direct-mode OS path and show USB mode
   changing from HID keyboard to direct USB.

## Project Shape

`alpha-emu/` will be a standalone Rust 2024 project.

Main modules:

- `os3kapp`: package parsing for header, metadata, entry offset, and payload.
- `cpu68k`: focused m68k CPU/interpreter state: `d0-d7`, `a0-a7`, `pc`, `sr`,
  stack, memory, and a bounded step loop.
- `neo_os`: emulated AlphaSmart OS services used by `Alpha USB`.
- `applet_host`: high-level message dispatch helpers for focus and USB attach.
- `gui`: desktop UI using `eframe/egui`, consistent with the existing
  `alpha-cli` GUI dependency.

## Instruction Handling

Use an existing m68k instruction/disassembly crate where useful, with Capstone
as the preferred first candidate if its Rust bindings work cleanly for m68k.
The emulator may still implement execution semantics itself for the small
instruction subset emitted by the current `Alpha USB` binary.

The first implementation should be trace-driven:

1. Disassemble the loaded applet payload.
2. Implement only the instructions reached by `Alpha USB`.
3. Fail clearly on unsupported opcodes, showing `pc`, opcode bytes, and recent
   trace context.

This keeps the implementation quick while preserving a real path toward
Calculator and other stock applets later.

## Emulated NEO Behavior

The first OS shim set only needs the behavior used by `Alpha USB`:

- display clear
- display row text write
- display flush/yield/idle handling
- status return storage
- process-message dispatch
- USB keyboard attach state
- direct USB switch marker

`idle_forever` must not hang the desktop app. The emulator should treat it as a
controlled yield/stop condition for the current process message.

## UI

The UI should be simple and desktop-first:

- NEO-like LCD display area
- loaded applet name/id
- current process-message status
- current emulated USB mode
- buttons:
  - `Open applet`
  - `Simulate USB attach`
  - `Reset emulator`

The UI should stay responsive even if the interpreted applet loops. The CPU
runner should execute bounded slices and report unsupported instructions or OS
calls in a visible error panel.

## Validation

Run validation after each implementation slice:

```sh
cargo check
```

Final first-slice validation:

```sh
cd alpha-emu
cargo check
cargo test
cargo run -- ../exports/applets/alpha-usb-native.os3kapp
```

Automated tests should cover:

- loading `alpha-usb-native.os3kapp`
- focus message renders the expected three display lines
- USB attach changes emulated mode to direct USB and returns the expected
  handled status

## Out Of Scope For First Slice

- full stock Calculator execution
- complete 68k instruction coverage
- real USB device access
- AlphaWord file storage
- full NEO filesystem emulation
- cycle-accurate CPU behavior

