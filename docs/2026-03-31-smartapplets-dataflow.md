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

Live physical-device validation:

- `real-check applets` successfully sends this read-only list command against a direct-mode NEO.
- The tested NEO returned these parsed metadata entries without retrieving applet binaries:
  - `0x0000` System 3.15, size `401408`
  - `0xa000` AlphaWord Plus 3.4, size `106684`
  - `0xa004` KeyWords 3.6, size `126888`
  - `0xa007` Control Panel 1.0, size `27412`
  - `0xa006` Beamer 1.0, size `32580`
  - `0xa001` AlphaQuiz 1.0, size `49828`
  - `0xa002` Calculator 3.0, size `24544`
  - `0xa027` Text2Speech Updater 1.4, size `11460`
  - `0xaf00` Neo Font - Small (6 lines) 1.0, size `4164`
  - `0xaf75` Neo Font - Medium (5 lines) 1.0, size `4360`
  - `0xaf02` Neo Font - Large (4 lines) 1.0, size `4264`
  - `0xaf73` Neo Font - Very Large (3 lines) 1.0, size `9392`
  - `0xaf03` Neo Font - Extra Large (2 lines) 1.0, size `13152`
  - `0xa005` SpellCheck Large USA 1.0, size `357312`
  - `0xa017` Thesaurus Large USA 1.1, size `366796`

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
- `0x16`: SmartApplet header version, normally `1`
- `0x17`: file count declared by the applet, normally `0` for applets with no owned files
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

## Betawise Cross-Check

External reference: `https://github.com/isotherm/betawise/tree/master`.

Betawise independently confirms the core SmartApplet container model:

- SmartApplet files start with a `0x84`-byte header.
- Header magic is `0xc0ffeead`.
- Header offset `0x04` is total file/ROM usage.
- Header offset `0x08` is RAM usage.
- Header offset `0x0c` is settings/info-table offset.
- Header offset `0x10` is the packed flags dword.
- Header offset `0x14..0x15` is the applet id.
- Header offset `0x16` is header version, normally `1`.
- Header offset `0x17` is file count, normally `0` for applets that do not own files.
- Header offset `0x18..0x3b` is the applet name.
- Header offset `0x3c..0x3e` is the UI-visible version.
- Header offset `0x3f` is language id / applet class in the shipped-header view.
- Header offset `0x40..0x7f` is the info/copyright string.
- Header offset `0x80..0x83` is file usage / extra memory in the shipped-header view.
- Valid applet files end with footer `0xcafefeed`.

Betawise builds applets with `m68k-elf-gcc`, a linker script, and A-line trap
stubs. Its `APPLET_HEADER_BEGIN` macro places a C `AppletHeader_t` in a kept
header section and its linker script keeps the footer at the end.

Important layout correction: Betawise's full `AppletHeader_t` is `0x94` bytes,
not only `0x84` bytes. The first `0x84` bytes are the same metadata record that
NeoManager lists and parses. The next 16 bytes are the entry pointer and fixed
ABI markers:

- `0x84..0x87`: `entryPoint`, normally `&BwProcessMessage`
- `0x88..0x8b`: marker dword `0`
- `0x8c..0x8f`: marker dword `1`
- `0x90..0x93`: marker dword `2`

That is exactly the layout this project had already inferred as a `0x84`-byte
metadata header followed by a four-dword payload prefix. Betawise therefore
confirms the generated-applet split:

```text
0x0000..0x0083  metadata parsed by NeoManager and the device list command
0x0084..0x0093  entry pointer plus 0, 1, 2 ABI markers
0x0094..        executable 68k code and data
EOF-4..EOF      ca fe fe ed footer
```

The Betawise linker script forces this ordering by keeping `os3k_header`, then
`BwProcessMessage`, then the remaining text/rodata, then `os3k_footer`.
`romUsage` is the linked binary size including header and footer. `ramUsage` is
the linked `.bss` size, not the size of the ROM payload.

Important correction from this cross-check: our early generator incorrectly
packed the applet version into header bytes `0x16..0x17`. That made `Alpha USB`
v1.20 declare `file_count=0x20`, even though the real UI version also lived at
`0x3c..0x3d`. The device accepted that image and Android backup worked, but it
was not structurally correct and could plausibly stress file/catalog handling.

The generator now emits:

```text
offset 0x14..0x17 = a1 30 01 00
```

for `Alpha USB` `0xa130`, meaning applet id `0xa130`, header version `1`, file
count `0`. The visible version remains:

```text
offset 0x3c..0x3f = 01 20 00 01
```

meaning version `1.20`, build byte `0`, language/class byte `1`.

Observed examples:

- `alphawordplus.os3kapp`: applet id `0xa000`, version `3.4`, class `0x01`
- `calculator.os3kapp`: applet id `0xa002`, version `3.0`, class `0x01`
- `keywordswireless.os3kapp`: applet id `0xa004`, version `4.0`, class `0x01`

## Betawise ABI, Stubs, And Callback Model

Betawise's public applet callback ABI is:

```c
void ProcessMessage(Message_e message, uint32_t param, uint32_t *status);
```

Compiled Betawise applets usually export a user `ProcessMessage`. The library
entry `BwProcessMessage` performs common setup for a few lifecycle messages and
then calls the user callback. Minimal or special applets can override
`BwProcessMessage` directly, as `NeoFontTerminal` does for font-private
messages.

Betawise message constants line up with the live NEO behavior we observed:

- `0x00`: `MSG_IDLE`
- `0x18`: `MSG_INIT`
- `0x19`: `MSG_SETFOCUS`, the menu-open/screen ownership path used by the
  working custom drawing probes
- `0x1a`: `MSG_KILLFOCUS`
- `0x20`: `MSG_CHAR`
- `0x21`: `MSG_KEY`
- `0x30001`: `MSG_USB_PLUG`, the USB attach callback used by `Alpha USB`
- `0x3000c`: `MSG_USB_UNPLUG`
- `0x30007`, `0x3001e`, `0x3001f`: other USB-family events with unknown exact
  meanings
- `0x1000000`: synthetic/private-message modifier namespace

Operational rules from Betawise plus live tests:

- Clear or set `*status` on entry. Most Betawise applets start by writing
  `*status = 0`.
- Draw normal applet UI from `MSG_SETFOCUS`, not from USB attach callbacks.
- Treat `MSG_USB_PLUG` (`0x30001`) as dispatcher-owned context. The stable
  `Alpha USB` path performs only the proven direct-mode switch sequence, writes
  the expected status, and returns immediately.
- Private applet messages are normal. The font applet `0xa1f0` answers
  `0x1000001..0x1000006`; the shared wrapper asks it for a font pointer through
  `AppletFindById(0xa1f0)` plus `AppletSendMessage(..., 0x1000002, ...)`.

Betawise's syscall layer is also direct and simple. `os3k/syscall.c` emits one
function per OS service, and each function body is a single Motorola 68k A-line
trap word:

```c
DEFINE_SYSCALL(index, name) => .word 0xA000 + 4 * index
```

Useful confirmed stubs:

- index `0`, trap `0xa000`: clear screen
- index `1`, trap `0xa004`: set cursor
- index `3`, trap `0xa00c`: centered string
- index `4`, trap `0xa010`: put character
- index `5`, trap `0xa014`: put raw string
- index `6`, trap `0xa018`: set cursor mode
- index `12`, trap `0xa030`: put string
- index `18`, trap `0xa048`: draw bitmap
- index `19`, trap `0xa04c`: raster operation
- index `34`, trap `0xa088`: wait for key
- index `37`, trap `0xa094`: get key
- index `39`, trap `0xa09c`: key/event ready
- index `41`, trap `0xa0a4`: scan keyboard / pump events
- index `53`, trap `0xa0d4`: sleep centiseconds
- index `54`, trap `0xa0d8`: sleep centimilliseconds
- index `102`, trap `0xa198`: file write buffer
- index `103`, trap `0xa19c`: file read buffer
- index `142`, trap `0xa238`: find applet by name
- index `143`, trap `0xa23c`: find applet by id
- index `144`, trap `0xa240`: get applet name
- index `145`, trap `0xa244`: send applet message
- index `151`, trap `0xa25c`: System dispatcher/yield helper
- index `174`, trap `0xa2b8`: internal system-info call

