# SmartApplets Dataflow

This note maps the SmartApplet binary flow in both directions:

- app to device: install or replace SmartApplets on the NEO
- device to app: retrieve SmartApplet binaries and the applet list from the NEO

The focus here is the direct updater protocol and the concrete app-side call chain that reaches it.

## Confirmed Updater Primitives

These functions are now pinned from string xrefs and decompilation:

- `UpdaterGetAppletList`
- `UpdaterAddApplet`
- `UpdaterRemoveApplet`
- `UpdaterRetrieveApplet`
- `UpdaterSaveAppletFileData`

Transport wrappers:

- `DirectUsbGetAppletList` wraps `UpdaterGetAppletList` for direct USB mode `2`
- `AlternateTransportGetAppletList` wraps `UpdaterGetAppletList` for alternate mode `3`
- `DirectUsbRetrieveApplet` wraps `UpdaterRetrieveApplet` for direct USB mode `2`
- `AlternateTransportRetrieveApplet` wraps `UpdaterRetrieveApplet` for alternate mode `3`

## Applet List Format

`UpdaterGetAppletList` sends:

- command `0x04`
- `arg32 = page_offset`
- `trailing = page_size`

Confirmed default request:

- `0x04`, `page_offset = 0`, `page_size = 7`
- packet bytes: `04 00 00 00 00 00 07 0b`

Expected response:

- header byte `0x44`
- payload length is a multiple of `0x84`
- checksum is 16-bit payload sum

The app normalizes the retrieved `0x84`-byte records after retrieval:

- `FetchRawSmartAppletListEntries`
- `QuerySmartAppletListAdapter`

Those functions swap selected big-endian dwords inside each `0x84` record before the UI uses them.

The important structural conclusion is that the device-side `0x84` list entry and the first `0x84` bytes of a host `.OS3KApp` file use the same metadata layout. The same parser object is used for both:

- list entry path: `QuerySmartAppletListAdapter` -> `BuildSmartAppletDetailsText` / `AppendSmartAppletStartupDetailsText` -> `ParseSmartAppletHeaderRecord`
- on-disk file path: `ParseSmartAppletImageMetadata` -> `ParseSmartAppletHeaderRecord`

Mapped shared `0x84` metadata layout:

- `0x00..0x03`: magic for on-disk `.OS3KApp` files, typically `0xc0ffeead`
- `0x04..0x07`: total SmartApplet file size
- `0x08..0x0b`: base memory requirement
- `0x0c..0x0f`: info-table offset inside the full `.OS3KApp` image, or `0` if absent
- `0x10..0x13`: packed flags dword
- `0x14..0x15`: applet id
- `0x16..0x17`: major/minor applet version in the `applet_id_and_version` dword
- `0x18..0x3f`: NUL-terminated applet name
- `0x3c`: BCD major version shown in UI
- `0x3d`: BCD minor version shown in UI
- `0x3e`: raw version/build byte
- `0x3f`: applet class byte used for a string lookup helper
- `0x40..0x7f`: NUL-terminated copyright string
- `0x80..0x83`: extra memory requirement

Confirmed bit extraction from the packed flags dword at `0x10..0x13`:

- bit `0x10000000` is extracted into the internal field at object offset `0x54`
- bit `0x00010000` is extracted through `FUN_004066b0` into the internal field at object offset `0x60`
- bit `0x40000000` is extracted into the internal field at object offset `0x5c`

The exact user-facing meaning of those extracted values is still unresolved, and the middle field is not a simple stable boolean in later runtime code. The extraction sites are pinned:

- on-disk path: `ParseSmartAppletImageMetadata`
- in-memory metadata path: `ParseSmartAppletListEntryMetadata`

Important runtime distinction:

- object offset `0x58` is not the parsed `0x00010000` flag bit
- offset `0x58` is a separate runtime marker set by the remove path `FUN_0043b010`
- `FUN_0042c190` checks that remove marker before deciding whether an applet still counts as present in the current session
- offset `0x60`, which initially receives the parsed `0x00010000` bit, is later reused by workflows such as `FUN_004072c0` and `FUN_00407320` as a traversal cursor / current-device index

Observed examples:

- `alphawordplus.os3kapp`: applet id `0xa000`, version `3.4`, class `0x01`
- `calculator.os3kapp`: applet id `0xa002`, version `3.0`, class `0x01`
- `keywordswireless.os3kapp`: applet id `0xa004`, version `4.0`, class `0x01`

## On-Disk `.OS3KApp` Container Layout

Cross-checking all shipped `.os3kapp` samples shows a consistent outer container:

1. first `0x84` bytes: SmartApplet metadata header
2. remaining bytes: raw big-endian Motorola 68k payload
3. optional trailing SmartApplet info table starting at header offset `0x0c`

The app streams the full file to the device during `UpdaterAddApplet`. It does not strip the info table off before sending. The info-table offset is therefore a pointer inside the uploaded image, not a separate host-only sidecar.

Confirmed layout:

- `0x0000..0x0083`: shared SmartApplet metadata header
- `0x0084..EOF`: payload bytes uploaded verbatim
- `0x0084..0x0093`: four big-endian payload prefix dwords
- `header[0x0c:0x10] != 0`: optional appended info table starts at that absolute file offset

Observed payload prefix dwords across all shipped samples:

- dword 0: variable offset inside the image, always within the file, always disassembles as plausible 68k code
- dword 1: always `0`
- dword 2: always `1`
- dword 3: always `2`

Strong current interpretation:

- dword 0 is the primary applet entry offset
- dwords 1..3 are fixed ABI/version markers for the embedded runtime image

That entry-offset interpretation is supported by the samples:

- when dword 0 is `0x94`, the image entry starts immediately after the 16-byte payload prefix
- when dword 0 is larger than `0x94`, the bytes from `0x94` up to the entry offset form a loader/helper stub region, and the pointed-to offset still begins with a normal 68k function prologue

Examples:

- `neofontmedium.os3kapp`: payload prefix `(0x94, 0, 1, 2)`
- `calculator.os3kapp`: payload prefix `(0x168, 0, 1, 2)`
- `alphaquiz.os3kapp`: payload prefix `(0x0e20, 0, 1, 2)`
- `spellcheck_small_usa.os3kapp`: payload prefix `(0x33b2, 0, 1, 2)`

Practical container split for tooling:

- `payload_total_size = file_size - 0x84`
- `loader_stub = file[0x94:entry_offset]` when `entry_offset > 0x94`
- `code_before_info_table = file[0x84:info_table_offset]` when `info_table_offset != 0`
- `info_table_bytes = file[info_table_offset:]` when `info_table_offset != 0`

This is enough to decode and extract shipped `.os3kapp` files structurally:

- header fields
- total uploaded payload
- inferred primary entry offset
- optional loader stub before that entry
- optional appended info table

What is not closed yet is the full calling convention of the 68k entry function. That is an inner SmartApplet runtime ABI question, not an outer `.os3kapp` container question.

## Inner SmartApplet ABI

The payload prefix dword at file offset `0x84` is not just a generic code pointer. In the raw `calculator.os3kapp` and `alphaquiz.os3kapp` images, it points to the top-level SmartApplet dispatcher function.

Confirmed entry signature from the `calculator.os3kapp` raw decompile:

```c
void SmartAppletEntryPoint(uint command, uint *call_block, uint *status_out)
```

Recovered top-level behavior:

- `*status_out` is cleared to `0` on entry
- lifecycle command `0x18` initializes applet state
- lifecycle command `0x19` shuts the applet down
- shutdown writes final status `7`
- other zero-valued lifecycle-range commands fall through as no-ops
- non-lifecycle commands are forwarded into the applet-specific command dispatcher

The current `calculator.os3kapp` decompile is now readable enough to treat that as proven rather than guessed:

- `SmartAppletEntryPoint`
- `InitializeCalculatorAppletState`
- `DispatchCalculatorAppletCommand`
- `ShutdownCalculatorAppletState`

Recovered call-block layout from the raw 68k stack setup used by the entry stub:

