# 2026-04-29 Basic Writer AlphaWord Findings

This note records the findings that actually made the Rust `Basic Writer`
SmartApplet work in the emulator, plus the SDK abstractions extracted from that
work.

## What Worked

1. Use the firmware textbox control, not a handwritten key loop.
   - The stable text-entry path is trap `A084`, exposed as
     `alpha_neo_sdk::keyboard::text_box`.
   - This path handles continuous printable typing and deletion in the way the
     stock text applets expect.

2. Treat `Basic Writer` as a modal focus-session applet.
   - Enter from `on_focus`.
   - Draw the document, set the screen cursor, run a textbox session, inspect
     the exit key, update local editor state, repeat.

3. Set the textbox cursor explicitly before each edit session.
   - The working primitive is trap `A004` with row/column/width, now exposed as
     `ctx.screen().set_cursor(...)`.
   - Without this, the firmware textbox resumes from the wrong screen position.

4. Drain stale keyboard events before entering the textbox.
   - The SmartApplets menu launch leaves queued key state behind.
   - The reusable helper is now `ctx.keyboard().drain()`.

5. Normalize textbox exit codes before comparing them.
   - The textbox return value can include modifier bits, especially Caps Lock.
   - The reusable helper is `alpha_neo_sdk::keyboard::normalize_textbox_exit`.

6. Use the AlphaWord-style exit key set.
   - Left, Right, Up, Down
   - File 1 through File 8
   - Escape
   - terminator `0xff`
   - The SDK now exposes this as `textbox_exit_keys()`.

7. Keep applet state at `A5 + 0x300`.
   - Using low A5-relative memory directly caused applet/runtime corruption in
     earlier experiments.
   - `Basic Writer` still owns this state pointer locally because it is applet
     storage policy, not generic SDK behavior.

8. Keep constant data out of global borrowed slices.
   - A shared `&[u8]` exit-key table reintroduced a forbidden `.got` section in
     the packaged applet.
   - Returning the key table by value from `textbox_exit_keys()` fixed that.

## What Did Not Work

1. Using `Forth Mini`'s interactive flow as the model.
   - `Forth Mini` is useful as a small callback example.
   - It is not the right reference for AlphaWord-style editor input.

2. Reimplementing the entire text-input path around raw key polling.
   - Multiple loops around `read_key`, `wait_for_key`, and custom dispatch were
     brittle under real SmartApplet menu launch conditions.
   - Some of those paths worked for one key and then stopped behaving like the
     stock firmware flow.

3. Assuming logical keys are ASCII.
   - `A094` returns firmware logical key codes, not characters.
   - Example: physical `1` becomes logical `0x38`, not byte `'1'`.

4. Using static borrowed data casually in applet code.
   - Even small-looking changes can introduce `.got` / `.got.plt`.
   - The packer rejection is correct and should be treated as a hard signal.

## Emulator Findings That Mattered

1. GUI input must preserve the old text path.
   - Printable typing in the GUI depends on `egui::Event::Text`.
   - Removing that path broke typing globally.

2. GUI physical keys must use stable press/release semantics.
   - The experimental mixed tap model broke key delivery broadly.
   - Restoring plain press/release fixed GUI responsiveness again.

3. Headless validation has to launch through the real SmartApplets chooser.
   - Direct callback injection gave false confidence for editor behavior.
   - Real menu launch plus OCR/state checks was the only reliable validation.

## SDK Abstractions Added

These validated pieces are now in `alpha-neo-sdk`:

- `ctx.screen().set_cursor(row, col, width)`
- `ctx.keyboard().drain()`
- `keyboard::textbox_exit_keys() -> [u8; 14]`
- `keyboard::normalize_textbox_exit(raw) -> u8`
- `keyboard::file_slot_for_exit_key(exit) -> Option<usize>`
- `export_applet!` reused by `Basic Writer` instead of applet-local entry glue

This keeps `Basic Writer` focused on editor policy:

- document bytes
- cursor/view restoration
- arrow movement semantics
- file-slot switching
- storage integration points

The SDK owns the proven firmware boundary.

## Remaining Constraint

True SmartApplet-owned retrievable file persistence is still not fully
implemented. The storage functions remain deliberately gated behind
`FirmwarePathUnproven` until the file load/save/commit trap sequence is proven
well enough to implement safely.

## Minimal Working Pattern

```rust
let exit_keys = alpha_neo_sdk::keyboard::textbox_exit_keys();
ctx.keyboard().drain();
ctx.screen().set_cursor(row, col, width);
let exit = alpha_neo_sdk::keyboard::normalize_textbox_exit(
    alpha_neo_sdk::keyboard::text_box(&mut buffer, &mut len, max_len, &exit_keys, false),
);
```

That is the current validated foundation for future AlphaWord-like Rust
SmartApplets.
