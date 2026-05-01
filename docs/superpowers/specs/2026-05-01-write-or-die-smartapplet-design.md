# WriteOrDie SmartApplet Design

## Goal

Build a new AlphaSmart NEO SmartApplet named `WriteOrDie`, inspired by the original Write or Die feedback loop:

`continuous writing = safe -> pause = pressure -> long pause = permanent penalty`

The applet should run through the validated Betawise-derived C workflow, use one applet-owned file, and save both in-progress drafts and completed sessions.

## Non-Goals

- No sound effects; the NEO has no useful sound path for this applet.
- No multi-file document switching.
- No rich document management, naming, export UI, or formatting.
- No firmware `TextBox` editor path; previous work showed the direct applet editor path is more reliable.

## User Flow

On launch, the applet opens to a four-line setup screen:

- goal row: `Goal: Words 500` or `Goal: Time 10m`
- grace row: `Grace: 10s`
- `Start`
- compact status/help row, including whether a saved draft/result exists

Controls:

- up/down moves the selected setup row
- left/right changes the selected value
- Enter starts the challenge
- Applets exits
- goal row toggles between word-count and time challenge modes

During a challenge:

- row 1 is a live status row
- rows 2-4 are the text editor viewport
- typing, Enter, and Backspace count as writing activity
- arrow movement does not reset the pressure timer

Status row examples:

- word mode: `123/500 safe`
- time mode: `09:42 123w safe`

Time mode displays remaining time, not elapsed time.

On success, the applet shows a compact congrats screen with the final word count and goal summary, then saves the completed snapshot.

## Defaults

- word goal: `500`
- time goal: `10 minutes`
- grace period: `10 seconds`

Values should be adjustable in setup. The exact increments can be implementation-local, but should be fast enough to configure on a small keyboard:

- word goal in coarse steps, with reasonable lower values available for emulator validation
- time goal in minute steps
- grace period in second steps

## Pressure Loop

The configured grace period drives all pressure thresholds.

For default `10s` grace:

- `0-10s`: safe
- `10-20s`: warning
- `20-30s`: danger
- `30s+`: penalty

General thresholds:

- warning starts at `grace`
- danger starts at `2 * grace`
- deletion starts at `3 * grace`

Indicators:

- safe: normal status text
- warning: status row says `write`
- danger: status row alternates or inverts warning text during idle redraws
- penalty: permanently delete one word every `max(grace / 2, 2s)` until typing resumes

Any printable character, Enter, or Backspace resets the pressure timer and stops active penalties. Deleted text is not recoverable through this applet.

## Completion Rules

Word-count mode completes when the current word count reaches the configured goal.

Time mode completes when remaining time reaches zero. The user does not need to hit a minimum word count in time mode for v1.

On completion:

- stop the pressure loop
- save the completed snapshot
- render the congrats screen
- keep the completed text available on the next launch as the last saved result

## Persistence

The applet owns exactly one file.

Use runtime file handle `1` through the validated Betawise-side `file_store` snapshot helper. The persisted snapshot stores:

- magic and version
- phase: setup, running, completed
- goal mode
- word goal
- time goal seconds
- grace seconds
- text bytes
- text length
- cursor
- viewport
- challenge start uptime
- last activity uptime
- last penalty uptime
- final word count and completion summary fields

Both partial and completed sessions are saved:

- in-progress drafts save on exit and important state transitions
- completed sessions save when the goal is reached
- penalty deletions mark the draft dirty and are persisted before exit or completion

Startup should load the one saved snapshot if present. If it is valid, setup opens with the saved settings and draft/result available. Starting a new challenge can reuse the current text buffer; v1 does not add a separate file-management or clear command.

## Architecture

Create `smartapplets/write_or_die_bw/` using the existing Betawise-derived applet workflow.

Recommended files:

- `WriteOrDieBW.c`: message dispatch and lifecycle only
- `app_state.h`: persisted state structure and constants
- `editor.c/h`: text buffer, cursor movement, wrapping, word count, delete-last-word
- `challenge.c/h`: goal completion, pressure-stage calculation, penalty timing
- `ui.c/h`: setup screen, challenge screen, congrats screen rendering
- `storage.c/h`: one-file snapshot load/save wrapper
- `Makefile`, linker script, and `applet.env`

Keep files under 200 lines where practical. If a file crosses that limit because of tables or syscall declarations, split responsibility rather than hiding unrelated logic together.

## Packaging

Add a new packer manifest:

- packer name: `write-or-die`
- applet id: `0xA133`
- applet name: `WriteOrDie`
- file count: `1`
- AlphaWord-style write metadata enabled
- memory size based on `basic_writer_bw`, adjusted only if validation shows it is required

The one-command workflow should support:

```sh
./scripts/build-smartapplet.sh write_or_die_bw
```

## Validation

Add a full-system headless validator, `--validate-write-or-die`.

Acceptance coverage:

- applet launches from SmartApplets menu
- setup menu appears with defaults
- word mode can start and complete a small validation goal
- congrats screen appears
- saved text/result survives exit and relaunch
- a pause past the penalty threshold permanently deletes at least one word
- time mode status row displays remaining time and counts down
- Applets key exits without crashing and saves the partial draft

Host-level tests should cover:

- word counting
- delete-last-word behavior
- cursor and viewport behavior inherited from the Basic Writer editor model
- pressure-stage threshold calculation
- completion conditions for word and time modes
- packer shape: id, name, one owned file, and metadata record for `0x8011`

## Risks

The largest implementation risk is applet size and m68k codegen stability. Start with the simple Basic Writer profile, then switch to the conservative `forth_mini_bw` profile if runtime behavior is unstable.

The second risk is UI density. The applet has only four lines; row 1 must stay concise and setup must avoid explanatory screens.

The third risk is destructive penalties. Because deletion is permanent by design, validation must prove that penalty deletion changes the persisted snapshot as well as the live buffer.