The Betawise build flags are part of the ABI contract:

- `-mpcrel` and assembler `--pcrel`: generated code must be position-relative.
- `-ffixed-a5`: do not let compiled code use A5 as a general register. Live
  experiments already showed AlphaWord-private A5-relative writes are unsafe
  outside AlphaWord.
- `-ffixed-d7`: reserve D7 for the OS/runtime dispatcher.
- `-fshort-enums`: enum argument size follows the firmware's expectations.

This explains why the hand-built applets in this repo work when they call local
BSR stubs containing A-line words and avoid clobbering OS-owned registers. It
also explains why future C-compiled applets should either use Betawise's flags
and linker script directly or replicate those constraints exactly.

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

Exact AlphaWordPlus trap-stub import table:

- a raw byte scan of `alphawordplus.os3kapp` at file offset `0x12d88` shows a contiguous A-line import block beginning `a008 a00c a010 a014 ...` and continuing through at least `a44c`
- that means the addresses `0x00012d88..0x00012f90` are direct runtime service stubs, not ordinary applet-local helper functions
- a few earlier behavior-level names at those addresses were therefore too local; the exact trap ids now take precedence

High-confidence trap-stub mappings from that block:

- `0x00012d88` `TrapA008_GetTextRowCol`
- `0x00012d8a` `TrapA00C_DrawPendingTextAndAdvanceRow`
  - conservative behavioral name; exact trap id is certain, detailed ABI still is not
- `0x00012d8c` `TrapA010_DrawPredefinedGlyph`
- `0x00012d8e` `TrapA014_DrawCStringAtCurrentPosition`
- `0x00012dbc` `TrapA094_ReadKeyCode`
- `0x00012dbe` `TrapA098_FlushTextFrame`
- `0x00012dc0` `TrapA09C_IsKeyReady`
- `0x00012dc8` `TrapA0A4_PumpUiEvents`
- `0x00012ddc` `TrapA0D4_DelayTicks`
- `0x00012e5c` `TrapA1D4_AssignCurrentFileNameFromPendingText`
- `0x00012e62` `TrapA1E0_QueryAdvancedFileIteratorOrdinal`
- `0x00012e68` `TrapA1EC_SyncCurrentSlotMapEntry`
- `0x00012e78` `TrapA20C_ReadChooserEventCode`
- `0x00012e7a` `TrapA210_ReadChooserActionSelector`
- `0x00012ed0` `TrapA2BC_CommitCurrentFileEditSession`
- `0x00012ed2` `TrapA2C0_FinalizeCurrentFileContext`
- `0x00012ed8` `TrapA2CC_BeginCurrentReplacement`
- `0x00012eda` `TrapA2D0_QueryCurrentReplacementStatus`
- `0x00012edc` `TrapA2D4_ResetCurrentSearchState`
- `0x00012ede` `TrapA2D8_ReadNextCharStreamUnit`
- `0x00012ee0` `TrapA2DC_SwitchToCurrentFileContext`
- `0x00012eea` `TrapA2EC_QueryCurrentWorkspaceFileStatus`
- `0x00012ef2` `TrapA2FC_InitializeEmptyWorkspaceFile`
- `0x00012f12` `TrapA36C_QueryActiveServiceStatus`
- `0x00012f1e` `TrapA378_RenderFormattedPendingText`
- `0x00012f22` `TrapA380_FormatPendingText`
- `0x00012f26` `TrapA388_QueryActiveServiceDisabledState`
- `0x00012f2a` `TrapA390_SharedRuntime`
- `0x00012f2e` `TrapA398_QueryPendingTextLength`
- `0x00012f30` `TrapA39C_SharedRuntime`

Remaining currently unresolved entries in that direct import table are still renamed structurally by exact trap id, for example:

- `TrapA018`, `TrapA01C`, `TrapA084`, `TrapA088`
- `TrapA0A0`, `TrapA0B4`, `TrapA0DC`, `TrapA0E0`
- `TrapA14C`, `TrapA158`, `TrapA15C`, `TrapA160`, `TrapA174`
- `TrapA1AC`, `TrapA1B0`, `TrapA1C0`, `TrapA1C4`, `TrapA1D8`, `TrapA1DC`, `TrapA1E4`
- `TrapA21C`, `TrapA224`, `TrapA22C`, `TrapA23C`, `TrapA244`, `TrapA248`, `TrapA250`, `TrapA260`, `TrapA26C`, `TrapA270`
- `TrapA2B0`, `TrapA2B4`, `TrapA2B8`, `TrapA2E0`, `TrapA2F0`, `TrapA32C`, `TrapA370`, `TrapA374`, `TrapA378_SharedRuntime`, `TrapA380`, `TrapA394`, `TrapA398`, `TrapA3AC`, `TrapA3B0`, `TrapA44C`

Current high-confidence interpretation of the newly named AlphaWordPlus file-workspace trap cluster:

- `TrapA1D4_AssignCurrentFileNameFromPendingText`
  - AlphaWordPlus calls this immediately after loading canned strings like `0xfb` into the pending text slot during create/load flows.
  - Best current reading: apply the pending text buffer as the current file name.
- `TrapA1E0_QueryAdvancedFileIteratorOrdinal`
  - Called only after `AdvanceAlphaWordFileIterator`.
  - Returns the resulting file ordinal, with nonpositive values meaning no available file slot.
- `TrapA1EC_SyncCurrentSlotMapEntry`
  - Paired around reads and writes of the eight-entry slot-to-file table in AlphaWordPlus local state.
  - Best current reading: sync the current slot-map entry between applet state and host/runtime state.
- `TrapA2BC_CommitCurrentFileEditSession`
  - Used after bulk overwrite/edit sequences such as the ROM test-file replacement path.
  - Best current reading: commit the active current-file edit session.
- `TrapA2C0_FinalizeCurrentFileContext`
  - Used at the end of create/load/prompt workflows and before some interactive prompts.
  - Best current reading: finalize or leave the current-file context/workspace binding.
- `TrapA2CC_BeginCurrentReplacement`
  - Used just before spell-check and clear-file replacement work, after the current cursor/selection location has been established.
- `TrapA2D0_QueryCurrentReplacementStatus`
  - Queried immediately after `TrapA2CC`.
  - Best current reading: return replacement-result status, including the empty/no-op case.
- `TrapA2D4_ResetCurrentSearchState`
  - Used around search/find/replace prompts and after one-character lookahead checks.
  - Best current reading: clear or reset the current search/match state.
- `TrapA2D8_ReadNextCharStreamUnit`
  - Low-level char-stream iterator primitive underneath the higher-level AlphaWordPlus read/preview/search helpers.
- `TrapA2DC_SwitchToCurrentFileContext`
  - Used before destructive or content-sensitive operations like delete, overwrite, search, and file loading.
  - Best current reading: switch/bind the active current-file workspace.
- `TrapA2EC_QueryCurrentWorkspaceFileStatus`
  - Queried after selecting or naming the current file.
  - Zero triggers the empty-file initialization path.
- `TrapA2FC_InitializeEmptyWorkspaceFile`
  - Called only when `TrapA2EC_QueryCurrentWorkspaceFileStatus` reports the zero/empty state for the active file.
  - Best current reading: initialize default/empty content for the current workspace file.
- `TrapA378_RenderFormattedPendingText`
  - Used across confirm/cancel, file-details/statistics, and queued-entry dialogs.
  - Best current reading: render a formatted pending text/template directly from the current stacked arguments.
- `TrapA380_FormatPendingText`
  - Used before wrapped-dialog display and before subsequent pending-text length queries.
  - Best current reading: build or format the current pending text buffer without drawing it immediately.
