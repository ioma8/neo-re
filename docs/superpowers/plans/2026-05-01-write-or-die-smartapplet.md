# WriteOrDie SmartApplet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a new `WriteOrDie` SmartApplet with setup goals, a pressure loop, permanent pause penalties, one-file persistence, and full headless validation.

**Architecture:** Implement a new Betawise-derived C SmartApplet under `smartapplets/write_or_die_bw/`, using the proven direct applet callback/editor path from `basic_writer_bw` and one-file snapshot persistence through `betawise-sdk/file_store`. Keep applet logic split into editor, challenge, UI, storage, and dispatch files so each part is testable and small.

**Tech Stack:** C for target applet code, `m68k-elf-gcc`, local `alpha-neo-pack`, Rust `alpha-emu` headless validation, existing Betawise-side SDK helpers.

---

## File Map

- Create `smartapplets/write_or_die_bw/WriteOrDieBW.c`: applet message dispatch and lifecycle.
- Create `smartapplets/write_or_die_bw/app_state.h`: persisted state, constants, enums.
- Create `smartapplets/write_or_die_bw/editor.h` and `editor.c`: text buffer, cursor movement, viewport, word count, delete-last-word.
- Create `smartapplets/write_or_die_bw/challenge.h` and `challenge.c`: pressure stages, completion rules, penalty timing.
- Create `smartapplets/write_or_die_bw/ui.h` and `ui.c`: setup, challenge, congrats rendering.
- Create `smartapplets/write_or_die_bw/storage.h` and `storage.c`: one-file snapshot load/save.
- Create `smartapplets/write_or_die_bw/Makefile`, `WriteOrDieBW.lds`, and `applet.env`.
- Modify `aplha-rust-native/crates/alpha-neo-pack/src/main.rs`: add `write-or-die` manifest.
- Modify `aplha-rust-native/crates/alpha-neo-pack/src/os3kapp.rs`: add packer test for `WriteOrDie`.
- Modify `alpha-emu/src/main.rs`: add `--validate-write-or-die` and applet state readers.
- Modify `alpha-emu/src/memory.rs`: include `exports/applets/write-or-die.os3kapp`.
- Modify `scripts/build-smartapplet.sh` only if the existing env-driven workflow cannot handle the new applet.
- Update `smartapplets/README.md` with the new reference applet after validation passes.

## Task 1: Packer Manifest And Workflow Skeleton

**Files:**
- Modify: `aplha-rust-native/crates/alpha-neo-pack/src/main.rs`
- Modify: `aplha-rust-native/crates/alpha-neo-pack/src/os3kapp.rs`
- Create: `smartapplets/write_or_die_bw/applet.env`
- Create: `smartapplets/write_or_die_bw/Makefile`
- Create: `smartapplets/write_or_die_bw/WriteOrDieBW.lds`
- Create: `smartapplets/write_or_die_bw/WriteOrDieBW.c`

- [ ] **Step 1: Write the failing packer test**

Add a test in `os3kapp.rs`:

```rust
#[test]
fn packages_write_or_die_shape() -> Result<(), Box<dyn Error>> {
    let manifest = AppletManifest {
        id: 0xA133,
        name: "WriteOrDie",
        version: Version::decimal(0, 1),
        flags: 0xFF00_00CE,
        base_memory_size: 0x4000,
        extra_memory_size: 0x2000,
        copyright: "neo-re Betawise WriteOrDie SmartApplet",
        file_count: 1,
        alphaword_write_metadata: true,
    };
    let image = build_image(&manifest, &[0x4E, 0x75])?;
    let slot_1 = [
        0xC0, 0x01, 0x80, 0x11, 0x00, 0x06, b'w', b'r', b'i', b't', b'e', 0x00,
    ];

    assert_eq!(&image[0x14..0x16], &[0xA1, 0x33]);
    assert_eq!(image[0x17], 1);
    assert!(image.windows(slot_1.len()).any(|window| window == slot_1));
    validate_image(&image)?;
    Ok(())
}
```

- [ ] **Step 2: Run the failing test**

Run:

```sh
cargo test --manifest-path aplha-rust-native/Cargo.toml -p alpha-neo-pack packages_write_or_die_shape
```

Expected: FAIL until the manifest path exists.

- [ ] **Step 3: Add `write-or-die` to the packer**

Add `WriteOrDie` to `AppletName`, parse `"write-or-die"`, usage text, and `manifest_for`.

- [ ] **Step 4: Add minimal applet skeleton**

Create `WriteOrDieBW.c`:

