# NeoManager Direct USB Driver Notes

Date: 2026-03-31

## Scope

This note covers only the direct USB path used for a single NEO device connected straight to the PC.

Out of scope for this note:

- AlphaHub transport
- Cart or lab broadcast behavior
- Higher-level application features unrelated to transport

## Relevant Components

Direct USB uses these binaries:

- `/Users/jakubkolcar/customs/neo-re/analysis/cab/asusbcomm.dll`
- `/Users/jakubkolcar/customs/neo-re/analysis/cab/asusbdrvxp.sys`
- `/Users/jakubkolcar/customs/neo-re/analysis/driver64_cab/asusbdrv64.sys`

The Windows INF also binds the device:

- `/Users/jakubkolcar/customs/neo-re/analysis/cab/asusbdrv.inf`

## Device Identity

From `asusbdrv.inf`:

- NEO device: `USB\\VID_081e&PID_bd01`
- AlphaHub: `USB\\VID_081e&PID_0100`

For the direct path we only care about:

- `USB\\VID_081e&PID_bd01`

## Driver Object Names

The 32-bit driver contains these Unicode strings:

- `\\Device\\AsUsbDrv`
- `\\DosDevices\\AsUsbDrv`

The user-mode DLL formats names as:

- `\\\\.\\AsUSBDrv%d`

Inference:

- The driver likely exposes one numbered DOS-visible device instance per attached device.
- `AsUSBCommIsAlphaSmartPresent` appears to probe a sequence of instances until it finds a working one.

## DriverEntry Mapping

`DriverEntry` in `asusbdrvxp.sys` initializes the major-function table directly.

Key assignments observed:

- `MajorFunction[0x00]` / create path starts near `0x102d2`
- `MajorFunction[0x02]` / close path starts near `0x10592`
- `MajorFunction[0x0E]` / device control starts near `0x108e4`

The `0x108e4` mapping is important because `0x0E` is `IRP_MJ_DEVICE_CONTROL`.

The 64-bit driver `asusbdrv64.sys` mirrors the same shape. `FUN_00011008` is the effective `DriverEntry` body and installs:

- `IRP_MJ_CREATE` -> `0x00011400`
- `IRP_MJ_CLOSE` -> `0x00011528`
- `IRP_MJ_DEVICE_CONTROL` -> `0x000115a4`
- `IRP_MJ_PNP` -> `0x000119e4`
- `IRP_MJ_POWER` -> `0x00012e98`
- `IRP_MJ_SYSTEM_CONTROL` -> `0x00013dd0`
- `DriverUnload` -> `0x00012af8`

The add-device path is `0x000110ac`. It:

- creates numbered device objects `\\Device\\AsUsbDrv0` .. `\\Device\\AsUsbDrv255`
- attaches to the lower PDO stack
- creates matching symbolic links `\\DosDevices\\AsUsbDrv0` .. `\\DosDevices\\AsUsbDrv255`
- initializes the device-extension state, events, queues, and WDM-version classification word

## User-Mode DLL Behavior

### Exported direct USB API

`asusbcomm.dll` exports:

- `AsUSBCommIsAlphaSmartPresent`
- `AsUSBCommReadData`
- `AsUSBCommWriteData`
- `AsUSBCommSwitchToApplet`
- `AsUSBCommResetConnection`
- `AsUSBUpdater_BuildCommand`
- `rlAsUSBUpdaterGetMACAddress`
- `rlAsUSBUpdaterSetMACAddress`

### `AsUSBCommWriteData`

Observed behavior:

- Fails immediately with return code `3` if the device handle is invalid.
- Uses `WriteFile`, not `DeviceIoControl`, for the bulk of the payload transfer.
- Splits outgoing writes into chunks of at most `0x40` bytes.
- Clears the DLL-global staged-read byte count before starting the write loop.
- On write failure it calls `AsUSBCommResetConnection` and returns `1`.
- On success it returns `0`.

Inference:

- The transport write path is stream-like.
- The `0x40` byte limit is probably a packet or endpoint max transfer size on this layer.

### `AsUSBCommReadData`

Observed behavior:

- Fails with return code `3` if the device handle is invalid.
- Validates requested read size against the caller-provided maximum; one failure path returns `0x0b`.
- Uses `GetTickCount` and a deadline argument for timeout handling.
- Uses `ReadFile` in 8-byte refill requests and stages the returned bytes in a persistent DLL-global buffer.
- Drains that staging buffer one byte at a time into the caller output.
- Tracks byte counts in globals and caller output pointers.
- Calls `AsUSBCommResetConnection` on error and returns `1`.
- Returns `0x0c` on timeout-style failure.
- Returns `0` on success.
- Exact return codes recovered from the decompile:
  - `3` if the handle is invalid
  - `0x0b` if `max_len < min_required`
  - `1` on `ReadFile` failure after reset
  - `0x0c` on timeout
  - `0` on success

Inference:

- Reads are not plain raw stream reads.
- The DLL maintains unread tail bytes across read-loop iterations, and likely across successive reads as long as the global handle stays valid.
- The transport naturally exposes 8-byte inbound chunks at this layer, even though the caller API asks for arbitrary byte counts.
- Live macOS/libusb testing confirmed the same behavior: a `0x28` AlphaWord attributes payload arrived as five 8-byte reads. User-mode `read_exact` logic must accumulate short reads before parsing the next 8-byte updater header.

### `AsUSBCommSwitchToApplet`

Observed behavior:

- Sends an 8-byte reset preamble before the switch command:
  - `3f ff 00 72 65 73 65 74`
  - ASCII form: `?\xff\x00reset`