- `TrapA398_QueryPendingTextLength`
  - Used to size wrapped dialogs and chooser-row preview buffers.
  - Best current reading: return the current pending/formatted text length.
- `TrapA14C_ReadTextInputChar`
  - Used by `HandleSingleLineTextFieldInput` and the numeric live-dialog workflow.
  - Best current reading: return the next typed input character when text-entry mode is active.

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

Live custom-applet validation tightened the drawing ABI further:

- the first visible custom applet is `USB Menu Probe`, applet id `0xa129`, version `1.8`
- selecting it from the Applets menu invokes the entry point with command `0x19`
- a direct inline `A014` C-string draw probe opened a blank, returnable applet screen
- a Calculator-style call pattern works: call local `bsr.w` stubs that contain the A-line trap opcodes, rather than embedding the traps inline in the drawing routine
- the working probe draws `USB` by clearing the text screen with `A000`, setting `(row=2, column=1, width=28)` with `A004`, drawing ASCII codes `0x55`, `0x53`, `0x42` through `A010`, flushing with `A098`, then idling through `A25C`
- the local stub table at the end of the generated payload is exactly `a000 a004 a010 a098 a25c`

The current authoring rule for minimal visible applets is therefore conservative:

- dispatch command `0x19` as the menu-open command for custom applets
- draw short status text with `A010` character-by-character through local `bsr.w` trap stubs
- flush with `A098`
- keep the applet returnable by idling in `A25C` instead of spinning without the runtime yield helper

### AlphaWord USB/Event Handler

AlphaWord Plus also shows that USB/external-transfer screens are not reached from the same branch as a normal Applets-menu launch.

The low command/event table in AlphaWord maps these lifecycle/event commands:

- `0x19` -> normal interactive menu open
- `0x20` -> applet event/dispatch path
- `0x21` -> applet event/dispatch path
- `0x26` -> identity query; AlphaWord writes status `0xa000`

The external-transfer UI helper `HandleAlphaWordExternalTransferRequest` is called from inside the `0x21` event path after the runtime has decoded event words. That helper receives a word whose high bits identify transfer class:

- `(event & 0xfc00) == 0x8000`: local/current-file transfer prompt path
- `(event & 0xfc00) == 0x4000`: wireless/alternate external-transfer prompt path
- anything else: writes status `0x0b` and returns

This explains why AlphaWord can show different text when a USB/external-transfer event arrives than when AlphaWord is opened from the applet menu: the visible applet entry is `0x19`, while the transfer UI is reached later through event commands and a decoded high-bit event word.

The custom `USB Menu Probe` version `1.8` mirrored this at the safest level available before the live USB attach observation:

- `0x19`: draws `USB` and idles with `A25C`
- `0x20` or `0x21`: draws `DIR`, calls the ROM direct-mode callback `0x00410b26`, then idles with `A25C`
- the callback path is intentionally only on the event-command side, not on normal menu open

The generated event-handler bytes disassemble as:

```asm
cmpi.l  #0x20,d0
beq.w   draw_direct
cmpi.l  #0x21,d0
beq.w   draw_direct
...
draw_direct:
  ; A000/A004/A010/A098 through local BSR trap stubs draws "DIR"
  jsr     0x00410b26
  ; A25C idle loop
```

Live result: with version `1.8` open, connecting USB still entered the stock `Attached to Mac/PC, emulating keyboard.` screen. The NEO did not draw `DIR`, so stock USB attach does not appear to dispatch custom applet `0x20`/`0x21` handlers the way AlphaWord reaches its external-transfer helper.

The next probe is `USB Menu Probe` version `1.9`. Its `0x19` menu-open path replicates the System lifecycle callback-registration side effects inline, then draws `ARM` and idles:

```asm
clr.l   -(a7)
jsr     0x00426bb0
move.b  #0,0x00000444
jsr     0x00412c82
jsr     0x004109ca
pea.l   0x00011111
pea.w   0x2580
jsr     0x00424fb0
pea.l   0x00410b26
jsr     0x00424f66
lea     16(a7),a7
```

This deliberately avoids `jsr 0x0041016e`: that System routine branches into its dispatcher epilogue and expects the System stack frame. Version `1.9` only copies the registration work, then stays inside the custom applet runtime contract.

Further AlphaWord comparison showed that the keyboard-send path is not reached only by matching the later transfer command ids. AlphaWord advertises the capability through its info table:

- `0x0105/0x100b = "write"`
- `0xc001/0x8011..0x8018 = "write"`

AlphaWord also accepts namespace initialization before transfer:

- `0x10001`: reset namespace 1 and return status `0x11`
- `0x20001`: reset namespace 2 and return status `0x11`

`USB Menu Probe` version `1.11` now mirrors those parts: it includes the AlphaWord-style write metadata, uses flags `0xff0000ce` and extra memory `0x2000`, draws `LINK` and returns `0x11` for namespace init, then draws `HOST` and returns `4` for the observed keyboard-send transfer command ids. This is the current probe for determining whether keyboard-send routing is metadata-based or hardcoded to AlphaWord applet id `0xa000`.

The current `1.11` build also mirrors the low command surface seen in AlphaWord's entry dispatcher:

- `0x20` / `0x21`: draw `HOST`, return status `4`
- `0x26`: return this probe's applet id (`0xa129`) through the status pointer

This matters because the previous metadata-only `1.11` build still reached the stock System `Attached to Mac/PC, emulating keyboard.` screen. The active hypothesis is now narrower: either System probes this low command surface before routing the attach UI, or the attach UI is hardcoded to AlphaWord/startup-app state rather than generic SmartApplet metadata.

NeoManager's shipped readme supports the second possibility: it warns that direct USB connection/update problems occur when the NEO is running an applet other than AlphaWord Plus or the SmartApplets menu, and when startup is not AlphaWord Plus.

The next probe is `USB AW Probe` version `1.12`, applet id `0xa12a`. It is intentionally separate from the earlier `USB Menu Probe` records because the NEO rejected visible applet-list row `14` for remove command `0x05` with status `0x8a`; removing by visible row is still not a safe replacement mechanism. Version `1.12` keeps the same write metadata and namespace handlers, but now declares base memory `0x240` and writes the AlphaWord-like local state bytes before returning namespace init status:

- `0x10001` and `0x30001`: set `A5+0x138 = 1`, clear `A5+0x142`, set `A5+0x143 = 1`, clear `A5+0xbc`, draw `LINK`, return status `0x11`.
- `0x20001`: set `A5+0x138 = 1`, clear `A5+0x142`, clear `A5+0x143`, clear `A5+0xbc`, draw `PC`, return status `0x11`.

This is a narrow test of the current AlphaWord-state hypothesis. It still does not clone the real `A5+0xba` command-stream helper routines; it only seeds the specific state bytes proven from AlphaWord's dispatcher before the next attach path is observed.

Live result: `USB AW Probe` v1.12 reached the custom attach branch but bus-faulted at `0x423044` before drawing. The address equals the attempted `A5+0x138` write, proving the attach path reached the applet but that writing AlphaWord-private A5 offsets from the custom applet's USB callback context is unsafe.

`USB Init Probe` v1.13 removed every A5-relative write. A read-back dump of applet `0xa12b` exactly matched `exports/usb-init-probe.os3kapp` and contained no `1b bc 00 01 48 00`, `42 35 48 00`, `28 3c 00 00 01 38`, or `28 3c 00 00 01 43` instruction bytes. It still faulted at `0x423044`; the likely cause is now the text/UI trap sequence (`A000`/`A004`/`A010`/`A098`) being unsafe in the USB attach callback context.

`USB Fault Probe` v1.14, applet id `0xa12c`, therefore avoids all UI drawing in USB attach handlers. It intentionally faults with command-encoded addresses:

- `0x10001` -> access `0x00581001`
- `0x30001` -> access `0x00583001`
- `0x20001` -> access `0x00582001`
- later transfer/event commands -> access `0x0058f00d`

