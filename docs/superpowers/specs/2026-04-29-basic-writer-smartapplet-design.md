# Basic Writer SmartApplet Design

## Goal

Build a basic Rust SmartApplet for writing text on the AlphaSmart NEO. It should
replicate the core AlphaWord writing flow, but only the minimal editor behavior:
typing, cursor movement with arrows, scrolling over the 4-line screen, autosave,
and direct switching between eight files with physical file keys.

The applet must expose its documents as SmartApplet-owned retrievable files, not
as RAM-only state.

## Non-Goals

- No menus, help screens, file naming, printing, sending, spellcheck, find, word
  count, clipboard, file clearing, section selection, or command shortcuts.
- No AlphaWord applet id reuse or compatibility claim.
- No UI beyond the editor text viewport and cursor.
- No persistence simulation. If owned-file load/save cannot be proven, the work
  stops at that boundary instead of pretending autosave works.

## Evidence And Constraints

AlphaWord Plus is the behavioral reference. A bounded radare2 pass over
`analysis/cab/alphawordplus.os3kapp` confirmed:

- metadata id `0xA000`, name `AlphaWord Plus`, and file-related strings
- applet message dispatch begins at the packaged entry area around `0x94`
- strings document physical file switching with `Cmd-File` variants and saved
  file messaging
- existing project notes map AlphaWord file retrieval/write protocol around
  applet file slots `1..8`

The existing Rust SDK examples are implementation references only. `Forth Mini`
shows how to build a stateful m68k Rust applet, keep applet-owned memory, and
draw four text rows, but its REPL loop is not the desired editor behavior.

The SDK currently supports applet callbacks for focus, chars, keys, display
traps, keyboard decode, and packaging. The packer already emits AlphaWord-style
write metadata records for keys `0x8011..0x8018`, but it currently declares
`file_count = 0`. This applet needs explicit eight-file ownership.

## Proposed Architecture

Add a new applet crate:

```text
aplha-rust-native/applets/basic_writer/
```

Primary units:

- `editor.rs`: host-testable pure editor model. Owns the active text buffer,
  cursor position, insert/delete behavior, arrow movement, wrapping, and
  viewport scrolling.
- `storage.rs`: applet file-slot adapter. Owns serialization and load/save
  calls for slots `1..8`. This module is where reverse engineering must prove
  the firmware path for SmartApplet-owned file persistence.
- `main.rs`: applet callback glue. Handles focus, char/key dispatch, active slot
  switching, autosave calls, and redraw.
- packer manifest support: add a way to declare eight owned files and emit the
  existing write metadata records for those slots.
- emulator launch support, if needed: extend the headless emulator’s applet
  discovery/validation helpers so `Basic Writer` can be launched like
  `Forth Mini`.

## File Model

The applet declares eight owned files. Physical keys map directly:

```text
file1 -> slot 1
file2 -> slot 2
...
file8 -> slot 8
```

Switching files autosaves the current slot, changes the active slot, loads the
new slot if needed, restores that slot's RAM cursor/viewport state when
available, otherwise places the cursor at end of file, and redraws.

Documents are plain byte text. Version 1 targets printable ASCII, newline, and
backspace. Unsupported bytes are dropped during load so file decoding is
deterministic.

Each file is limited to 4096 bytes for the first implementation. This keeps the
active editor buffer small enough for the current SDK memory model and gives
tests a concrete boundary. If reverse engineering proves the firmware-owned
file path has a lower safe limit, implementation must lower the constant before
editor work continues.

## State Persistence

The retrievable file contents are persistent. Editor navigation state is not
embedded in the files.

- Active slot is RAM-only and resets to slot 1 on a fresh applet focus after
  restart.
- Cursor and viewport positions are tracked per slot in applet RAM while the
  applet is running.
- Switching away from a slot preserves that slot's cursor and viewport in RAM.
- Loading a slot with no RAM navigation state places the cursor at end of file
  and scrolls so the cursor is visible.
- Restarting the applet loses cursor and viewport positions, but not file text.

## Editor Behavior

On `SETFOCUS`:

- initialize editor state if needed
- set active slot to slot 1 unless persisted applet state already has another
  active slot in RAM
- load active file contents
- draw the visible 4-line viewport
- return `Status::OK`

On `CHAR`:

- printable ASCII inserts at the cursor
- enter inserts a newline
- backspace deletes before the cursor
- each mutation autosaves the active slot and redraws

On `KEY`:

- left/right/up/down move the cursor through the wrapped document
- `file1..file8` switch slots
- decoded printable bytes may be accepted if firmware delivers them through
  key messages
- all other keys are ignored

The editor scrolls vertically when cursor movement or insertion crosses the
visible 4-line viewport. Horizontal behavior is kept simple: text wraps to the
available display width and arrows move through the logical text positions.

## Display

Use the existing SDK text display traps. The first implementation should use
the current validated text width of 28 columns and 4 rows. Every redraw writes
all four rows so stale characters do not remain on screen.

Cursor display is required. It will follow the simplest reliable SDK/emulator
path available: either the firmware cursor if the text traps expose it safely,
or an explicit visible marker strategy if needed. The exact method is
implementation detail, but the emulator validation must prove the cursor is
visible and that arrow movement changes insertion position correctly.

## Autosave And Persistence

Autosave runs:

- after every text mutation
- before switching away from a slot

Persistence must use SmartApplet-owned retrievable files. The implementation
will first prove the load/save firmware path with a small reverse-engineering
loop and tests. If the SDK lacks the needed traps, add narrow SDK wrappers only
for the proven calls.

The updater-visible retrieval path must be validated with existing host tooling
or an added equivalent test path. The applet should not claim autosave success
until saved text can be retrieved from the corresponding owned file slot.

## Error Handling

The applet has no user-facing error UI beyond preserving editor usability.

- If load fails, use an empty buffer for that slot and keep the applet running.
- If save fails, keep the in-memory buffer and continue; emulator/headless tests
  must catch this as a persistence failure.
- If a file reaches 4096 bytes, reject additional inserted bytes.
- Unknown keys and unsupported commands return `Status::UNHANDLED` or no-op
  according to the SDK pattern that keeps focus stable.

## Testing And Validation

Use test-driven development.

Host unit tests:

- insertion at cursor
- newline handling
- backspace
- left/right movement across line boundaries
- up/down movement across wrapped visual rows
- viewport scroll when cursor leaves the 4-line area
- active slot switching preserves independent buffers
- serialization bounds and unsupported byte handling
- cursor/viewport state survives file switches in RAM and resets after a fresh
  applet restart

Packer tests:

- `Basic Writer` declares `file_count = 8`
- package includes write metadata records for `0x8011..0x8018`
- package has valid OS3KApp header/footer shape

SDK/storage tests:

- any newly added trap wrapper has a focused host-side API test where possible
- file-slot save/load encoding is tested without emulator dependencies

Validation commands:

- run the relevant Rust unit tests after each small code change
- run `cargo check` for the affected workspace/package after each change
- build with `./build.sh basic_writer`
- run headless emulator with scripted typing, arrows, file switching, and LCD
  output
- retrieve owned files through existing updater tooling or a narrow added
  validator and confirm slot contents match typed text

## Implementation Boundaries

The first usable version is complete when:

- the applet builds into `exports/applets/basic-writer.os3kapp`
- emulator launch opens the editor
- typed text appears in the 4-line viewport
- arrows affect insertion position
- cursor is visible
- text scrolls over four visible rows
- file keys `file1..file8` switch independent slots
- autosaved slot contents are retrievable as SmartApplet-owned files
