# Forth Mini Redesign

## Goal

Redesign `Forth Mini` from scratch as a standard AlphaSmart NEO SmartApplet
that follows the normal firmware-owned lifecycle and input dispatch model.

The redesign must eliminate the current freeze-after-first-input behavior and
replace the current mixed architecture with a smaller, cleaner structure that
is easier to validate in the emulator and safer to run on the real device.

## Product Shape

`Forth Mini` should behave like a small REPL designed for the NEO's four-line
screen:

- line 1: static title
- line 2: stack summary
- line 3: most recent output or error
- line 4: prompt and current input buffer

Input is line-based. The applet evaluates only when the user presses `Enter`.
The applet starts fresh every time it is opened.

## Core Constraints

- The firmware remains in charge of applet lifecycle and keyboard delivery.
- The applet must not run its own keyboard polling loop in `on_focus`.
- The applet must not depend on synthetic validation-only control flow that
  differs from the real firmware-launched path.
- The applet must use only stable SDK drawing/input helpers that already match
  validated SmartApplet behavior.
- The applet must avoid partial-row rendering tricks and custom prompt drawing
  helpers that have already produced m68k-specific codegen faults.

## Recommended Architecture

Use a firmware-driven message applet with a small explicit session state block.

Files:

- `main.rs`: SmartApplet adapter only
- `forth.rs`: pure REPL core and screen model

Responsibilities:

- `main.rs`
  - define the applet type and callback entrypoints
  - reset session state in `on_focus`
  - route `on_char` for printable input
  - route `on_key` for `Enter`, `Backspace`, and exit keys
  - call one redraw function after every state change
- `forth.rs`
  - hold the small session state
  - implement stack operations and token evaluation
  - expose screen lines as fixed 28-byte arrays
  - contain no SDK, trap, or firmware-specific code

This keeps platform coupling at the edges and lets the REPL logic stay pure and
fully unit-testable.

## Session State

The applet keeps one bounded state struct for the current open session:

- operand stack
- current input buffer
- one visible output line
- optional status/error line content if needed

The applet resets this state every time `on_focus` runs. There is no attempt to
preserve state across exits or applet switches.

## Lifecycle Model

`on_focus`

- initialize a fresh session state
- render the initial four-line screen
- return normally

`on_char`

- accept printable ASCII bytes
- append to the input buffer if there is room
- redraw the screen

`on_key`

- `Enter`: evaluate the current input line, update stack/output, clear input,
  redraw
- `Backspace`: delete one byte from the input buffer, redraw
- `Esc` or applet-exit keys: return `APPLET_EXIT`
- all other keys: ignore or return unhandled

There is no custom event loop, no manual `pump_events`, and no `yield_once`
loop inside the applet.

## UI Rules

Render the full four-line screen every time. Do not use incremental partial-row
updates.

Screen format:

- line 1: `Forth Mini`
- line 2: compact stack summary such as `S:<2> 1 2`
- line 3: most recent result or error such as `3` or `stack underflow`
- line 4: `> ` plus the current input buffer

Prompt rendering should be plain text only. No custom cursor glyph, inverse
field, or per-character helper beyond the standard row write used elsewhere in
the SDK.

## Forth Feature Set

First supported words:

- numbers
- `+`
- `-`
- `*`
- `/`
- `mod`
- `dup`
- `drop`
- `swap`
- `over`
- `.`
- `.s`
- `clear`

Errors:

- divide by zero
- stack underflow
- stack overflow
- unknown word
- input full

This is enough for a real minimal Forth REPL on the device without adding
control-flow words, variables, or dictionary mutation.

## Validation Strategy

Validation should shift away from direct synthetic applet-entry assumptions and
toward real firmware-launched behavior.

Required test layers:

1. Pure REPL unit tests in `forth.rs`
   - arithmetic
   - stack words
   - errors
   - prompt/output line formatting
2. Packaging/build validation
   - `cargo +nightly test --manifest-path aplha-rust-native/Cargo.toml -p forth-mini-applet`
   - `cd aplha-rust-native && ./build.sh forth_mini`
3. Emulator integration validation
   - open `Forth Mini` through the normal firmware applet list
   - inject real matrix key events, not only direct message calls
   - verify the screen changes after printable input
   - verify `Enter` evaluates and updates output

Direct message-based validation can remain as a narrow ABI sanity check, but it
must not be treated as sufficient proof of correct launched behavior.

## Implementation Notes

- Remove the current custom focus-loop design entirely.
- Remove the current custom prompt rendering helper from the Forth Mini applet
  path.
- Keep the SDK changes minimal unless the redesign proves an SDK bug with clear
  evidence.
- Prefer a simpler applet over a richer one. Reliability is the priority.

## Out of Scope

- persistent session state across applet exits
- blinking custom cursor inside the applet
- full ANS Forth dictionary support
- variables, loops, conditionals, or user-defined words
- device-side file persistence