The normal Applets-menu `0x19` path still draws `USB`, so the applet remains selectable before connecting USB.

Live result from v1.14: USB attach faulted at `0x00583001`, proving the System calls the running custom applet with command `0x30001` on this Mac-side attach path.

`USB Silent Probe` v1.15, applet id `0xa12d`, now returns status `0x11` for `0x30001` without drawing and without intentional faulting. All other attach/test branches still fault with command-encoded addresses. The installed device state was cleaned to stock applets plus only `USB Silent Probe` because keeping multiple probe applets caused add-begin timeouts even though read-only applet listing stayed healthy.

Live result from v1.15: after selecting the applet and connecting USB, the NEO stayed on `Making USB connection...`. The host nevertheless saw `081e:bd04` HID keyboard mode, not `081e:bd01`. So a silent successful `0x30001 -> 0x11` response is accepted enough to alter the UI/control flow, but it does not itself activate direct USB mode and did not visibly advance to a later command-fault branch.

`USB Switch Probe` v1.16, applet id `0xa12e`, is the next narrow test. It keeps the same AlphaWord-style write metadata and the same no-UI USB callback discipline as v1.15, but its proven `0x30001` branch now calls the ROM callback at `0x00410b26` before returning status `0x11`:

- `0x30001` -> `jsr 0x00410b26`, write `0x11` to the status pointer, return.
- `0x10001`, `0x20001`, and later transfer/event commands still intentionally fault with command-encoded addresses.

The callback address is the same one NeoManager reaches through the HID report sequence when switching `081e:bd04` keyboard mode to `081e:bd01` direct USB mode. The device was cleaned to stock applets plus only `USB Switch Probe` v1.16, then restarted; the host reported normal HID mode (`081e:bd04`) before the live applet-side USB attach test. This probe tests whether the callback can be safely invoked from the custom applet's `0x30001` attach callback, not just from the firmware's HID-switch path.

Live result from v1.16: after selecting the applet and connecting USB, the NEO showed the stock `emulating keyboard.` path and the host reported `081e:bd04`. This disproves the bare-callback hypothesis: `0x00410b26` is the connected-status/direct-packet callback, not the low-level HID identity-switch primitive.

ROM tracing of `analysis/cab/os3kneorom.os3kos` found the real HID unlock sequence handler around runtime `0x00440b8e`. The validated sequence table starts at runtime `0x0044f6a8`:

- `e0 e1 e2 e3 e4`
- `01 02 04 03 07`
- `f0 f1 f2 f3 f4`
- `07 03 01 04 02`

When a five-byte sequence matches, the first sequence path performs the same completion sequence NeoManager triggers indirectly from host HID reports: writes a one-byte control transfer through `0x0041f9a0`, waits through `0x00424780`, sets byte `0x00013cf9 = 1`, calls runtime `0x0044044e`, waits again, then calls runtime `0x0044047c`. `USB HID Complete` v1.17, applet id `0xa12f`, invokes that HID-completion path from the proven `0x30001` branch without UI drawing, then returns status `0x11`. It was installed as the only custom probe after restoring the stock applets, and the NEO was restarted back to normal `081e:bd04` HID mode for live testing.

Live result from v1.17: after launching `USB HID Complete` and connecting USB, the host reported `081e:bd01` direct mode with bulk OUT `0x01` and bulk IN `0x82`. This proves device-side SmartApplet activation is possible without host access to the initial HID keyboard interface. The successful path is not a direct call to `0x00410b26`; it is the ROM HID-sequence completion path (`0x0044044e` / `0x0044047c`) invoked from the applet's `0x30001` USB attach callback.

`Alpha USB` is the production form of this probe. It uses applet id `0xa130`, version `1.20`, and the menu name `Alpha USB`. Its `0x19` menu-open branch draws brief instructions telling the user to connect the NEO to a computer or smartphone via USB. Its `0x30001` branch keeps the proven v1.17 ROM HID-completion sequence unchanged, then calls the normal direct-mode status callback `0x00410b26` before returning status `0x11`. Unlike the diagnostic probes, it has no intentional fault branches; other USB/event branches return quiet status values and avoid UI traps in USB callback context. Build command:

Failed follow-up: v1.21 tried to draw a custom post-switch message and then idle in the applet screen from inside the `0x30001` USB attach callback. That was the wrong ownership boundary. The callback is part of the System USB/SmartApplet dispatcher path and must return a status; blocking there caused a finalize-response checksum mismatch during install, and the next boot reported `File 127 MaxSize overflow...`. After recovery, the applet catalog contained duplicate `Alpha USB` records, so the best-supported failure model is an interrupted/inconsistent install-finalize/catalog state, not a proven direct write by the applet into a specific file table slot. Future display experiments must not call display traps, flush traps, or `A25C`/idle loops from USB attach callbacks.

Hard safety rule for custom USB applets:

- `0x19` may draw UI and idle; it is normal menu-open applet context.
- `0x30001` must be treated as an interrupt-like USB dispatcher callback.
- `0x30001` may run the minimal HID-completion/direct-mode sequence, set status `0x11`, and return.
- `0x30001` must not own the screen, wait for keys, enter an applet idle loop, or perform multi-step UI flows.
- Any install checksum mismatch or unexpected finalize response is a stop condition; do not install another candidate until `real-check applets` has been read and archived.

```bash
uv run --project poc/neotools neotools build-benign-smartapplet \
  --output exports/alpha-usb.os3kapp \
  --applet-id 0xa130 \
  --name "Alpha USB" \
  --draw-on-menu-command \
  --host-usb-message-handler \
  --alphaword-write-metadata \
  --alpha-usb-production
```

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
- `0xa14c` `read_text_input_char`: `stack_argument_count = 0`, return value used as the next typed input character while AlphaWordPlus field-entry handlers are in text-entry mode
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
- `0xa378` `render_formatted_pending_text`: `stack_argument_count = 0`, no return value used; best current reading is a direct formatted-text render helper for dialogs and status screens
- `0xa380` `format_pending_text`: `stack_argument_count = 0`, no return value used; best current reading is a pending-text formatter/builder used before wrapped rendering
- `0xa388` `query_active_service_disabled_state`: `stack_argument_count = 0`, scalar/state return; in AlphaWordPlus spell-check toggles, zero means enabled and nonzero means turned off
- `0xa398` `query_pending_text_length`: `stack_argument_count = 0`, scalar return; used to measure the current pending/formatted text buffer
- `0xa38c` `shared_runtime_a38c`: shared A3xx helper, still unresolved
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

- `BeginAlphaWordScreenRedraw`
  - is the common AlphaWordPlus-local screen reset helper used before rebuilding chooser, prompt, statistics, and settings screens
  - current best interpretation is “clear/reset the current AlphaWord screen frame before drawing”
- `FlushAlphaWordScreenRedraw`
  - is used after a screen body has been laid out and immediately before modal key reads or span/statistics capture
  - current best interpretation is “finalize/flush the current AlphaWord screen frame for interaction”
- `GetCurrentAlphaWordFileName`
  - returns the current AlphaWord file name string pointer
  - this is the string rendered by current-file prompts and compared against generated default names
- `AdvanceAlphaWordFileIterator`
  - advances or rebinds the current AlphaWord file iterator state used by the open-file selector and duplicate-name scans
  - repeated callers pair it with `QueryCurrentAlphaWordFileOrdinal` and `GetCurrentAlphaWordFileName`
- `QueryCurrentAlphaWordFileOrdinal`
  - returns the ordinal/id of the current AlphaWord file selected by that iterator state
  - the selector, slot reassignment, and duplicate-name scans all use it as the “current file record” identity
- `QuerySelectedAlphaWordSlotFileOrdinal`
  - returns the file ordinal currently assigned to the selected AlphaWord slot/workspace
  - the slot-cache refresh and selector-state builders use it to map slots back to file records
- `DrawPendingWrappedText`
  - renders the currently prepared longer prompt/help text block
  - confirmation and file-name prompts call it for the main wrapped body, then render the footer prompt on the next row
