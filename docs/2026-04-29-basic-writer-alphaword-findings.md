# 2026-04-29 Basic Writer AlphaWord Findings

This note originally recorded the Rust `Basic Writer` prototype. The current
maintained and validated Basic Writer applet is `smartapplets/basic_writer_bw`,
built through the Betawise-derived C workflow documented in
`smartapplets/README.md`.

The Rust textbox findings below are historical: they explain why the first
prototype behaved like AlphaWord in the emulator, but they are not the current
reliable applet path.

## Current Status

- Build with `./scripts/build-smartapplet.sh basic_writer_bw --no-validate`.
- Use the Betawise syscall stubs and headers, but replace the Betawise wrapper
  with the repo-owned no-global runtime.
- Keep applet state at `A5 + 0x300`.
- Package with local `alpha-neo-pack`.
- Use `smartapplets/betawise-sdk/file_store.h` for applet-owned retrievable
  file snapshots.
- Use `smartapplets/betawise-sdk/screen_lines.h` for stable four-row rendering.
- Basic Writer owns eight retrievable SmartApplet files and switches them from
  physical File keys.
- The current full Basic Writer validator has a known menu-selection caveat
  when multiple repo applets are installed; build validation is reliable, and
  shared SDK behavior is covered through the Forth Mini and WriteOrDie
  validators until that path is tightened.

## Historical Rust Prototype Findings

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

## Historical Rust SDK Abstractions

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

The Rust SDK owns that historical firmware boundary. Current applet-owned file
persistence is proven in the Betawise-side SDK, not through these Rust storage
gates.

## Minimal Working Pattern

```rust
let exit_keys = alpha_neo_sdk::keyboard::textbox_exit_keys();
ctx.keyboard().drain();
ctx.screen().set_cursor(row, col, width);
let exit = alpha_neo_sdk::keyboard::normalize_textbox_exit(
    alpha_neo_sdk::keyboard::text_box(&mut buffer, &mut len, max_len, &exit_keys, false),
);
```

That remains useful context for future AlphaWord-like Rust SmartApplets, but
new reliable applets should start from `smartapplets/basic_writer_bw` or the
Betawise-side SDK templates.