- `call_block[0]`: input length / input element count
- `call_block[1]`: input pointer
- `call_block[2]`: output capacity / minimum required output space
- `call_block[3]`: output length written back by the applet
- `call_block[4]`: output buffer pointer

That layout is visible directly in the 68k call sequence:

- `move.l 0x10(a4), -(a7)` pushes `call_block[4]`
- `pea 0x0c(a4)` pushes `&call_block[3]`
- `move.l 0x08(a4), -(a7)` pushes `call_block[2]`
- `move.l a3, -(a7)` pushes `status_out`
- `move.l 0x04(a4), -(a7)` pushes `call_block[1]`
- `move.l (a4), -(a7)` pushes `call_block[0]`
- `move.l d7, -(a7)` pushes the 32-bit command word

That layout is now modeled in the PoC runtime helper:

- [os3kapp_runtime.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/os3kapp_runtime.py)

Useful commands:

```bash
uv run --project poc/neotools python -m neotools os3kapp-entry-abi "<full .OS3KApp hex>"
uv run --project poc/neotools python -m neotools os3kapp-command 0x12040000
```

### Custom Command Encoding

The lifecycle commands are plain low-valued integers (`0x18`, `0x19`), but the applet-specific commands use additional namespace bytes inside the 32-bit command word.

Cross-checking the `alphaquiz.os3kapp` raw dispatcher shows:

- top byte `0x00`: lifecycle path
- top byte nonzero: custom-dispatch path
- inside the custom path, `alphaquiz` extracts selector byte `((command & 0x00ff0000) >> 16)` and branches on values `4`, `5`, and `6`

The `calculator.os3kapp` custom dispatcher decompiles as if it compares command values `1` and `2` directly. That does not line up cleanly with the outer top-byte gate, so the safest current interpretation is:

- lifecycle dispatch is universal and proven
- the exact custom-command selector extraction is applet-specific
- at least one applet (`alphaquiz`) uses the second-highest command byte as the selector namespace
- `calculator` really does compare literal low command values `1` and `2` after the entry stub hands off into `DispatchCalculatorAppletCommand`

This means the safe authoring rule for a custom SmartApplet is:

- the universal part of the ABI is the entry signature plus lifecycle opcodes `0x18` and `0x19`
- custom command numbers are private to the applet payload and may be encoded however that payload expects
- a minimal custom SmartApplet can therefore start with only lifecycle handling and no custom commands at all

## Runtime Trap ABI

The raw 68k payloads import host/runtime services by embedding Motorola A-line trap opcodes directly into the image. For `calculator.os3kapp`, the PoC now scans those imports and reports the real dense trap blocks:

- `0x34ce..0x34ee`: `0xa000..0xa03c`
- `0x34f0..0x3640`: `0xa06c..0xa308`
- `0x3640..0x3684`: `0xa32c..0xa3b0`

So the imported runtime surface is not a small hand-written jump table. It is a large contiguous family of host services spanning at least A-line families `0xa0`, `0xa1`, `0xa2`, and `0xa3`.

Cross-sample check:

- `calculator.os3kapp` has three dense blocks ending at `0xa03c`, `0xa308`, and `0xa3b0`
- `alphaquiz.os3kapp` has the same three-block pattern with the same first/last trap opcodes
- `spellcheck_small_usa.os3kapp` again starts with the same `0xa000..0xa03c` and `0xa06c..0xa308` ranges, then extends the final block past `0xa3b0`
- `neofontmedium.os3kapp` has no dense imported trap blocks and also uses `entry_offset = 0x94`

That makes two authoring patterns visible:

- UI-heavy / logic-heavy SmartApplets import a substantial shared runtime through dense A-line trap tables
- very small payloads can be self-contained and omit those imports entirely

Confirmed or strongly inferred shared trap meanings from the currently renamed call sites:

- `0xa000`: clear text screen
- `0xa004`: set text row / column / width
- `0xa010`: draw a predefined glyph or marker id
- `0xa014`: draw a C-string at the current text position
- `0xa094` `TrapA094`: read key code
- `0xa09c`: test whether a key / event is ready
- `0xa0a4`: pump pending UI events
- `0xa25c`: yield while waiting for the next event
- `0xa1f8`: begin chooser row-builder session
- `0xa1fc`: advance or commit the current output row
- `0xa200`: append the current chooser row
- `0xa204`: highlight the current chooser row
- `0xa208`: begin chooser input session
- `0xa20c`: read chooser event code
- `0xa210`: read chooser action selector
- `0xa214`: read chooser selection value

Evidence for those names:

- `RunCalculatorFunctionMenu` uses `TrapA000`, `TrapA004`, `TrapA010`, `TrapA014`, `TrapA094`, `TrapA09C`, `TrapA0A4`, and `TrapA25C` in a menu redraw + key polling loop
- `RunAlphaWordChooserDialog` and several other AlphaWordPlus chooser loops use the stable `0xa1f8 -> row text helpers -> 0xa1fc -> 0xa200 -> optional 0xa204 -> 0xa208 -> 0xa20c/0xa210/0xa214` sequence
- `CountNewlineDrivenRowAdvances` in AlphaWordPlus scans newline-delimited text and calls `0xa1fc`; the nonzero return path acts like a stop/overflow condition, which is why `0xa1fc` is now treated as a row-advance helper rather than an object resolver

Recovered call shapes from raw 68k in `RunCalculatorFunctionMenu`:

- `0xa000`: no explicit stack arguments
- `0xa004`: three pushed arguments, strongly inferred as `(row, column, width)`
- `0xa010`: one pushed small integer argument, acting like a glyph or marker id
- `0xa014`: one pushed pointer argument, acting like a C-string pointer
- `0xa094`, `0xa09c`, `0xa0a4`, `0xa25c`: no explicit stack arguments in the observed calculator menu loop

The `0xa004` call evidence is especially useful for custom applet work. The calculator menu loop emits calls equivalent to:

- `set_text_row_column_width(2 + item_index, 1, 12)` before drawing menu rows
- `set_text_row_column_width(2, 40, 12)` before drawing the selection marker column

That makes `0xa004` the first concrete host layout primitive we can describe beyond a trap number.

The PoC now carries those prototype fragments explicitly:

```bash
uv run --project poc/neotools python -m neotools os3kapp-trap-prototype 0xa004
```

Current prototype table encoded in the PoC:

- `0xa000` `clear_text_screen`: `stack_argument_count = 0`, no return value used
- `0xa004` `set_text_row_column_width`: `stack_argument_count = 3`, no return value used
- `0xa008` `get_text_row_col`: `stack_argument_count = 2`, no return value used; writes two byte-sized coordinate-like outputs
- `0xa010` `draw_predefined_glyph`: `stack_argument_count = 1`, no return value used
- `0xa014` `draw_c_string_at_current_position`: `stack_argument_count = 1`, no return value used
- `0xa020` `prepare_text_row_span`: `stack_argument_count = 3`, no return value used; prepares a row/column/width text region before redraw work
- `0xa094` `read_key_code`: `stack_argument_count = 0`, return value used as key code
- `0xa09c` `is_key_ready`: `stack_argument_count = 0`, return value used as readiness flag
- `0xa0a4` `pump_ui_events`: `stack_argument_count = 0`, no return value used
- `0xa0d4` `delay_ticks`: `stack_argument_count = 1`, no return value used; argument behaves like a pacing or timeout value
- `0xa190` `begin_output_builder`: `stack_argument_count = 3`, no return value used
- `0xa198` `append_output_bytes`: `stack_argument_count = 4`, no return value used
- `0xa1b4` `query_numeric_state`: `stack_argument_count = 1`, return value used as a scalar runtime value
- `0xa1c8` `query_object_metric`: `stack_argument_count = 2`, return value used as a scalar property query on a previously resolved runtime object or token
- `0xa1f8` `begin_chooser_row_builder`: `stack_argument_count = 0`, no return value used
- `0xa1fc` `advance_current_output_row`: `stack_argument_count = 2`, return value used as a row-advance success or stop/overflow result
- `0xa25c` `yield_until_event`: `stack_argument_count = 0`, no return value used
- `0xa364` `query_active_service_available`: `stack_argument_count = 0`, scalar/state return; used as a feature-availability query before beamer, wireless-transfer, and spell-check flows
- `0xa36c` `query_active_service_status`: `stack_argument_count = 0`, scalar/state return; used after feature-specific setup and best understood as a current-service status/session query rather than a single feature-specific boolean
- `0xa368` `shared_runtime_a368`: shared A3xx helper, still unresolved
- `0xa388` `query_active_service_disabled_state`: `stack_argument_count = 0`, scalar/state return; in AlphaWordPlus spell-check toggles, zero means enabled and nonzero means turned off
- `0xa38c` `shared_runtime_a38c`: shared A3xx helper, still unresolved
- `0xa378` `shared_runtime_a378`: shared across calculator and alphaquiz, but still unresolved beyond “common A3xx helper”
- `0xa390` `shared_runtime_a390`: shared across calculator and alphaquiz, returns a scalar or pointer-like value from at least one explicit argument
- `0xa39c` `shared_runtime_a39c`: shared across calculator and alphaquiz, side-effect helper likely related to copy/unpack behavior but not pinned further

