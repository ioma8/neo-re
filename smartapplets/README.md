# SmartApplets

Repo-owned workflow for reliable AlphaSmart NEO SmartApplets using the validated
Betawise-derived C path.

## One-command build

From the repo root:

```sh
./scripts/build-smartapplet.sh basic_writer_bw
```

That command does all of the working steps:

1. builds the applet ELF with `m68k-elf-gcc` / `m68k-elf-ld`
2. packs the ELF with local `alpha-neo-pack`
3. writes the final `.os3kapp` into `exports/applets/`
4. runs the headless emulator validator

To launch the GUI emulator after a successful build:

```sh
./scripts/build-smartapplet.sh basic_writer_bw --gui
```

To skip validation:

```sh
./scripts/build-smartapplet.sh basic_writer_bw --no-validate
```

The same workflow applies to every applet with an `applet.env`, for example:

```sh
./scripts/build-smartapplet.sh write_or_die_bw
./scripts/build-smartapplet.sh floppy_bird_bw
./scripts/build-smartapplet.sh snake_bw
```

## Tool requirements

On macOS with Homebrew:

```sh
brew install m68k-elf-binutils m68k-elf-gcc
```

## Writing applets this way

Use these rules. They come directly from the working path:

- keep Betawise syscall stubs and headers
- do not use Betawise `os3k.c`
- use a custom no-global applet runtime
- store all mutable applet state at `A5 + 0x300`
- pack the final ELF with local `alpha-neo-pack`
- for applet-owned persistence, use the proven runtime file handle and trap path,
  not guessed metadata ids

In practice that means:

- include `smartapplets/betawise-sdk/bw.h` for normal applets
- link `smartapplets/betawise-sdk/syscall.c`
- link `smartapplets/betawise-sdk/bw_store.c` if the applet persists state
- link `smartapplets/betawise-sdk/bw_screen.c` if the applet writes full LCD
  rows
- link `smartapplets/betawise-sdk/bw_challenge.c` if the applet has
  countdown or pressure timing
- link `smartapplets/betawise-sdk/bw_alphaword.c` if the applet appends
  text into visible AlphaWord File slots
- include `smartapplets/betawise-sdk/os3k.h` directly only for low-level
  firmware-specific work
- dispatch `MSG_SETFOCUS`, `MSG_CHAR`, and `MSG_KEY` yourself
- handle Enter as `MSG_CHAR` byte `0x0d`/`0x0a` where the screen flow expects
  a confirmation key; under full firmware dispatch it is not guaranteed to
  arrive as `MSG_KEY KEY_ENTER`
- do not rely on writable global C variables
- keep the linker script simple and discard `.bss`, `.data`, `.got`, `.rela`, `.rel`

## Shared SDK pieces

The repo-owned Betawise-side SDK now exposes these validated helpers through
`bw.h`:

- app entry and status helpers
  - `BW_APP_ENTRY(handler)`
  - `BW_APP_STATE(Type)`
  - `BW_STATUS_EXIT`
  - `BW_STATUS_UNHANDLED`
  - `bw_message_id(...)`
  - `bw_key_id(...)`
- timers and drawing
  - `BwTimer`
  - `bw_now_ms(...)`
  - `bw_timer_reset(...)`
  - `bw_timer_due_and_advance(...)`
  - `BwRect`
  - `bw_fill_rect(...)`
  - `bw_fill_rect_clipped(...)`
- `bw_store.h` / `bw_store.c`
  - `bw_store_load_snapshot(...)`
  - `bw_store_save_snapshot(...)`
- `bw_screen.h` / `bw_screen.c`
  - `BW_SCREEN_SAFE_COLS`
  - `bw_screen_put_line(...)`
  - `bw_screen_put_cached_line(...)`
  - `bw_screen_set_reverse(...)`
  - `bw_screen_set_reverse_cached(...)`
  - `bw_screen_clear_reverse_cache(...)`
  - row formatting, cache invalidation, and fixed-width line copying helpers
- `bw_challenge.h` / `bw_challenge.c`
  - `bw_seconds_to_milliseconds(...)`
  - `bw_milliseconds_to_seconds(...)`
  - `bw_elapsed_milliseconds(...)`
  - `bw_remaining_seconds(...)`
  - `bw_penalty_interval_milliseconds(...)`
  - `bw_pressure_stage(...)`
  - `bw_flash_phase(...)`
- `bw_alphaword.h` / `bw_alphaword.c`
  - `bw_alphaword_append_text_block(...)`

This means applet code no longer needs to carry:

- the raw `alpha_usb_entry` assembly shim
- direct `A5 + 0x300` pointer boilerplate
- direct `A2DC -> A2EC/A2FC -> A190 -> FileReadBuffer/FileWriteBuffer -> FileClose`
  choreography for one-file snapshot persistence
