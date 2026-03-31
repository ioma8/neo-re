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

- `TransportWriteExact`: generic transport write loop
- `TransportReadExact`: generic transport read loop

`AsUSBCommReadData` stages inbound traffic in 8-byte chunks, so updater responses above this layer are assembled by repeated small reads rather than one raw device transfer.

Before the updater commands start, the direct USB path also uses the switch protocol:

- `DirectUsbEnterUpdaterApplet` and `AlternateTransportEnterUpdaterApplet` both do:
  - `AsUSBCommResetConnection()`
  - `AsUSBCommSwitchToApplet(0)`
  - `AsUSBCommIsAlphaSmartPresent()`
- in practice this means NeoManager first switches the device into updater-side mode using applet id `0`, not directly into AlphaWord

The offline PoC now models this bootstrap sequence at:

- [alphaword_flow.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_flow.py)

## Generic Updater Command Frame

`BuildUpdaterCommandPacket` builds every updater command packet.

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

`UpdaterGetRawFileAttributes` is the raw attribute retrieval routine.

Behavior confirmed by Ghidra:

- calls `UpdaterSendCommandAndGetResponse(..., timeout=600, command=0x13, argument=file_slot & 0xff, trailing=applet_id, ...)`
- expects first response byte `0x90` for the simple success case
- also handles response byte `0x5a` as the data-bearing attributes path
- reads the returned attribute bytes with `read_data1`
- verifies the returned 16-bit checksum against the byte sum of the attribute payload
- caller-provided receive storage is `0x28` bytes
- `CopyAppletFileNameFromRawAttributes` copies a NUL-terminated string directly from the first `0x18` bytes
- `NormalizeAlphaWordAttributeWords` endian-swaps two 32-bit words from offsets `0x18` and `0x1c`
- `UpdaterSaveAppletFileData` treats the big-endian word at offset `0x1c` as the file content length

Current best layout of the `0x28`-byte record, validated against both retrieve and restore/save callers:

- `0x00..0x17`: file name field
  - copied as a NUL-terminated byte string by `CopyAppletFileNameFromRawAttributes`
  - effectively a fixed `0x18`-byte name slot
- `0x18..0x1b`: big-endian reserved length / storage-footprint field
  - this is a strong inference from `FUN_00485950` and its callers, which use this word as a size-planning metric distinct from the real payload byte count
- `0x1c..0x1f`: big-endian file payload length
  - this is explicitly consumed by both `UpdaterSaveAppletFileData` and `UpdaterRestoreAppletFileData` to delimit the following payload bytes
- `0x20..0x27`: trailing opaque bytes
  - preserved when NeoManager round-trips raw records through save/restore
  - no validated semantic decoding of these final 8 bytes was found in the main AlphaWord path

The most concrete attribute command packet currently confirmed is:

```text
command = 0x13
argument = file_slot
trailing = applet_id
```

The offline PoC models this as `build_raw_file_attributes_command`.

### File content retrieval

`UpdaterRetrieveAppletFileData` retrieves the actual file contents.

Behavior confirmed by Ghidra:

- calls `UpdaterSendCommandAndGetResponse(..., timeout=10000, command=0x12 or 0x1c, argument=(requested_length << 8) | file_slot, trailing=applet_id, ...)`
- `param_7` selects command `0x12` vs `0x1c`
- expects first response byte `0x53` (`'S'`)
- the initial response `arg32` field is the total payload length NeoManager expects to receive
- then repeatedly requests chunk payloads using command `0x10`
- each chunk cycle expects response byte `0x4d` (`'M'`)
- each chunk response `arg32` field is the chunk byte count
- each chunk response trailing field is the expected 16-bit checksum of the chunk payload
- chunk data is read with `read_data1`
- chunk bytes are written to the destination sink with `WriteRetrievedBytesToSink`
- the byte-sum checksum of each chunk must equal that chunk response’s 16-bit trailing field

### Exact `UpdaterRetrieveAppletFileData` Flow

Fresh decompilation of `UpdaterRetrieveAppletFileData` now pins the exact root retrieval loop:

1. clear local accumulators and set `DAT_004c2b28 = 0`
2. if `param_9 == 0`:
   - format a progress caption string using `file_slot`
   - call the progress UI percentage helper with `0`
3. else:
   - compute aggregate progress as `(param_8 * 100) / param_9`
   - call the progress UI percentage helper with that value
4. choose the start command byte:
   - `0x12` when `param_7 == 0`
   - `0x1c` when `param_7 != 0`
5. send the start command through `UpdaterSendCommandAndGetResponse` with:
   - timeout `10000`
   - `arg32 = (requested_length << 8) | (file_slot & 0xff)`
   - `trailing = applet_id`
6. require response status byte `0x53`
7. if the caller provided `param_6`, write the reported total length into `*param_6`
8. cap the remaining transfer length to `min(reported_total_length, requested_length)`
9. loop while remaining length is nonzero:
   - update progress
   - send chunk command `0x10` with timeout `600`
   - require response status byte `0x4d`
   - take response `arg32` as the chunk length
   - repeatedly call `TransportReadExact(...)` until that many bytes are read
   - call `WriteRetrievedBytesToSink(...)` for each returned slice
   - sum every payload byte into a 16-bit running checksum
   - compare the final checksum against the chunk response trailing field
10. on success:
   - if `param_9 == 0`, set progress to `100` and close the standalone progress scope
   - else set aggregate progress to `((param_8 + capped_total_length) * 100) / param_9`
   - return `0`
11. on failure:
   - record a structured updater error entry
   - call `SetLastUpdaterErrorCode(...)`
   - if `param_9 == 0`, close the standalone progress scope
   - return `-1`

The exact error cases that are explicitly distinguished in the function are:

- start command transport failure
- start response byte not equal to `0x53`
- chunk command transport failure
- chunk response byte not equal to `0x4d`
- transport read failure while draining a chunk payload
- sink write failure while writing the retrieved bytes
- payload checksum mismatch

Important exact details from the root function:

- the root function itself enforces the `0x53` then `0x4d` response sequence
- the root function, not `RetrieveFullAlphaWordText`, caps the transfer length to the caller-requested maximum
- the root function accepts both standalone progress mode and aggregate nested-progress mode
- the root function returns `0` on success and `-1` on failure
- `command 0x12` vs `0x1c` is selected solely by `param_7`

### Meaning of `0x12` vs `0x1c`

Fresh caller analysis closes most of this distinction:

- `0x12` is the normal interactive retrieval opcode used by:
  - `RetrieveFullAlphaWordText`
  - `RetrieveAlphaWordPreviewText`
  - the `Get/Print AlphaWord Files` UI flows
- `0x1c` is the save/export retrieval opcode used by `UpdaterSaveAppletFileData`

Concrete evidence:

- `UpdaterSaveAppletFileData` calls `UpdaterRetrieveAppletFileData(..., param_7 = 1, ...)`
- `UpdaterRetrieveAppletFileData` converts `param_7 != 0` into opcode `0x1c`
- the symmetric send-side restore path already has the paired alternate opcode split:
  - normal put: `0x14`
  - restore/import put: `0x1f`

Best current interpretation:

- `0x12` = retrieve file content for normal interactive consumption
- `0x1c` = retrieve file content in the archive/save format used when exporting a complete applet-file bundle to a host file

This is still an inference from caller behavior and send/retrieve symmetry, but it is now a strong one.

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
- [alphaword_get_print.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_get_print.py) for the exact branch-level `UpdaterRetrieveAppletFileData` control flow

## Applet Discovery Layer

NeoManager does not hardcode only the AlphaWord applet id at the transport layer. It also walks the applet list.

`UpdaterGetAppletList`:

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

Two small wrappers select transport context before entering `UpdaterRetrieveAppletFileData`:

- `DirectUsbRetrieveAppletFileData`
- `AlternateTransportRetrieveAppletFileData`

These do not change the updater opcode. They prepare the transport context passed as `param_1` to `UpdaterRetrieveAppletFileData`:

- mode `2`: direct USB / single-device path
- mode `3`: alternate port-aware context used by the multi-device path