New concrete `alphaquiz` evidence for the `0xa190..0xa1fc` family:

- `AppendCStringToAlphaQuizOutputBuffer` first computes the string length, then calls `0xa190`, then `0xa198`
- the call shape is stable: `0xa190(id, 0, 2)` followed by `0xa198(id, src, len, 1)`
- a second helper in `alphaquiz` uses `0xa190(0, 0, 2)` once, then repeatedly calls `0xa198(0, &byte, 1, 1)` for each character in a NUL-terminated string
- `0xa1fc(0x0d, 0)` in earlier `alphaquiz` work was initially interpreted as a selector/object lookup, but AlphaWordPlus now shows that the stronger cross-sample interpretation is a shared row-builder helper instead
- `0xa1b4(key)` returns a scalar value that is compared numerically and also forwarded into nearby update/finalize traps
- the safer current reading is:
  - `0xa1b4` and `0xa1c8` are still scalar query helpers
  - `0xa1fc` should no longer be treated as a proven object resolver for custom-app authoring purposes

These are still behavioral names, not fully proven SDK names, but they are now specific enough to help build higher-level tooling around text-output and handle/state queries.

## AlphaWordPlus Runtime Evidence

`alphawordplus.os3kapp` is useful because it is much smaller than `alphaquiz` or the spellcheck applets and still exercises a compact but nontrivial runtime/UI surface. Its payload prefix is:

- `(0x94, 0, 1, 2)`

So the main entry logic starts immediately after the 16-byte payload prefix, with no loader stub.

AlphaWordPlus also gives the clearest evidence so far for the chooser/menu-oriented `0xa200..0xa214` runtime family. The applet builds lists of AlphaWord rows, enters a chooser session, then reads back event codes and selection values.

Confirmed AlphaWordPlus-local support functions now renamed in Ghidra:

- `LookupAlphaWordPlusString`
- `RunAlphaWordChooserDialog`
- `HandleAlphaWordSlotSelection`

Chooser/session trap evidence from AlphaWordPlus:

- `0xa200` `append_current_chooser_row`
  - called immediately after a row has been assembled from string/data helpers
  - repeated once per list row before the chooser becomes interactive
- `0xa204` `highlight_current_chooser_row`
  - called when the just-appended row matches the currently selected slot
- `0xa208` `begin_chooser_input_session`
  - called once after chooser rows are prepared and just before input/event reads begin
- `0xa20c` `read_chooser_event_code`
  - returns command bytes such as `H`, `@`, `0x18`, `0x17`, `0x1c`, or applet-specific action codes
- `0xa210` `read_chooser_action_selector`
  - used after `@`-style activation events to identify the selected row/action slot
- `0xa214` `read_chooser_selection_value`
  - returns a 16-bit selection value consumed after chooser completion

AlphaWordPlus also strengthens the earlier `0xa1cc` interpretation:

- `0xa1cc` `commit_editable_buffer`
  - AlphaWordPlus uses the sequence `BeginEditableBufferEdit -> GetEditableBufferPointer -> mutate -> CommitEditableBuffer`
  - that is now strong enough to treat `0xa1cc` as the end of an editable-buffer transaction rather than a generic query

What AlphaWordPlus closes further:

- the `0xa1f8 -> 0xa1fc -> 0xa200 -> 0xa204 -> 0xa208` sequence is now best treated as a chooser/list-row builder API
- `0xa1fc` is no longer safe to document as an object resolver; the shared map now uses the more conservative row-advance name instead
- `0xa364` and `0xa36c` are shared current-service queries rather than purely spell-check helpers
- `0xa388` is now tied more tightly to the spell-check enable/disable path

AlphaWordPlus also exposes a richer applet-private command ABI layered on top of the universal SmartApplet entry contract. The currently pinned custom namespace handlers are:

- `HandleAlphaWordNamespace1Commands`
- `HandleAlphaWordNamespace2Commands`
- `HandleAlphaWordNamespace4Commands`
- `HandleAlphaWordNamespace7Commands`

Recovered high-confidence AlphaWordPlus command contracts:

- `0x10001`
  - handled by `HandleAlphaWordNamespace1Commands`
  - resets command-stream state through `ResetAlphaWordCommandStreamState`
  - sets status `0x11`
- `0x10004`
  - handled by `HandleAlphaWordNamespace1Commands`
  - returns one byte containing the current selected slot number `1..8`
- `0x20001`
  - handled by `HandleAlphaWordNamespace2Commands`
  - resets the namespace-2 import/export stream state
  - sets status `0x11`
- `0x20002`
  - handled by `HandleAlphaWordNamespace2Commands`
  - accepts incoming transferred bytes
  - decodes them through the transferred-byte tables
  - terminates on in-band byte `0xbb`
- `0x40001`
  - handled by `HandleAlphaWordNamespace4Commands`
  - resets the namespace-4 command stream
  - sets status `0x11`
- `0x40002`
  - handled by `HandleAlphaWordNamespace4Commands`
  - routes into `HandleAlphaWordNamespace4PayloadCommand`
  - the payload helper either emits an immediate error reply byte or starts/continues a streamed response
- `0x70001`
  - handled by `HandleAlphaWordNamespace7Commands`
  - resets the namespace-7 command stream
  - sets status `0x11`
- `0x70002`
  - handled by `HandleAlphaWordNamespace7Commands`
  - routes namespace-7 payload bytes through the same encoded transfer machinery used by the other AlphaWordPlus stream handlers

Supporting AlphaWordPlus-local transfer helpers now pinned from raw 68k:

- `ResetAlphaWordCommandStreamState`
- `DecodeAlphaWordTransferredByte`
- `WriteEncodedAlphaWordTransferredByte`
- `AppendEncodedAlphaWordTransferredByte`
- `RebuildAlphaWordTransferPointersFromCurrentFile`

Additional AlphaWordPlus-local helpers that are now clear enough to name:

- `RefreshAlphaWordSlotHandleCache`
  - clears the cached 8-slot handle table
  - enumerates or creates per-slot handles through `0x12e66/0x12e6e`
  - restores the current active slot through `EnsureAlphaWordSlotHandleSelected`
- `EnsureAlphaWordSlotHandleSelected`
  - lazily creates a slot handle when the cached entry is `-1`
  - switches the active slot if needed and replays the per-slot selection/update path
- `PromptForUniqueAlphaWordName`
  - prompts for a new AlphaWord file name
  - rejects duplicates by comparing against the existing slot-name list
- `DecodeAlphaWordMarkupTagKind`
  - classifies inline markup tokens such as `INPUT`, `NOBR`, `FORM`, and `FORMDATA`
