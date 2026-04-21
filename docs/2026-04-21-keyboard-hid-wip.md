# Keyboard Matrix HID Bridge WIP

Scope: map AlphaSmart NEO matrix bytes to physical key names for `alpha-emu`.

## Confirmed Firmware Evidence

- Small ROM password check contains raw matrix bytes `3a 3d 7f 30 3a`
  for password `ernie`.
- Full OS contains a logical-key-to-USB-HID usage table at:
  - file: `analysis/cab/os3kneorom.os3kos`
  - file offset: `0x3c32a`
  - mapped address with `-m 0x400000`: `0x0043c32a`
- Minimal radare2 check:

```sh
r2 -q -m 0x400000 -e scr.color=false -e log.quiet=true \
  -c 'p8 0x51 @ 0x43c32a' analysis/cab/os3kneorom.os3kos
```

Expected bytes:

```text
30402f2a3c392b17e11c3f34e3513e3d0a0b0e0f333107160409ff0d0c12134a081a1415182ee2412d58ff423b3a3522232526274543201f1e212436e6374d4428061b1d19e5103829504f522ce005114c
```

Validation anchors:

| key | raw | logical | HID usage |
|---|---:|---:|---:|
| E | `0x3a` | `0x20` | `0x08` |
| R | `0x3d` | `0x23` | `0x15` |
| N | `0x7f` | `0x4f` | `0x11` |
| I | `0x30` | `0x1c` | `0x0c` |

These four anchors match both the Small ROM password bytes and the USB HID
usage table, so the table is the validated bridge from firmware logical keys to
physical key identity.

Full OS disassembly also confirms the raw-matrix decode shape:

```sh
r2 -q -a m68k -b 16 -m 0x400000 -e scr.color=false -e log.quiet=true \
  -c 'pd 90 @ 0x413d7a' analysis/cab/os3kneorom.os3kos
```

Key instructions:

```text
0x00413d86  moveq 0x0, d0
0x00413d88  move.b 0xf(a7), d0
0x00413d8c  addi.l 0x44c37b, d0
0x00413d92  movea.l d0, a0
0x00413d94  move.b (a0), d7
```

This routine takes a raw matrix byte and indexes the logical-key map.

## Current Full Mapping

| key | raw | logical | HID |
|---|---:|---:|---:|
| File 1 | `0x4b` | `0x2d` | `0x3a` |
| File 2 | `0x4a` | `0x2c` | `0x3b` |
| File 3 | `0x0a` | `0x04` | `0x3c` |
| File 4 | `0x1a` | `0x0f` | `0x3d` |
| File 5 | `0x19` | `0x0e` | `0x3e` |
| File 6 | `0x10` | `0x0a` | `0x3f` |
| File 7 | `0x02` | `0x01` | `0x40` |
| File 8 | `0x42` | `0x27` | `0x41` |
| Print | `0x49` | `0x2b` | `0x42` |
| Spell Check | `0x59` | `0x35` | `0x43` |
| Find | `0x67` | `0x3f` | `0x44` |
| Clear File | `0x54` | `0x34` | `0x45` |
| Applets | `0x47` | `0x2a` | `0xff` |
| Send | `0x46` | `0x29` | `0x58` |
| Backspace | `0x09` | `0x03` | `0x2a` |
| Tab | `0x0c` | `0x06` | `0x2b` |
| Caps Lock | `0x0b` | `0x05` | `0x39` |
| Enter | `0x69` | `0x40` | `0x28` |
| Shift | `0x0e` | `0x08` | `0xe1` |
| Right Shift | `0x6e` | `0x45` | `0xe5` |
| Ctrl | `0x7c` | `0x4d` | `0xe0` |
| Alt/Option | `0x41` | `0x26` | `0xe2` |
| Command | `0x14` | `0x0c` | `0xe3` |
| Space | `0x79` | `0x4c` | `0x2c` |
| Esc | `0x74` | `0x48` | `0x29` |
| Delete | `0x61` | `0x50` | `0x4c` |
| Up | `0x77` | `0x4b` | `0x52` |
| Left | `0x75` | `0x49` | `0x50` |
| Down | `0x15` | `0x0d` | `0x51` |
| Right | `0x76` | `0x4a` | `0x4f` |
| Home | `0x34` | `0x1f` | `0x4a` |
| End | `0x65` | `0x3e` | `0x4d` |
| A | `0x2c` | `0x18` | `0x04` |
| B | `0x7d` | `0x4e` | `0x05` |
| C | `0x6a` | `0x41` | `0x06` |
| D | `0x2a` | `0x16` | `0x07` |
| E | `0x3a` | `0x20` | `0x08` |
| F | `0x2d` | `0x19` | `0x09` |
| G | `0x1d` | `0x10` | `0x0a` |
| H | `0x1f` | `0x11` | `0x0b` |
| I | `0x30` | `0x1c` | `0x0c` |
| J | `0x2f` | `0x1b` | `0x0d` |
| K | `0x20` | `0x12` | `0x0e` |
| L | `0x22` | `0x13` | `0x0f` |
| M | `0x6f` | `0x46` | `0x10` |
| N | `0x7f` | `0x4f` | `0x11` |
| O | `0x32` | `0x1d` | `0x12` |
| P | `0x33` | `0x1e` | `0x13` |
| Q | `0x3c` | `0x22` | `0x14` |
| R | `0x3d` | `0x23` | `0x15` |
| S | `0x2b` | `0x17` | `0x16` |
| T | `0x0d` | `0x07` | `0x17` |
| U | `0x3f` | `0x24` | `0x18` |
| V | `0x6d` | `0x44` | `0x19` |
| W | `0x3b` | `0x21` | `0x1a` |
| X | `0x6b` | `0x42` | `0x1b` |
| Y | `0x0f` | `0x09` | `0x1c` |
| Z | `0x6c` | `0x43` | `0x1d` |

Number and punctuation raw keys are decoded from the same HID table in
`alpha-emu/src/keyboard.rs`.

## Open Point

The table proves the key identities used by firmware and HID mode. The top-row
local keys `Applets` and `Send` are currently assigned by matching the remaining
top-row slots against the HID table and the user-provided physical layout:

- `Applets`: raw `0x47`, no host HID usage.
- `Send`: raw `0x46`, HID usage `0x58` (`Keypad Enter`).

This is the only remaining mapping that is not separately validated by a direct
live key capture.

## Print Key Validation

`Print = raw 0x49 / logical 0x2b / HID 0x42` is validated by the firmware table
chain and physical top-row layout:

1. Raw `0x49` decodes through the matrix map at `0x0044c37b` to logical
   `0x2b`.
2. Logical `0x2b` decodes through the HID table at `0x0043c32a` to HID usage
   `0x42`.
3. HID usage `0x42` is `F9`; in the NEO physical top row the ninth function-key
   slot after File 1..File 8 is `Print`.

There is also firmware behavior evidence that logical `0x2b` is treated as a
special top-row key, not as printable text. In full OS disassembly, the event
dispatch around `0x0041898e` and `0x0041f2cc` compares the current logical key
against a set of top-row/system keys:

```text
0x0041899c  cmpi.b 0x2b, d0
0x004189a0  beq.b 0x4189c8

0x0041f2da  cmpi.b 0x2b, d0
0x0041f2de  beq.b 0x41f306
```

Both branches group `0x2b` with other non-text keys such as `0x34`, `0x35`,
`0x3f`, and `0x49..0x4b`.

What is not yet proven: a direct named xref from logical `0x2b` to a routine
labelled "print". The print/printer strings are present in the firmware, but I
have not tied those strings to the `0x2b` branch yet.