- local copies of the validated `row 1..4`, `col = 1`, `CURSOR_MODE_HIDE`
  full-row rendering pattern
- local copies of the uptime-based challenge countdown and pressure-stage
  arithmetic
- direct LCD command writes for whole-display reverse/highlight feedback
- local interval phase loops for periodic warning/punishment flashing
- direct AlphaWord visible-file descriptor scanning and append patching

For the currently validated one-file persistence path used by `forth_mini_bw`:

- runtime file handle is `1`
- workspace/file sequence is `A2DC -> A2EC/A2FC -> A190 -> FileReadBuffer/FileWriteBuffer -> FileClose`
- persisting a binary app state or machine snapshot is safer than replaying source on target when the code uses struct returns under `-fshort-enums`

## Compiler profiles

There is a real codegen boundary here.

For small, straightforward applets such as `basic_writer_bw`, the default compact
profile is fine:

```make
CFLAGS += -Os
```

For more complex applets with denser control flow or REPL-style logic such as
`forth_mini_bw`, use the conservative profile instead:

```make
CFLAGS += -O1 -fno-inline -fno-optimize-sibling-calls
```

Why this matters:

- `forth_mini_bw` built successfully with more aggressive optimization
- but the resulting applet crashed at runtime under real firmware dispatch
- backing optimization down and disabling inlining/tail-call shaping made the
  applet stable in the emulator validation path

So the rule is:

- start with the compact profile for simple applets
- switch to the conservative profile as soon as a complex applet shows unstable
  runtime behavior that is not explained by the applet logic itself

## Layout

Each applet lives in `smartapplets/<name>/` and should contain:

- applet source, usually `<Name>.c`
- linker script
- `Makefile`
- `applet.env`

`applet.env` tells the wrapper script how to:

- locate the built ELF
- choose the `alpha-neo-pack` manifest
- choose the output `.os3kapp`
- run an applet-specific validator

## Minimal state pattern

The validated pattern is:

```c
typedef struct {
    /* applet state */
} AppState_t;

BW_APP_STATE(AppState_t);
```

Everything mutable should hang off that state block.

## Current references

- `smartapplets/basic_writer_bw`: simple editor applet, good default template
- `smartapplets/forth_mini_bw`: more complex REPL applet, reference for the
  conservative compiler profile
- `smartapplets/write_or_die_bw`: challenge editor applet, reference for setup
  menus, live timers, pressure states, one-file autosave, and full-system
  headless validation of completion plus persistence
- `smartapplets/floppy_bird_bw`: simple LCD game applet, reference for
  uptime-driven frame ticks, transient in-memory game state, and key-controlled
  animation
- `smartapplets/snake_bw`: full-pixel Snake game applet, reference for using
  the whole 264x64 LCD area through 4x4 square-cell rectangle raster operations
  with edge wraparound

## Validated screen feedback helpers

Use `bw_screen_set_reverse(true)` for strong whole-display highlight
feedback, and always call `bw_screen_set_reverse(false)` before leaving a
screen, redrawing cached text from a normal baseline, or returning to another
applet. The helper writes both LCD controllers, turns the display on first, and
uses the same command path that validates in the headless emulator.

For applets that store display state in their `A5 + 0x300` state block, prefer
`bw_screen_set_reverse_cached(&state->reverse_flag, enabled)` and
`bw_screen_clear_reverse_cache(&state->reverse_flag)`. That avoids redundant
LCD command writes and prevents stale reverse mode from leaking across screens.

For periodic visual feedback, use:

```c
bool on = bw_flash_phase(idle_ms, 500u);
```

`write_or_die_bw` combines that with `bw_pressure_stage(...)`: the full
grace period is safe, the next four seconds flash as a warning, and the penalty
state flashes while deleting one trailing character every 700 ms. Normal typing
clears the highlight immediately and starts a new full grace period. The
headless validator checks the visible LCD diff, no deletion during warning,
repeated character deletion, and immediate recovery.

## Validated AlphaWord append helper

Use `bw_alphaword_append_text_block(slot, title, text, text_len)` when an applet
needs to append plain text to a visible AlphaWord File key slot. `slot` is the
user-visible file number, `1..8`; the helper handles the backing descriptor
rotation discovered from AlphaWord. Newlines in `text` are written as AlphaWord
carriage returns, and unsupported bytes are skipped.

The helper intentionally performs the narrow proven sequence only:

- find the AlphaWord applet
- temporarily focus AlphaWord and press the requested File key so firmware
  initializes the visible file state
- locate the last initialized backing descriptor for that visible file
- append `title`, a newline, then filtered text bytes
- update the used-length mirror fields and refill the remaining capacity with
  AlphaWord's `0xa7` tail byte

This is AlphaWord-specific and should not be used as general applet-owned
storage. The WriteOrDie headless validator verifies the exported session by
checking the selected AlphaWord backing descriptor buffer and repeated append
growth after export.