```c
#include "../betawise-sdk/applet.h"
#include "os3k.h"

typedef struct {
    uint32_t marker;
} AppState_t;

APPLET_ENTRY(alpha_neo_process_message);
APPLET_STATE(AppState_t);

void alpha_neo_process_message(uint32_t message, uint32_t param, uint32_t* status) {
    (void)param;
    AppState_t* state = State();
    *status = 0;
    if(message == MSG_SETFOCUS) {
        state->marker = 0x574F4431;
        ClearScreen();
        PutStringRaw("WriteOrDie");
    } else if(message == MSG_KEY && ((param & 0xff) == KEY_APPLETS || (param & 0xff) == KEY_ESC)) {
        *status = APPLET_EXIT_STATUS;
    } else {
        *status = APPLET_UNHANDLED_STATUS;
    }
}
```

Create `applet.env`:

```sh
APPLET_SLUG="write_or_die_bw"
PACKER_NAME="write-or-die"
ELF_NAME="WriteOrDieBW.elf"
OUTPUT_PATH="exports/applets/write-or-die.os3kapp"
VALIDATE_FLAG="--validate-write-or-die"
```

Use `basic_writer_bw/Makefile` and linker script as templates.

- [ ] **Step 5: Verify packer and skeleton**

Run:

```sh
cargo test --manifest-path aplha-rust-native/Cargo.toml -p alpha-neo-pack packages_write_or_die_shape
./scripts/build-smartapplet.sh write_or_die_bw --no-validate
```

Expected: test passes and `exports/applets/write-or-die.os3kapp` is written.

- [ ] **Step 6: Commit**

```sh
git add aplha-rust-native/crates/alpha-neo-pack/src/main.rs aplha-rust-native/crates/alpha-neo-pack/src/os3kapp.rs smartapplets/write_or_die_bw
git commit -m "Add WriteOrDie applet packaging skeleton"
```

## Task 2: Editor Core

**Files:**
- Create: `smartapplets/write_or_die_bw/app_state.h`
- Create: `smartapplets/write_or_die_bw/editor.h`
- Create: `smartapplets/write_or_die_bw/editor.c`
- Modify: `smartapplets/write_or_die_bw/Makefile`

- [ ] **Step 1: Define state and editor API**

In `app_state.h`, define:

```c
#define WOD_SCREEN_COLS 28
#define WOD_TEXT_ROWS 3
#define WOD_MAX_TEXT_BYTES 768

typedef struct {
    uint32_t len;
    uint32_t cursor;
    uint32_t viewport;
    char bytes[WOD_MAX_TEXT_BYTES];
} WodEditor_t;
```

In `editor.h`, expose:

```c
bool editor_insert_byte(WodEditor_t* editor, char byte);
bool editor_backspace(WodEditor_t* editor);
void editor_move_left(WodEditor_t* editor);
void editor_move_right(WodEditor_t* editor);
void editor_move_up(WodEditor_t* editor);
void editor_move_down(WodEditor_t* editor);
uint32_t editor_word_count(const WodEditor_t* editor);
bool editor_delete_last_word(WodEditor_t* editor);
void editor_render_row(const WodEditor_t* editor, uint8_t row, char* output);
```

- [ ] **Step 2: Port minimal editor implementation**

Copy the proven Basic Writer cursor/viewport logic into `editor.c`, changing visible rows to `WOD_TEXT_ROWS`. Add word counting and delete-last-word:

- word is a run of non-whitespace bytes
- whitespace is space, tab, CR, LF
- delete-last-word removes trailing whitespace, then the preceding word, then fixes cursor/viewport

- [ ] **Step 3: Compile-check after editor split**

Temporarily link `editor.o` in `Makefile`.

Run:

```sh
./scripts/build-smartapplet.sh write_or_die_bw --no-validate
```

Expected: applet still builds.

- [ ] **Step 4: Commit**

```sh
git add smartapplets/write_or_die_bw
git commit -m "Add WriteOrDie editor core"
```

## Task 3: Challenge State Machine

**Files:**
- Modify: `smartapplets/write_or_die_bw/app_state.h`
- Create: `smartapplets/write_or_die_bw/challenge.h`
- Create: `smartapplets/write_or_die_bw/challenge.c`
- Modify: `smartapplets/write_or_die_bw/Makefile`

- [ ] **Step 1: Define challenge model**

Add enums and state:

```c
typedef enum {
    WOD_PHASE_SETUP = 0,
    WOD_PHASE_RUNNING = 1,
    WOD_PHASE_COMPLETED = 2
} WodPhase_t;

typedef enum {
    WOD_GOAL_WORDS = 0,
    WOD_GOAL_TIME = 1
} WodGoalMode_t;

typedef enum {
    WOD_PRESSURE_SAFE = 0,
    WOD_PRESSURE_WARNING = 1,
    WOD_PRESSURE_DANGER = 2,
    WOD_PRESSURE_PENALTY = 3
} WodPressure_t;
```

