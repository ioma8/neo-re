# AlphaSmart NEO Emulator Memory Map Notes

Brief record of the memory and MMIO findings validated while booting
`analysis/cab/smallos3kneorom.os3kos` in `alpha-emu`.

## ROM and RAM

| Range | Meaning | Evidence |
| --- | --- | --- |
| `0x00000000..` | Reset-vector mirror of the Small ROM image. | CPU reset vectors are read from address `0`; initial SSP is `0x0007fff0`, reset PC is `0x0040042a`. |
| `0x00400000..` | Executable Small ROM mapping. | Small ROM code and strings disassemble/run at runtime base `0x00400000`. |
| `0x00410000..` | Whole `os3kneorom.os3kos` System 3 Neo package mapping. | Full firmware branches and absolute calls line up only when the whole package, including its `0x70`-byte header, is mapped at `0x00410000`; the entry stub at `0x00410070` jumps to valid code at `0x00417914`. |
| `0x00470000..` | Persistent SmartApplet package storage used by the full OS emulator path. | Full OS routine `0x004130fc` clears the runtime applet tables, then rebuilds them by walking a contiguous `0xc0ffeead`/`0xcafefeed` package chain starting at the storage pointer in `0x00000e8a`. |
| `0x00000e0a` | Full-OS RAM applet pointer table, slot 0. | Full OS dispatch reads this table and then calls through the applet entry pointer at package offset `0x84`. Seeding slot 0 with the embedded System package address lets the full image reach LCD drawing and keyboard idle code. |
| `0x00000e8a` | Full-OS persistent SmartApplet storage start pointer. | Firmware initialization writes `0x00470000` here before rebuilding the applet table. The emulator maps backed-up stock applets contiguously at that address. |
| `0x00000e8e` | Full-OS persistent SmartApplet storage end pointer. | The firmware normally derives this from flash CFI geometry. The emulator currently protects the synthetic chain end because the flash query path is only minimally modeled and otherwise produces an invalid `0x003f0000` bound. |
| `0x00000e92` | Full-OS runtime applet ID table. | After `0x004130fc` walks the package chain, slot 1 contains `0xa000` for AlphaWord and following slots contain the stock applet IDs. |
| `0x0000355e` | Full-OS per-applet A5/base adjustment table. | During table rebuild, the firmware stores `0x0007d800 - package.data_offset` for each valid SmartApplet. AlphaWord slot 1 yields `0x0007ca70`. |
| `0x00003e8a` | Current SmartApplet callback entry pointer. | The dispatcher stores `applet_base + header.entryPoint` here before calling the current applet. With stock AlphaWord loaded, this becomes `0x00470094`. |
| `0x00000000..0x007fffff` | Current emulator backing memory. | Large enough for reset mirror, ROM mapping, RAM state, and observed Small ROM stack use. |

`analysis/cab/os3kneorom.os3kos` is not reset-vector bootable as a flat ROM.
Its header starts with `0xffffffff 00015379`, and treating those bytes as reset
vectors immediately fails. The updater segment table at file offset `0x50`
describes real-device erase/program bookkeeping:

| Segment address | Length |
| --- | --- |
| `0x00410000` | `0x00060000` |
| `0x00406000` | `0x00000014` |
| `0x005ffc00` | `0x00000400` |

For emulator execution, the package is mapped as a whole image at `0x00410000`;
mapping only segment payloads shifts the code by `0x70` bytes and makes absolute
branches land in the wrong instructions.

The two full-OS entry stubs have different behavior:

| Entry | Behavior |
| --- | --- |
| `0x00410070` | Updater path. It reaches the LCD message `Attempting to enter the Updater Mode. Attach an Updater cable. Start AlphaSmart Manager.` |
| `0x00410082` | Normal System boot path. With the current RAM/timebase/storage seeds, it reaches the stock AlphaWord applet through the firmware SmartApplet dispatcher. |

Current full-OS boot status:

- The data-change prompt is no longer bypassed. On a blank virtual persistent
  store, the firmware shows:
  `An unexpected data change occurred. Did you recently remove or replace the AlphaSmart's lithium backup battery?`
  Answering `Y` and pressing Enter lets the System firmware run its own repair
  and format path.
- The TimeModule divide fault is avoided by seeding `0x00007dd8 = 0x00000830`
  and returning the timer-ready bit from `0xf449`.
- The stock applet package chain is loaded from
  `analysis/device-dumps/applets/A*.os3kapp`, excluding font-only `AF*`
  packages, at `0x00470000`.
- The normal boot path now reaches AlphaWord without synthetic AlphaWord file
  records. The System firmware formats the virtual file workspace and writes the
  AlphaWord state itself.

## AlphaWord Virtual Filesystem Format

Earlier emulator builds seeded AlphaWord-visible file records directly, including
the applet file table around `0x00000fd2` and synthetic file records at
`0x00060000`. That was the wrong layer. It overlapped low System file-descriptor
state and could trip the firmware memmove/file-size diagnostics before
AlphaWord opened `File 1`.

The validated path is:

1. Leave the emulated persistent store blank on full OS boot.
2. Let the System firmware detect the data-change condition.
3. Answer the firmware prompt with `Y`, then Enter.
4. Let the firmware repair/format routine initialize the virtual filesystem.
5. Continue normal boot. AlphaWord opens `Opening "File 1"...`, then reaches the
   blank editor.