- `NormalizeAlphaWordMarkupAndLineEndings`
  - walks the current AlphaWord text buffer
  - rewrites or strips recognized markup constructs
  - normalizes LF/CRLF into the applet's expected CR-oriented line layout
  - expands bracket-style markup shorthands into longer inline sequences
- `ShowAlphaWordHelpTopicsMenu`
  - displays the topic chooser headed by `Select help topic:           (esc=exit)`
  - the visible topics are `Features`, `Settings/Status`, `Navigating`, `Editing`, `International`, and `Symbols/Greek`
  - after selection it opens the corresponding long-form help text through the standard modal text viewer path
- `PromptOverwriteCurrentFileWithRomTestFile`
  - displays the `Are you sure you want to overwrite this file with the ROM test file?` prompt
  - if accepted it performs a fixed `0x100`-iteration reset/write loop through the lower AlphaWord file primitives
- `RunAlphaWordSectionSeparatorSettings`
  - controls the per-file section-break mode stored in the AlphaWord workspace state
  - the UI strings show this is specifically the `Change section separator from ... to ... ? (y/n)` workflow
  - the current mode is a count-like blank-line separator setting rather than a generic boolean
  - `SetDefaultSectionSeparatorMode` seeds that mode byte to `1` during AlphaWordPlus-local initialization
  - `BuildVisibleSectionPreviewWindow`, `ShiftSectionPreviewWindowBackward`, `ShiftSectionPreviewWindowForward`, `FindPreviousSectionBreak`, `FindNextSectionBreak`, `ReadSectionPreviewLine`, and `DecodeSectionSeparatorHotkey` are the local helper stack behind that screen
- `ShowAlphaWordFileSelectorPleaseWaitBanner`
  - is the tiny selector-page-specific `Please wait...` helper used by `RenderAlphaWordFileSelectorPage`
  - it is narrower than the generic `ShowPleaseWaitBanner` helper used by the section-separator settings workflow
- `ShowOpenFileCharacterTotals`
  - is the separate open-files aggregate screen headed by `Characters in open files (1000s)`
  - it is distinct from `ShowOpenAlphaWordFilesList`, which only prints the open-file slot list
- `SelectAlphaWordSlotWorkspace`
  - validates slot numbers `1..8`
  - switches the active AlphaWord slot when the requested slot is not already current
  - returns the per-slot workspace base used by the local helper stack
- `SelectAlphaWordSlotAndMaybePersistMap`
  - ensures the requested slot handle exists
  - switches the active slot byte in local AlphaWordPlus state
  - optionally persists the slot map when the caller requests it
- `InitializeAlphaWordLocalState`
  - is the AlphaWordPlus-local startup helper that seeds the default section-separator mode, sets the still-unresolved applet-local flag at `A5+0x21c`, and resets the Find/Replace state block
- `GetAlphaWordLineCountForSlot`
  - selects the requested slot workspace and returns its current line-count metric
- `IsValidAlphaWordSlotNumber`
  - is the narrow slot validator used before workspace switching
- `RunAlphaWordFileStatisticsWorkflow`
  - drives the summary/details UI reached from the AlphaWord file statistics path
  - `RenderAlphaWordFileSummaryScreen` renders the first page
  - `ShowAlphaWordFileStatisticsDialog` renders the expanded statistics/details page
  - `DecodeAlphaWordModalKey`, `NormalizeFileStatsDialogKey`, and `WaitForNormalizedFileStatsKey` normalize the chooser/dialog key loop around that workflow
- `RunFindReplaceWorkflow`
  - is the AlphaWordPlus-local Find/Replace controller
  - `ShowFindReplaceDialog` presents the multi-field `Find:` / `Replace with:` form
  - `InitializeFindReplaceFieldBuffers` seeds the editable field descriptors
  - `AdvanceFindReplaceDialogFieldFocus` handles tab/arrow focus movement between those fields
  - `ExecuteFindReplaceScan` performs the actual scan/update pass
  - `ConfirmFindReplaceOperation` is the prompt path for replace-all / capacity checks
  - `SetFindReplaceSelectionBounds` restores or updates the current selection bounds after the operation
  - `PromptFindReplaceMatchAction` is the interactive per-match chooser used during replace flows
  - `IsSpellCheckCandidateBoundary` is the boundary test reused while stepping candidate tokens
  - `ClassifySpellCheckTokenCapitalization` classifies the matched token casing before replacement text is generated
  - `ApplySpellCheckReplacementText` applies the replacement bytes and updates the tracked editor counts
  - `HasActiveAlphaWordSelection` gates the selection-aware branches
  - `GetAlphaWordSelectionAnchorLineIndex` plus `GetCurrentAlphaWordLineIndex` reconstruct the current selected line range for the active slot
  - `QueryAlphaWordSlotTransferEnd` is the slot-aware end/capacity helper used by replace-capacity checks

Additional AlphaWordPlus edit-state findings:

- `QueryCurrentAlphaWordEditorSpan`
  - is still the main current-span query used by namespace-2 and the statistics flows
  - current best interpretation remains a two-part editor span descriptor
- `QueryCurrentAlphaWordTransferCursor`
  - is distinct from the editor span helper and is reused heavily by spell-check, find/replace, and stream export paths
  - current best interpretation remains a transfer/export cursor or offset, not the whole file size
- `QueryAlphaWordSlotTransferEnd`
  - is slot-aware in raw disassembly
  - for a valid slot it looks up that slot handle and sums two per-slot queries
  - the fallback path does the same against a fixed default handle
  - current best interpretation is a slot-specific transfer end / capacity boundary rather than a generic current-file query

Helpers intentionally left unnamed for now:

Additional conservative AlphaWordPlus-local names from the final unnamed helper slice:

- `ShowAlphaWordRecoveryStatusAndFinalize`
  - prints either `Recovering "` plus the current file name or `Performing emergency recovery of`
  - then runs the same low-level finalize/close sequence used by slot/file-management paths
  - the exact underlying recovery primitive is still opaque, so the name intentionally stays at the status/finalize level
- `HandleAlphaWordQueuedWordBatch`
  - appends one or more 16-bit values into the small queue at `A5+0x13a`
  - dispatches special queued values through a local jump table
  - when more than one word is queued it reports output status `3`, calls the batch output primitive, and writes `count * 2` as the emitted size
  - this name is intentionally queue/batch-oriented because the deeper protocol meaning of those words is still not fully pinned
- `SetAlphaWordLocalInitFlag`
  - only sets the AlphaWordPlus-local byte flag at `A5+0x21c`
  - it is only called from the startup/local-init helper that also seeds the section-separator mode and resets Find/Replace state
- `SelectAlphaWordSlotAndPrimeCharStream`
  - selects the requested slot workspace, primes the low-level character stream, and performs one initial read-step
  - the decompiler collapses this into a very small wrapper, so the name stays at the observed slot-and-stream behavior level

## AlphaWordPlus Typing And Edit Command Path

AlphaWordPlus does not appear to handle normal document typing through the same trap-driven key loop used by chooser/help menus. The clearer path for live document editing is the applet-private command ABI.

The main write-side entrypoint is:

- `HandleAlphaWordNamespace2Commands`
  - this is the strongest current candidate for normal document typing and live edit mutation
  - on command `0x20002` it consumes incoming payload bytes, maps them through the encoding table at `DAT_0001488c`, and writes them with `WriteEncodedAlphaWordTransferredByte`
  - this is the clearest “incoming text bytes are being inserted into the current document” path

The paired read/control side is:

- `HandleAlphaWordNamespace1Commands`
  - initializes or resets the current AlphaWord command stream
  - selects slots and reports lengths or offsets
  - returns encoded file bytes through `DecodeAlphaWordTransferredByte`
  - this looks like the stream-control and readback half of the same document/channel ABI rather than the primary typing path

Additional stream-oriented command handlers:

- `HandleAlphaWordNamespace4Commands`
  - wraps `HandleAlphaWordNamespace4PayloadCommand`
  - provides another payload-command family with chunked return-data behavior
  - current evidence makes it look more like command/control and structured payload exchange than ordinary text entry
