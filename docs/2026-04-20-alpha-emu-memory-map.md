# AlphaSmart NEO Emulator Memory Map Notes

Brief record of the memory and MMIO findings validated while booting
`analysis/cab/smallos3kneorom.os3kos` in `alpha-emu`.

## ROM and RAM

| Range | Meaning | Evidence |
| --- | --- | --- |
| `0x00000000..` | Reset-vector mirror of the Small ROM image. | CPU reset vectors are read from address `0`; initial SSP is `0x0007fff0`, reset PC is `0x0040042a`. |
| `0x00400000..` | Executable Small ROM mapping. | Small ROM code and strings disassemble/run at runtime base `0x00400000`. |
| `0x00000000..0x007fffff` | Current emulator backing memory. | Large enough for reset mirror, ROM mapping, RAM state, and observed Small ROM stack use. |

## Internal Register Window

| Range | Meaning | Evidence |
| --- | --- | --- |
| `0x0000f000..0x0000ffff` | DragonBall-style internal register/MMIO window. | Small ROM accesses byte and word registers in this range during early hardware setup. |
| `0xfffff000..0xffffffff` | Sign-extended alias of the same register window. | 68k absolute-short addressing reaches registers such as `0xfffff419`; preserving state between low and sign-extended aliases lets boot continue. |

## LCD Controller Ports

The NEO LCD behaves like two page/column byte-addressed controllers, each
covering half of the 320x128 display.

| Address | Meaning | Evidence |
| --- | --- | --- |
| `0x01008000` | Left LCD controller command port. | Small ROM writes page/column commands here; rendering this as the left half aligns the password prompt at x=0. |
| `0x01008001` | Left LCD controller data port. | Data bytes set vertical 8-pixel columns and auto-increment the controller column. |
| `0x01000000` | Right LCD controller command port. | Small ROM writes the second controller through this port. |
| `0x01000001` | Right LCD controller data port. | Data bytes render the right half of the display. |

## Keyboard Matrix

| Address | Meaning | Evidence |
| --- | --- | --- |
| `0xfffff419` / `0x0000f419` | Active-low keyboard matrix input byte. | Small ROM idle scanner and password checker read this byte, invert it, then test column bits. Idle/no key is `0xff`. |
| `0xfffff411` / `0x0000f411` | Observed row-select byte for the Small ROM scanner. | The scanner writes a row value before repeated reads from `0xf419`. |
| `0x00400650` | Small ROM routine that tests one encoded key. | It takes an encoded byte, drives the low-nibble row, reads `0xf419`, then tests bit `encoded >> 4`. |
| `0x00400732` | Small ROM debounce/scanner routine. | It cycles row values, samples `0xf419` until stable, and returns pressed/no-key state. |
| `0x004053ee` | Small ROM password key-code table. | Bytes `3a 3d 7f 30 3a` are compared against the password `ernie`. |

The encoded keyboard byte format is:

```text
encoded = (column_bit_index << 4) | row_index
```

Known labelled key codes from the Small ROM password:

| Key | Encoded byte | Row | Column bit | Active-low `0xf419` value when selected |
| --- | --- | --- | --- | --- |
| `e` | `0x3a` | `0x0a` | `3` | `0xf7` |
| `r` | `0x3d` | `0x0d` | `3` | `0xf7` |
| `n` | `0x7f` | `0x0f` | `7` | `0x7f` |
| `i` | `0x30` | `0x00` | `3` | `0xf7` |

For full firmware-backed matrix layout and the complete raw-logical mapping, see:
[AlphaSmart NEO Full Keyboard Matrix Map](/Users/jakubkolcar/customs/neo-re/docs/2026-04-21-keyboard-matrix-map.md).

This proves the matrix encoding and full matrix placement across all rows/columns.
