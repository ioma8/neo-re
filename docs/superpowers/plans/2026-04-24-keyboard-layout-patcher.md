# Keyboard Layout Patcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a precise `uv`-managed Python tool that patches one stock non-default NEO OS keyboard layout to `czech` or `polish`, then verify the patched image boots in the headless emulator.

**Architecture:** Create a small standalone Python package under `layout-patcher/` with explicit firmware constants, embedded replacement mapping data, and a narrow CLI. Keep binary size unchanged by rewriting only known in-place layout-table and string slots after validating anchor bytes and strings.

**Tech Stack:** Python 3.12, `uv`, `pytest`, stock full-OS binary `analysis/cab/os3kneorom.os3kos`, Rust `alpha-emu`

---

### Task 1: Scaffold The New `uv` Project

**Files:**
- Create: `layout-patcher/pyproject.toml`
- Create: `layout-patcher/README.md`
- Create: `layout-patcher/src/layout_patcher/__init__.py`
- Create: `layout-patcher/src/layout_patcher/cli.py`
- Test: `layout-patcher/tests/test_cli.py`

- [ ] **Step 1: Write the failing CLI help test**

Add a CLI test that imports `layout_patcher.main`, runs `["--help"]`, and asserts the exit code is `0` and the help mentions `--replace` and `--with`.

- [ ] **Step 2: Run test to verify it fails**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_cli.py -q`

Expected: FAIL because the project and package do not exist yet.

- [ ] **Step 3: Write minimal project scaffold and CLI help implementation**

Create the package, expose `main`, parse arguments with `argparse`, and make `--help` work.

- [ ] **Step 4: Run test to verify it passes**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_cli.py -q`

Expected: PASS

### Task 2: Identify And Validate The Stock Firmware

**Files:**
- Create: `layout-patcher/src/layout_patcher/firmware.py`
- Test: `layout-patcher/tests/test_firmware.py`

- [ ] **Step 1: Write a failing test for stock firmware identification**

Add a test that loads `analysis/cab/os3kneorom.os3kos` and asserts the validator recognizes required string/table anchors.

- [ ] **Step 2: Run test to verify it fails**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_firmware.py -q`

Expected: FAIL because validation logic is missing.

- [ ] **Step 3: Write minimal validator implementation**

Implement a small firmware model that checks expected file offsets, required strings, and layout-table anchor bytes.

- [ ] **Step 4: Run test to verify it passes**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_firmware.py -q`

Expected: PASS

- [ ] **Step 5: Write a failing rejection test**

Add a test that mutates an anchor byte/string and asserts validation fails with a useful error.

- [ ] **Step 6: Run test to verify it fails**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_firmware.py -q`

Expected: FAIL because the validator does not yet reject the altered image correctly.

- [ ] **Step 7: Extend validator minimally**

Add explicit failure messages for anchor mismatches.

- [ ] **Step 8: Run test to verify it passes**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_firmware.py -q`

Expected: PASS

### Task 3: Encode Replacement Layout Data

**Files:**
- Create: `layout-patcher/src/layout_patcher/layouts.py`
- Modify: `layout-patcher/src/layout_patcher/firmware.py`
- Test: `layout-patcher/tests/test_firmware.py`

- [ ] **Step 1: Write a failing test for supported layout metadata**

Add a test that asserts `czech` and `polish` exist as embedded patch definitions and that `dvorak`, `right`, and `left` resolve to stock patch slots.

- [ ] **Step 2: Run test to verify it fails**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_firmware.py -q`

Expected: FAIL because the data tables are missing.

- [ ] **Step 3: Add minimal explicit mapping constants**

Create constants for replacement layouts and stock slot metadata, including the exact target remap-column offsets.

- [ ] **Step 4: Run test to verify it passes**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_firmware.py -q`

Expected: PASS

### Task 4: Patch One Layout Column In Place