- `HandleAlphaWordNamespace7Commands`
  - handles another transfer/control family
  - includes slot selection, stream reset, pointer rebuild, and chunked return-data operations
  - again, this looks stream/control-oriented rather than the simplest live typing path

The key AlphaWordPlus-local primitives behind that document ABI are:

- `WriteEncodedAlphaWordTransferredByte`
  - write-side encoded byte insertion into the current AlphaWord document stream
- `DecodeAlphaWordTransferredByte`
  - read-side decoding of outgoing AlphaWord bytes
- `AppendEncodedAlphaWordTransferredByte`
  - helper for accumulating outbound encoded bytes
- `ResetAlphaWordCommandStreamState`
  - clears the active document-stream state machine before a new command session
- `RebuildAlphaWordTransferPointersFromCurrentFile`
  - resynchronizes the stream pointers against the current open AlphaWord file

Current best interpretation:

- menu/help/topic selection uses the shared runtime key/event traps
- actual document typing, file transfer, and edit/update work is mainly mediated through the AlphaWordPlus namespace command handlers
- among those, namespace `2` is the clearest write/mutate path for normal text insertion

### Namespace 2 Sub-ABI

The currently pinned namespace-2 command layer is:

- `0x20001`
  - initialize or reset the active AlphaWord command stream
  - calls `ResetAlphaWordCommandStreamState`
- `0x20002`
  - main write-side payload path
  - in normal mode it consumes incoming payload bytes, translates them through `DAT_0001488c`, and forwards them to `WriteEncodedAlphaWordTransferredByte`
  - this is the strongest current candidate for ordinary typed-character insertion into the current document
- `0x20006`
  - continue chunked readback when the local stream state is `3`
  - returns decoded bytes via `DAT_0001498c`
  - when the buffered readback is exhausted, it returns a one-byte `0xfe` terminator marker
- `0x2011f`
  - returns status code `1`
  - exact user-visible semantic still unresolved

The important namespace-2 payload/control bytes currently pinned inside `0x20002` are:

- `0xbb`
  - immediate write/session terminator on the incoming payload path
  - clears the local stream state and returns status code `0x0d`
- `0xbc`
  - enters local state `6`, meaning “interpret the next payload byte as a namespace-2 control selector”
- `0xbd`
  - immediate one-byte reply path returning `0xd4`
  - exact semantic still unresolved

When namespace-2 is in local state `6`, the next payload byte acts as a control selector:

- `0x01..0x08`
  - select AlphaWord slot `1..8`
  - handled through `EnsureAlphaWordSlotHandleSelected(slot - 1, ...)`
- `0x83`
  - `RebuildAlphaWordTransferPointersFromCurrentFile`
- `0x84`
  - begin chunked readback of the current file
  - loads a length through `FUN_00012ee8`, primes the char stream with `FUN_00012ed6`, sets local stream state `3`, and returns the first decoded chunk
- `0x87`
  - performs a size/limit check using `FUN_00012ee8` and `FUN_00012e4a`
  - on overflow/failure it sets status code `0x0c`
  - current best interpretation is “does the current editor span reach the end of the file yet”
- `0x88`
  - returns the current slot number plus one
- `0x90`
  - returns a 16-bit value derived from `FUN_00012ee8`
  - current best interpretation is the low 16-bit end position of the current editor span
- `0x91`
  - returns a 16-bit value from `FUN_00012e4a`
  - current best interpretation is a current file/document size query

The local namespace-2 stream-state byte at `A5+0x202` now has the following observed meanings:

- `0`
  - idle
- `3`
  - chunked readback active
- `5`
  - special handshake/readback mode used by namespace-1 `0x10016` / `0x1011e`
- `6`
  - namespace-2 control-selector mode entered by payload byte `0xbc`

So the current best write-side model is:

- host/app sends namespace-2 `0x20002` with encoded text bytes for normal insertion
- AlphaWordPlus maps those bytes through `DAT_0001488c`
- the mapped bytes are committed to the current document through `WriteEncodedAlphaWordTransferredByte`
- special bytes such as `0xbb` and `0xbc` escape out of plain text mode into stream/session control

Related host-backed size/range helpers now distinguished by caller behavior:

- `QueryCurrentAlphaWordFileSize`
  - behaves like full current document or file size
  - it is used as the cap/upper bound in statistics screens and in the namespace-2 `0x87` validation path
  - namespace-2 selector `0x91` returns this value directly
- `QueryCurrentAlphaWordTransferCursor`
  - behaves like a stream-position or available-length metric rather than the full file size
  - it participates in range arithmetic with `QueryCurrentAlphaWordTransferExtent`
  - it is also used directly by the namespace-1 and namespace-2 readback setup paths
- `QueryCurrentAlphaWordTransferExtent`
  - behaves like a companion range/span metric used with `FUN_00012e4c`
  - AlphaWordPlus combines the two when rebuilding or exporting transfer pointers
- `QueryCurrentAlphaWordEditorSpan`
  - returns a two-part range/value structure that feeds:
    - namespace-2 selector `0x84` readback setup
    - namespace-2 selector `0x87` validation
    - namespace-2 selector `0x90` 16-bit span-end query
  - current best interpretation is “current editor span descriptor” rather than a simple scalar

Current best interpretation of the still-narrow namespace-2 unknowns:

- payload byte `0xbd`
  - is an immediate one-byte reply command returning `0xd4`
  - it does not enter the slot/control-selector mode
  - the safest current reading is a small status or capability probe opcode
- selector `0x90`
  - reports the low 16-bit end position derived from `QueryCurrentAlphaWordEditorSpan`
- selector `0x87`
  - validates whether that same `QueryCurrentAlphaWordEditorSpan` already reaches the current full document size from `QueryCurrentAlphaWordFileSize`

### AlphaWordPlus Byte Translation Tables

The AlphaWordPlus payload contains two 256-byte translation tables:

- `DAT_0001488c`
  - write-side source-byte to internal-byte mapping
- `DAT_0001498c`
  - read-side internal-byte to output-byte mapping

The PoC now extracts and models them directly from `alphawordplus.os3kapp`.

What is now pinned:

- printable ASCII `0x20..0x7e` is identity in both directions
  - plain letters, digits, punctuation, and space are not remapped at all
- the non-identity region is concentrated in:
  - control bytes `0x00..0x1f`
  - high bytes `0x80..0xff`
- the two tables are not perfect inverses
  - current inverse-match count is `212 / 256` in both directions
  - the remaining `44` values are alias or reserved cases

Concrete write-side examples from `DAT_0001488c`:

- `0x00 -> 0xe7`
- `0x01 -> 0xfc`
- `0x02 -> 0xd6`
- `0x1d -> 0xac`
- `0x1e -> 0x00`
- `0x1f -> 0x00`
- `0xbc -> 0xb6`
- `0xbd -> 0xbd`
- `0xc0 -> 0xf8`

Concrete read-side examples from `DAT_0001498c`:

- `0x80 -> 0x19`
- `0x8c -> 0x04`
- `0x9d -> 0xfd`
- `0xb6 -> 0xbc`
- `0xbe -> 0x1c`
- `0xfc -> 0x01`

Important structural property:

- multiple source bytes collapse onto the same encoded byte on the write side
- for example, encoded `0x00` currently has these known source aliases:
  - `0x1e 0x1f 0xa9 0xaa 0xab 0xac 0xad 0xae 0xc1 0xd9 0xec 0xfa 0xfe 0xff`
- that aliasing is why the read table cannot be a full inverse of the write table

Current best interpretation:

- normal typing stays simple because printable ASCII is identity
- the tables mainly exist to carry:
  - AlphaWord-internal control bytes
  - document/stream command markers
  - non-ASCII or applet-specific symbols and glyph selectors
- bytes such as `0xbb`, `0xbc`, and nearby high-byte values should be treated as command-space markers rather than plain text

More detailed structure from the current table sweep:

- among high source bytes `0x80..0xff`:
  - `43` map into control-space outputs `< 0x20`
  - only `6` map into printable ASCII outputs
  - the remaining `79` map into high-byte outputs `>= 0x80`