- Writes an 8-byte command block via `WriteFile`.
- The command starts with the string prefix `"?Swtch"`.
- The remaining bytes are the target applet ID encoded as a big-endian 16-bit value.
- Sleeps `500` ms before reading the reply.
- It then reads back an 8-byte response via `ReadFile`.
- Response strings checked in the DLL:
  - `Switched`
  - `NoSwitch`
  - `NoApplet`

Return behavior:

- `3` if the handle is invalid
- `1` on write or read failure after resetting the connection
- `5` if the response length is not 8 bytes
- `4` or `2` depending on negative textual response
- `0` on successful switch
- `3` on any other 8-byte textual response

Inference:

- Applet switching is a very small command/response protocol.
- The command and response framing is fixed-width at 8 bytes.
- The on-wire switch packet format is:
  - bytes `0..5`: ASCII `?Swtch`
  - bytes `6..7`: applet ID in big-endian order
- The reset preamble is distinct from the switch packet and should be modeled as a separate fixed 8-byte write.

Relevant higher-level callers in `neomanager.exe`:

- `DirectUsbEnterUpdaterApplet`
- `AlternateTransportEnterUpdaterApplet`

Both do the same direct USB bootstrap pattern:

1. `AsUSBCommResetConnection()`
2. `AsUSBCommSwitchToApplet(0)`
3. `AsUSBCommIsAlphaSmartPresent()`

Interpretation:

- NeoManager first switches the device into updater-side mode using applet id `0`.
- Later AlphaWord retrieval happens through updater packets on top of that transport, rather than by switching directly to applet `0xa000` at the USB DLL boundary.

### `AsUSBCommIsAlphaSmartPresent`

Observed behavior:

- Formats `\\\\.\\AsUSBDrv%d` and probes up to `0x100` possible instances.
- Opens each candidate with `CreateFileA`.
- Uses `DeviceIoControl` with control code `0x80002000`.
- Expects an 18-byte output buffer on that transaction.
- Validates the returned descriptor as vendor `0x081e`, product `0xbd01`.
- Classifies matching devices by the descriptor `bcdDevice` field:
  - `1` => caches `1` and returns `1`
  - `2` => caches `3` and returns `3`
  - anything else => caches `2` and returns `2`
- Falls back to `EnumerateAsUsbHidInterfacesFallback` only if the numbered `AsUSBDrv` probe fails.

Inference:

- The 18-byte buffer length matches a USB device descriptor size, which is a strong hint that this call is retrieving a device descriptor for identification.
- The practical user-mode discriminator for the direct NEO path is the USB device descriptor plus the `bcdDevice` classification value.

### `AsUSBCommResetConnection`

Observed behavior:

- Closes the current handle.
- Restores the handle to `INVALID_HANDLE_VALUE`.
- Clears the cached presence classification.
- Clears the staged-read byte count.

Interpretation:

- Reset clears the whole DLL-global transport state, not just the open handle.

### MAC helper exports

`rlAsUSBUpdaterSetMACAddress`:

- Builds updater opcode `0x20`.
- Packs bytes `2..7` of the caller buffer into the updater packet:
  - `arg32 = source[2:6]` as big-endian
  - `trailing = source[6:8]` as big-endian
- Writes the 8-byte updater packet.
- Sleeps `600` ms.
- Reads back 8 bytes via `AsUSBCommReadData(..., max_len=8, min_required=0, timeout=0)`.
- If the read succeeds, it returns `-1` unless the first reply byte is `0x20`.

`rlAsUSBUpdaterGetMACAddress`:

- Builds updater opcode `0x00`.
- Writes the 8-byte updater packet and requires exactly 8 bytes written.
- Reads an 8-byte header via `AsUSBCommReadData(..., timeout=200)`.
- Continues only if the first reply byte is `'@'`.
- Clears a 64-byte local buffer and fills it with eight 8-byte reads:
  - one read into the first 8 bytes
  - seven more reads into the remaining 56 bytes
- On a complete 64-byte receive, copies the final 8 bytes to the caller output buffer.
- The function still returns the original write result code even if later reads fail, so a partial receive can leave return code `0` with no MAC output.

### HID fallback path

`EnumerateAsUsbHidInterfacesFallback` and `ProbeAndInitializeAsUsbHidInterface` are now decompiled well enough to characterize:

- `EnumerateAsUsbHidInterfacesFallback` uses `HidD_GetHidGuid` and `SetupDi*` enumeration first.
- If that path exhausts, it retries with literal GUID `{884B96C3-56EF-11D1-BC8C-00A0C91405DD}`.
- `ProbeAndInitializeAsUsbHidInterface` opens each HID path, validates vendor `0x081e` and product `0xbd04`, then performs one of two exact init sequences followed by `Sleep(2000)`:
  - newer Windows branch (`os_major_version > 4`):
    - `DeviceIoControl(0x0b0040, in=NULL, in_len=0, out_len=4)`
    - `DeviceIoControl(0x0b0008, payload=05 00 00 00)`
    - `DeviceIoControl(0x0b0008, payload=02 00 00 00)`
    - `DeviceIoControl(0x0b0008, payload=04 00 00 00)`
    - `DeviceIoControl(0x0b0008, payload=01 00 00 00)`
    - `DeviceIoControl(0x0b0008, payload=06 00 00 00)`
    - `DeviceIoControl(0x0b0008, payload=07 00 00 00)`
  - legacy branch (`os_major_version <= 4`):
    - `WriteFile(00 e0)`
    - `WriteFile(00 e1)`
    - `WriteFile(00 e2)`
    - `WriteFile(00 e3)`
    - `WriteFile(00 e4)`

Interpretation:

- HID is part of discovery and direct-mode initialization. A NEO can attach first as `081e:bd04`, a standard HID boot keyboard with only interrupt IN plus keyboard LED output-report support.
- The physical NEO tested on macOS did **not** switch when given the newer 4-byte `05 02 04 01 06 07` sequence, even though those USB/HID writes completed successfully.
- The same physical NEO **did** switch to direct USB mode when given the legacy one-byte output-report sequence `e0 e1 e2 e3 e4`.
- After the switch the device re-enumerates as `081e:bd01` with direct endpoints:
  - interface `0`
  - bulk OUT endpoint `0x01`, max packet `64`
  - bulk IN endpoint `0x82`, max packet `64`
- HID is not the main AlphaWord direct-USB data path after this point. AlphaWord traffic runs over the `081e:bd01` bulk endpoints, corresponding to the Windows `\\\\.\\AsUSBDrv%d` `WriteFile` / `ReadFile` path.

macOS transport notes from live testing:

- hidapi can enumerate the `081e:bd04` keyboard path but fails to open it as a keyboard-class HID device.
- PyUSB can find the device, but its managed `ctrl_transfer` path tries to claim interface `0` and fails on macOS for the HID keyboard interface.
- Direct `libusb_control_transfer` works when it sends HID class `SET_REPORT` requests without claiming the interface.
- The working macOS switch request is:
  - `bmRequestType = 0x21` (`host-to-device | class | interface`)
  - `bRequest = 0x09` (`SET_REPORT`)
  - `wValue = 0x0200` (`output report`, report id `0`)
  - `wIndex = 0`
  - payloads: `e0`, `e1`, `e2`, `e3`, `e4`

### Device-side direct-mode status screen

When the physical NEO switches into normal direct USB mode, its LCD shows:

```text
Connected to NEO Manager.
```

That message is generated by device firmware, not sent by NeoManager or by `real-check`.

The exact path was found in the installer ROM image `analysis/cab/os3kneorom.os3kos`:

This ROM area maps as `runtime address = file offset + 0x410000`.

- Runtime `0x00410b26`, file offset `0x0b26`, is the normal direct-mode entry/status setup callback.
- At runtime `0x00410b38`, file offset `0x0b38`, it writes `0x05` to global byte `0x444`.
- It then calls runtime `0x00412c82`, file offset `0x2c82`, to initialize direct-mode globals.
- It then calls runtime `0x004109ca`, file offset `0x09ca`, the status-screen renderer.
- The renderer switches on global byte `0x444`; state `5` reaches runtime `0x00410ae0`, file offset `0x0ae0`.
- That state pushes string resource id `0x8c`, resolves it through runtime `0x00424212`, and draws it through runtime `0x00421e08`.

Relevant disassembly:

```asm
; runtime 0x00410b26, file offset 0x0b26
0x00410b26  jsr     0x426b18
0x00410b2c  pea.l   0x410b60
0x00410b32  jsr     0x424f66
0x00410b38  move.b  #0x05, 0x00000444
0x00410b40  bsr.w   0x412c82
0x00410b44  bsr.w   0x4109ca

; renderer state 5, runtime 0x00410ae0, file offset 0x0ae0
0x00410ae0  jsr     0x421fe0
0x00410ae2  pea.l   0x008c
0x00410ae6  jsr     0x424212
0x00410aea  move.l  d0, -(a7)
0x00410aec  jsr     0x421e08
```

The resource pointer table maps resource `0x8c` to the exact string bytes:

```text
table offset 0x3b26c -> ptr 0044:693b -> file offset 0x3693b -> "Connected to NEO Manager."
```

The ROM contains duplicate copies of this same text at file offsets `0x3660d`, `0x36904`, `0x3693b`, and `0x36955`. The normal direct-mode visible status path uses the `0x8c` resource at file offset `0x3693b`.

Static inventory of the status byte and callback registration:

- Global byte `0x444` is the visible USB/status-screen state consumed by the renderer at runtime `0x004109ca`.
- The only immediate write of state `5` is at runtime `0x00410b38`, file offset `0x0b38`.
- The startup/event dispatcher registers the state-5 callback exactly once:

```asm
; dispatcher entry, runtime 0x004100a0, file offset 0x00a0
; command/event 0x19 reaches the branch below through the lifecycle jump table
; runtime 0x00410184, file offset 0x0184
0x00410184  pea.l   0x11111
0x0041018a  pea.l   0x2580
0x0041018e  jsr     0x424fb0
0x00410194  pea.l   0x410b26
0x0041019a  jsr     0x424f66
```

- Control-flow trace, not string search: `0x004100a0` is the top-level System/ROM SmartApplet dispatcher. For low lifecycle commands it subtracts `0x18` and jumps through the table at runtime `0x0041014a`. Table entry `0x19` is the only entry that reaches runtime `0x0041016e`, which clears status, initializes USB/direct globals, calls `0x00424fb0(0x2580, 0x11111)`, and registers callback `0x00410b26` through `0x00424f66`.
- `0x00424f66` is a callback registration helper. It stores the function pointer in RAM global `0x5d9c` and updates hardware/status flags at `0x5da4`. The registered callback is invoked indirectly by low-level routines at `0x00413908` and `0x00425260` when their hardware status bits show pending input. Those callers pass two byte-sized values from hardware registers; they are not local menu/key handlers that directly enter NEO Manager mode.
- Once callback `0x00410b26` runs and marks the device connected, it installs `0x00410b60` as the subsequent direct packet handler:

```asm
0x00410b2c  pea.l   0x410b60
0x00410b32  jsr     0x424f66
```