- `RunAlphaWordFileNameEntryDialog`
  - is the inline modal file-name editor used by the unique-name workflow
  - it returns the same accept/cancel style modal status codes used elsewhere in AlphaWordPlus
- `WaitForAlphaWordModalKey`
  - is the common “wait for one modal keypress before continuing” helper used by details, stats, spell-check, file-management, and settings prompts
- `BeginAlphaWordChooserBuilder`
  - begins an AlphaWordPlus-local chooser/menu build sequence
- `AppendAlphaWordChooserRowText`
  - appends one chooser row using the currently prepared display text
- `AdvanceAlphaWordChooserRow`
  - advances to the next chooser row, including explicit spacer rows used by help and spell-check menus
- `HighlightAlphaWordChooserRow`
  - marks the current chooser row as highlighted/selected before input begins
- `BeginAlphaWordChooserInput`
  - finalizes the chooser layout and enters chooser input mode
- `ReadAlphaWordChooserEventCode`
  - returns the current chooser event code such as accept, cancel, or navigation events
- `ReadAlphaWordChooserSelectionIndex`
  - returns the current chooser row index
- `ReadAlphaWordChooserSelectionValue`
  - returns the chooser row value associated with that current selection
- `ReadAlphaWordChooserSelectionPayload`
  - returns the chooser payload pointer/value used by the spell-check suggestion chooser
- `QueryCurrentAlphaWordEditorSpan`
  - is still the main current-span query used by namespace-2 and the statistics flows
  - current best interpretation remains a two-part editor span descriptor
- `QueryCurrentAlphaWordTransferCursor`
  - is distinct from the editor span helper and is reused heavily by spell-check, find/replace, and stream export paths
  - current best interpretation remains a transfer/export cursor or offset, not the whole file size
- `LoadCurrentAlphaWordTransferState`
  - is called before querying `QueryCurrentAlphaWordTransferCursor` and `QueryCurrentAlphaWordTransferExtent`
  - current best interpretation is a host-backed load/refresh of the current file transfer state before those values are read
- `SyncCurrentAlphaWordFileState`
  - is called after markup normalization, ROM test overwrite writes, namespace-1 append writes, and namespace-2 span validation
  - current best interpretation is a host-backed sync/flush of the current file state after mutations or before span-sensitive queries
- `QueryAlphaWordSlotTransferEnd`
  - is slot-aware in raw disassembly
  - for a valid slot it looks up that slot handle and sums two per-slot queries
  - the fallback path does the same against a fixed default handle
  - current best interpretation is a slot-specific transfer end / capacity boundary rather than a generic current-file query

Late AlphaWordPlus helper block now mapped:

- `ClassifyFindReplaceCharKind`
  - classifies a byte for the Find/Replace scanner using the local `DAT_00014b35` table
- `ApplyFindReplaceReplacementCaseMode`
  - rewrites replacement text to match the source token capitalization mode
- `ShowFindReplaceSummaryPrompt`
  - renders the summary/result prompt after a replace operation
- `DecodeFindReplaceCaretEscapes`
  - decodes `^t` and `^p` style escapes in the replacement field
- `ShowFindReplaceCapacityExceededPrompt`
  - renders the “replacement would exceed capacity” warning
- `ShowFindReplaceNoMatchesPrompt`
  - renders the “no matches found” warning
- `WaitForInteractiveDialogKey`
  - shared interactive dialog key wait loop used by selector, statistics, single-line editor, and Find/Replace prompts
- `SetAlphaWordCurrentFileOrdinal`
  - updates the current AlphaWord file ordinal state and refreshes the backing workspace binding
- `LoadAlphaWordFileOrdinalIntoCurrentWorkspace`
  - host-backed helper that loads the requested file ordinal into the current workspace, creating or reusing a backing file as needed
- `LoadAlphaWordSlotIntoCurrentWorkspace`
  - higher-level slot workflow that optionally prompts, then loads a slot’s file into the current workspace
- `ConfirmCurrentAlphaWordFileAvailable`
  - returns whether a current AlphaWord file is available after applying the standard confirmation/prompt flow
- `RunConfirmCancelDialog`
  - shared confirm/cancel wrapped-text dialog wrapper
- `SelectAlphaWordSlotWithCurrentFilePrompt`
  - switches slots, optionally prompting to preserve or load the current file first
- `ReadNextSectionLineIndexIfPresent`
  - returns the next line index only if the preview stream yields another line
- `ReadNextSectionLineIndex`
  - unconditional “next line index” helper for section preview traversal
- `PrimeSectionPreviewCharStream`
  - primes the character stream used by section preview scanners
- `AdvanceTextColumnOnce`
  - one-column wrapper used by the generic wrapped-text dialog helpers
- `IsModalAcceptOrCancelKey`
  - default key filter for wrapped dialogs that accepts only OK/cancel keys
- `ShowFindReplaceScanBanner`
  - renders the standard Find/Replace scan banner before range-wide operations
- `BeginWrappedTextRow`
  - wrapped-text dialog row setup helper
- `GetWrappedTextCursorPosition`
  - returns the current wrapped-text cursor position
- `RenderWrappedTextBlock`
  - renders a wrapped block of text starting at the current offset
- `RunPagedWrappedTextDialog`
  - generic paged wrapped-text dialog engine used by help and confirmation flows
- `FindPreviousWrappedPageStart`
  - computes the starting offset of the previous wrapped-text page
- `FindNextWrappedLineBreakFrom`
  - finds the next wrapped line break from a starting offset
- `IsWrappedTextBreakChar`
  - classifies wrapped-text break characters
- `FindWrappedLineBreak`
  - core wrapped-text line-breaking helper
- `AdvanceTextColumns`
  - advances multiple columns in a row
- `AcceptAnyDialogKey`
  - permissive wrapped-dialog key filter
- `HandleSingleLineTextFieldInput`
  - core keystroke handler for the inline single-line text editor
- `RunFindReplaceToggleFieldEditor`
  - editor for the boolean Find/Replace option rows
- `ShowAlphaWordConfirmFooterPrompt`
  - shared footer prompt used by confirmation screens
- `ShowSpellCheckCompletionBanner`
  - renders the spell-check completion banner for selection vs full-file runs
- `InitializeSpellCheckSessionState`
  - initializes selection bounds and ignore-list state for a spell-check run
- `FinalizeSpellCheckSessionState`
  - tears down that spell-check session state
- `IsSpellCheckCandidateWithinSessionRange`
  - range gate for the active spell-check target
- `PromptIgnoreAllRemainingSpellCheckMatches`
  - prompts to ignore all remaining occurrences of the current misspelling
- `CountRemainingSpellCheckMatches`
  - counts remaining occurrences of the current misspelling
- `AppendSpellCheckIgnoreToken`
  - appends a token to the per-session ignore list
- `IsSpellCheckTokenIgnored`
  - checks the current token against that ignore list
- `CompareSpellCheckToken`
  - token comparison helper with optional case-folding
- `ApplySpellCheckReplacementAcrossSessionRange`
  - applies a replacement across the active spell-check session range
- `MatchFindReplaceLiteralAtCharStream`
  - matches a prepared literal against the current AlphaWord character stream using exact byte comparison
- `MatchFindReplaceLiteralCaseFoldedAtCharStream`
  - matches that same prepared literal using the AlphaWord case-folding helper instead of raw byte equality
- `UpdateSingleLineTextFieldViewportForKey`
  - updates the inline single-line editor cursor/window state for home/end and left/right style navigation keys
- `LocateNthNewlineDelimitedEntry`
  - walks a newline-delimited in-memory entry table and returns the start pointer plus byte length of the requested entry
- `LoadCurrentCharStreamPreviewIntoBuffer`
  - conservative name: primes the current AlphaWord char stream, optionally skips one initial unit, and loads up to the requested preview length into the caller buffer
- `ValidateAlphaWordFileNameCharacters`
  - validates a proposed AlphaWord file name against the local filename-character classification table