That distinction matters because earlier assembly could be misread as if `2` and `3` were file-retrieval command numbers. Ghidra makes clear they are transport selectors, not updater opcodes.

## Higher-Level AlphaWord Retrieval Helpers

Two UI-facing helpers sit on top of the raw retrieval routines.

### Full text retrieval helper

`RetrieveFullAlphaWordText`

Behavior:

- dispatches the current MFC thread hook once if a thread object is present
- calls `ResetRetrievedTextWorkspace`
- opens a temporary retrieved-text sink through `OpenTemporaryRetrievedTextFile`
- for direct USB mode `2` and mode `5`, calls `DirectUsbRetrieveAppletFileData(..., max_len=0x80000, ...)`
- for alternate transport mode `3`, calls `AlternateTransportRetrieveAppletFileData(..., max_len=0x80000, ...)`
- closes the file descriptor after the transport returns
- calls `LoadRetrievedTextFileAsCString`
- assigns the returned text into the caller-provided `CString`
- calls `FinalizeRetrievedTextWorkspace(0)` before returning

Interpretation:

- this path requests a large cap of `0x80000` bytes
- it is the "full retrieval" path for AlphaWord data that can later be printed or saved
- the helper always uses the temporary file workspace, even for direct USB

### Exact `RetrieveFullAlphaWordText` Subfunction Chain

Fresh decompilation now pins the exact helper chain for the full `0xa000` retrieval path:

1. `RetrieveFullAlphaWordText`
2. `ResetRetrievedTextWorkspace`
3. `OpenTemporaryRetrievedTextFile`
4. inside `OpenTemporaryRetrievedTextFile`:
   - `BuildRetrievedTextTempPathBase`
   - `OpenRetrievedTextSinkForWrite`
   - `WriteRetrievedBytesToSink`
   - `ReopenRetrievedTextSinkForRead`
5. one transport wrapper:
   - `DirectUsbRetrieveAppletFileData`
   - or `AlternateTransportRetrieveAppletFileData`
6. `UpdaterRetrieveAppletFileData`
7. `CloseFileDescriptor`
8. `LoadRetrievedTextFileAsCString`
9. `FinalizeRetrievedTextWorkspace`

Verified exact branch behavior:

- if the temp sink open fails, the function skips transport entirely and returns the current default result
- transport modes `2` and `5` share the direct-USB branch
- transport mode `3` uses the alternate-transport branch with the per-device selector byte from `this + param_1 * 4 + 0x10`
- other transport modes skip the transport call
- the full path does not apply the preview-only truncation logic

## Updater Error Log Table

The repeated writes to `DAT_004c31e0` inside `UpdaterRetrieveAppletFileData`, `UpdaterPutFileData`, `UpdaterPutRawFileAttributes`, and related helpers now make the global error table layout clear.

Each error record is a 7-field structure, 0x1c bytes wide:

1. `message` pointer
2. `operation` pointer
3. `source_file` pointer
4. `reserved` / context dword, currently written as `0` in these flows
5. `source_line` dword
6. `error_code` u16 plus `response_byte` u8 packed into the next slots
7. `detail` pointer or `0`

For the retrieve path, the most common concrete values are:

- `message`
  - `"Error retrieving file."`
  - `"Invalid response"`
  - `"Bad checksum."`
  - `"Error writing file."`
- `operation`
  - `"UpdaterRetrieveFile"`
- `source_file`
  - `"C:\\AS Software\\OS3000\\Tool\\HostU..."`
- `error_code`
  - `0x12a` generic retrieve failure
  - `0x102` invalid response
  - `0x105` bad checksum
  - `0x129` sink write failure
- `response_byte`
  - set for invalid-response cases
- `detail`
  - either `0`
  - or a decoded response-detail string from the `"Number of Device(s) compacted = %d."` table when the response byte is in the documented `0x80..0x92` range

The PoC now models this record shape as `UpdaterErrorLogEntry`.

### Exact `LoadRetrievedTextFileAsCString` Behavior

Fresh decompilation of `LoadRetrievedTextFileAsCString` supports these concrete claims:

- it rebuilds the same temporary path using string resource `0xf1a8` and `BuildRetrievedTextTempPathBase`
- it opens the temp file through MSVCP60 `basic_filebuf` / `basic_istream` types
- it reads exactly the byte count passed by the caller
- it rewrites embedded `0x00` bytes to ASCII space (`0x20`)
- when a caller-supplied flag is set, it also remaps some space bytes to `0xff`
- it converts the final byte buffer into a `CString`
- it deletes the temporary file before returning

That means the previously assumed generic "newline normalization" should not be treated as a proven property of the full retrieval path. The validated full-path behavior is byte cleanup plus temp-file-to-`CString` conversion.

### Exact Helper Replacement Strings

The data items used by the temp-file helpers are now concrete enough to name:

- `param_1_004bd0a0` = carriage return (`"\r"`, byte `0x0d`)
- `_Str2_004bd0a4` = CRLF (`"\r\n"`, bytes `0x0d 0x0a`)
- `param_2_004bd0a8` = space (`" "`, byte `0x20`)
- `_Format_004bd32c` = decimal slot formatter (`"%d"`)

What that means in code:

- `OpenTemporaryRetrievedTextFile` conditionally replaces `"\r\n"` with `"\r"` before seeding the temp sink
- `RetrieveAlphaWordPreviewText` later replaces `"\r\n"` with `" "` in the preview `CString`
- `LoadRetrievedTextFileAsCString` itself does not prove a CR-to-CRLF conversion step in the full retrieval path

### Preview / short printable text helper

`RetrieveAlphaWordPreviewText`

Behavior:

- opens a temporary output sink through `OpenTemporaryRetrievedTextFile`
- for direct USB mode, calls `DirectUsbRetrieveAppletFileData(..., max_len=0xb4, ...)`
- for alternate transport mode, calls `AlternateTransportRetrieveAppletFileData(..., max_len=0xb4, ...)`
- then converts the retrieved data through `LoadRetrievedTextFileAsCString`
- applies one final `CString::Replace(...)` post-processing step
- truncates the resulting string to `0xb3` bytes if needed

Interpretation:

- this is the short preview path used to populate UI summaries or list displays
- it is not the full-file retrieval path used for complete printing/export

## Confirmed UI-Side Callers

The tree-control message-map binding for the `Get/Print AlphaWord Files` page is now pinned even though the literal resource string itself still lives only in `.rsrc`.

`RefreshAlphaWordPreviewCacheForTreeItem`:

- repeatedly calls `RetrieveAlphaWordPreviewText(..., applet_id=0xa000, ...)`
- iterates through AlphaWord file slots
- stores the returned short text and size back into local cache objects
- is consistent with preview/list population for the `Get/Print AlphaWord Files` view

`RetrieveAllAlphaWordSlotsForDevice` and `RetrieveSelectedAlphaWordSlotsForDevice`:

- iterate through 8 AlphaWord file slots
- call `RetrieveFullAlphaWordText(..., applet_id=0xa000, ...)` for full retrieval
- then call `GetAppletFileNameForSlot(..., applet_id=0xa000, ...)` to obtain the human-facing per-slot file name used by the local cache
- update per-slot local storage with:
  - full retrieved text
  - retrieved byte count
  - file name
  - status `2`

This is enough to say the app path above the transport is not a single monolithic call. It is:

1. preview-oriented slot enumeration using `RetrieveAlphaWordPreviewText`
2. full slot retrieval using `RetrieveFullAlphaWordText`
3. cache/status updates for each of the 8 AlphaWord slots

`ScanAlphaWordPreviewSlots` adds one more confirmed layer above the single-slot preview helper:

- calls `RetrieveAlphaWordPreviewText(..., applet_id=0xa000, file_slot=1..8, ...)` in a loop
- scans all 8 AlphaWord slots for preview content
- treats preview text longer than `0x32` bytes specially for UI formatting
- is consistent with the higher-level dialog/page that assembles the printable selection summary

The PoC now models this app behavior as a session-level 8-slot scan at:

- [alphaword_session.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_session.py)

## Top-Level Controller Chain

Fresh Ghidra decompilation of the renamed controller functions shows the main app-side execution chain is:

1. `HandleAlphaWordTreeNotifications`
2. one of:
   - `HandleAlphaWordTreeExpandStateChange`
   - `UpdateAlphaWordButtonsForTreeSelection`
   - `HandleAlphaWordTreeDoubleClick`
3. `RefreshAlphaWordPreviewCacheForTreeItem` for preview fills
4. `ExecuteGetPrintAlphaWordFlow` for full retrieval
5. one of:
   - `RetrieveAllAlphaWordSlotsForDevice`
   - `RetrieveSelectedAlphaWordSlotsForDevice`
   - `RetrieveSingleAlphaWordSlotForDevice`
6. `RetrieveFullAlphaWordText`
7. `GetAppletFileNameForSlot`

What each layer does:

`HandleAlphaWordTreeNotifications`

- is the exact tree notification dispatcher bound to control id `0x8a8a`
- routes:
  - notification `-2` to `ToggleAlphaWordTreeCheckByMouse`
  - notification `-3` to `HandleAlphaWordTreeDoubleClick`
  - notification `-0x19c` to `ToggleAlphaWordTreeCheckByKeyboard`
  - notification `-0x192` to `UpdateAlphaWordButtonsForTreeSelection`
  - notification `-0x195` to `HandleAlphaWordTreeExpandStateChange`

`HandleAlphaWordTreeExpandStateChange`

- handles tree expand/collapse state changes for the AlphaWord tree
- when the tree action field is `2`, it calls `RefreshAlphaWordPreviewCacheForTreeItem`
- when the tree action field is `1`, it clears cached slot-selection/check state under the affected item instead of fetching preview text
- refreshes visible list/tree state after the preview/cache work

`UpdateAlphaWordButtonsForTreeSelection`

- is the separate plain-selection-state helper
- updates enabled/disabled button state based on whether the current tree selection resolves to a device item plus slot item
- does not fetch preview text

`RefreshAlphaWordPreviewCacheForTreeItem`

- is the preview-only population path
- repeatedly calls `RetrieveAlphaWordPreviewText`
- stores preview text and preview byte count into the local per-slot cache
- does not call `GetAppletFileNameForSlot`
- does not populate the full-text cache used by later retrieval/print flows

`HandleAlphaWordTreeDoubleClick`

- dispatches to `OpenSelectedAlphaWordSlotDialog`
- ensures the selected slot has been fully retrieved first when its cache status is not already `2`
- then opens the per-slot modal viewer dialog using the cached full text

`ExecuteGetPrintAlphaWordFlow`

- creates the progress UI
- sets the progress caption text
- chooses one of three full-retrieval branches:
  - all slots on the current device
  - selected slots only
  - one explicit slot

`RetrieveAllAlphaWordSlotsForDevice`

- walks slot numbers `1..8`
- calls `RetrieveFullAlphaWordText(..., applet_id=0xa000, ...)`
- calls `GetAppletFileNameForSlot(..., applet_id=0xa000, ...)`
- writes full text, byte count, and file name into the local cache
- marks each populated slot with status `2`

### Exact `RetrieveAllAlphaWordSlotsForDevice` Flow

Fresh decompilation of `RetrieveAllAlphaWordSlotsForDevice` with the renamed helper accessors gives the exact function-local sequence:

1. call `FUN_00401260()` to obtain the global empty `CString` sentinel
2. if `param_1 < 1`, return `0x51`
3. load string resource `0xf1ba`
4. initialize:
   - `slot_number = 1`
   - `slot_index = 0`