- [ ] **Step 2: Implement pure challenge helpers**

In `challenge.c`:

```c
WodPressure_t challenge_pressure(uint32_t idle_ms, uint32_t grace_seconds);
uint32_t challenge_penalty_interval_ms(uint32_t grace_seconds);
bool challenge_words_complete(uint32_t words, uint32_t goal);
bool challenge_time_complete(uint32_t now_ms, uint32_t start_ms, uint32_t goal_seconds);
uint32_t challenge_remaining_seconds(uint32_t now_ms, uint32_t start_ms, uint32_t goal_seconds);
```

- [ ] **Step 3: Build-check**

Run:

```sh
./scripts/build-smartapplet.sh write_or_die_bw --no-validate
```

Expected: build passes.

- [ ] **Step 4: Commit**

```sh
git add smartapplets/write_or_die_bw
git commit -m "Add WriteOrDie challenge state helpers"
```

## Task 4: UI Rendering And Setup Menu

**Files:**
- Create: `smartapplets/write_or_die_bw/ui.h`
- Create: `smartapplets/write_or_die_bw/ui.c`
- Modify: `smartapplets/write_or_die_bw/app_state.h`
- Modify: `smartapplets/write_or_die_bw/WriteOrDieBW.c`
- Modify: `smartapplets/write_or_die_bw/Makefile`

- [ ] **Step 1: Add app state**

In `app_state.h`, define `WodAppState_t` with:

- phase
- selected setup row
- goal mode
- word goal
- time goal seconds
- grace seconds
- editor
- start time ms
- last activity ms
- last penalty ms
- dirty flag

- [ ] **Step 2: Implement setup rendering**

Render four fixed rows:

- selected row marker in column 1
- `Goal: Words 500` or `Goal: Time 10m`
- `Grace: 10s`
- `Start`
- `Draft saved` or `Ready`

- [ ] **Step 3: Implement setup input**

In dispatch:

- up/down changes selected setup row
- left/right adjusts goal/grace
- Enter starts challenge
- Applets exits

Use validation-friendly lower bounds:

- word goal minimum `5`
- time goal minimum `1 minute`
- grace minimum `2 seconds`

- [ ] **Step 4: Build-check**

Run:

```sh
./scripts/build-smartapplet.sh write_or_die_bw --no-validate
```

Expected: build passes.

- [ ] **Step 5: Commit**

```sh
git add smartapplets/write_or_die_bw
git commit -m "Add WriteOrDie setup UI"
```

## Task 5: Challenge Editing, Pressure, And Completion

**Files:**
- Modify: `smartapplets/write_or_die_bw/WriteOrDieBW.c`
- Modify: `smartapplets/write_or_die_bw/ui.c`
- Modify: `smartapplets/write_or_die_bw/challenge.c`
- Modify: `smartapplets/write_or_die_bw/editor.c`

- [ ] **Step 1: Wire typing to editor**

During `WOD_PHASE_RUNNING`:

- `MSG_CHAR` inserts printable bytes, Enter as `\n`, Backspace as delete
- printable/Enter/Backspace reset `last_activity_ms`
- arrows move cursor but do not reset activity

- [ ] **Step 2: Render challenge screen**

Row 1:

- word mode: `12/500 safe`
- time mode: `09:42 12w safe`
- pressure text changes to `safe`, `write`, `DANGER`, or `DELETE`

Rows 2-4 call `editor_render_row`.

- [ ] **Step 3: Implement idle pressure loop**

On `MSG_IDLE` while running:

- compute current pressure
- if penalty and enough time elapsed, call `editor_delete_last_word`
- redraw when pressure text changes, timer changes, or penalty deletes text
- mark dirty after penalty deletion

- [ ] **Step 4: Implement completion**

After each edit and idle tick:

- word mode completes when `editor_word_count >= word_goal`
- time mode completes when remaining seconds reaches zero
- set phase completed
- draw congrats screen

- [ ] **Step 5: Build-check**

Run:

```sh
./scripts/build-smartapplet.sh write_or_die_bw --no-validate
```

Expected: build passes.

- [ ] **Step 6: Commit**

```sh
git add smartapplets/write_or_die_bw
git commit -m "Implement WriteOrDie challenge loop"
```

## Task 6: One-File Persistence