- `LoadActiveSelectionIntoSearchBuffer`
  - loads the active document selection into the shared search/replace buffer
- `LoadCurrentWordIntoSearchBuffer`
  - loads the current word under the char-stream cursor into that same search buffer
- `ExtractDelimitedTokenFromCharStream`
  - skips forward to the next delimiter boundary, then extracts the following token until the next delimiter or NUL
- `CopyCharStreamPrefixToBuffer`
  - copies a bounded prefix of the current AlphaWord char stream into a caller buffer
- `IsDelimitedTokenBoundary`
  - classifies bytes accepted as token boundaries by the local scanner
- `LookupQueuedEntryKindString`
  - maps small queued-entry kind codes to the corresponding AlphaWordPlus string ids
- `AdvanceQueuedEntryCursor`
  - advances the current queued-entry iterator before querying the entry kind
- `ReadQueuedEntryKind`
  - returns the small kind code for the current queued entry
- `ReadQueuedEntryCount`
  - returns the queued-entry count used by the chooser/action workflows
- `IsQueuedEntryMarked`
  - returns whether the current queued entry is already marked/selected
- `ShowCurrentFileSearchBanner`
  - draws the current-file banner used by cross-file prepared-snippet search
- `ShowAlphaWordModalStatusPrompt`
  - renders a short modal status prompt and waits for acknowledgment
- `RunValidatedSingleLineTextFieldPrompt`
  - runs the inline single-line text field until accept/cancel, rejecting empty input when the current pending text length is zero
- `SearchCurrentFileForPreparedSnippet`
  - searches the current file’s char stream for the prepared pending-text snippet
- `SearchAcrossAlphaWordFilesForPreparedSnippet`
  - iterates AlphaWord files and rebinds the current workspace until that prepared snippet is found or the file ring is exhausted
- `RefreshNumericStatusDisplayIfChanged`
  - redraws the numeric status area only when the sampled value differs from the cached value
- `RenderPrimaryNumericStatusValue`
  - formats and draws the primary numeric status field
- `RenderSecondaryNumericStatusValue`
  - formats and draws the secondary numeric status field
- `RunNumericStatusSummaryDialog`
  - conservative name: draws a summary screen for the two numeric status values and loops on modal dialog keys
- `RunNumericStatusLiveDialog`
  - conservative name: drives the live numeric status monitor, refreshing until user exit or a local limit condition
- `RunQueuedEntryPreflightPrompt`
  - conservative name: performs the intro/preflight prompt before the queued-entry chooser flow starts
- `RunQueuedEntryChooserWorkflow`
  - conservative name: chooser workflow over the full queued-entry set
- `RunQueuedEntryActionChooserWorkflow`
  - conservative name: action chooser layered on top of that queued-entry set
- `RunMarkedQueuedEntryChooserWorkflow`
  - conservative name: chooser workflow restricted to entries already marked in the queued-entry state
- `RunMarkedQueuedEntryActionChooserWorkflow`
  - conservative name: action chooser layered on top of the marked queued-entry subset
- `RunPreparedSnippetActionChooser`
  - chooser dialog for the prepared snippet buffer, including edit/confirm actions
- `InitializeAlphaWordAdvancedWorkflowState`
  - conservative name: initializes a larger late-stage AlphaWordPlus workflow state block and dispatches into a jump table
- `RunAlphaWordSingleValuePrompt`
  - conservative name: runs a labeled single-value prompt built on the single-line text editor

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

Shared compiler/runtime helper layer now named in the AlphaWordPlus project:

- `NormalizeInternalFloat32`, `DecodeIeeeFloat32ToInternal`, `EncodeInternalToIeeeFloat32`
  - compiler-style float32 normalization and pack/unpack helpers reused by the applet-local numeric status code
- `DecodeIeeeFloat64ToInternal`, `EncodeInternalToIeeeFloat64`
  - corresponding float64 conversion helpers
- `ShiftRightU64PairWithFill`, `ShiftLeftU64PairWithCarry`, `AddU64PairWithCarry`, `DivideU64PairByU64Pair`
  - 64-bit pair arithmetic helpers used by the float64 normalization/conversion path
- `ShiftRightU32WithFill`, `ShiftLeftU32WithCarry`, `AddU32WithCarry`
  - matching 32-bit helper layer used by the float32 path
- `NormalizeMantissaU32`, `NormalizeMantissaU64Pair`
  - mantissa normalization helpers built on the shift/count-leading-zero primitives
- `CountLeadingZerosU32`, `CountLeadingZerosU64`, `MultiplyU32Fragments`
  - low-level compiler runtime helpers rather than AlphaWordPlus-specific document logic

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

The first generated benign applet for direct-mode experiments is intentionally lifecycle-only. It has applet id `0xa123`, menu name `Direct USB Test`, no runtime trap imports, and no direct-mode calls. It is meant only to prove that a custom applet can appear in the NEO SmartApplets menu and launch/exit safely before any direct-mode patch is attempted.

Build it locally:

```bash
uv run --project poc/neotools neotools build-benign-smartapplet --output exports/direct-usb-test.os3kapp
```

The generated host-side file is ignored by git under `exports/`. Installing it on a physical NEO would modify the device's SmartApplet storage, so the current implementation stops at producing and validating the `.os3kapp` image.

Live/Ghidra correction: valid SmartApplet images must also end with the 4-byte trailer `0xca 0xfe 0xfe 0xed`. NeoManager's file classifier reads the first `0x84` bytes, checks magic `0xc0ffeead`, then seeks to `file_size - 4`; only a trailing `0xcaffeeed` returns SmartApplet type `0x11`. The benign `Direct USB Test` generator now emits a 176-byte image whose executable stub still ends in `4e 75`, followed by the `ca fe fe ed` trailer.

Live lifecycle correction: selecting a generated custom applet from the physical NEO Applets menu first invoked the applet entry point with command `0x19`, not `0x18`. A deliberate fault probe encoded that command as the visible bus-error address `0x580019`. This matches Calculator better than the original minimal stub did: Calculator uses command `0x19` to run its interactive menu loop, then performs cleanup and returns status `7`. A generated probe that handled `0x19` as menu entry stayed active and the NEO could return to the Applets menu, proving the lifecycle path is reachable.

Live custom UI correction: the blank `0x19` probe failed because the drawing path did not match the Calculator-style trap-call ABI closely enough. The working visible probes use local `bsr.w` calls into A-line trap stubs, set `(row=2, column=1, width=28)` with `A004`, draw ASCII bytes through `A010`, flush with `A098`, and idle through `A25C`. The physical NEO accepted and listed `USB Menu Probe` `0xa129` version `1.8` at `364` bytes, and then accepted version `1.9` at `328` bytes after the direct-callback arming sequence replaced the older USB-event experiment.

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

### Live System 3.15 Dump

The retrieve-applet path is now validated against the physical NEO for applet id `0x0000`:

```bash
uv run --project real-check real-check dump-applet 0x0000 --output analysis/device-dumps/neo-system-3.15.os3kapp
```

Validated output:

- bytes read: `401408` (`0x62000`)
- SHA-256: `304a32fb548c8d605351cdef5389976ac2346cace5e9cafcc1e96f7737a37fa6`
- header magic: `0xc0ffeead`
- header file size: `0x00062000`
- base memory size: `0x00015098`
- header offset `0x0c`: `0x0003968a`
- flags/version field: `0x00000011`
- applet id/version field: `0x00000100`
- name: `System`
- copyright: `TESTING VERSION. DO NOT DISTRIBUTE. (c) 2012 AlphaSmart, Inc.`
- extra memory size at header offset `0x80`: `0`
- live applet-list display version: `3.15`

The System applet is special compared with normal `.os3kapp` files. The generic parser reads a body size of `0x39606` and no normal info-table records. In Ghidra, loading the dump as a flat Motorola 68k big-endian image at base `0` gives a single segment `0x00000000..0x00061fff`.

