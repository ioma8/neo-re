# NeoManager "Get/Print AlphaWord Files" Dataflow

Date: 2026-03-31

## Scope

This note tracks the direct dataflow behind the NeoManager UI feature labeled:

- `Get/Print AlphaWord Files`

This is focused on the direct USB path and the higher-level updater protocol used by `neomanager.exe`.

Out of scope:

- AlphaHub broadcast behavior
- MacOS transport implementation
- Other applets unrelated to AlphaWord file retrieval and printing

## Core Finding

The `Get/Print AlphaWord Files` feature is split into two distinct layers:

1. retrieval from the NEO through the updater protocol
2. local post-processing of the retrieved bytes into previewable and printable text

The transport and protocol layer is shared with other applet operations. The AlphaWord-specific behavior is in the choice of updater commands, expected response bytes, record lengths, and the later text formatting path.

The current strongest applet-level identifier for AlphaWord is:

- applet id `0xa000`

## Direct USB Transport Layer

For direct USB, the executable ultimately reaches these imported DLL calls:

- `AsUSBCommWriteData`
- `AsUSBCommReadData`
- `AsUSBCommIsAlphaSmartPresent`
- `AsUSBCommSwitchToApplet`
- `AsUSBCommResetConnection`

The shared transport wrappers inside `neomanager.exe` dispatch to `AsUSBComm*` when the transport mode is direct USB.

Relevant functions:

- `FUN_00430050`: generic transport write loop
- `FUN_00430180`: generic transport read loop

`AsUSBCommReadData` stages inbound traffic in 8-byte chunks, so updater responses above this layer are assembled by repeated small reads rather than one raw device transfer.

Before the updater commands start, the direct USB path also uses the switch protocol:

- `FUN_00438200` and `FUN_00438290` both do:
  - `AsUSBCommResetConnection()`
  - `AsUSBCommSwitchToApplet(0)`
  - `AsUSBCommIsAlphaSmartPresent()`
- in practice this means NeoManager first switches the device into updater-side mode using applet id `0`, not directly into AlphaWord

The offline PoC now models this bootstrap sequence at:

- [alphaword_flow.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_flow.py)

## Generic Updater Command Frame

`FUN_004374b0` builds every updater command packet.

Ghidra decompilation shows the exact on-wire format:

- byte `0`: command byte
- bytes `1..4`: 32-bit argument, big-endian
- bytes `5..6`: 16-bit trailing field, big-endian
- byte `7`: 8-bit checksum equal to the low byte of the sum of bytes `0..6`

So the generic frame is:

```text
[cmd][arg32-be][arg16-be][sum]
```

This framing is now modeled in the PoC at:

- [updater_packets.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/updater_packets.py)

## AlphaWord Retrieval Commands

### Raw file attributes

`FUN_00436200` is the raw attribute retrieval routine.

Behavior confirmed by Ghidra:

- calls `FUN_004377c0(..., timeout=600, command=0x13, argument=file_slot & 0xff, trailing=applet_id, ...)`
- expects first response byte `0x90` for the simple success case
- also handles response byte `0x5a` as the data-bearing attributes path
- reads the returned attribute bytes with `read_data1`
- verifies the returned 16-bit checksum against the byte sum of the attribute payload
- caller-provided receive storage is `0x28` bytes
- `FUN_00436080` endian-swaps two 32-bit words from offsets `0x18` and `0x1c`
- `FUN_00435b00` treats the big-endian word at offset `0x1c` as the file content length

The most concrete attribute command packet currently confirmed is:

```text
command = 0x13
argument = file_slot
trailing = applet_id
```

The offline PoC models this as `build_raw_file_attributes_command`.

### File content retrieval

`FUN_00434100` retrieves the actual file contents.

Behavior confirmed by Ghidra:

- calls `FUN_004377c0(..., timeout=10000, command=0x12 or 0x1c, argument=(requested_length << 8) | file_slot, trailing=applet_id, ...)`
- `param_7` selects command `0x12` vs `0x1c`
- expects first response byte `0x53` (`'S'`)
- the initial response `arg32` field is the total payload length NeoManager expects to receive
- then repeatedly requests chunk payloads using command `0x10`
- each chunk cycle expects response byte `0x4d` (`'M'`)
- each chunk response `arg32` field is the chunk byte count
- each chunk response trailing field is the expected 16-bit checksum of the chunk payload
- chunk data is read with `read_data1`
- chunk bytes are written to the destination sink with `FUN_004383d0`
- the byte-sum checksum of each chunk must equal that chunk response’s 16-bit trailing field

The primary retrieve command packet is therefore:

```text
command = 0x12 or 0x1c
argument = (requested_length << 8) | file_slot
trailing = applet_id
```

The offline PoC models this as `build_retrieve_file_command`.

Concrete values confirmed from higher-level call sites:

- full retrieval uses `requested_length = 0x80000`
- preview retrieval uses `requested_length = 0x0b4`
- AlphaWord retrieval uses `applet_id = 0xa000`

So for AlphaWord slot `0x12`, the direct USB full-retrieve command is:

```text
12 08 00 00 12 a0 00 cc
```

and the preview command is:

```text
12 00 00 b4 12 a0 00 78
```

The follow-up chunk request remains:

```text
10 00 00 00 00 00 00 10
```

The PoC also models the 8-byte response headers and chunk reconstruction at:

- [updater_responses.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/updater_responses.py)
- [alphaword_transfer.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_transfer.py)

## Applet Discovery Layer

NeoManager does not hardcode only the AlphaWord applet id at the transport layer. It also walks the applet list.

`FUN_00430470`:

- sends updater command `0x04`
- uses `arg32` as the page offset
- uses trailing field `7`
- expects response byte `0x44`
- reads applet records in `0x84` byte entries
- paginates in batches of 7 entries

This matches the current PoC bootstrap sequence:

```text
04 00 00 00 00 00 07 0b
```

## UI-Facing Retrieval Wrappers

Two small wrappers select transport context before entering `FUN_00434100`:

- `FUN_00434080`
- `FUN_004340c0`

These do not change the updater opcode. They prepare the transport context passed as `param_1` to `FUN_00434100`:

- mode `2`: direct USB / single-device path
- mode `3`: alternate port-aware context used by the multi-device path

That distinction matters because earlier assembly could be misread as if `2` and `3` were file-retrieval command numbers. Ghidra makes clear they are transport selectors, not updater opcodes.

## Higher-Level AlphaWord Retrieval Helpers

Two UI-facing helpers sit on top of the raw retrieval routines.

### Full text retrieval helper

`FUN_00483bf0`

Behavior:

- opens a temporary output sink through `FUN_00485af0`
- for direct USB mode, calls `FUN_00434080(..., max_len=0x80000, ...)`
- for alternate transport mode, calls `FUN_004340c0(..., max_len=0x80000, ...)`
- then passes the resulting sink to `FUN_00485c50`
- returns a `CString` built from the retrieved content

Interpretation:

- this path requests a large cap of `0x80000` bytes
- it is the "full retrieval" path for AlphaWord data that can later be printed or saved

### Preview / short printable text helper

`FUN_00483eb0`

Behavior:

- opens a temporary output sink through `FUN_00485af0`
- for direct USB mode, calls `FUN_00434080(..., max_len=0xb4, ...)`
- for alternate transport mode, calls `FUN_004340c0(..., max_len=0xb4, ...)`
- then converts the retrieved data through `FUN_00485c50`
- replaces CRLF with spaces
- truncates the resulting string to `0xb3` bytes if needed

Interpretation:

- this is the short preview path used to populate UI summaries or list displays
- it is not the full-file retrieval path used for complete printing/export

## Confirmed UI-Side Callers

The exact resource-to-handler binding for the literal UI string is still unresolved, but the higher-level AlphaWord callers are now visible.

`FUN_00406920`:

- repeatedly calls `FUN_00483eb0(..., applet_id=0xa000, ...)`
- iterates through AlphaWord file slots
- stores the returned short text and size back into local cache objects
- is consistent with preview/list population for the `Get/Print AlphaWord Files` view

`FUN_00407e20` and `FUN_00408160`:

- iterate through 8 AlphaWord file slots
- call `FUN_00483bf0(..., applet_id=0xa000, ...)` for full retrieval
- then call `FUN_00483b40(..., applet_id=0xa000, ...)` to obtain associated metadata/text used by the local cache
- update per-slot status in local storage after each retrieval

This is enough to say the app path above the transport is not a single monolithic call. It is:

1. preview-oriented slot enumeration using `FUN_00483eb0`
2. full slot retrieval using `FUN_00483bf0`
3. cache/status updates for each of the 8 AlphaWord slots

## Text Formatting and Print-Side Handoff

`FUN_00485c50` is the local formatter / file-backed text loader that runs after retrieval.

Key observed behavior:

- opens the temporary file with MSVCP60 stream types
- loads a UI string resource with id `0xf1a8`
- reads text from the temp file into a `CString`
- normalizes line endings by replacing `"\r"` with `"\r\n"`
- may delete the temporary file after loading it

Interpretation:

- the updater layer retrieves raw AlphaWord bytes into a local sink
- `FUN_00485c50` re-reads that local file and turns it into printable Windows text
- this is the seam where "retrieve from device" ends and "prepare for display/printing" begins

## End-to-End Direct Dataflow

For the direct USB case, the current best reconstruction is:

1. User enters `Get/Print AlphaWord Files`.
2. NeoManager identifies the direct NEO transport and uses the `AsUSBComm*` path.
3. The direct USB setup path resets the transport and issues `AsUSBCommSwitchToApplet(0)` to enter updater-side mode.
4. The updater layer issues command `0x04` to enumerate applets and confirms the AlphaWord applet context.
5. AlphaWord file metadata is fetched through `FUN_00436200` using opcode `0x13`, `argument=file_slot`, and `trailing=0xa000`.
6. Full AlphaWord file contents are fetched through `FUN_00434100` using opcode `0x12` or `0x1c`, `argument=(requested_length << 8) | file_slot`, and `trailing=0xa000`.
7. The device returns an initial `0x53` response containing the total byte count.
8. NeoManager repeatedly issues command `0x10`, receives `0x4d` chunk headers, reads each chunk body, and verifies each chunk checksum.
9. Retrieved bytes are written to a local temporary sink.
10. `FUN_00485c50` opens the temporary file, converts it into `CString` text, normalizes line endings, and returns printable text.
11. The UI uses:
   - `FUN_00483eb0` for short preview text capped at `0xb4`
   - `FUN_00483bf0` for full retrieval capped at `0x80000`

## Protocol Facts Confirmed So Far

- updater command framing is fixed at 8 bytes
- frame fields are big-endian for the 32-bit and 16-bit arguments
- frame checksum is the low byte of the sum of the first seven bytes
- raw attribute command opcode is `0x13`
- file retrieval command opcode is `0x12` or `0x1c`
- AlphaWord applet id is `0xa000`
- raw attribute command trailing field is the applet id
- file retrieval command trailing field is the applet id
- file retrieval command `arg32` field is `(requested_length << 8) | file_slot`
- file retrieval uses command `0x10` for subsequent chunk fetches
- file retrieval success bytes include:
  - initial `'S'` / `0x53`
  - chunk `'M'` / `0x4d`
- initial `'S'` response `arg32` is total transfer length
- chunk `'M'` response `arg32` is chunk length
- chunk `'M'` response trailing field is the expected payload checksum
- attribute retrieval success/data bytes include:
  - `0x90`
  - `0x5a`
- raw attribute record length is `0x28`
- raw attribute offsets `0x18` and `0x1c` are big-endian 32-bit values used by NeoManager
- raw attribute offset `0x1c` is the file content length
- applet list command is `0x04` with page size `7`
- applet list entry size is `0x84`

## Still Unresolved

- the exact semantic difference between retrieval opcodes `0x12` and `0x1c`
- the exact semantics of the attribute word at offset `0x18`
- the remaining internal layout of the raw AlphaWord attribute block returned by opcode `0x13`
- the exact UI function that binds the literal `Get/Print AlphaWord Files` label to these retrieval helpers
- the exact purpose of `FUN_00483b40` in the full-retrieval caller chain
- whether print uses the full retrieval path in all cases, or can sometimes reuse the preview path for very short records
