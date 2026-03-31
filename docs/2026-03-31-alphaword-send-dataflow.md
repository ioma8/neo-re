# NeoManager "AlphaWord Files to Send" Dataflow

Date: 2026-03-31

## Scope

This note tracks the host-to-device AlphaWord send path, centered on the UI area labeled:

- `AlphaWord Files to Send`

This is focused on the direct USB path and the updater protocol used by `neomanager.exe`.

## Core Finding

The send flow is the mirror of retrieval, but it is not symmetric packet-for-packet.

NeoManager restores or sends AlphaWord file records by transmitting:

1. a `0x28`-byte raw file attributes record
2. the file payload bytes themselves

Each stage has its own updater-side handshake.

## Confirmed Low-Level Helpers

### Raw file attributes put

`UpdaterPutRawFileAttributes` is the raw-file-attributes put routine.

For the direct USB / generic updater path:

- begin command: opcode `0x1d`
- expected begin response: `0x5b`
- payload staging handshake: opcode `0x02`
- expected payload ack: `0x42`
- sends exactly `0x28` bytes of attribute data
- completion probe: opcode `0xff`
- expected completion ack: `0x43`
- final close command: opcode `0x1e`
- expected final response: `0x5c`

Command fields:

- `0x1d`: `arg32=file_slot`, `trailing=applet_id`
- `0x02`: `arg32=0x28`, `trailing=sum16(attribute_record)`
- `0x1e`: `arg32=file_slot`, `trailing=applet_id`

### File payload put

`UpdaterPutFileData` is the file-payload put routine.

For the direct USB / generic updater path:

- begin command: opcode `0x14` or `0x1f`
- expected begin response: `0x50`
- chunk handshake: opcode `0x02`
- expected chunk ack: `0x42`
- chunk bytes are sent with `TransportWriteExact`
- completion probe after each chunk: opcode `0xff`
- expected completion ack after each chunk: `0x43`
- final close command: opcode `0x15`
- expected final response: `0x51`

Confirmed packet construction:

- begin command `arg32 = (file_slot << 24) | file_length`
- begin command `trailing = applet_id`
- if `param_5` is set, bit `31` is ORed into the begin-command argument
- opcode is `0x14` by default, or `0x1f` when `param_6 != 0`
- chunk handshake `arg32 = chunk_length`
- chunk handshake `trailing = sum16(chunk_payload)`
- chunk payloads are capped at `0x400` bytes
- finish command is `0x15` with zero argument and zero trailing field

## Record-Level Send Wrapper

`UpdaterRestoreAppletFileData` is the record-level send wrapper.

It reads a local stream structured as:

1. 4-byte total length prefix
2. repeated records:
   - `0x28` raw attributes block
   - file payload bytes of length stored in attribute offset `0x1c`

For each record:

1. read `0x28` bytes
2. send them through `UpdaterPutRawFileAttributes`
3. decode big-endian file length from attribute offset `0x1c`
4. send that many file payload bytes through `UpdaterPutFile`
5. increment the file slot counter

This confirms that the send flow is record-oriented, not just “send arbitrary text”.

## Transport Wrappers

`SendAlphaWordFileRecordToDevice` is the higher-level wrapper that chooses the transport:

- mode `2` and `5` call `DirectUsbRestoreAppletFileData`, which wraps `UpdaterRestoreAppletFileData` for direct USB
- mode `3` calls `AlternateTransportRestoreAppletFileData`, which wraps the same logic for the alternate port-aware context

## Confirmed UI-Side Call Chain

The top-level send-side controller chain is now visible:

1. `SendSelectedAlphaWordFilesToDevices`
2. `SendAlphaWordFilesToDevice`
3. `SendChosenAlphaWordFileRecords`
4. `SendOneAlphaWordFileRecord`
5. `SendAlphaWordFileRecordToDevice`
6. `UpdaterRestoreAppletFileData`
7. `UpdaterPutRawFileAttributes` then `UpdaterPutFileData`

What each layer does:

### `SendSelectedAlphaWordFilesToDevices`

- initializes the send UI state
- displays progress text from `s_Sending_retrieved_AW_files_004bd61c`
- iterates a table at `param_1 + 0x33f4`
- checks each entry with `FUN_00482a80`
- for active entries, calls `SendAlphaWordFilesToDevice(param_1, device_index)`

Interpretation:

- this is the controller-level action that sends retrieved AlphaWord files to selected connected targets
- the loop bound is transport-dependent and derived from the current device mode

### `SendAlphaWordFilesToDevice`

- resolves the AlphaWord file collection for the selected target/device slot
- gets a count with `CountPopulatedAlphaWordFileRecords`
- if more than one candidate is present, delegates to `SendChosenAlphaWordFileRecords`
- otherwise iterates file records directly and calls `SendOneAlphaWordFileRecord`

Interpretation:

- this function is the per-target sender for the `AlphaWord Files to Send` workflow
- it decides whether to walk one record directly or use the helper that searches for the next sendable record

### `SendChosenAlphaWordFileRecords`

- scans file records until it finds one marked sendable
- builds the per-file path and label strings
- calls `SendOneAlphaWordFileRecord`

Interpretation:

- this is the per-target file-selection helper used when multiple AlphaWord file entries exist

### `SendOneAlphaWordFileRecord`

- checks that the host-side file path exists
- opens the file and rejects zero-length content
- calls `OpenHostAlphaWordFile(...)` to obtain a handle/stream for the local file
- calls `SendAlphaWordFileRecordToDevice(..., file_slot, local_file_id)`
- marks the record as sent on success through `FUN_00482a70`

Interpretation:

- this is the concrete per-file send worker
- it is the boundary where the UI/controller layer hands control to the updater/transport layer

## Direct USB Bootstrap

As with retrieval, the direct USB path still depends on the DLL-level reset and updater switch path:

1. `?\xff\x00reset`
2. `?Swtch\x00\x00`

After that, the send flow enters updater commands.

## Reconstructed Direct USB AlphaWord Send Path

For one AlphaWord file slot, the current best reconstruction is:

1. reset direct USB connection
2. switch to updater mode with applet id `0`
3. send `0x1d` begin-attributes command for slot `n`, applet `0xa000`
4. send `0x02` attributes handshake with length `0x28` and checksum
5. send the `0x28` raw attributes bytes
6. poll `0xff` and expect `0x43`
7. send `0x1e` finish-attributes command
8. send `0x14` begin-file command with slot and file length
9. for each payload chunk up to `0x400` bytes:
   - send `0x02` chunk handshake with chunk length and checksum
   - send chunk bytes
   - poll `0xff` and expect `0x43`
10. send `0x15` finish-file command

## Confirmed AlphaWord-Specific Facts

- AlphaWord applet id is `0xa000`
- file slot is carried in the low byte for attributes begin/finish
- file slot is carried in the top byte of the put-file begin command
- attribute record size is `0x28`
- attribute offset `0x1c` is the big-endian file length
- send chunk size cap is `0x400`

## PoC Coverage

The offline PoC now models this send path at:

- [alphaword_send.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_send.py)

It can build:

- raw attribute send begin/data/finish steps
- file send begin/chunk/finish steps
- a direct USB single-record AlphaWord send sequence

## Remaining Unknowns

- the exact semantic role of `param_5` and `param_6` in `UpdaterPutFile`
- the exact resource id used by the `AlphaWord Files to Send` page/tab title at runtime
- whether there is an AlphaWord-specific prevalidation pass that modifies the `0x28` attribute record before transmit