5. for each AlphaWord slot while `slot_index < 8`:
   - call `FormatAlphaWordSlotProgressText(this, ..., device_ref=param_1, slot_ref=slot_number)`
   - call `SetProgressDialogTextIfWindowAlive(param_2, formatted_text)`
   - call `GetAlphaWordSlotStatus(device_cache, slot_index)`
   - if status is already `2`, skip the rest of this slot and continue
   - otherwise:
     - remember the current slot status as `previous_status`
     - clear the destination `CString`
     - call `RetrieveFullAlphaWordText(..., device_ref=param_1, file_slot=slot_number, applet_id=0xa000, ..., max_len=0x80000, alternate_mode=0)`
     - if that returns `-1`:
       - call `SetAlphaWordSlotStatus(device_cache, slot_index, previous_status)`
       - return `0x2a`
     - call `SetAlphaWordSlotText(device_cache, slot_index, full_text)`
     - call `SetAlphaWordSlotByteCount(device_cache, slot_index, retrieved_length)`
     - call `GetAppletFileNameForSlot(..., device_ref=param_1, file_slot=slot_number, applet_id=0xa000, ...)`
     - call `SetAlphaWordSlotFileName(device_cache, slot_index, file_name)`
     - call `SetAlphaWordSlotStatus(device_cache, slot_index, 2)`
     - call `GetProgressDialogCancelCode(param_2)`
     - if that returns `0x33`:
       - call `FUN_00473470(this->+0x6c, 0)` to tear down the progress binding
       - return `0x29`
   - increment both:
     - `slot_number += 1`
     - `slot_index += 1`
6. after all 8 slots, return `0`

Important exact details from the decompile:

- `slot_number` is 1-based and is the value passed on wire to `RetrieveFullAlphaWordText` and `GetAppletFileNameForSlot`
- `slot_index` is 0-based and is the index used into the local cache object
- the function does not clear old text or metadata for slots already at status `2`; it skips them entirely
- the previous slot status is only restored on retrieval failure, not on user cancellation
- cancellation is checked only after a successful full retrieval, filename fetch, and status promotion to `2`

`RetrieveSelectedAlphaWordSlotsForDevice`

- same as the all-slot variant, but skips slots whose selection bit is clear in the local cache object

`RetrieveSingleAlphaWordSlotForDevice`

- performs the same full retrieval and filename fetch sequence for exactly one slot
- preserves the prior slot status on retrieval failure and returns `0x2a`
- returns `0x29` when the progress UI reports cancellation

This resolves the earlier ambiguity around `GetAppletFileNameForSlot`: it is part of the full `Get/Print AlphaWord Files` cache population path and exists specifically to attach the user-visible filename to the retrieved full text.

## Text Formatting and Print-Side Handoff

`LoadRetrievedTextFileAsCString` is the local formatter / file-backed text loader that runs after retrieval.

Key observed behavior:

- rebuilds the temporary path through resource `0xf1a8` and `BuildRetrievedTextTempPathBase`
- opens the temporary file with MSVCP60 stream types
- reads exactly the requested byte count from the temp file
- rewrites embedded `0x00` bytes to spaces
- optionally remaps some spaces to `0xff` depending on the caller flag
- deletes the temporary file after loading it

Interpretation:

- the updater layer retrieves raw AlphaWord bytes into a local sink
- `LoadRetrievedTextFileAsCString` re-reads that local file and turns it into a `CString`
- this is the seam where "retrieve from device" ends and "prepare for display/printing" begins

## End-to-End Direct Dataflow

For the direct USB case, the current best reconstruction is:

1. User enters `Get/Print AlphaWord Files`.
2. NeoManager identifies the direct NEO transport and uses the `AsUSBComm*` path.
3. The direct USB setup path resets the transport and issues `AsUSBCommSwitchToApplet(0)` to enter updater-side mode.
4. The updater layer issues command `0x04` to enumerate applets and confirms the AlphaWord applet context.
5. AlphaWord file metadata is fetched through `UpdaterGetRawFileAttributes` using opcode `0x13`, `argument=file_slot`, and `trailing=0xa000`.
6. Full AlphaWord file contents are fetched through `UpdaterRetrieveAppletFileData` using opcode `0x12` or `0x1c`, `argument=(requested_length << 8) | file_slot`, and `trailing=0xa000`.
7. The device returns an initial `0x53` response containing the total byte count.
8. NeoManager repeatedly issues command `0x10`, receives `0x4d` chunk headers, reads each chunk body, and verifies each chunk checksum.
9. Retrieved bytes are written to a local temporary sink.
10. `LoadRetrievedTextFileAsCString` opens the temporary file, converts it into `CString` text, normalizes line endings, and returns printable text.
11. The UI uses:
   - `RetrieveAlphaWordPreviewText` for short preview text capped at `0xb4`
   - `RetrieveFullAlphaWordText` for full retrieval capped at `0x80000`
