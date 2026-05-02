# WriteOrDie SmartApplet Findings

This note records the implementation details that made
`smartapplets/write_or_die_bw` reliable enough in the real device and GUI
emulator.

## Working Build Path

- Build through the Betawise-derived C workflow.
- Keep Betawise syscall stubs and headers.
- Use the custom no-global applet runtime.
- Store applet state at `A5 + 0x300`.
- Package with local `alpha-neo-pack`.
- The packed OS3K applet image must have an even total length. The firmware
  validates applet record footers with long-word reads; an odd applet length
  caused `address error vector=3 at 0x00413358` during full-system boot.

Validated command:

```sh
./scripts/build-smartapplet.sh write_or_die_bw --no-validate
cargo run --manifest-path alpha-emu/Cargo.toml -- --headless --validate-write-or-die analysis/cab/os3kneorom.os3kos
```

## State And Persistence

- `WriteOrDie` owns exactly one SmartApplet file.
- Runtime file handle `1` works for applet-owned snapshot persistence.
- Persist the whole `WodAppState_t` snapshot through the Betawise
  `file_store` helper.
- Completed, exported, and setup phases are stored in the same snapshot.
- Starting a new challenge clears the draft in memory before entering the
  running phase.

## Timer Findings

- Challenge countdown must be based on firmware uptime, not keypress count.
- Reading low-level timer registers directly produced real-device jumps.
- The applet should compute elapsed time as `now_ms - start_ms`, using
  `GetUptimeMilliseconds()`.
- Keypresses reset pressure timing, but must not advance countdown time.
- The validated countdown, elapsed-time, penalty-interval, and pressure-stage
  helpers now live in `smartapplets/betawise-sdk/challenge_timer.h`.
- Punishment timing follows the requested WriteOrDie loop:
  - safe idle phase lasts the configured grace period, default `10` seconds and
    configurable from `0` to `30` seconds
  - warning phase lasts `4` seconds, flashes the writing area, and does not
    damage text
  - punishment starts immediately after warning
  - kamikaze deletion removes one trailing character every `700` ms during
    punishment
  - any typed character immediately clears warning/punishment and starts a new
    full grace period

## GUI Emulator Input Findings

- GUI typing must use `egui::Event::Text` for printable characters.
- GUI taps should expire by emulated CPU cycles, not by matrix-read counts.
  Read-count based taps queued up during sustained typing and made the GUI look
  frozen while stale key phases drained.
- Do not advance huge CPU-cycle chunks after each host keypress. That made time
  mode lose seconds per typed character.

## Rendering Findings

- Betawise `_OS3K_SetCursor` has the signature:

```c
void _OS3K_SetCursor(uint8_t row, uint8_t col, CursorMode_e cursor_mode);
```

- The third argument is a cursor mode, not a width. Passing `WOD_SCREEN_COLS`
  broke every WriteOrDie screen.
- The working cursor convention is `col = 1` with `CURSOR_MODE_HIDE`, matching
  the other Betawise applets.
- `WOD_SCREEN_COLS` must be `41` for this applet. Wider values looked correct
  briefly, but trailing cells from the last row wrapped back into row 1 and hid
  the first status character.
- The GUI renderer must not synthesize cursor blinking by masking guessed LCD
  pixels. WriteOrDie hides its cursor, and the heuristic could mistake text
  stems for a cursor.
- Cache rendered challenge rows and skip unchanged row writes. This reduces
  keypress flicker because stable rows are not redrawn for every typed byte.
- The validated fixed-width row helpers now live in
  `smartapplets/betawise-sdk/screen_lines.h`.

## Text Layout Findings

- Word wrapping belongs in `editor.c`, not the UI renderer.
- Spaces before a wrapped word should not be drawn at the start of the next
  visual line.
- The editor viewport must follow the cursor over the three writing rows.
- Status row is not part of the editor viewport.

## AlphaWord Export Findings

- Locating a fixed AlphaWord text buffer and appending bytes directly is not
  safe. AlphaWord files are firmware-owned logical records with length,
  attributes, reserved size, and workspace invariants.
- Cross-applet AlphaWord `MSG_CHAR`/`MSG_KEY` calls do not reliably mutate the
  selected AlphaWord file. The applet can report success while no file buffer
  changes.
- The working route is descriptor-based, not fixed-address based. The reusable
  helper is `smartapplets/betawise-sdk/alphaword_export.h`:
  `alphaword_append_text_block(slot, title, text, text_len)`.
- The helper preselects AlphaWord and the requested visible File key, maps the
  visible slot to AlphaWord's backing slot rotation, scans low-RAM AlphaWord
  file descriptors for the last initialized matching `File N` record, reads
  the descriptor's text-buffer pointer, then appends before the `0xa7` fill
  tail.
- There are two relevant descriptor states. If AlphaWord's live edit buffers
  exist, records point at `0xa7`-filled 512-byte buffers. If switching away from
  AlphaWord has released those buffers, persistent records can have length `0`
  with valid payload pointers. In that state WriteOrDie writes the payload at
  the pointer and updates the record length fields to the actual byte count.
- The descriptor scan must include the primary records and later duplicate
  groups, then update only the last initialized matching descriptor. Updating
  all duplicates made the same append appear in multiple visible files; updating
  only the first primary descriptor was invisible to AlphaWord.
- Visible File keys are rotated in the backing descriptor records. The validated
  helper maps visible File `1` to backing File `8`, and visible File `N > 1` to
  backing File `N - 1`.
- Headless validation checks the selected AlphaWord backing descriptor buffer
  directly and verifies repeated append growth. Recent File 2 validation prints
  `write_or_die_export_visible_file2_backing_slot1_buffers=0x0001889c` and
  `write_or_die_second_export_file2_buffers=0x0001889c`.

## Practical Validation

Useful checks after changes:

```sh
cargo check --manifest-path alpha-emu/Cargo.toml
cargo run --manifest-path alpha-emu/Cargo.toml -- --headless --validate-write-or-die analysis/cab/os3kneorom.os3kos
cargo run --manifest-path alpha-emu/Cargo.toml -- --headless --steps=5000000 analysis/cab/os3kneorom.os3kos
```

The first validates Rust-side emulator changes. The second catches most
WriteOrDie state, screen, timing, punishment, and AlphaWord export regressions.
The third catches full-system boot regressions with the recovery seed and
injected applet block.