Ghidra found the System string-resource resolver at `FUN_0002cebc`:

```c
undefined * FUN_0002cebc(ushort resource_id) {
  return &DAT_000269fe + *(int *)(&DAT_0003ad72 + (uint)resource_id * 4);
}
```

Confirmed bases:

- resource text/data base: `0x269fe`
- dword offset table: `0x3ad72`

USB/user-visible resource ids pinned from that table:

- `0x04c`: `Attached to Mac, emulating keyboard.`
- `0x04d`: `Attached to PC, emulating keyboard.`
- `0x04e..0x053`: send-file prompts and "Sending file" status
- `0x054..0x055`: Get Utility connected / keys inactive status
- `0x056..0x057`: AlphaHub connected / keys inactive status
- `0x058`: `Receiving file `
- `0x059`: `Connected to computer (infrared mode).`
- `0x0fe..0x104`: multi-line Mac/PC send and suspended-format strings

Correction from the AlphaWord Plus applet itself: the distinct AlphaWord screen is not only a System fallback. `analysis/cab/alphawordplus.os3kapp` contains its own copies of the exact attach strings. The applet's resource lookup function at `0x064be` indexes the pointer table at `0x14374`, and Ghidra xrefs confirm the table entries:

- pointer table entry `0x04c` at `0x144a4` -> string `0x153e1`: `Attached to Mac, emulating keyboard.`
- pointer table entry `0x04d` -> string `0x15406`: `Attached to PC, emulating keyboard.`
- pointer table entry `0x0fe` at `0x1476c` -> string `0x19829`: `Attached to MAC, emulating keyboard.` plus the send-file instructions and current filename format.
- pointer table entry `0x0ff` -> string `0x198a3`: `Attached to PC, emulating keyboard.` plus the PC-side send-file instructions and current filename format.
- pointer table entries `0x100..0x104` -> `MAC`, `PC`, `Attached to %s.\nSending "%s".`, `Attached to %s.\nAborting send of  "%s".`, and `Attached to %s.\nUSB connection suspended.`

The short strings are rendered by the namespace handlers:

- `HandleAlphaWordNamespace1Commands` at `0x07a44` handles `0x10001`, returns status `0x11`, calls `ResetAlphaWordCommandStreamState`, sets the applet-local Mac flag byte at `A5+0x143` to `1`, then redraws `0x04c`, `0x04e`, `0x052`, and `0x053`.
- `HandleAlphaWordNamespace2Commands` at `0x08210` handles `0x20001`, returns status `0x11`, resets the same command stream, and redraws `0x04d`, `0x04e`, `0x04f`, and `0x050` on the PC-side path.

The uppercase `Attached to MAC/PC...` message is driven by AlphaWord's main entry dispatcher, not by a trivial one-command draw handler. The executable entry at `0x0094` dispatches namespace commands first, then calls the command-stream helpers around the applet-local state block at `A5+0xba`. The important local bytes are:

- `A5+0xba+2`: command-stream state byte used as an index into the dispatcher jump table at `0x02e0`.
- `A5+0x143`: Mac/PC selector. Command `0x10001` sets it to `1`; command `0x30001` also sets it when its input payload is non-empty; the generic `command low byte == 1` path otherwise clears it.

When the stream state reaches case `0`, the dispatcher builds the formatted attach screen. If `A5+0x143` is nonzero it formats resource `0x0fe`; otherwise it formats resource `0x0ff`. Later state cases format `0x102`, `0x103`, and `0x104` for sending, aborting, and suspended USB states. This explains why our custom probe can advertise AlphaWord-like write metadata and answer namespace ids but still fall through to the stock `Connected to computer, emulating keyboard.` UI: the proven AlphaWord behavior depends on its private command-stream state machine, not only on the metadata records or direct `0x10001`/`0x20001` handlers.

Relevant System-side handlers located so far:

- `FUN_0002e442`: Mac/keyboard-send command handler; displays resources `0x04c`, `0x04e`, `0x051`, `0x052`, and `0x053`.
- `FUN_0002ec0e`: PC/keyboard-send peer path; displays resources `0x04d`, `0x04e`, `0x04f`, `0x050`, `0x051`, and `0x058`.
- `FUN_0002f242`: Get Utility handler; displays `0x054` and `0x055`.
- `FUN_0002f464`: infrared computer mode handler; displays `0x059`, handles `0xbc` command subcodes, and routes some receive/status paths through `0x058`.
- `FUN_00042404`: on-device attached-computer UI loop; it renders the attached Mac/PC/send prompts and handles return/control bytes such as `0x40`, `0x48`, `0x4b`, `0x06`, and `0x0d`.

The System dump also contains a USB descriptor/data blob near `0x60a6..0x6211`.

Important descriptor offsets:

- `0x60c2`: HID report descriptor, length `0x3f`; it includes the normal keyboard modifier usage range `0xe0..0xe7` and LED output usages.
- `0x6102`: `081e:bd04` device descriptor.
  - bcdUSB `1.00`
  - device class/subclass/protocol `0/0/0`
  - max packet `64`
  - VID/PID `0x081e:0xbd04`
  - bcdDevice `0x0002`
  - manufacturer string id `1`
  - product string id `2` (`AlphaSmart`)
- `0x6114`: `081e:bd01` device descriptor.
  - bcdUSB `1.00`
  - device class/subclass/protocol `2/0/0`
  - max packet `64`
  - VID/PID `0x081e:0xbd01`
  - bcdDevice `0x0002`
  - manufacturer string id `1`
  - product string id `3` (`AlphaSmart Communication Driver`)
- `0x6126`: configuration descriptor with total length `0x22`, matching the HID keyboard-side interface set.
- `0x6130`: configuration descriptor with total length `0x20`, matching the direct communication-driver side.
- `0x613a`: HID boot-keyboard interface descriptor.
- `0x6143`: communication-driver interface descriptor with two endpoints.
- `0x614c`: HID descriptor pointing at the `0x3f`-byte report descriptor.
- `0x6156`: endpoint `0x82`, interrupt IN, max packet `64`, interval `10`.
- `0x615e`: endpoint `0x82`, bulk IN, max packet `64`.
- `0x6166`: endpoint `0x01`, bulk OUT, max packet `64`.
- `0x6172`: UTF-16 string `AlphaSmart, Inc.`
- `0x6194`: UTF-16 string `AlphaSmart`
- `0x61aa`: UTF-16 string `AlphaSmart Communication Driver`
- `0x61ea`: UTF-16 string `AlphaSmart Keyboard`

Immediately before the HID report descriptor, the System dump contains this candidate HID output-report sequence table:

- `0x60a6`: `e0 e1 e2 e3 e4`
- `0x60ab`: `01 02 04 03 07`
- `0x60b0`: `f0 f1 f2 f3 f4`
- `0x60b5`: `07 03 01 04 02`
- `0x60ba`: `00 02 03 04 01 02 03 04`

The same table appears in the installer ROM image `analysis/cab/os3kneorom.os3kos` at file offsets `0x3f6a8`, `0x3f6ad`, `0x3f6b2`, `0x3f6b7`, and `0x3f6bc`. The first sequence, `e0 e1 e2 e3 e4`, is dynamically proven on the physical NEO as the `081e:bd04` keyboard-mode to `081e:bd01` direct-mode activation sequence. The other adjacent sequences are firmware candidates that still need live switching tests.

The separate Small ROM updater image `analysis/cab/smallos3kneorom.os3kos` contains a `081e:bd01` descriptor at file offset `0x5458`, no `081e:bd04` descriptor, and no HID report descriptor. That supports the second proven route to a direct USB identity: booting into the Small ROM/updater path. Live testing showed that this mode is not equivalent to the normal AlphaWord direct path; the AlphaWord file-attribute command `0x13` returned status `0x92`.