Observed firmware-written state after the repair/format path:

| Address | Observed value | Meaning |
| --- | --- | --- |
| `0x00000400` | `0x4920616d` | Start of restored `I am...` integrity marker. |
| `0x00000e94` | `0xa000a001` | Runtime applet ID table includes AlphaWord and following applet IDs. |
| `0x00000fda` | `0x00001822` | AlphaWord file/workspace table pointer/state written by firmware. |
| `0x00000fde` | `0x08000001` or `0x08ff0000` during transition | AlphaWord slot/file count and current-state bytes. |
| `0x000035e2` | `0x00000001` | Current applet slot state. |
| `0x000035e6` | `0xa0000000` | Current applet id includes AlphaWord `0xa000`. |
| `0x000035ec` | `0x00000001` | Current file/slot selector state. |

The emulator must not synthesize AlphaWord file records in `EmuMemory`. The full
System firmware owns this format path, and the emulator only needs to provide
enough RAM, applet storage, timers, keyboard, and LCD behavior for it to run.

Two hardware details were required for this to work:

- `0xf608` is a 16-bit timer source, but firmware combines it with a high-word
  base at `0x00005d94`. The emulator advances `0xf608` and increments
  `0x00005d94` on 16-bit wrap so firmware delay arithmetic remains monotonic.
- Firmware `STOP` at `0x00426752` resumes at `0x00426756` after an interrupt.
  Treat it as a wakeable low-power wait, not as a reset. Resetting there caused
  repeated splash-screen cycles after the filesystem was already formatted.

Validated headless flow:

```sh
cargo run -q --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=1700000000 \
  --type-at=9000000:Y \
  --key-at=18000000:enter \
  --type-at=1400000000:hello \
  --lcd-pbm=/tmp/exact-type-lower-1_7b.pbm \
  --lcd-ascii \
  analysis/cab/os3kneorom.os3kos
```

The resulting LCD shows `hello` in the AlphaWord editor. Headless text injection
must use exact-row key taps; all-row text taps are useful for early boot probes
but can be decoded as the wrong character once the full OS matrix scanner is
running.

## Line-A Trap Vectoring

SmartApplets call OS services through Motorola 68k Line-A opcodes such as the
`0xa2b8` stub reached by AlphaWord at `0x00482ece`. The `m68000` crate reports
Line-A as an exception to the host instead of automatically entering the vector.

The full OS initializes vector `10` at low-memory vector table address
`0x00000028` to `0x00426768`. The emulator now handles Line-A by pushing the
standard 68000 exception frame (`SR`, return `PC`) on the supervisor stack and
jumping to that firmware vector. This keeps the firmware responsible for the OS
trap instead of reintroducing Rust-side A-line service shims.

## Internal Register Window

| Range | Meaning | Evidence |
| --- | --- | --- |
| `0x0000f000..0x0000ffff` | DragonBall-style internal register/MMIO window. | Small ROM accesses byte and word registers in this range during early hardware setup. |
| `0xfffff000..0xffffffff` | Sign-extended alias of the same register window. | 68k absolute-short addressing reaches registers such as `0xfffff419`; preserving state between low and sign-extended aliases lets boot continue. |
| `0xfffff202` / `0x0000f202` | PLL frequency select/status high byte (`PLLFSR`). | The MC68VZ328 map names `0xfffff202` as `PLLFSR`; firmware polls high-byte bit 7 at `0x0042673c`/`0x00426744`, matching the `CLK32` indicator. It must not increment on every read. The emulator now toggles bit 7 from simulated 33 MHz CPU time at a 32.768 kHz edge rate and preserves the other byte bits. |
| `0xfffffb00..0xfffffb03` / `0x0000fb00..0x0000fb03` | Timebase high/low words. | Full OS routine `0x004247ca` combines these words with `0xfb1a` into an elapsed-time value used by initialization delays. |
| `0xfffffb1a` / `0x0000fb1a` | Timebase fractional/low counter word. | The same routine masks this value with `0x01ff`; keeping it at zero traps the firmware on the initialization screen. |
| `0x02000000..0x02000007` | ASIC/board register window used by System firmware. | Normal full-OS boot writes `0x0200000/2/6` through routines near `0x0043ff54`; treating it as byte-preserving MMIO avoids a bus error and matches command/data-style access. |

## LCD Controller Ports

The NEO LCD behaves like two page/column byte-addressed controllers, each
covering half of a 320x128 controller buffer. The GUI displays the active NEO
viewport as the top-left 256x64 square-pixel crop. The lower and right-side
controller rows/columns remain in the backing buffer for firmware compatibility
but are not part of the normal visible NEO screen.

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
| `0x00401378` | Small ROM entry-chord gate. | It checks encoded keys `0x6e`, `0x60`, `0x62`, and `0x73`; only then does the boot flow call the password routine at `0x004013c0`. |
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

The emulator's normal boot path does not hold this chord. The GUI's explicit
`Reboot Small ROM with activating key chord` action uses the entry-chord gate as
boot state, then releases those keys before the password scanner. The firmware
then waits for user input instead of receiving a scripted password.

For full firmware-backed matrix layout and the complete raw-logical mapping, see:
[AlphaSmart NEO Full Keyboard Matrix Map](/Users/jakubkolcar/customs/neo-re/docs/2026-04-21-keyboard-matrix-map.md).

This proves the matrix encoding and full matrix placement across all rows/columns.
