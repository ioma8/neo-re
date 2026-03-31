# SmartApplets Dataflow

This note maps the SmartApplet binary flow in both directions:

- app to device: install or replace SmartApplets on the NEO
- device to app: retrieve SmartApplet binaries and the applet list from the NEO

The focus here is the direct updater protocol and the concrete app-side call chain that reaches it.

## Confirmed Updater Primitives

These functions are now pinned from string xrefs and decompilation:

- `FUN_00430470` = `UpdaterGetAppletList`
- `FUN_004309d0` = `UpdaterAddApplet`
- `FUN_004328f0` = `UpdaterRemoveApplet`
- `FUN_00433b10` = `UpdaterRetrieveApplet`
- `FUN_00435b00` = `UpdaterSaveAppletFileData`

Transport wrappers:

- `FUN_00430410` wraps `UpdaterGetAppletList` for direct USB mode `2`
- `FUN_00430440` wraps `UpdaterGetAppletList` for alternate mode `3`
- `FUN_00433ab0` wraps `UpdaterRetrieveApplet` for direct USB mode `2`
- `FUN_00433ae0` wraps `UpdaterRetrieveApplet` for alternate mode `3`

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

- `FUN_00484c50`
- `FUN_00484e40`

Those functions swap selected big-endian dwords inside each `0x84` record before the UI uses them.

The important structural conclusion is that the device-side `0x84` list entry and the first `0x84` bytes of a host `.OS3KApp` file use the same metadata layout. The same parser object is used for both:

- list entry path: `FUN_00484e40` -> `FUN_004667e0` / `FUN_00466f40` -> `FUN_00476d60`
- on-disk file path: `FUN_00476650` -> `FUN_00476d60`

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

- bit `0x10000000` is extracted into one UI boolean
- bit `0x00010000` is extracted into a second UI boolean
- bit `0x40000000` is extracted into a third UI boolean

The exact user-facing names for those three booleans are still unresolved, but the extraction sites are now pinned:

- on-disk path: `FUN_00476650`
- in-memory metadata path: `FUN_00476a40`

Observed examples:

- `alphawordplus.os3kapp`: applet id `0xa000`, version `3.4`, class `0x01`
- `calculator.os3kapp`: applet id `0xa002`, version `3.0`, class `0x01`
- `keywordswireless.os3kapp`: applet id `0xa004`, version `4.0`, class `0x04`

## Retrieve Applet From Device

`UpdaterRetrieveApplet` is `FUN_00433b10`.

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
6. host writes those bytes to the destination file with `FUN_004383d0`
7. verify the 16-bit checksum for each chunk

The minimal direct USB session is therefore:

1. `?\xff\x00reset`
2. `?Swtch 0000`
3. `0x0f` retrieve-applet command
4. repeated `0x10` chunk requests until the announced size is satisfied

## Send Applet To Device

`UpdaterAddApplet` is `FUN_004309d0`.

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

`UpdaterSaveAppletFileData` is `FUN_00435b00`.

This is the device-to-host bulk saver for an applet’s associated file data:

1. `FUN_00435990` discovers the total payload size and record count
2. host writes a 4-byte total-size prefix
3. for each file slot:
4. `FUN_00436200` retrieves the raw `0x28` file-attribute record
5. host writes the `0x28` record
6. file length is decoded from the attributes
7. `FUN_00434100` retrieves the actual file bytes

This mirrors the AlphaWord file-data flow, but under the currently selected SmartApplet id.

## Embedded SmartApplet Info Table

Header offset `0x0c` is not another size field. It is an offset inside the `.OS3KApp` image to a variable-length info table that the UI uses for SmartApplet details, settings labels, and file-information strings.

Confirmed from the on-disk parser path:

- `FUN_00476650` reads header offset `0x0c`
- if nonzero, `FUN_00478840` walks a table at that file offset
- `FUN_00477e90` resolves strings from individual table records by a `(group, key)` pair

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

Those are instantiated by `FUN_004774b0` / `FUN_00477ac0` after `FUN_00477020` walks the info table.

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

