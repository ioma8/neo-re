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

Inference:

- Reads are not plain raw stream reads.
- The DLL maintains unread tail bytes across read-loop iterations, and likely across successive reads as long as the global handle stays valid.
- The transport naturally exposes 8-byte inbound chunks at this layer, even though the caller API asks for arbitrary byte counts.

### `AsUSBCommSwitchToApplet`

Observed behavior:

- Sends an 8-byte reset preamble before the switch command:
  - `3f ff 00 72 65 73 65 74`
  - ASCII form: `?\xff\x00reset`
- Writes an 8-byte command block via `WriteFile`.
- The command starts with the string prefix `"?Swtch"`.
- The remaining bytes are the target applet ID encoded as a big-endian 16-bit value.
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

Inference:

- Applet switching is a very small command/response protocol.
- The command and response framing is fixed-width at 8 bytes.
- The on-wire switch packet format is:
  - bytes `0..5`: ASCII `?Swtch`
  - bytes `6..7`: applet ID in big-endian order
- The reset preamble is distinct from the switch packet and should be modeled as a separate fixed 8-byte write.

### `AsUSBCommIsAlphaSmartPresent`

Observed behavior:

- Formats `\\\\.\\AsUSBDrv%d` and probes up to `0x100` possible instances.
- Opens each candidate with `CreateFileA`.
- Uses `DeviceIoControl` with control code `0x80002000`.
- Expects an 18-byte output buffer on that transaction.
- Returns distinct status codes depending on descriptor-type values.

Inference:

- `0x80002000` is probably not a private `AsUsbDrv` IOCTL.
- It likely targets a lower USB/HID stack request that the filter/function driver passes through.
- The 18-byte buffer length matches a USB device descriptor size, which is a strong hint that this call is retrieving a device descriptor for identification.
- The practical user-mode discriminator for the direct NEO path is therefore the USB device descriptor VID/PID pair:
  - vendor `0x081e`
  - product `0xbd01`

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
  - Looks up a pointer chain and copies a variable-length data block to the caller output buffer.
  - This is still relevant to the driver’s read-side plumbing, but the user-mode `AsUSBCommReadData` function itself now looks like a staged `ReadFile` loop rather than a `DeviceIoControl` wrapper.
- `0x220004`
  - Reaches a separate helper around the `0x109ba` branch.
  - Likely another transport control operation, not yet fully resolved.
- `0x220000`
  - Reaches a separate helper around the `0x109c6` branch.
  - Likely another transport or reset/setup control operation, not yet fully resolved.

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

## Protocol Framing Observed So Far

Direct USB appears to use two layers:

1. Control and setup:
   - driver private IOCTLs such as `0x220000`, `0x220004`, `0x220008`
   - lower-stack or pass-through descriptor query `0x80002000`

2. Data path:
   - `WriteFile` for outbound stream data
   - `ReadFile` for inbound stream data
   - user-mode chunking capped at `0x40` bytes on write

Small command transactions exist on top of this:

- 8-byte `?Swtch` command
- 8-byte textual response

## Working Hypothesis

The direct transport likely works like this:

1. Enumerate numbered `AsUSBDrv` device instances.
2. Query a USB descriptor with `0x80002000` to verify the device is a NEO.
3. Use private IOCTLs to prepare transfers or expose driver-managed buffers.
4. Use `WriteFile` and `ReadFile` for the main data exchange.
5. Use small fixed-width command packets for mode changes such as applet switching.

## High-Value Unknowns

These still need confirmation:

- Exact semantics of `0x220000`
- Exact semantics of `0x220004`
- Exact semantics of `0x220008`
- Whether `WriteFile` talks to a single USB bulk OUT endpoint
- Whether `ReadFile` talks to a single USB bulk IN endpoint
- Whether any HID transport remains in the actual data path after enumeration
- Exact layout of the 8-byte `?Swtch` command beyond the embedded applet ID
- Whether the 8-byte inbound read staging maps directly to USB max-packet size or to a higher-level record framing choice

## Best Next Reverse-Engineering Targets

If continuing locally in radare2:

- Follow the `0x109ba` branch in the `IRP_MJ_DEVICE_CONTROL` handler
- Follow the `0x109c6` branch in the `IRP_MJ_DEVICE_CONTROL` handler
- Trace helper calls reached from the `0x220008` branch
- Trace `AsUSBCommReadData` around the 8-byte control call and the timeout loop

If using Ghidra next, request decompilation for:

- `AsUSBCommReadData`
- `AsUSBCommSwitchToApplet`
- `IRP_MJ_DEVICE_CONTROL` handler at `0x108e4`
- The helpers reached from `0x109ba` and `0x109c6`

## Practical Takeaways

What is already firm enough to rely on:

- Direct USB is separate from AlphaHub and should be documented independently.
- The driver exposes a named DOS device path consumed from user mode.
- User-mode writes are stream writes in `0x40` byte chunks.
- Applet switching is an 8-byte request and 8-byte response exchange.
- Applet switching is preceded by a fixed 8-byte reset preamble `?\xff\x00reset`.
- The `?Swtch` applet ID field is big-endian.
- The presence-check path can be modeled offline as a standard 18-byte USB device descriptor parse plus VID/PID classification.
- The driver has at least three private user-visible IOCTLs and one descriptor-oriented pass-through request.