- `0x00410b60` accumulates 8-byte direct-mode command headers and dispatches them through `0x00410c82`. It is not another HID-to-direct activation path; it runs after the direct-mode callback has already executed.

The other writes to global `0x444` are init/disconnect/intermediate status values:

| File offset | Runtime address | Write | Meaning from surrounding code |
| ---: | ---: | --- | --- |
| `0x0178` | `0x00410178` | clear | startup/event init |
| `0x020e` | `0x0041020e` | `1` | non-direct intermediate status |
| `0x0940` | `0x00410940` | `4` | `0x50001` communication-driver event, followed by direct globals init and status repaint |
| `0x09be` | `0x004109be` | clear | `0x5000c` disconnect/suspend cleanup |
| `0x0b3c` | `0x00410b3c` | `5` | normal direct-mode connected callback |

This means duplicate `Connected to NEO Manager.` strings are not, by themselves, evidence of additional activation paths. The ROM has one normal state-5 status callback; the remaining question is which lower-level USB/HID events can reach it.

Current answer for device-side activation: no local UI/key path has been found that directly calls `0x00410b26` or directly writes visible state `5` to global `0x444`. Static scans across the installer ROM, Small ROM, live installed applets, and shipped applets found only one exact `pea.l 0x410b26 ; jsr 0x424f66` registration site, and it is the lifecycle `0x19` path above. That means a device-side action can at most put the System applet into the manager-ready/deactivated state that arms this callback; normal `Connected to NEO Manager.` still depends on the lower-level USB/HID callback firing.

Custom SmartApplet probe status:

- `USB Menu Probe` (`0xa129`, version `1.8`) had two visible paths.
- Normal Applets-menu launch (`0x19`) drew `USB` and idled.
- Event-command paths (`0x20` and `0x21`) drew `DIR`, called ROM callback `0x00410b26`, then idled.
- Live negative result: with version `1.8` open, connecting USB still showed the stock `Attached to Mac/PC, emulating keyboard.` path. The applet did not draw `DIR`, so the System USB attach path appears to bypass custom applet `0x20`/`0x21` handlers.
- `USB Menu Probe` (`0xa129`, version `1.9`, `328` bytes) is the next installed probe. It does not call `0x0041016e` directly because that branch expects the System dispatcher stack/epilogue. Instead, its menu-open command `0x19` replicates the registration side effects inline, then draws `ARM` and idles:

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

The version `1.9` hypothesis is: if the applet is selected first and shows `ARM`, the lower-level USB/HID event may call the registered direct-mode callback on later USB attach. This is not proven yet.

### Firmware activation inventory for `081e:bd01`

Current firmware evidence comes from three local images:

- live physical-device System applet dump: `analysis/device-dumps/neo-system-3.15.os3kapp`
- installer ROM image: `analysis/cab/os3kneorom.os3kos`
- Small ROM updater image: `analysis/cab/smallos3kneorom.os3kos`

The ways found so far to reach a USB device whose descriptor is `081e:bd01` are:

1. Normal System keyboard mode (`081e:bd04`) -> hidden HID output-report switch.
   - Dynamically proven on the physical NEO.
   - Host sends five report-id-0 output reports with one-byte payloads `e0 e1 e2 e3 e4`.
   - The device disconnects/re-enumerates as `081e:bd01`.
   - This is the path used by `real-check switch-to-direct`.
2. Small ROM/updater boot path -> direct updater USB identity.
   - Dynamically proven as a non-HID USB mode visible to Android `UsbManager`.
   - Firmware evidence: `analysis/cab/smallos3kneorom.os3kos` contains a `081e:bd01` device descriptor at file offset `0x5458` and no `081e:bd04` descriptor or HID report descriptor.
   - This mode is an updater/Small ROM transport. It did not support the normal AlphaWord read-only file-list command; `0x13` returned status `0x92` in live testing.
3. Left-shift+tab power-on recovery/manager-ready path.
   - Public recovery instructions say to start with the NEO off, hold left shift and tab, press on/off, release when the SmartApplet menu appears, then connect USB with NEO Manager running.
   - This is a device-side way to force a clean SmartApplet-menu/manager-ready state before USB attach, and it is especially relevant when the startup applet is damaged or Two-Button On interferes with normal startup.
   - It is not currently proven to autonomously enter the normal AlphaWord direct USB protocol without a host-side USB/HID event. Static firmware evidence still points to the lifecycle `0x19` path arming callback `0x00410b26`, then the low-level USB/HID callback firing later.
4. `Alpha USB` SmartApplet device-side activation path.
   - Dynamically proven on the physical NEO.
   - Production applet id is `0xa130`, version `1.18`, menu name `Alpha USB`.
   - Launching the applet and then connecting USB invokes the ROM HID-sequence completion path (`0x0044044e` / `0x0044047c`) from the applet's proven `0x30001` USB attach callback.
   - The device disconnects/re-enumerates as normal `081e:bd01` direct USB mode with bulk OUT `0x01` and bulk IN `0x82`.
   - This path does not require the host to open the initial `081e:bd04` HID keyboard interface, so it is the practical unrooted Android path.

Firmware-level candidate sequences adjacent to the HID report descriptor:

| Sequence | Live switch status | System dump offset | Installer ROM offset | Notes |
| --- | --- | ---: | ---: | --- |
| `e0 e1 e2 e3 e4` | proven activates `081e:bd01` | `0x60a6` | `0x3f6a8` | This is the validated host HID switch sequence. |
| `01 02 04 03 07` | needs live test | `0x60ab` | `0x3f6ad` | Values fit the HID keyboard LED output bit range; possible standards-compatible LED-state path. |
| `f0 f1 f2 f3 f4` | needs live test | `0x60b0` | `0x3f6b2` | Adjacent candidate; exact role unknown. |
| `07 03 01 04 02` | needs live test | `0x60b5` | `0x3f6b7` | Adjacent candidate; exact role unknown. |
| `00 02 03 04 01 02 03 04` | needs further investigation | `0x60ba` | `0x3f6bc` | Eight-byte tail before the HID report descriptor; not currently classified as a switch sequence. |