12. Higher-level UI code loops over all 8 AlphaWord slots:
   - preview/session scan callers include `RefreshAlphaWordPreviewCacheForTreeItem` and `ScanAlphaWordPreviewSlots`
   - full retrieval callers include `RetrieveAllAlphaWordSlotsForDevice` and `RetrieveSelectedAlphaWordSlotsForDevice`

## Non-Core Sibling Path

Fresh caller checks for `RetrieveFullAlphaWordText` also found `0x0044a140` and `0x0044a5c0`, but those pass applet id `0xa004`, not `0xa000`.

That means they are not the core `Get/Print AlphaWord Files` path documented here. They belong to a sibling AlphaWord-like document parser / renderer flow and should not be used as evidence for the main `0xa000` direct-USB retrieval sequence.

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
- raw attribute offsets `0x00..0x17` are the fixed name field copied by `CopyAppletFileNameFromRawAttributes`
- raw attribute offsets `0x18` and `0x1c` are big-endian 32-bit values used by NeoManager
- raw attribute offset `0x18` is best understood as a reserved length / storage-footprint field distinct from payload byte count
- raw attribute offset `0x1c` is the file content length
- raw attribute offsets `0x20..0x27` are preserved trailing bytes with no validated higher-level semantics yet
- applet list command is `0x04` with page size `7`
- applet list entry size is `0x84`

## Offline PoC Coverage

The PoC now covers both the updater protocol and the controller-level `Get/Print AlphaWord Files` flow:

- [alphaword_flow.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_flow.py): raw AlphaWord updater packets for preview and full retrieval
- [alphaword_transfer.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_transfer.py): start/chunk reconstruction with checksum checks
- [alphaword_session.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_session.py): 8-slot preview and full direct-USB sessions
- [alphaword_get_print.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_get_print.py): controller-level preview refresh and full get/print retrieval flows, including the exact tree notification dispatch for control `0x8a8a`
- [alphaword_get_print.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_get_print.py): also includes an exact branch-level model of `RetrieveAllAlphaWordSlotsForDevice`, including skip-on-status-2, restore-on-error, and cancel handling
- [alphaword_get_print.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_get_print.py): also includes an exact helper-level model of `RetrieveFullAlphaWordText`, including temp-sink setup, transport dispatch, temp-file load, and workspace finalization
- [alphaword_attributes.py](/Users/jakubkolcar/customs/neo-re/poc/neotools/src/neotools/alphaword_attributes.py): exact `0x28` raw-attribute parsing for the name field, reserved-length word, payload-length word, and trailing opaque bytes

Example commands:

```bash
uv run --project poc/neotools python -m neotools direct-usb-alphaword-session preview 0xa000
uv run --project poc/neotools python -m neotools alphaword-get-print-flow preview-all 0xa000 1 2
uv run --project poc/neotools python -m neotools alphaword-get-print-flow full-selected 0xa000 2 4
uv run --project poc/neotools python -m neotools retrieve-all-alphaword-slots-flow 0xa000 --file-slots 1 2 3 --initial-statuses 1=2 2=0 3=2
uv run --project poc/neotools python -m neotools retrieve-full-alphaword-text-flow 2 0xa000 2 --retrieved-length 0x1234
uv run --project poc/neotools python -m neotools updater-retrieve-applet-file-data-flow 0x12 0xa000 2 0x80000 --reported-total-length 0x30 --chunk-lengths 0x10 0x20 --chunk-checksums 0x10 0x20
```

## Still Unresolved

- the final semantic name of the size word at offset `0x18`
  - current best reading is reserved length / storage footprint, but NeoManager does not label it explicitly
- the exact semantic meaning of the trailing raw-attribute bytes at offsets `0x20..0x27`
- the exact tree-population helper that inserts the literal `Get/Print AlphaWord Files` resource string into the UI hierarchy
- the exact downstream print formatter function that consumes the fully cached `0xa000` text after `ExecuteGetPrintAlphaWordFlow`
