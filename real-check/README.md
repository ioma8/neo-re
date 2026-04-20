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

## Operational Command Guide

This section groups every `real-check` command by device risk. Commands that
write to the NEO are marked clearly. When in doubt, run the read-only inspection
commands first and save the output.

### USB Mode And Endpoint Checks

These commands do not read or modify AlphaWord contents or SmartApplet flash.
They either observe USB descriptors or change only the current USB mode.

Check what the host sees:

```bash
uv run --project real-check real-check watch --timeout 5
```

Expected keyboard mode:

```text
vendor_id=0x081e product_id=0xbd04 mode=keyboard detail=AlphaSmart HID keyboard mode; no direct USB OUT endpoint
```

Expected direct USB mode:

```text
vendor_id=0x081e product_id=0xbd01 mode=direct detail=NEO direct USB mode
```

Switch a NEO in HID keyboard mode to direct USB mode from the host:

```bash
uv run --project real-check real-check switch-to-direct
uv run --project real-check real-check watch --timeout 5
```

Probe the direct USB interface and endpoint selection:

```bash
uv run --project real-check real-check probe
```

Run a candidate HID switch sequence for diagnostics:

```bash
uv run --project real-check real-check switch-hid-sequence e0 e1 e2 e3 e4
uv run --project real-check real-check switch-hid-sequence 01 02 04 03 07 --delay 2 --wait 5
```

`switch-hid-sequence` sends each provided byte as a one-byte HID output report.
Use it only for switch-sequence experiments.

### Read-Only Direct USB Commands

These require direct USB mode. They send read commands to the device and write
only to the host filesystem.

List AlphaWord slot metadata:

```bash
uv run --project real-check real-check list
```

Debug AlphaWord attribute responses when `list` is suspicious:

```bash
uv run --project real-check real-check debug-attributes
```

List installed SmartApplet metadata:

```bash
uv run --project real-check real-check applets
```

Dump raw SmartApplet metadata records:

```bash
uv run --project real-check real-check debug-applets
```

Dump one installed SmartApplet package to the host:

```bash
uv run --project real-check real-check dump-applet 0xa130 --output analysis/device-dumps/applets/A130-Alpha_USB.os3kapp
```

Verify one AlphaWord slot download without saving or printing the document:

```bash
uv run --project real-check real-check verify-get 1
```

Download one AlphaWord slot to a host file:

```bash
uv run --project real-check real-check get 1 --output exports/alphaword-slot1.raw
```

`get` without `--output` prints the file bytes as hex. Avoid that on devices
with private data unless you intentionally want terminal output.

### SmartApplet Write Commands

These commands rewrite the NEO SmartApplet area. Back up first with `applets`
and `dump-applet`. They should not directly modify AlphaWord document slots, but
they do modify persistent device flash and can make the applet catalog invalid
if interrupted or if a bad applet is installed.

Install the validated native Alpha USB applet:

```bash
uv run --project real-check real-check switch-to-direct
uv run --project real-check real-check install-applet exports/applets/alpha-usb-native.os3kapp
uv run --project real-check real-check applets
```

Use `--assume-updater` only when the NEO is already showing the SmartApplet
loading/updater screen and the `?Swtch` bootstrap must be skipped:

```bash
uv run --project real-check real-check install-applet exports/applets/alpha-usb-native.os3kapp --assume-updater
```

Remove one applet by table index:

```bash
uv run --project real-check real-check applets
uv run --project real-check real-check remove-applet-index 3
uv run --project real-check real-check applets
```

The index is the applet table index used by the device protocol, not the
`applet_id`. Use only after inspecting the current table.

Clear the writable SmartApplet area:

```bash
uv run --project real-check real-check clear-applet-area
uv run --project real-check real-check applets
```

This is destructive for installed custom/stock applets in the writable applet
area. It is useful for catalog repair only when backups exist.

Restore applets from a backup directory:

```bash
uv run --project real-check real-check restore-stock-applets \
  --backup-dir analysis/device-dumps/applets \
  --yes
```

By default this skips applet id `0x0000` System. Add repeated `--skip` options
to omit applets that should not be restored:

```bash
uv run --project real-check real-check restore-stock-applets \
  --backup-dir analysis/device-dumps/applets \
  --skip 0xa017 \
  --yes
```

Add `--restart` only after the restore is known-good:

```bash
uv run --project real-check real-check restore-stock-applets \
  --backup-dir analysis/device-dumps/applets \
  --skip 0xa017 \
  --yes \
  --restart
```

The physically safer recovery pattern was one applet install at a time with
`applets` verification after each install. The broad restore command exists, but
do not use it on a fragile device unless that tradeoff is intentional.

### OS Flash And Recovery Commands

These are destructive OS-flash/recovery commands. They are for Small ROM Updater
or serious recovery work, not normal applet iteration.

Flash a NEO OS image:

```bash
uv run --project real-check real-check install-os-image \
  /tmp/os3kneorom-disable-fscheck.os3kos \
  --yes-flash-os
```

`--yes-flash-os` is required. Keep using the patched validator-disabled OS as
the recovery baseline while applet/catalog work is in progress; it proved more
useful than immediately returning to the original stock OS.

Only use `--reformat-rest-of-rom` when intentionally reproducing NeoManager's
tail-segment erase behavior during OS repair:

```bash
uv run --project real-check real-check install-os-image \
  /tmp/os3kneorom-disable-fscheck.os3kos \
  --yes-flash-os \
  --reformat-rest-of-rom
```

Restart the device from direct USB:

```bash
uv run --project real-check real-check restart-device
```

The detailed physical-device recovery record is in
`docs/2026-04-18-neo-recovery-runbook.md`.

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

This path is now physically validated end to end on Android: with `Alpha USB`
`0xa130` version `1.20` launched on the NEO before USB attach, the Android GUI
sees direct USB through `UsbManager` and successfully backs up AlphaWord files.
That is the validated no-root, no-proxy, no-typing-fallback Android path.