**Files:**
- Create: `smartapplets/write_or_die_bw/storage.h`
- Create: `smartapplets/write_or_die_bw/storage.c`
- Modify: `smartapplets/write_or_die_bw/WriteOrDieBW.c`
- Modify: `smartapplets/write_or_die_bw/Makefile`

- [ ] **Step 1: Add storage wrapper**

Use `applet_load_snapshot(1, magic, state, sizeof(*state))` and `applet_save_snapshot(1, magic, state, sizeof(*state))`.

Use magic:

```c
static const char WOD_MAGIC[4] = {'W', 'O', 'D', '1'};
```

- [ ] **Step 2: Load on focus**

On `MSG_SETFOCUS`:

- zero state
- set defaults
- load snapshot if valid
- clamp invalid fields
- render setup or completed screen based on loaded phase

- [ ] **Step 3: Save on transitions**

Save:

- when challenge starts
- after completion
- on Applets/Esc exit
- after penalty deletion

Avoid saving on every character unless validation shows it is required.

- [ ] **Step 4: Build-check**

Run:

```sh
./scripts/build-smartapplet.sh write_or_die_bw --no-validate
```

Expected: build passes.

- [ ] **Step 5: Commit**

```sh
git add smartapplets/write_or_die_bw
git commit -m "Add WriteOrDie snapshot persistence"
```

## Task 7: Headless Emulator Integration

**Files:**
- Modify: `alpha-emu/src/memory.rs`
- Modify: `alpha-emu/src/main.rs`
- Modify: `smartapplets/write_or_die_bw/applet.env`

- [ ] **Step 1: Load exported applet**

Add `../exports/applets/write-or-die.os3kapp` to `EXTRA_APPLET_PATHS`.

- [ ] **Step 2: Add CLI flag**

Add `--validate-write-or-die` near existing validation flags.

- [ ] **Step 3: Add applet launch helper**

Follow `launch_basic_writer_through_menu`. Locate `WriteOrDie` in the SmartApplets menu by text/OCR or by deterministic menu navigation after the applet is included.

- [ ] **Step 4: Add state readers**

Read `WodAppState_t` from `A5 + 0x300` like Basic Writer validation does. Assert:

- phase
- goal mode
- word goal
- time goal
- grace
- text preview
- word count

- [ ] **Step 5: Add validation scenario**

Validation should:

1. launch applet
2. assert setup defaults
3. reduce word goal to validation value if needed
4. start word challenge
5. type enough words to complete
6. assert completed phase and congrats screen
7. exit and relaunch
8. assert completed text persists
9. start/enter a running draft, pause past penalty threshold, assert word deletion
10. switch to time mode and assert remaining timer decreases

- [ ] **Step 6: Run emulator check**

Run:

```sh
cargo check --manifest-path alpha-emu/Cargo.toml
./scripts/build-smartapplet.sh write_or_die_bw
```

Expected: `write_or_die_validation=ok ... exception=none`.

- [ ] **Step 7: Commit**

```sh
git add alpha-emu/src/main.rs alpha-emu/src/memory.rs smartapplets/write_or_die_bw/applet.env
git commit -m "Add WriteOrDie headless validation"
```

## Task 8: Documentation And Regression Pass

**Files:**
- Modify: `smartapplets/README.md`
- Possibly modify: `hypotheses.tsv` if debugging produced confirmed findings

- [ ] **Step 1: Document the applet**

Add `write_or_die_bw` to current references and include the build command:

```sh
./scripts/build-smartapplet.sh write_or_die_bw
```

- [ ] **Step 2: Run full targeted validations**

Run:

```sh
./scripts/build-smartapplet.sh write_or_die_bw
./scripts/build-smartapplet.sh basic_writer_bw
./scripts/build-smartapplet.sh forth_mini_bw
cargo test --manifest-path aplha-rust-native/Cargo.toml -p alpha-neo-pack
cargo check --manifest-path alpha-emu/Cargo.toml
```

Expected:

- all three applet validators report `..._validation=ok`
- packer tests pass
- emulator check passes

- [ ] **Step 3: Commit docs and final fixes**

```sh
git add smartapplets/README.md hypotheses.tsv
git commit -m "Document WriteOrDie SmartApplet workflow"
```

## Final Verification

Before claiming completion, run:

```sh
git status --short
./scripts/build-smartapplet.sh write_or_die_bw
cargo test --manifest-path aplha-rust-native/Cargo.toml -p alpha-neo-pack
cargo check --manifest-path alpha-emu/Cargo.toml
```

Expected final state:

- `WriteOrDie` builds through the one-command workflow
- headless validation passes
- one-file snapshot survives relaunch
- pause penalty permanently deletes words
- Basic Writer and Forth Mini workflows remain unbroken
