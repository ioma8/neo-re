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

Test another firmware-candidate HID output-report sequence:

```bash
uv run --project real-check real-check switch-hid-sequence 01 02 04 03 07
```

This is diagnostic only. It sends the provided hex bytes as one-byte HID output reports and waits for `081e:bd01` re-enumeration. It does not read or write AlphaWord file contents.

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

List installed SmartApplet metadata records:

```bash
uv run --project real-check real-check applets
```

This uses the read-only applet-list opcode `0x04`. It prints only metadata from the device's `0x84`-byte applet records and does not retrieve applet binaries.

Dump one installed SmartApplet binary to the host:

```bash
uv run --project real-check real-check dump-applet 0x0000 --output analysis/device-dumps/neo-system-3.15.os3kapp
```

This uses the read-only retrieve-applet command `0x0f` followed by repeated chunk pulls with command `0x10`. It writes only to the host output path. The validated System 3.15 dump is `401408` bytes with SHA-256 `304a32fb548c8d605351cdef5389976ac2346cace5e9cafcc1e96f7737a37fa6`.

Verify that an AlphaWord slot can be retrieved without printing or writing its contents:

```bash
uv run --project real-check real-check verify-get 2
```

This uses the same read-only retrieve path as `get`, but prints only `reported_length`, `bytes_read`, `sum16`, and `sha256`.

Download one AlphaWord slot and print the payload as hex:

```bash
uv run --project real-check real-check get 2
```

Download one AlphaWord slot and write it to a file:

```bash
uv run --project real-check real-check get 2 --output slot2.bin
```

Export one AlphaWord slot as a host-side text file:

```bash
mkdir -p exports
uv run --project real-check real-check get 1 --output exports/alphaword-slot1.raw
python3 - <<'PY'
from pathlib import Path
raw = Path("exports/alphaword-slot1.raw").read_bytes()
text = raw.replace(b"\x00", b" ").replace(b"\r\n", b"\n").replace(b"\r", b"\n")
Path("exports/alphaword-slot1.txt").write_text(text.decode("latin-1"), encoding="utf-8")
PY
```

The tested slot 1 export produced `22712` raw bytes and a `22712` byte UTF-8 text file after CR-to-LF normalization. Keep `exports/` ignored; these files can contain private device data.

## Current assumptions

- HID keyboard-mode device match is `VID=0x081e`, `PID=0xbd04`
- direct USB device match is `VID=0x081e`, `PID=0xbd01`
- keyboard-to-direct mode switch is confirmed as HID output report payloads `e0 e1 e2 e3 e4`
- device-side keyboard-to-direct mode switch is confirmed through the `Alpha USB` SmartApplet (`applet_id=0xa130`, version `1.20`)
- firmware-adjacent candidate HID sequences are `01 02 04 03 07`, `f0 f1 f2 f3 f4`, and `07 03 01 04 02`; these still need physical-device switch testing
- endpoint selection prefers a bulk OUT + bulk IN pair, then falls back to interrupt
- updater bootstrap is `?\xff\x00reset` then `?Swtch\x00\x00`
- SmartApplet listing uses opcode `0x04`, response `0x44`, and `0x84`-byte metadata records
- SmartApplet dump uses opcode `0x0f`, response `0x53`, repeated opcode `0x10`, response `0x4d`, and 16-bit payload checksums
- AlphaWord applet id is `0xa000`
- file listing is based on raw attribute opcode `0x13` across slots `1..8`
- file download uses retrieve opcode `0x12` and repeated chunk opcode `0x10`
- direct USB reads can return short packets; callers must accumulate until the requested byte count is satisfied

## Physical-device safety

The `watch`, `switch-to-direct`, `switch-hid-sequence`, and `probe` commands do not read or modify AlphaWord file contents. `switch-to-direct` and `switch-hid-sequence` only change USB mode by sending HID output reports. `probe` only inspects the direct USB descriptor/endpoints.

`list` sends read-only AlphaWord file-attribute requests and prints slot names/lengths. `applets` sends a read-only applet-list request and prints installed applet metadata. `dump-applet` retrieves an installed SmartApplet binary and writes only to the host output path. `verify-get` retrieves AlphaWord bytes but prints only host-side verification summaries, not document contents. `get` retrieves file bytes from a slot and writes only to the host output path if `--output` is provided; without `--output`, it prints the slot contents as hex, so avoid it when the device contains private data. Host-side exports under `exports/` are ignored by git.

## Status

The protocol layer is validated against reverse-engineered Windows binaries and offline tests.
The live macOS path is confirmed against a physical AlphaSmart NEO:

```text
081e:bd04 keyboard mode -> switch-to-direct -> 081e:bd01 direct mode
direct endpoints: OUT 0x01 bulk, IN 0x82 bulk
```

For hosts that cannot open the initial HID keyboard interface, install and launch the `Alpha USB` SmartApplet on the NEO before connecting USB. The applet uses the proven ROM HID-completion path from the device side and re-enumerates as the same `081e:bd01` direct-mode device.
