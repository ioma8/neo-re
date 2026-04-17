# real-check

Minimal live USB checker for AlphaSmart NEO direct USB communication on macOS/Linux.

## Commands

Observe whether the connected NEO is in HID keyboard mode (`081e:bd04`) or direct USB mode (`081e:bd01`):

```bash
uv run --project real-check real-check watch --timeout 5
```

Switch a freshly attached NEO from HID keyboard mode to direct USB mode:

```bash
uv run --project real-check real-check switch-to-direct
```

The confirmed macOS path sends five HID output reports with one-byte payloads:

```text
e0 e1 e2 e3 e4
```

These are USB HID class `SET_REPORT` control transfers sent through `libusb_control_transfer` without claiming the keyboard interface. This matters on macOS: PyUSB's managed `ctrl_transfer` path tries to claim the HID keyboard interface and fails, and hidapi can enumerate `081e:bd04` but cannot open it reliably as a keyboard-class device. No `sudo` is required for the working path.

Probe the connected device and print the selected interface and endpoints:

```bash
uv run --project real-check real-check probe
```

Print a read-only raw trace of AlphaWord file-attribute responses:

```bash
uv run --project real-check real-check debug-attributes
```

Use this when `list` fails. It prints the reset/switch response, each slot's raw attribute command/header, and payload checksums for data-bearing responses.

List AlphaWord files:

```bash
uv run --project real-check real-check list
```

Download one AlphaWord slot and print the payload as hex:

```bash
uv run --project real-check real-check get 2
```

Download one AlphaWord slot and write it to a file:

```bash
uv run --project real-check real-check get 2 --output slot2.bin
```

## Current assumptions

- HID keyboard-mode device match is `VID=0x081e`, `PID=0xbd04`
- direct USB device match is `VID=0x081e`, `PID=0xbd01`
- keyboard-to-direct mode switch is confirmed as HID output report payloads `e0 e1 e2 e3 e4`
- endpoint selection prefers a bulk OUT + bulk IN pair, then falls back to interrupt
- updater bootstrap is `?\xff\x00reset` then `?Swtch\x00\x00`
- AlphaWord applet id is `0xa000`
- file listing is based on raw attribute opcode `0x13` across slots `1..8`
- file download uses retrieve opcode `0x12` and repeated chunk opcode `0x10`
- direct USB reads can return short packets; callers must accumulate until the requested byte count is satisfied

## Physical-device safety

The `watch`, `switch-to-direct`, and `probe` commands do not read or modify AlphaWord file contents. `switch-to-direct` only changes USB mode by sending HID output reports. `probe` only inspects the direct USB descriptor/endpoints.

`list` sends read-only AlphaWord file-attribute requests and prints slot names/lengths. `get` retrieves file bytes from a slot and writes only to the host output path if `--output` is provided.

## Status

The protocol layer is validated against reverse-engineered Windows binaries and offline tests.
The live macOS path is confirmed against a physical AlphaSmart NEO:

```text
081e:bd04 keyboard mode -> switch-to-direct -> 081e:bd01 direct mode
direct endpoints: OUT 0x01 bulk, IN 0x82 bulk
```