Negative findings:

- The exact newer NeoManager Windows fallback bytes `05 02 04 01 06 07` do not appear as a contiguous firmware sequence in the live System dump, installer ROM, or Small ROM image.
- The `?Swtch`/`Switched` strings are not in the live System applet dump. They are present in the installer ROM and Small ROM, but they are direct-transport applet/updater commands after `081e:bd01` is already active, not a `081e:bd04` HID-to-direct activation mechanism.
- Ghydra reports no direct xrefs to the live System descriptor/magic table addresses (`0x60a6`, `0x60c2`, `0x6102`, `0x6114`, etc.). Treat the table as firmware data evidence plus live behavioral evidence, not as a fully decompiled switch handler yet.

Practical diagnostic command for the unresolved firmware candidates:

```bash
uv run --project real-check real-check switch-hid-sequence 01 02 04 03 07
uv run --project real-check real-check switch-hid-sequence f0 f1 f2 f3 f4
uv run --project real-check real-check switch-hid-sequence 07 03 01 04 02
```

These commands only send HID output reports to the keyboard-mode device and wait for `081e:bd01` re-enumeration. They do not read or write AlphaWord contents.

### Device-side System 3.15 USB evidence

The physical NEO's installed System applet was dumped through the read-only SmartApplet retrieve path:

```bash
uv run --project real-check real-check dump-applet 0x0000 --output analysis/device-dumps/neo-system-3.15.os3kapp
```

Validated dump:

- size: `401408` bytes (`0x62000`)
- SHA-256: `304a32fb548c8d605351cdef5389976ac2346cace5e9cafcc1e96f7737a37fa6`
- live applet-list identity: `System` version `3.15`

Ghidra load assumptions:

- processor: Motorola 68000, big-endian
- base address: `0`
- flat segment: `0x00000000..0x00061fff`

The System image contains both USB identities as descriptors in the late data region:

- `0x6102`: `081e:bd04` device descriptor for the keyboard identity
  - bcdUSB `1.00`
  - class/subclass/protocol `0/0/0`
  - max packet `64`
  - product string id `2`: `AlphaSmart`
- `0x6114`: `081e:bd01` device descriptor for the communication-driver identity
  - bcdUSB `1.00`
  - class/subclass/protocol `2/0/0`
  - max packet `64`
  - product string id `3`: `AlphaSmart Communication Driver`
- `0x613a`: HID boot-keyboard interface descriptor
- `0x6143`: communication-driver interface descriptor
- `0x6156`: `0x82` interrupt IN endpoint, max packet `64`
- `0x615e`: `0x82` bulk IN endpoint, max packet `64`
- `0x6166`: `0x01` bulk OUT endpoint, max packet `64`
- `0x61ea`: product/interface string `AlphaSmart Keyboard`

The same region contains the HID report descriptor starting at `0x60c2`, including keyboard modifier usages `0xe0..0xe7` and LED output usages. The exact `e0 e1 e2 e3 e4` byte run appears once at `0x60a6`, adjacent to keyboard/report-descriptor data. That matches the validated host switch payload sequence, but the current Ghidra evidence does not show it as an alternate executable switch routine; it is embedded in the System-side HID data table and has no direct Ghidra xrefs.

The System string-resource resolver is:

```c
undefined * FUN_0002cebc(ushort resource_id) {
  return &DAT_000269fe + *(int *)(&DAT_0003ad72 + (uint)resource_id * 4);
}
```

USB/status resources mapped from that table include:

- `0x04c`: `Attached to Mac, emulating keyboard.`
- `0x04d`: `Attached to PC, emulating keyboard.`
- `0x054`: `Connected to Get Utility. All AlphaSmart`
- `0x056`: `Attached to AlphaHub. All AlphaSmart`
- `0x059`: `Connected to computer (infrared mode).`
- `0x104`: suspended-format string containing `USB connection suspended.`

Negative findings from the System image:

- no ASCII `Swtch`, `Switched`, `NEO Manager`, `NeoManager`, `bd01`, or `bd04`
- the only `Manager` string is unrelated wireless-file-transfer UI text
- no decompiled System function found so far provides a separate host-visible path from stock Android's hidden `bd04` HID keyboard state into `bd01`

Interpretation:

- The device firmware/System side clearly carries descriptors for both `bd04` keyboard mode and `bd01` direct mode.
- The Windows/macOS host-side transition remains the validated HID output-report sequence `e0 e1 e2 e3 e4`.
- The System dump does not currently weaken the Android conclusion: if stock Android hides `bd04` boot keyboards from `UsbManager`, a normal unrooted app still needs the NEO to become `bd01` by some mechanism outside Android's public USB Host access to the HID keyboard.

## Private Driver IOCTL Surface

The `IRP_MJ_DEVICE_CONTROL` handler at `0x108e4` compares incoming control codes against:

- `0x220000`
- `0x220004`
- `0x220008`
- `0x80002000`

Unknown codes fall through to:

- `STATUS_INVALID_DEVICE_REQUEST` (`0xC0000010`)

The handler also rejects requests when the device-extension state is not ready:

- `STATUS_DEVICE_NOT_READY` (`0xC00000A3`) style checks appear in some paths
- `STATUS_INVALID_DEVICE_STATE` / `STATUS_INVALID_BUFFER_SIZE` style failures also appear