- the printable-ASCII aliases from the high-byte region are currently:
  - `0xd3 -> 0x60` `` ` ``
  - `0xe7 -> 0x67` `'g'`
  - `0xea -> 0x6a` `'j'`
  - `0xf0 -> 0x70` `'p'`
  - `0xf1 -> 0x71` `'q'`
  - `0xf9 -> 0x79` `'y'`

That means the high-byte region is not “extended ASCII text” in any normal sense. It is mostly a compact command/glyph namespace that happens to include a few printable aliases.

Important command-space preservation examples:

- write-side:
  - `0xbc -> 0xb6`
  - `0xbd -> 0xbd`
  - `0xbf -> 0xbc`
  - `0xca -> 0xff`
- read-side:
  - `0xb6 -> 0xbc`
  - `0xbd -> 0xbd`
  - `0xbc -> 0xbf`
  - `0xff -> 0xca`

So the namespace command markers are not raw pass-through bytes. They are carried through the AlphaWordPlus channel in this translated high-byte domain and must be encoded/decoded through the tables to round-trip correctly.

The most useful clarification from the recent AlphaWordPlus pass is that the `0xa364/0xa36c/0xa388` family is shared and context-sensitive:

- beamer path:
  - `0xa364` behaves like "is this service installed/available?"
  - `0xa36c` behaves like "is this prepared beamer session currently usable?"
- wireless file transfer path:
  - the same pair drives the "not installed" vs "disabled in AlphaSmart Manager" messages
- spell-check path:
  - `0xa364` gates whether the Spell Check SmartApplet is present
  - `0xa388` gates whether spell check is turned off
  - `0xa36c` is used after spell-check setup and appears to enter or query the active spell-check session state

That is why the PoC now uses generic service-oriented names for those traps instead of earlier calculator-specific placeholders.

New cross-sample evidence for additional shared API surface:

- `0xa008` is used by both `calculator` and `alphaquiz` with two byte-pointer out-parameters, and the resulting bytes feed later text-layout calls
- `0xa020` is used by both applets with a stable 3-argument row/column/width-like shape immediately before redraw work
- `0xa0d4` is used by both applets with one timing-like argument such as `0x32` or `0xc8`, strongly suggesting a pacing primitive
- `0xa098` is shared across calculator, alphaquiz, and spellcheck, and consistently appears after UI/text batches with no explicit arguments; the safest current name is `flush_text_frame`
- raw-opcode scans across `calculator`, `alphaquiz`, and `spellcheck_large_usa` show that `0xa0f0` and `0xa0f4` are not applet-local oddities; they are part of the shared runtime and are heavily exercised by the spellcheck UI paths
- `0xa0f0` is still best described as `render_wrapped_text_block`: the observed call shape remains “text/source plus row/column/height/width-like layout arguments”
- `0xa0f4` remains a setup companion to `0xa0f0`; the safest current name is `define_text_layout_slot`
- `0xa0f8` is only exercised in the smaller calculator/alphaquiz UI loops, but the stable one-argument call shape still supports the provisional name `register_allowed_key`
- `0xa390` and `0xa39c` are no longer justified as calculator-only helpers because both are directly used by `alphaquiz` as well
- `spellcheck_large_usa` gives the clearest transaction pattern for `0xa1cc`: after a begin/edit phase exposes mutable buffer storage, `0xa1cc` closes that edit session and makes the changes visible again; the PoC now models it as `commit_editable_buffer`

What is still not safe to treat as generic runtime API:

- `0xa368`, `0xa36c`, and `0xa38c` do not yet have convincing cross-sample behavioral evidence beyond calculator-centric usage
- `0xa390` and `0xa39c` are shared, but still only safe as neutral placeholders rather than semantic SDK names
- `0xa200` and `0xa208` still do not have a defensible semantic name; raw opcode hits exist in the shipped images, but the recovered callsites are too sparse or poorly bounded to name them without overclaiming

What this means for custom SmartApplet authoring:

- a structurally valid `.os3kapp` does not need any trap imports at all
- a functional UI-heavy SmartApplet almost certainly does
- the trap words are embedded directly in the payload; there is no separate relocation/import table
- the first practical custom applet target should therefore be a minimal lifecycle-only applet that uses no traps, then incrementally add specific runtime services once their semantics are pinned

## AlphaQuiz Command ABI

`alphaquiz.os3kapp` is now the clearest sample of a nontrivial applet-specific command ABI layered on top of the universal entry contract.

Renamed handlers in Ghidra:

- `AlphaQuizEntryPoint`
- `LookupAlphaQuizString`
- `HandleAlphaQuizNamespace4Commands`
- `HandleAlphaQuizNamespace5Commands`
- `HandleAlphaQuizNamespace6Commands`
- `HandleAlphaQuizBytePayloadCommand`
- `AppendCStringToAlphaQuizOutputBuffer`

Validated entry dispatch:

- lifecycle `0x18` initializes state and iterates through the applet's quiz/session slots
- lifecycle `0x19` performs shutdown/cleanup and writes status `7`
- lifecycle `0x1a` performs an extra cleanup path and also ends with status `7`
- lifecycle `0x26` returns status `0xa001`
- selector byte `((command & 0x00ff0000) >> 16)` dispatches namespaces `4`, `5`, and `6`

Recovered applet-specific command families:

- namespace `4`
  - `0x40001`: returns status `0x11`, refreshes namespace-4 state, then redraws a three-line status block
  - `0x40002`: routes to `HandleAlphaQuizBytePayloadCommand`
  - `0x4000c`: side-effect-only cleanup/reset path
- namespace `5`
  - `0x50001`: returns status `0x11`, refreshes namespace-5 state, then redraws a two-line status block
  - `0x50002`: routes to `HandleAlphaQuizBytePayloadCommand`
  - `0x50005`: shares the same byte-payload helper as `0x50002`
  - `0x5000c`: side-effect-only cleanup/reset path
- namespace `6`
  - `0x60001`: returns status `0x11`, copies up to `0x27` input bytes into an applet-global title buffer, then NUL-terminates it
  - `0x6000d`: when two runtime selection values differ, returns status `4` and writes one 32-bit value to the output buffer
  - `0x60010`: sets redraw mode `2` and redraws the common namespace-6 three-line status block; status remains `0`
  - `0x60011`: sets redraw mode `1` and redraws the same three-line status block; status remains `0`
  - `0x60020`: only when input begins with ASCII `H`, shows a one-line prompt, waits, and sets status `8`; otherwise status remains `0`

`HandleAlphaQuizBytePayloadCommand` is especially useful because it shows a compact sub-protocol that operates on the first input byte and writes short response payloads:

- input byte `0x0a` -> immediate status `0x0f`
- input byte `0x1d` -> status `4`, clears the UI, and writes the fixed 2-byte reply `0x5d 0x02`
- input byte `0x1e` -> status `4` and writes the fixed 2-byte reply `0x5e 0x02`
- input byte `0x1a` -> delegates into a byte-decoder helper and only returns status `4` when the helper produced a nonzero reply length
- input byte `0x1b` -> delegates into a second byte-decoder helper and has the same conditional status/length behavior
- input byte `0x3f` -> immediate status `8`
- any other byte -> status `4` and a 1-byte reply of `(input | 0x80)`

This sub-protocol is shared by parent commands `0x40002`, `0x50002`, and `0x50005`. The only validated behavioral split so far is that the `0x1a` helper receives mode `0x3f` under the namespace-5 parent commands and mode `8` under the namespace-4 parent command.

The PoC now exposes that recovered command contract directly:

```bash
uv run --project poc/neotools python -m neotools os3kapp-applet-command alphaquiz 0x60001
uv run --project poc/neotools python -m neotools os3kapp-payload-subcommand alphaquiz 0x50002 0x1d
```

This does not yet solve the remaining ambiguous host trap semantics in the `0xa190..0xa1fc` family, but it does close an important part of the “complex applet” story: nontrivial applets can and do define private selector namespaces and byte-level payload contracts on top of the shared SmartApplet entry ABI.

The PoC now exposes this import surface directly:

```bash
uv run --project poc/neotools python -m neotools os3kapp-traps "<full .OS3KApp hex>"
```

That command prints the dense A-line trap blocks plus any currently known inferred names.

## Minimum Viable Custom SmartApplet

The current PoC can already synthesize a minimal custom `.os3kapp` image from scratch:

- valid `0xc0ffeead` SmartApplet header
- correct `file_size`
- correct base/extra memory fields
- payload prefix words `(entry_offset, 0, 1, 2)`
- a minimal 68k entry stub
- universal lifecycle handling for `0x18` and `0x19`
- no runtime trap imports

The generated minimal entry stub does exactly this:

1. load `status_out`
2. write `0` to `*status_out`
3. compare `command` with `0x19`
4. if equal, write `7` to `*status_out`
5. return

That is enough to produce a parseable, structurally valid SmartApplet image and proves that custom authoring is not blocked on the outer container format anymore. The remaining blocker for sophisticated custom applets is semantic coverage of the runtime trap services, not the container or entry ABI.

### Host Runtime Boundary

The applet payloads are not fully self-contained binaries. They call out to the NEO SmartApplet runtime through embedded Motorola 68k A-line trap opcodes.

Concrete evidence from `calculator.os3kapp`:

- file offset `0x365e..0x3682` contains a dense run of trap words:
  - `0xa368`, `0xa36c`, `0xa370`, `0xa374`, `0xa378`, `0xa37c`, `0xa380`, `0xa384`, `0xa388`, `0xa38c`, `0xa390`, `0xa394`, `0xa398`, `0xa39c`, `0xa3a0`, `0xa3a4`, `0xa3a8`, `0xa3ac`, `0xa3b0`
- callers branch or `jsr` through those trap stubs as if they were imported runtime services

Practical implication:

- we can now build structurally valid `.os3kapp` containers and the outer dispatcher ABI correctly
- a truly functional custom SmartApplet still requires mapping the semantics of the runtime trap services it uses
- a deliberately minimal applet may be possible with only a small subset of those services, but that subset is not fully named yet

## Retrieve Applet From Device

`UpdaterRetrieveApplet` is the device-to-host SmartApplet binary retrieval routine.

Start command:

- command `0x0f`
- `arg32 = 0`
- `trailing = applet_id`

For example, applet id `0xa123` becomes:

- `0f 00 00 00 00 a1 23 d3`

Expected retrieve responses:

- initial response `0x53`
- `arg32 = total_applet_size`
- repeated chunk pulls with command `0x10`
- each chunk response `0x4d`
- `arg32 = chunk_length`
- trailing field = 16-bit payload checksum

Read path:

1. send `0x0f`
2. expect `0x53`
3. loop sending `0x10`
4. expect `0x4d`
5. `read_data1(...)` drains the payload bytes
6. host writes those bytes to the destination file with `WriteRetrievedBytesToSink`
7. verify the 16-bit checksum for each chunk

The minimal direct USB session is therefore:

1. `?\xff\x00reset`
2. `?Swtch 0000`
3. `0x0f` retrieve-applet command
4. repeated `0x10` chunk requests until the announced size is satisfied

## Send Applet To Device

`UpdaterAddApplet` is the host-to-device SmartApplet install routine.

It first reads the first `0x84` bytes of the host `.OS3KApp` file and derives the add-applet start fields from that header.

What is confirmed:

- add-begin command `0x06`
- response must be `0x46`
- chunk handshake command `0x02`
- chunk handshake response must be `0x42`
- post-chunk completion poll command `0xff`
- completion response must be `0x43`
- program-applet command `0x0b`
- program response must be `0x47`
- finalize command `0x07`
- finalize response must be `0x48`
- payload chunk size is capped at `0x400`

The add-begin fields are now mapped precisely from the first `0x84` bytes of the `.OS3KApp` file:

- header offset `0x00`: big-endian magic `0xc0ffeead`
- header offset `0x04`: big-endian total SmartApplet file size
- header offset `0x08`: big-endian base memory size
- header offset `0x80`: big-endian extra memory size
- combined memory size used by `0x06` is `header[0x08:0x0c] + header[0x80:0x84]`
- the `0x06` `arg32` packs:
  - low 24 bits = total file size
  - high 8 bits = bits `16..23` of the combined memory size
- the `0x06` trailing field is the low 16 bits of the combined memory size

Equivalent expression from the recovered code path:

```text
combined_memory = be32(header[0x08:0x0c]) + be32(header[0x80:0x84])
arg32 = be32(header[0x04:0x08]) | ((combined_memory & 0xffff0000) << 8)
trailing = combined_memory & 0xffff
```

For the sampled files extracted from the installer:

- `calculator.os3kapp`: `file_size=0x00005fe0`, `combined_memory=0x0000056c`, so `0x06` starts with `arg32=0x00005fe0`, `trailing=0x056c`
- `alphawordplus.os3kapp`: `file_size=0x0001a0bc`, `combined_memory=0x00002d90`, so `0x06` starts with `arg32=0x0001a0bc`, `trailing=0x2d90`
- `keywordswireless.os3kapp`: `file_size=0x0002d350`, `combined_memory=0x00001d60`, so `0x06` starts with `arg32=0x0002d350`, `trailing=0x1d60`

Send path:

1. open the SmartApplet file on disk
2. read the first `0x84` bytes
3. derive the `0x06` argument and trailing fields from the header
4. send `0x06`
5. expect `0x46`
6. seek the file to the start and stream the full file in `0x400` chunks
7. for each chunk:
8. send `0x02` with `arg32 = chunk_length`, `trailing = sum16(chunk)`
9. expect `0x42`
10. write the chunk payload bytes
11. send `0xff`
12. expect `0x43`
13. after all bytes are staged, send `0x0b`
14. expect `0x47`
15. send `0x07`
16. expect `0x48`

## Save Retrieved Applet File Data

`UpdaterSaveAppletFileData` is the device-to-host saver for an applet's associated file data.

This is the device-to-host bulk saver for an applet’s associated file data:

1. `UpdaterGetAppletUsedFileSpace` discovers the total payload size and record count
2. host writes a 4-byte total-size prefix
3. for each file slot:
4. `UpdaterGetRawFileAttributes` retrieves the raw `0x28` file-attribute record
5. host writes the `0x28` record
6. file length is decoded from the attributes
7. `UpdaterRetrieveAppletFileData` retrieves the actual file bytes

This mirrors the AlphaWord file-data flow, but under the currently selected SmartApplet id.

## Embedded SmartApplet Info Table

Header offset `0x0c` is not another size field. It is an offset inside the `.OS3KApp` image to a variable-length info table that the UI uses for SmartApplet details, settings labels, and file-information strings.

Confirmed from the on-disk parser path:

- `ParseSmartAppletImageMetadata` reads header offset `0x0c`
- if nonzero, `ParseSmartAppletInfoTableAtOffset` walks a table at that file offset
- `ResolveSmartAppletInfoString` resolves strings from individual table records by a `(group, key)` pair

The record format is:

- `be16 group`
- `be16 key`
- `be16 payload_length`
- `payload[payload_length]`
- optional one-byte padding if `payload_length` is odd

The table terminates when the next `group` field is `0`.

The app also treats `group` as a typed record family when building the settings/file-info UI:

- `0x0101`: single 32-bit value record
- `0x0102`: three 32-bit values
- `0x0103`: list of 16-bit references to string records
- `0x0104`: inline text record
- `0x0105`: inline text record with alternate handling flag
- `0x0106`: inline text record used as a display-only string

Those are instantiated by `ParseSmartAppletSettingRecord` / `ParseSmartAppletTextRecord` after `WalkSmartAppletInfoTable` walks the info table.

Concrete AlphaWord Plus records at file offset `0x19fa4`:

- `(0x0001, 0x8002)` -> `Passwords Enabled`
- `(0x0001, 0x8003)` -> `Delete all files`
- `(0x0105, 0x100b)` -> `write`

Concrete KeyWords Wireless records at file offset `0x2d100`:

- `(0x0001, 0x8000)` -> `Delete all students`
- `(0x0001, 0x8001)` -> `Set custom WPM goals`
- `(0x0001, 0x8002)` -> `Set custom error goals`

`calculator.os3kapp` has header offset `0x0c = 0`, which matches the absence of an extra info table.

## Confirmed App-Side Call Chains

### Retrieve Applet List / Device Inventory

- `RefreshSmartAppletDeviceView`
- `LoadSmartAppletInventoryIntoUi`
- `QuerySmartAppletListAdapter`
- `DirectUsbGetAppletList` or `AlternateTransportGetAppletList`
- `UpdaterGetAppletList`

Meaning:

- `RefreshSmartAppletDeviceView` refreshes the SmartApplet/device view
- `LoadSmartAppletInventoryIntoUi` triggers the list retrieval and hands the normalized `0x84` entries to the UI layer
- `QuerySmartAppletListAdapter` is the concrete applet-list adapter over `UpdaterGetAppletList`

### Install SmartApplet On Device

- `InstallQueuedSmartAppletsToDevice`
- `InstallSmartAppletToTargets`
- `InstallSmartAppletWithRetry`
- `InstallOs3kOsImage`
- `InstallSmartAppletFromPath` or `InstallPreparedSmartApplet`
- `DirectUsbAddApplet` or `AlternateTransportAddApplet`
- `UpdaterAddApplet`

Meaning:

- `DirectUsbAddApplet` is the direct USB wrapper into `UpdaterAddApplet`
- `AlternateTransportAddApplet` is the alternate mode `4` wrapper into `UpdaterAddApplet`
- `InstallSmartAppletFromPath` and `InstallPreparedSmartApplet` open the host SmartApplet file and dispatch it to those wrappers
- `InstallSelectedSmartApplets` is now pinned as the top-level send/install workflow controller that eventually reaches `InstallQueuedSmartAppletsToDevice` and the SmartApplet install helpers
- the higher-level controller entry points above are the install-side callers currently confirmed by xrefs

### Retrieve SmartApplet To Host

- `RetrieveMissingSmartAppletsToWorkspace`
- `RetrieveOneSelectedSmartApplet`
- `RetrieveSmartAppletToHostFile`
- `DirectUsbRetrieveApplet` or `AlternateTransportRetrieveApplet`
- `UpdaterRetrieveApplet`

and also:

- `RetrieveChosenSmartAppletsToHost`
- `RetrieveSmartAppletToHostFile`
- `DirectUsbRetrieveApplet` or `AlternateTransportRetrieveApplet`
- `UpdaterRetrieveApplet`

Meaning:

- `RetrieveSmartAppletToHostFile` opens a host-side output file and retrieves one SmartApplet binary into it
- `RetrieveOneSelectedSmartApplet` retrieves one selected SmartApplet entry through that helper
- `RetrieveMissingSmartAppletsToWorkspace` iterates the SmartApplet send-list entries and retrieves missing applets to the host workspace
- `RetrieveChosenSmartAppletsToHost` is another controller path that can retrieve selected applets to host storage
- `RetrieveSelectedSmartApplets` is the higher-level retrieval controller that sets up the progress dialog, iterates selected devices, then calls `RetrieveChosenSmartAppletsToHost`

### Refresh SmartApplet Info / Details View

- `RefreshSmartAppletDetailsPane`
- `RefreshSmartAppletDeviceView`
- `LoadSmartAppletInventoryIntoUi`
- `QuerySmartAppletListAdapter`

Meaning:

- `RefreshSmartAppletDetailsPane` is the controller that clears and rebuilds the SmartApplet info/details pane
- it calls `RefreshSmartAppletDeviceView`, which refreshes the applet inventory and injects AlphaWord-specific preview text when applet id `0xa000` is present
- `LoadSmartAppletInventoryIntoUi` and `QuerySmartAppletListAdapter` provide the normalized `0x84` SmartApplet records that feed that view

## Resource Strings And Popup Menus

`r2` resource decoding and `ghydra` decompilation now pin the AlphaWord-specific SmartApplet labels and the three relevant popup menus.

Confirmed STRINGTABLE entries from resource block `3860`:

- `0xf138` = `Maximum File Size (in characters)`
- `0xf139` = `Minimum File Size (in characters)`
- `0xf13a` = `Updating AlphaWord file size limits.`

Those strings are consumed by the AlphaWord-specific SmartApplet settings code:

- `ParseSmartAppletSettingRecord` compares generated SmartApplet setting labels against `0xf138` and `0xf139` when applet id `0xa000` is active
- `UpdateAlphaWordMaxFileSizeSettingRow` uses `0xf138` to find and update the current AlphaWord maximum-size row
- `ValidateAlphaWordMaxFileSizeEdit` uses `0xf138` to validate the edited numeric value against the row-local min/max bounds at offsets `0x48` and `0x4c`

That closes the AlphaWord SmartApplet settings mapping: the SmartApplet info-table produces editable rows, and the AlphaWord-specialized UI logic recognizes two of those rows by their resource labels instead of by numeric record key alone.

Confirmed popup menus from `r2` MENU resources:

- resource `163`:
  - `0x800e` `Startup`
  - `0x800f` `Startup Lock`
  - `0x8010` `Remove`
  - `0x8012` `Get Info`
  - `0x8013` `Help`
- resource `208`:
  - `0x801d` `Startup`
  - `0x801e` `Startup Lock`
  - `0x801f` `Get Info`
  - `0x8020` `Help`
- resource `219`:
  - `0x8021` `Undo`
  - `0x8022` `Cut`
  - `0x8023` `Copy`
  - `0x8024` `Paste`
  - `0x8025` `Delete`
  - `0x8026` `Select All`

The remaining command-dispatch gap is narrower than before: the popup contents and ids are now decoded, but not every id has been tied to a uniquely named MFC handler.

## Minimal PoC Coverage

The offline PoC now models the confirmed packet layer in:

- `poc/neotools/src/neotools/smartapplets.py`

Covered operations:

- list applets command construction
- retrieve-applet command construction
- direct USB retrieve session planning
- SmartApplet header parsing
- full `.os3kapp` container parsing and rebuilding
- shared SmartApplet list-entry / header metadata parsing
- embedded SmartApplet info-table parsing
- typed info-table record-family classification
- extraction of the three currently proven `0x10` flag bits
- offline lookup of the confirmed AlphaWord SmartApplet size-limit resource labels
- offline lookup of the decoded SmartApplet popup-menu command maps
- `0x06` add-begin field derivation from a real `.OS3KApp` header
- add-applet begin command construction
- direct USB add session planning directly from a full SmartApplet image
- direct USB add session planning with:
  - `0x06`
  - `0x02`
  - `0xff`
  - `0x0b`
  - `0x07`

Useful commands:

```bash
uv run --project poc/neotools python -m neotools smartapplet-retrieve-plan 0xa123
uv run --project poc/neotools python -m neotools smartapplet-add-plan 0x12345678 0x9abc "41 42 43 44 45"
uv run --project poc/neotools python -m neotools smartapplet-header "<0x84-byte header hex>"
uv run --project poc/neotools python -m neotools smartapplet-metadata "<0x84-byte header hex>"
uv run --project poc/neotools python -m neotools os3kapp-image "<full .OS3KApp hex>"
uv run --project poc/neotools python -m neotools smartapplet-string 0xf138
uv run --project poc/neotools python -m neotools smartapplet-menu 163
uv run --project poc/neotools python -m neotools smartapplet-add-plan-from-image "<full .OS3KApp hex>"
```

## Remaining Unknowns

- the exact semantic names of the three proven flag bits in the flags dword at offset `0x10`
- the exact human-readable mapping behind every `applet_class` byte value at offset `0x3f`
- the final MFC command-handler binding for every decoded popup-menu id
