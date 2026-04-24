# Keyboard Layout Patcher Design

Date: 2026-04-24

## Goal

Build a precise Python patcher in a new `uv`-managed directory that replaces
one existing non-default AlphaSmart NEO firmware keyboard layout with either a
`czech` or `polish` layout in the stock full OS image
`analysis/cab/os3kneorom.os3kos`.

The patcher must:

- patch exactly one existing non-default slot: `dvorak`, `right`, or `left`
- support replacement layouts: `czech`, `polish`
- keep the binary size unchanged
- patch in place using known offsets and validated anchors
- patch both the layout mapping data and user-visible names
- crop replacement names if a string slot is too short
- be tested with TDD
- be externally verified by booting the patched image in `alpha-emu` headless

## Non-Goals

- adding a fifth layout
- patching `qwerty`
- repointing firmware resources or relocating data
- patching arbitrary firmware revisions heuristically
- supporting non-ASCII replacement names

## Constraints

The stock firmware uses a fixed layout model:

- selector byte at `0x00005d36`
- selector values `0..3`
- `3` is the pass-through default QWERTY mode
- values `0..2` select one of three alternate remap columns
- the remap table lives at runtime address `0x0044c3fb`
- in the file that table is at offset `0x0003c3fb`

The stock firmware also contains fixed-size string blocks for long layout
prompts and compact layout names. The patcher must preserve file size and only
rewrite bytes in place.

## Chosen Approach

Implement a narrow in-place patcher with embedded replacement data.

Why this approach:

- safest for a firmware binary with fixed branch structure
- deterministic and reviewable
- no runtime guessing about offsets or table structure
- easiest to verify with exact byte-level tests

Rejected alternatives:

- deriving layout mappings algorithmically at runtime: less transparent and
  harder to review
- adding a new layout slot: requires OS code patching beyond this tool's scope
- repointing strings for longer names: riskier and unnecessary for this task

## Tool Structure

Create a new directory:

- `layout-patcher/`

Proposed files:

- `layout-patcher/pyproject.toml`
- `layout-patcher/README.md`
- `layout-patcher/src/layout_patcher/__init__.py`
- `layout-patcher/src/layout_patcher/cli.py`
- `layout-patcher/src/layout_patcher/firmware.py`
- `layout-patcher/src/layout_patcher/layouts.py`
- `layout-patcher/tests/test_firmware.py`
- `layout-patcher/tests/test_cli.py`
- `layout-patcher/tests/test_patch_integration.py`

## CLI

Proposed command shape:

```sh
uv run --project layout-patcher layout-patcher \
  --input analysis/cab/os3kneorom.os3kos \
  --output /tmp/os3kneorom-czech.os3kos \
  --replace dvorak \
  --with czech
```

Arguments:

- `--input`: source OS image
- `--output`: patched OS image path
- `--replace`: one of `dvorak`, `right`, `left`
- `--with`: one of `czech`, `polish`

Behavior:

- read the full image
- validate anchor bytes and strings
- patch the chosen layout column
- patch the chosen visible name strings
- write the patched output
- print a concise summary of what changed

## Firmware Knowledge Embedded In Tool

The patcher will encode:

- expected firmware size and anchor bytes
- fixed file offsets for the layout remap table
- mapping from stock layout name to alternate table column
- fixed file offsets for long status strings and compact layout names
- replacement layout columns for `czech` and `polish`

The patcher must fail closed if any required anchor does not match.

## Layout Mapping Data

Replacement layout mappings will be stored as explicit firmware logical-key
remap columns, not as inferred text tables.

This means:

- each replacement layout is represented as a list of logical-key targets
- the patcher rewrites exactly one existing alternate column
- the mapping data is stable, explicit, and testable

ASCII handling:

- accented variants are normalized to ASCII equivalents before being encoded
- for this tool, practical visible names are `Czech` and `Polish`
- replacement key behavior also uses ASCII-oriented mappings

## String Patching

The patcher will update only strings associated with the selected replaced slot.

String classes:

- long prompt/status strings such as `Key layout changed to Dvorak.`
- compact menu/status names such as `Dvorak`, `Right`, `Left`

Patch rule:

- write replacement text as ASCII bytes
- if the target slot is longer, NUL-pad the remaining bytes
- if the replacement text is longer than the slot, crop it to fit exactly

This is intentionally asymmetric: names are allowed to crop because preserving
layout and binary stability is more important than preserving full label text.

## Safety Checks

Before patching, the tool must verify:

- file exists and is readable
- file matches expected stock full-OS size or anchor pattern
- key layout strings are present at expected offsets
- layout remap table anchor bytes match expected stock bytes
- requested `--replace` and `--with` values are supported

Failure mode:

- no partial output written
- actionable error message describing which validation failed

## Testing Strategy

Follow strict TDD in small iterations.

Initial test sequence:

1. failing test for identifying the stock OS image correctly
2. failing test for rejecting altered or unsupported images
3. failing test for patching exactly one layout column
4. failing test for patching long strings for the selected slot only
5. failing test for patching compact names for the selected slot only
6. failing test for CLI argument validation
7. failing integration test for output image byte diffs at exact offsets

Each implementation step will be followed by an external validity check:

- `uv run --project layout-patcher pytest ...`

## External Verification

After implementation is complete:

1. run the patcher on the stock OS image
2. boot the patched image in `alpha-emu` headless mode
3. confirm the emulator still reaches the normal full-OS boot path without
   crashing

Expected verification form:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  /tmp/os3kneorom-czech.os3kos
```

The emulator check is a boot smoke test, not full behavioral proof of the new
layout mapping.

## Open Technical Item

Before finalizing the exact layout-column patching constants, the implementation
must pin down the exact mapping between stock layout names and alternate table
columns for:

- `dvorak`
- `right`
- `left`

This should be resolved during the first TDD cycles and then encoded as a fixed
constant mapping in the tool.

## Success Criteria

The work is successful when:

- the tool patches one stock non-default layout with `czech` or `polish`
- output size is unchanged
- tests pass
- `alpha-emu` headless can boot the patched OS image successfully
- the patched image updates the selected layout name in firmware-visible text
  slots