What each code appears to do:

- `0x80002000`
  - Copies 18 bytes from a device-resident descriptor pointer into the caller output buffer.
  - This matches the user-mode presence check.
  - The copied layout is consistent with a standard USB device descriptor (`bLength == 0x12`, `bDescriptorType == 0x01`).
- `0x220008`
  - Requires a file-context-derived endpoint pointer.
  - Builds a `0x28` internal transfer request with function code `0x1e`.
  - Forwards that request through internal IOCTL `0x220003`.
  - This is narrower than the normal stream data path; the ordinary direct USB reads and writes go through `IRP_MJ_READ` / `IRP_MJ_WRITE`, not user-mode `DeviceIoControl(0x220008, ...)`.
- `0x220004`
  - Reaches a separate helper around the `0x109ba` branch.
  - Likely another transport control operation, not yet fully resolved.
- `0x220000`
  - Reaches a separate helper around the `0x109c6` branch.
  - Likely another transport or reset/setup control operation, not yet fully resolved.

The 64-bit driver `FUN_000115a4` confirms the same four control codes and clarifies their roles:

- `0x220000`
  - copies a cached device-managed block from `device_extension + 0x60` to the caller output buffer
  - uses the cached block's `ushort` length field at offset `+2`
  - returns `STATUS_INVALID_DEVICE_STATE` if the cached block pointer is null
  - returns `STATUS_BUFFER_TOO_SMALL` if the caller output buffer is shorter than the cached block length
- `0x220004`
  - runs an internal lower-stack probe sequence through `GetPortStatusAndMaybeResetPort`
  - first sends internal IOCTL `0x220013`
  - if the returned flags word has bit `1` set, it then sends internal IOCTL `0x220007`
- `0x220008`
  - requires a non-null file-context pointer chained through the IRP stack location
  - uses that context to trigger `SubmitResetPipeUrbForEndpoint`, which builds a small `0x28` request object with function code `0x1e` and forwards it through internal IOCTL `0x220003`
- `0x80002000`
  - copies the cached 18-byte USB device descriptor from `device_extension + 0x58`
  - requires an output buffer of at least `0x12` bytes

The 64-bit statuses line up with the observed branch structure:

- `STATUS_INVALID_DEVICE_STATE` when the device is not in the started state
- `STATUS_INVALID_DEVICE_REQUEST` for unknown IOCTLs
- `STATUS_INVALID_BUFFER_SIZE` / `STATUS_BUFFER_TOO_SMALL` on undersized outputs
- `STATUS_INVALID_PARAMETER` when `0x220008` arrives without the needed file-context chain

## Lower-Stack Internal Request

A helper used by the driver create/setup path builds a lower request with:

- `IoBuildDeviceIoControlRequest(0x220013, ...)`

That helper also:

- Initializes kernel events
- Waits for completion
- Stores completion status back into driver-managed state

Inference:

- `0x220013` is likely an internal driver-to-lower-stack control path, not the user-mode API surface.
- It probably participates in configuration or endpoint setup.

In the 64-bit driver that helper is `GetPortStatusAndMaybeResetPort`, and its exact sequence is:

1. send internal IOCTL `0x220013`
2. inspect a returned flags word written through the IRP stack location
3. if bit `0x2` is set, send internal IOCTL `0x220007`

This makes `0x220004` a wrapper around a two-step lower-stack readiness or reset/probe sequence, not a direct data transfer.

The observed probe-flag handling is:

- if returned flags bit `0x1` is set, stop after `0x220013`
- if bit `0x1` is clear and bit `0x2` is set, follow with `0x220007`
- otherwise stop after `0x220013`

The 64-bit setup path also clarifies the create/start flow:

- `StartDeviceAndLoadUsbDescriptors` clears the cached descriptor/configuration pointers, forwards the start IRP down, then calls `FetchUsbDeviceDescriptor`
- `FetchUsbDeviceDescriptor` sends a `0x220003` request with request type `1` and a `0x12`-byte buffer to fetch the USB device descriptor
- `FetchUsbConfigurationDescriptor` sends another `0x220003` request with request type `2`:
  - first with a 9-byte buffer to fetch the configuration-descriptor header
  - then with the full configuration length from `wTotalLength`
- `ConfigureDeviceInterfacesFromDescriptor` parses the returned configuration descriptor with:
  - `USBD_ParseConfigurationDescriptorEx`
  - `USBD_CreateConfigurationRequestEx`
- It caches:
  - the full USB device descriptor at `device_extension + 0x58`
  - the full USB configuration descriptor at `device_extension + 0x60`
  - the generated URB/configuration request at `device_extension + 0x68`
  - a per-interface “busy” byte array at `device_extension + 0x70`

The internal `0x220003` request objects now have a concrete partial layout:

- descriptor/configuration fetch request (`0x88` bytes):
  - offset `0x00`: size = `0x88`
  - offset `0x02`: function = `0x0b`
  - offset `0x28`: transfer buffer length
  - offset `0x30`: response buffer pointer
  - offset `0x38`: endpoint/pipe pointer placeholder = `0`
  - offset `0x83`: request type
    - `1` for USB device descriptor
    - `2` for USB configuration descriptor
- endpoint-trigger request used by `0x220008` (`0x28` bytes):
  - offset `0x00`: size = `0x28`
  - offset `0x02`: function = `0x1e`
  - offset `0x18`: endpoint pointer copied from the file-bound endpoint descriptor