- `FUN_004665f0`
- `FUN_00467980`
- `FUN_00484e40`
- `FUN_00430410` or `FUN_00430440`
- `FUN_00430470`

Meaning:

- `FUN_004665f0` refreshes the SmartApplet/device view
- `FUN_00467980` triggers the list retrieval and hands the normalized `0x84` entries to the UI layer
- `FUN_00484e40` is the concrete applet-list adapter over `UpdaterGetAppletList`

### Install SmartApplet On Device

- `FUN_0041eb80`
- `FUN_00427270`
- `FUN_00427810`
- `FUN_00472ee0`
- `FUN_00484160` or `FUN_00485690`
- `FUN_00430970` or `FUN_004309a0`
- `FUN_004309d0`

Meaning:

- `FUN_00430970` is the direct USB wrapper into `UpdaterAddApplet`
- `FUN_004309a0` is the alternate mode `4` wrapper into `UpdaterAddApplet`
- `FUN_00484160` and `FUN_00485690` open the host SmartApplet file and dispatch it to those wrappers
- `FUN_0041afd0` is now pinned as the top-level send/install workflow controller that eventually reaches `FUN_0041eb80` and the SmartApplet install helpers
- the higher-level controller entry points above are the install-side callers currently confirmed by xrefs

### Retrieve SmartApplet To Host

- `FUN_00423df0`
- `FUN_004286c0`
- `FUN_004851f0`
- `FUN_00433ab0` or `FUN_00433ae0`
- `FUN_00433b10`

and also:

- `FUN_004191d0`
- `FUN_004851f0`
- `FUN_00433ab0` or `FUN_00433ae0`
- `FUN_00433b10`

Meaning:

- `FUN_004851f0` opens a host-side output file and retrieves one SmartApplet binary into it
- `FUN_004286c0` retrieves one selected SmartApplet entry through that helper
- `FUN_00423df0` iterates the SmartApplet send-list entries and retrieves missing applets to the host workspace
- `FUN_004191d0` is another controller path that can retrieve selected applets to host storage
- `FUN_00417d30` is the higher-level retrieval controller that sets up the progress dialog, iterates selected devices, then calls `FUN_004191d0`

### Refresh SmartApplet Info / Details View

- `FUN_004662a0`
- `FUN_004665f0`
- `FUN_00467980`
- `FUN_00484e40`

Meaning:

- `FUN_004662a0` is the controller that clears and rebuilds the SmartApplet info/details pane
- it calls `FUN_004665f0`, which refreshes the applet inventory and injects AlphaWord-specific preview text when applet id `0xa000` is present
- `FUN_00467980` and `FUN_00484e40` provide the normalized `0x84` SmartApplet records that feed that view

## Resource Strings And Popup Menus

`r2` resource decoding and `ghydra` decompilation now pin the AlphaWord-specific SmartApplet labels and the three relevant popup menus.

Confirmed STRINGTABLE entries from resource block `3860`:

- `0xf138` = `Maximum File Size (in characters)`
- `0xf139` = `Minimum File Size (in characters)`
- `0xf13a` = `Updating AlphaWord file size limits.`

Those strings are consumed by the AlphaWord-specific SmartApplet settings code:

- `FUN_004774b0` compares generated SmartApplet setting labels against `0xf138` and `0xf139` when applet id `0xa000` is active
- `FUN_004867d0` uses `0xf138` to find and update the current AlphaWord maximum-size row
- `FUN_00487440` uses `0xf138` to validate the edited numeric value against the row-local min/max bounds at offsets `0x48` and `0x4c`

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
uv run --project poc/neotools python -m neotools smartapplet-string 0xf138
uv run --project poc/neotools python -m neotools smartapplet-menu 163
uv run --project poc/neotools python -m neotools smartapplet-add-plan-from-image "<full .OS3KApp hex>"
```

## Remaining Unknowns

- the exact semantic names of the three proven flag bits in the flags dword at offset `0x10`
- the exact human-readable mapping behind every `applet_class` byte value at offset `0x3f`
- the final MFC command-handler binding for every decoded popup-menu id
