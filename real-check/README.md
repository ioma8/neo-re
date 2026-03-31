# real-check

Minimal live USB checker for AlphaSmart NEO direct USB communication on macOS/Linux.

## Commands

Probe the connected device and print the selected interface and endpoints:

```bash
uv run --project real-check real-check probe
```

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

- device match is `VID=0x081e`, `PID=0xbd01`
- endpoint selection prefers a bulk OUT + bulk IN pair, then falls back to interrupt
- updater bootstrap is `?\xff\x00reset` then `?Swtch\x00\x00`
- AlphaWord applet id is `0xa000`
- file listing is based on raw attribute opcode `0x13` across slots `1..8`
- file download uses retrieve opcode `0x12` and repeated chunk opcode `0x10`

## Status

The protocol layer is validated against reverse-engineered Windows binaries and offline tests.
The live libusb path is implemented, but it still needs confirmation against a real NEO on macOS/Linux hardware.