- chunked data-transfer request used by `DispatchReadWrite` (`0x80` bytes):
  - offset `0x00`: size = `0x80`
  - offset `0x02`: function = `9`
  - offset `0x20`: transfer code
    - `3` for read
    - `2` for write
  - offset `0x24`: current chunk length
  - offset `0x28`: MDL pointer for the current chunk
  - offset `0x18`: endpoint pointer copied from the configured pipe entry
- active-transfer cancel request used by `AbortActivePipeTransfers` (`0x28` bytes):
  - offset `0x00`: size = `0x28`
  - offset `0x02`: function = `2`
  - offset `0x18`: endpoint pointer for the active configured pipe

Interpretation:

- internal `0x220003` is the generic lower transport submit path
- function `0x0b` is the descriptor-fetch style request family
- function `9` is the normal chunked data-transfer request family
- function `2` is the active-transfer cancel request family
- function `0x1e` is the endpoint-trigger style request family used by the public `0x220008` wrapper

## Create, Close, and Endpoint Binding

The 64-bit driver makes the file-handle semantics much clearer.

`DispatchCreate`:

- succeeds only when the device state at `device_extension + 0x78` is `2` and the configured-interface cache at `device_extension + 0x68` is non-null
- opens with an empty file name as a control handle:
  - clears the file object's context pointer
  - increments the outstanding-open count at `device_extension + 0x14c`
  - cancels the pending timer if one is active
- opens with a suffixed file name as an endpoint-specific handle:
  - `ResolveInterfaceStateByteFromFileName` parses the trailing decimal suffix from the Unicode file name
  - valid suffix range is `0..5`
  - the parsed suffix indexes the per-interface busy-byte array at `device_extension + 0x70`
  - the matching endpoint descriptor pointer from the cached URB/configuration request is stored in the file object's context slot
  - the busy byte is set to `1`
  - the outstanding-open count is incremented
  - the pending timer is cancelled if active

`DispatchClose`:

- if the file object carried an endpoint-specific context and a named suffix, it resolves that suffix again and clears the corresponding busy byte back to `0`
- always decrements the outstanding-open count at `device_extension + 0x14c`

Interpretation:

- the driver supports both a control handle and endpoint-bound per-file-object handles
- endpoint names are numeric suffixes, and the busy-byte table prevents ambiguous reuse of those logical interface slots

## Read and Write Transfer Path

The 64-bit driver resolves the main transport path more precisely than the earlier 32-bit pass.

`DispatchReadWrite`:

- treats major function `0x03` as read and `0x04` as write
- requires device state `2`
- waits on the timer event if a timer IRP is active
- resolves the target endpoint descriptor in one of two ways:
  - from the file object's stored endpoint context, if present
  - otherwise by scanning the cached configured pipes at `device_extension + 0x68` and picking the first endpoint whose direction bit matches the read/write request
- accepts only endpoint types `2` or `3`
- rejects transfer lengths above `0x10000`
- returns immediate success for zero-length requests
- allocates:
  - a `0x28` transfer-tracking context
  - an MDL over the caller buffer
  - an `0x80` lower request block
- programs the lower request for internal IOCTL `0x220003`
- uses transfer code `3` for reads and `2` for writes
- limits each submitted chunk to `0x100` bytes
- installs `ContinueChunkedReadTransfer` as the completion routine
- holds an outstanding-I/O reference across the lower-driver call
- if the immediate lower-driver status is a hard failure other than the two tolerated pending-style statuses, it tries `SubmitResetPipeUrbForEndpoint` and, if that also fails, runs `GetPortStatusAndMaybeResetPort`

`ContinueChunkedReadTransfer`:

- adds the completed byte count from the lower request to the IRP's accumulated `Information` value
- if bytes remain, rebuilds the partial MDL for the next `0x100`-byte-or-smaller slice
- resubmits the same IRP down with IOCTL `0x220003`
- when all bytes are consumed, releases the outstanding-I/O reference and frees the request block, MDL, and tracking context

Interpretation:

- normal direct USB data movement in the 64-bit driver is a chunked lower-stack `0x220003` pipeline, not `0x220008`
- user-mode `ReadFile` / `WriteFile` on `\\\\.\\AsUSBDrv%d` are the important operations to replicate, with the driver's own chunk size capped at `0x100`
- the lower `0x80` transfer request block used by `DispatchReadWrite` is distinct from the `0x88` descriptor-fetch block and carries:
  - function `9`
  - transfer code `3` for read or `2` for write
  - endpoint pointer from the configured pipe entry
  - current chunk length
  - MDL pointer for the current chunk
- `AbortActivePipeTransfers` uses a separate `0x28` / function `2` request to tear down still-marked active pipe transfers during stop/remove-style paths

## PnP State Machine

`DispatchPnP` maps the main minor codes as follows:

- `0x00` -> `StartDeviceAndLoadUsbDescriptors`
- `0x01` -> query remove:
  - saves the previous state
  - sets current state to `4`
  - clears the started flag
  - waits for outstanding I/O to drain
  - forwards the IRP down
- `0x02` -> `HandleRemoveDevice`
- `0x03` -> `HandleCancelRemoveDevice`
- `0x04` -> `HandleStopDevice`
- `0x05` -> query stop:
  - saves the previous state
  - sets current state to `3`
  - clears the started flag
  - waits for outstanding I/O to drain
  - forwards the IRP down
- `0x06` -> `HandleCancelStopDevice`
- `0x09` -> `HandleQueryCapabilities`
- `0x17` -> `HandleSurpriseRemoval`
- other minors -> pass through to the lower driver

## Protocol Framing Observed So Far

Direct USB appears to use two layers:

1. Control and setup:
   - driver private IOCTLs such as `0x220000`, `0x220004`, `0x220008`
   - lower-stack internal IOCTL `0x220003` for descriptor fetch and transfer submission
   - lower-stack or pass-through descriptor query `0x80002000`