The normal direct-mode LCD message `Connected to NEO Manager.` is in the installer ROM, not in the live System applet dump. In `analysis/cab/os3kneorom.os3kos`, the relevant ROM area maps as `runtime = file offset + 0x410000`. The state-5 callback is runtime `0x00410b26` / file offset `0x0b26`; it writes `0x05` to global status byte `0x444`, calls the direct-mode global initializer at runtime `0x00412c82`, then repaints through the status renderer at runtime `0x004109ca`. Renderer state `5` uses string resource `0x8c`, whose pointer-table entry at file offset `0x3b26c` resolves to file offset `0x3693b`, the exact `Connected to NEO Manager.` bytes.

Static ROM inventory found only one immediate write of `0x05` to global `0x444` and only one registration of the state-5 callback (`pea.l 0x410b26` followed by `jsr 0x424f66` at file offsets `0x0194..0x019a`). Duplicate copies of `Connected to NEO Manager.` in the ROM are therefore not by themselves evidence of additional HID-to-direct activation paths.

Control-flow trace: the System/ROM dispatcher at runtime `0x004100a0` routes lifecycle event `0x19` through its jump table to runtime `0x0041016e`; that path initializes the USB/direct globals, calls runtime `0x00424fb0`, then registers callback `0x00410b26` with runtime `0x00424f66`. The registration helper stores the callback pointer in RAM global `0x5d9c`; low-level hardware/USB handlers later call through that pointer. No installed SmartApplet, shipped SmartApplet, Small ROM image, or alternate ROM site has been found that directly calls `0x00410b26`, registers it a second time, or directly writes visible state `5` to global `0x444`.

Implication for device-side activation: a local key/menu action can plausibly force the System applet through lifecycle `0x19` and therefore arm the direct-mode callback, but that is not the same as autonomously entering the normal AlphaWord direct protocol. The currently documented local recovery workflow is left-shift+tab during power-on to reach the SmartApplet menu before connecting USB; the actual `Connected to NEO Manager.` state still appears to require the lower-level USB/HID callback to fire.

Strings that were explicitly searched and not present as ASCII in the System dump:

- `NEO Manager`
- `NeoManager`
- `Swtch`
- `Switched`
- `bd01` / `BD01`
- `bd04` / `BD04`

The only `Manager` hit is the unrelated wireless-file-transfer message `disabled in AlphaSmart Manager.`. The direct USB descriptor data is present, but this dump has not revealed an alternate Android-accessible switch path beyond the already validated HID-output-report transition.

Ghidra caveat: auto-analysis creates false functions inside late data/descriptor regions, including the `0x5f80..0x62000` area. Treat apparent functions in that range as suspect unless they have clear code flow and sane callers.

## Send Applet To Device

`UpdaterAddApplet` is the host-to-device SmartApplet install routine.

It first reads the first `0x84` bytes of the host `.OS3KApp` file and derives the add-applet start fields from that header.

What is confirmed:

- add-begin command `0x06`
- response must be `0x46`
- chunk handshake command `0x02`
- chunk handshake response must be `0x42`
- post-chunk completion response is read after the raw chunk payload
- completion response must be `0x43`
- program-applet command `0x0b`
- program response must be `0x47`
- finalize command `0x07`
- finalize response must be `0x48`
- payload chunk size is capped at `0x400`

Important correction from `UpdaterSendCommandAndGetResponse`: NeoManager calls the helper with command byte `0xff` after writing each chunk, but the helper treats `0xff` as a sentinel and skips `BuildUpdaterCommandPacket` / `TransportWriteExact`. In other words, post-chunk completion is a read-only wait for status `0x43`; it is not an actual `ff 00 00 00 00 00 00 ff` packet on the direct USB transport.

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
11. read the post-chunk completion response
12. expect `0x43`
13. send `0x0b` for the just-staged chunk
14. expect `0x47`
15. after all chunks have been staged and programmed, send `0x07`
16. expect `0x48`

Validated correction: NeoManager sends the `0x0b` program command inside the `0x400`-byte chunk loop, immediately after the read-only `0x43` chunk-completion wait. Sending all chunks first and only one `0x0b` at the end can leave the device waiting or fail during applet restore.

Live failure note from the first custom applet install attempt:

- A malformed 172-byte generated applet without the `0xcaffeeed` trailer was staged and then rejected at program time.
- The NEO displayed `Error: ROM not Erased`; the direct response to a later `0x06` add-begin was status `0x81`.
- NeoManager's higher-level retry path treats `0x81` as a compaction-needed condition.
- The remove command is `0x05` with `arg32=5`, expecting response `0x45`. It is not applet-id-based. Later Ghidra tracing indicates the trailing value is an internal listen/slot-table index, not the visible direct-mode applet-list row; sending both `0xa123` and visible row `7` produced `Error: Invalid SmartApplet index`/status `0x8a`.
- The broad applet-area clear command used by NeoManager's compaction path is `0x11`, expecting response `0x4f`, with a 90-second timeout. That is not a narrow custom-applet repair command; it clears the SmartApplet area and NeoManager's UI flow backs up/restores applets around it.
- After a factory reset left only `System`, the backed-up stock applets were successfully restored from `exports/smartapplet-backups/20260418-145415` using the corrected per-chunk `0x0b` sequencing.

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
- live read-only applet metadata listing through `real-check applets`
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
uv run --project real-check real-check applets
uv run --project poc/neotools python -m neotools smartapplet-retrieve-plan 0xa123
uv run --project poc/neotools python -m neotools smartapplet-add-plan 0x12345678 0x9abc "41 42 43 44 45"
uv run --project poc/neotools python -m neotools smartapplet-header "<0x84-byte header hex>"
uv run --project poc/neotools python -m neotools smartapplet-metadata "<0x84-byte header hex>"
uv run --project poc/neotools python -m neotools os3kapp-image "<full .OS3KApp hex>"
uv run --project poc/neotools python -m neotools smartapplet-string 0xf138
uv run --project poc/neotools python -m neotools smartapplet-menu 163
uv run --project poc/neotools python -m neotools smartapplet-add-plan-from-image "<full .OS3KApp hex>"
```

Safety note: `real-check applets` is a read-only live metadata list probe. The offline `smartapplet-add-plan*` commands only print modeled packets, but any future live implementation of the `0x06` / `0x02` / `0xff` / `0x0b` / `0x07` add/program/finalize sequence would modify the device and must not be used as a probe on a data-bearing NEO.

## 2026-04-18 Recovery Note

Custom SmartApplet USB experiments produced duplicate `Alpha USB` entries and a
damaged writable catalog on the physical NEO. The successful repair did not use
the broad restore flow as the final mechanism. Instead, after a patched
validator-disabled OS restored normal direct USB access, the applet area was
left with System only and stock applets were installed one at a time, validating
with `real-check applets` after each install.

The Thesaurus applet was intentionally not restored. The final validated applet
set was System, AlphaWord Plus, five NEO fonts, KeyWords, Control Panel, Beamer,
AlphaQuiz, Calculator, Text2Speech Updater, and SpellCheck Large USA.

Full details and exact commands are in
[2026-04-18-neo-recovery-runbook.md](/Users/jakubkolcar/customs/neo-re/docs/2026-04-18-neo-recovery-runbook.md).

## 2026-04-19 Android End-To-End Validation

`Alpha USB` `0xa130` version `1.20` is now validated as the production bridge
for Android backup:

1. launch `Alpha USB` on the NEO,
2. connect the NEO to a physical Android device over USB Host/OTG,
3. the applet switches the NEO from the otherwise Android-hidden `081e:bd04`
   HID keyboard identity into `081e:bd01` direct USB,
4. the Android GUI opens the direct device through `UsbManager`,
5. the Android GUI backs up AlphaWord files successfully.

This closes the Android constraint set: no root, no proxy device, and no
typewriter/keyboard fallback are needed when the stable `Alpha USB` applet is
used.

## Remaining Unknowns

- the exact semantic names of the three proven flag bits in the flags dword at offset `0x10`
- the exact human-readable mapping behind every `applet_class` byte value at offset `0x3f`
- the final MFC command-handler binding for every decoded popup-menu id