**Files:**
- Modify: `layout-patcher/src/layout_patcher/firmware.py`
- Test: `layout-patcher/tests/test_patch_integration.py`

- [ ] **Step 1: Write a failing integration test for column patching**

Add a test that copies the stock OS bytes in memory, patches one requested slot, and asserts only the expected layout-table column bytes change.

- [ ] **Step 2: Run test to verify it fails**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_patch_integration.py -q`

Expected: FAIL because patching logic is missing.

- [ ] **Step 3: Write minimal column patcher**

Implement in-place remap-column rewriting based on explicit byte offsets for the chosen stock slot.

- [ ] **Step 4: Run test to verify it passes**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_patch_integration.py -q`

Expected: PASS

### Task 5: Patch Visible Layout Names

**Files:**
- Modify: `layout-patcher/src/layout_patcher/firmware.py`
- Test: `layout-patcher/tests/test_patch_integration.py`

- [ ] **Step 1: Write a failing integration test for long and compact strings**

Add a test that patches one stock slot and asserts the corresponding long status/prompt strings and compact names are rewritten, NUL-padded, or cropped exactly as required.

- [ ] **Step 2: Run test to verify it fails**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_patch_integration.py -q`

Expected: FAIL because string patching is missing.

- [ ] **Step 3: Write minimal string patcher**

Implement exact-width ASCII rewriting for the selected slot's visible strings only.

- [ ] **Step 4: Run test to verify it passes**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_patch_integration.py -q`

Expected: PASS

### Task 6: Finish The CLI

**Files:**
- Modify: `layout-patcher/src/layout_patcher/cli.py`
- Modify: `layout-patcher/src/layout_patcher/__init__.py`
- Test: `layout-patcher/tests/test_cli.py`

- [ ] **Step 1: Write a failing CLI command test**

Add a test that runs the CLI with `--input`, `--output`, `--replace`, and `--with`, then asserts an output file is written and a concise summary is printed.

- [ ] **Step 2: Run test to verify it fails**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_cli.py -q`

Expected: FAIL because the CLI does not yet patch files end-to-end.

- [ ] **Step 3: Write minimal end-to-end CLI implementation**

Wire the CLI to firmware loading, validation, patching, writing, and summary output.

- [ ] **Step 4: Run test to verify it passes**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_cli.py -q`

Expected: PASS

### Task 7: Document Usage

**Files:**
- Modify: `layout-patcher/README.md`
- Test: `layout-patcher/tests/test_cli.py`

- [ ] **Step 1: Write a failing test for the documented command example if needed**

If helpful, add a small test asserting the help text includes the supported replacement and replacement-target choices.

- [ ] **Step 2: Run test to verify it fails**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_cli.py -q`

Expected: FAIL if the help text is incomplete.

- [ ] **Step 3: Update README and help text minimally**

Document installation, usage, supported stock slots, supported replacement layouts, and the emulator smoke-check command.

- [ ] **Step 4: Run test to verify it passes**

Run: `uv run --project layout-patcher pytest layout-patcher/tests/test_cli.py -q`

Expected: PASS

### Task 8: Full Verification

**Files:**
- Modify: `layout-patcher/README.md` if command details need correction from real verification

- [ ] **Step 1: Run the full test suite**

Run: `uv run --project layout-patcher pytest -q`

Expected: PASS

- [ ] **Step 2: Produce a real patched OS image**

Run: `uv run --project layout-patcher layout-patcher --input analysis/cab/os3kneorom.os3kos --output /tmp/os3kneorom-czech.os3kos --replace dvorak --with czech`

Expected: output file written with concise summary

- [ ] **Step 3: Run the emulator smoke test**

Run: `cargo run --manifest-path alpha-emu/Cargo.toml -- --headless --steps=120000000 /tmp/os3kneorom-czech.os3kos`

Expected: emulator boots the full OS image and prints the normal headless summary line without an unhandled exception