2. Data path:
   - `WriteFile` for outbound stream data
   - `ReadFile` for inbound stream data
   - user-mode chunking capped at `0x40` bytes on write
   - driver-side chunking capped at `0x100` bytes per lower-stack submission

Small command transactions exist on top of this:

- 8-byte `?Swtch` command
- 8-byte textual response
- 8-byte updater command frames built as `[cmd][arg32-be][arg16-be][sum]`

For AlphaWord retrieval specifically, the reconstructed fresh direct USB sequence is now:

1. If the device is attached as `081e:bd04`, send HID output report payloads `e0 e1 e2 e3 e4`.
2. Wait for re-enumeration as `081e:bd01`.
3. Open the direct USB bulk endpoints (`OUT 0x01`, `IN 0x82` on the tested NEO).
4. Send `?\xff\x00reset`.
5. Send `?Swtch\x00\x00`.
6. Send updater command `0x04` to enumerate applets.
7. Send updater command `0x13` to get file attributes.
8. Send updater command `0x12` or `0x1c` to begin file retrieval.
9. Send repeated updater command `0x10` chunk pulls.

Live physical-device validation on the tested NEO now confirms the read-only parts of that sequence:

- `real-check watch --timeout 5` observed `081e:bd01` direct mode with bulk OUT `0x01` and bulk IN `0x82`.
- `real-check probe` selected interface `0`, OUT endpoint `0x01`, and IN endpoint `0x82`.
- `real-check switch-to-direct` is idempotent once already direct and reports `sent_manager_switch_reports=0`.
- `real-check list` successfully reads all eight AlphaWord `0x13` file-attribute records.
- `real-check applets` successfully reads the SmartApplet `0x04` list pages and parses `0x84`-byte metadata records.
- `real-check verify-get 1..8` successfully exercises read-only AlphaWord retrieval (`0x12` plus repeated `0x10`) without printing or writing document contents.

## Working Hypothesis

The direct transport likely works like this:

1. If only `081e:bd04` is present, NeoManager's HID fallback path switches the device into `081e:bd01` direct mode using HID output reports.
2. Enumerate numbered `AsUSBDrv` device instances.
3. Query a USB descriptor with `0x80002000` to verify the device is a NEO.
4. Start-device handling fetches the USB device and configuration descriptors and builds a configured-pipe cache.
5. Optional named opens bind a file object to a specific configured endpoint.
6. Use `WriteFile` and `ReadFile` for the main data exchange, which the driver translates into chunked internal `0x220003` requests.
7. Use small fixed-width command packets for applet switching after direct mode is active.

For the AlphaWord retrieval and print-side flow built on top of this transport, see:

- [2026-03-31-alphaword-get-print-dataflow.md](/Users/jakubkolcar/customs/neo-re/docs/2026-03-31-alphaword-get-print-dataflow.md)

## High-Value Unknowns

These still need confirmation:

- Exact user-visible semantics of the cached block returned by `0x220000`
- Exact lower-stack meaning of internal IOCTLs `0x220013` and `0x220007`
- Whether NeoManager ever opens named endpoint handles explicitly, or relies only on the default direction-matched path
- Whether `WriteFile` consistently lands on one bulk OUT pipe or can switch depending on the configured-interface cache
- Whether `ReadFile` consistently lands on one bulk IN pipe or can switch depending on the configured-interface cache
- Exact layout of the 8-byte `?Swtch` command beyond the embedded applet ID

## Best Next Reverse-Engineering Targets

If continuing locally in Ghidra/radare2:

- Follow the remaining lower-stack status handling around `DispatchReadWrite`
- Inspect which configured pipe entries NeoManager actually opens by name versus using the default direction scan
- Follow the power/timer path around `RequestDevicePowerIrp` and the timer object at `device_extension + 0x150`
- Follow the 32-bit driver's create/read/write dispatchers and confirm they mirror the 64-bit chunking and endpoint-selection logic

## Practical Takeaways

What is already firm enough to rely on:

- Direct USB is separate from AlphaHub and should be documented independently.
- A directly connected NEO may first attach as `081e:bd04`, a HID keyboard interface, not `081e:bd01`.
- The confirmed `081e:bd04` -> `081e:bd01` switch for the tested NEO is HID output report payloads `e0 e1 e2 e3 e4`.
- On macOS, direct `libusb_control_transfer` works for that switch when it avoids claiming the HID keyboard interface.
- The tested `081e:bd01` direct endpoint pair is bulk OUT `0x01` and bulk IN `0x82`.
- The read-only live probe set now validates descriptor selection, applet listing, AlphaWord file-attribute listing, and AlphaWord content retrieval checksums against a physical NEO.
- The driver exposes a named DOS device path consumed from user mode.
- User-mode writes are stream writes in `0x40` byte chunks.
- Applet switching is an 8-byte request and 8-byte response exchange.
- Applet switching is preceded by a fixed 8-byte reset preamble `?\xff\x00reset`.
- The `?Swtch` applet ID field is big-endian.
- The presence-check path can be modeled offline as a standard 18-byte USB device descriptor parse plus `bcdDevice`-based return-code classification.
- The driver has at least three private user-visible IOCTLs and one descriptor-oriented pass-through request.
- The DLL exports two MAC helper commands on top of the same 8-byte updater framing.
- The 64-bit driver confirms that `0x220004` is an internal probe wrapper around lower-stack IOCTLs `0x220013` and `0x220007`.
- The 64-bit driver confirms that `0x220003` is the lower-stack request used to fetch descriptors and to move data via URB-style request objects.
