# AlphaSmart NEO Emulator Memory Map Notes

Brief record of the memory and MMIO findings validated while booting
`analysis/cab/smallos3kneorom.os3kos` in `alpha-emu`.

## Whole-Device Address Diagram

This is the current evidence-backed map used by the emulator and by the
firmware/app analysis. Addresses are 68k physical addresses as observed by the
firmware. Some flash/persistent-storage ownership is still inferred from the
System firmware behavior, so those regions are labelled carefully.

```text
68k address space, not drawn to scale

0x00000000  +--------------------------------------------------------------+
            | Low RAM / reset mirror                                      |
            |                                                              |
            | 0x00000000..0x0000581a  Small ROM reset-vector mirror        |
            |                         when booting smallos3kneorom.os3kos  |
            |                                                              |
            | Low System RAM records observed during full OS boot:          |
            | 0x00000e0a  runtime System applet pointer table slot 0        |
            | 0x00000e8a  persistent SmartApplet storage start = 0x00470000 |
            | 0x00000e8e  persistent SmartApplet storage end               |
            | 0x00000e92  runtime SmartApplet ID table                     |
            | 0x00000fda  AlphaWord workspace/file table pointer/state      |
            | 0x0000355e  per-applet A5/base adjustment table              |
            | 0x000035e2  current applet slot state                        |
            | 0x000035e6  current applet id, e.g. 0xa000 for AlphaWord      |
            | 0x00003e8a  current SmartApplet callback entry pointer       |
            | 0x00005d94  high word paired with timer register 0xf608       |
            | 0x00007dd8  seeded TimeModule value used by full OS boot      |
0x0000f000  | DragonBall internal register/MMIO window                     |
0x00010000  +--------------------------------------------------------------+
            | General RAM / System-owned persistent workspace in emulator   |
            |                                                              |
            | AlphaWord files are not fixed raw offsets here. The System    |
            | firmware formats and owns the virtual filesystem. Direct USB  |
            | exposes them as logical slots 1..8; observed file/workspace   |
            | state is rooted through the low-RAM records above.            |
0x00400000  +--------------------------------------------------------------+
            | Small ROM executable mapping                                 |
            |                                                              |
            | 0x00400000..0x0040581a  smallos3kneorom.os3kos               |
            | 0x0040042a              Small ROM reset PC                   |
            | 0x00401378              Small ROM entry-chord gate           |
            | 0x004013c0              Small ROM password path              |
            | 0x004053ee              password key-code table              |
0x0040581a  +--------------------------------------------------------------+
            | Full OS secondary segment area                               |
            |                                                              |
            | 0x00406000..0x00406014  os3kneorom segment from updater      |
0x00410000  +--------------------------------------------------------------+
            | Full System 3 Neo firmware package / main OS segment         |
            |                                                              |
            | 0x00410000..0x00470000  main executable segment              |
            | 0x00410070              updater entry stub                   |
            | 0x00410082              normal System boot entry             |
            | 0x00426768              Line-A trap vector handler target    |
            | 0x00470000..0x00470800  tail of whole host package mapping;  |
            |                         applet storage starts at 0x00470000   |
0x00470000  +--------------------------------------------------------------+
            | Persistent SmartApplet package chain                         |
            |                                                              |
            | 0x00470000..0x0048a0bc  AlphaWord Plus                       |
            | 0x0048a0bc..0x00496360  AlphaQuiz                            |
            | 0x00496360..0x0049c340  Calculator                           |
            | 0x0049c340..0x004bb2e8  KeyWords                             |
            | 0x004bb2e8..0x005126a8  SpellCheck Large USA                 |
            | 0x005126a8..0x0051a5ec  Beamer                               |
            | 0x0051a5ec..0x00521100  Control Panel                        |
            | 0x00521100..0x0057a9cc  Thesaurus Large USA                  |
            | 0x0057a9cc..0x0057d690  Text2Speech Updater                  |
            | 0x0057d690..0x0057db2c  Alpha USB native                     |
            | 0x0057db2c..0x0058087c  Forth Mini                           |
0x0058087c  +--------------------------------------------------------------+
            | High firmware/update segment area                            |
            |                                                              |
            | 0x005ffc00..0x00600000  os3kneorom segment from updater      |
0x00600000  +--------------------------------------------------------------+
            | Unmapped/unused in current evidence-backed emulator map       |
0x007fffff  +--------------------------------------------------------------+

0x01000000  +--------------------------------------------------------------+
            | LCD controller MMIO                                          |
            | 0x01000000  right LCD command port                           |
            | 0x01000001  right LCD data port                              |
            | 0x01008000  left LCD command port                            |
            | 0x01008001  left LCD data port                               |
0x01008002  +--------------------------------------------------------------+

0x02000000  +--------------------------------------------------------------+
            | ASIC/board register window used by full System firmware      |
            | 0x02000000..0x02000007                                      |
0x02000008  +--------------------------------------------------------------+

0xffff0000  +--------------------------------------------------------------+
            | Sign-extended alias of low internal registers                |
            |                                                              |
            | 0xfffff202  PLLFSR high byte / CLK32 status bit              |
            | 0xfffff408  GPIO keyboard row-select helper                  |
            | 0xfffff410  GPIO keyboard row-select register group          |
            | 0xfffff411  Small ROM observed row-select byte               |
            | 0xfffff419  active-low keyboard matrix input byte            |
            | 0xfffff440  GPIO keyboard row-select register group          |
            | 0xfffff449  timer-ready bit used by full OS boot             |
            | 0xfffff608  16-bit timer source                              |
            | 0xfffffb00..0xfffffb03  timebase high/low words              |
            | 0xfffffb1a  timebase fractional/low counter word             |
0xffffffff  +--------------------------------------------------------------+
```

### AlphaWord File Placement

AlphaWord document contents are best described as a firmware-owned logical
filesystem, not as a fixed address range we can safely edit directly.

```text
Direct USB protocol view:

  slot 1..8
     |
     +-- read-only attribute command 0x13 gives name/length/reserved size
     +-- read-only get command retrieves text bytes by slot

Firmware memory view observed during boot:

  0x00000fda / 0x00000fde  AlphaWord workspace table/state
       |
       +-- System firmware-created file/workspace records
             |
             +-- AlphaWord applet opens "File 1" and edits through OS file APIs

Rule: do not synthesize or patch AlphaWord records by address. Let the System
firmware format/repair them, then use direct USB read-only commands for backups.
```

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

The original validated path was:

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

To avoid re-entering the battery/recovery prompt on every fresh emulator boot,
the emulator can now persist a recovery seed produced by the real firmware
recovery path. The seed is written by booting with blank low RAM, pressing
`Y`, pressing Enter, letting the firmware repair/format path run, and then
saving only these RAM ranges:

| Range | Why it is persisted |
| --- | --- |
| `0x00000400..0x00000800` | Firmware recovery marker page. |
| `0x00000e00..0x00001b00` | Applet runtime tables plus AlphaWord records observed after recovery. |

The default local seed path is
`alpha-emu/state/full-system-recovery.seed`; that directory is ignored by git.
If the seed exists, the full-OS emulator overlays these ranges before the CPU
starts. It does not hardcode only the marker or serialize the full 8 MiB RAM
image.

### Recovery-Gate Diff Findings

With a completely blank low RAM marker, the full OS shows the recovery prompt:

```text
An unexpected data change occurred.
Did you recently remove or replace the
AlphaSmart's lithium backup battery?
(Y for yes, N for no)
```

To isolate the cause, two 8 MiB emulator memory images were captured:

1. Fresh initialized memory before executing any firmware.
2. Memory after booting the full OS, pressing `Y`, pressing Enter, and letting
   firmware complete its repair/format path.

Then fresh memory was overlaid with specific post-recovery ranges and booted
without pressing `Y`.

Overlay result:

| Overlay copied from post-recovery image | Recovery prompt skipped? | Notes |
| --- | --- | --- |
| `0x00000400..0x00000800` | yes | LCD hash matched the recovered-memory non-recovery path. |
| `0x00000e00..0x00001200` | no | Applet/runtime tables alone still reached the recovery screen. |
| `0x00000fda..0x00001b00` | no | AlphaWord records alone still reached the recovery screen. |
| `0x00000e00..0x00001200` + `0x00000fda..0x00001b00` | no | Applet tables plus AlphaWord records still reached the recovery screen. |
| `0x00000400..0x00001b00` | yes | Expected, because it includes the `0x400` marker page. |
| `0x00018400..0x00019500` | no | Workspace block alone did not satisfy the recovery gate. |
| `0x0006f800..0x00070900` | no | Workspace block alone did not satisfy the recovery gate. |
| `0x00071000..0x00072400` | no | Workspace block alone did not satisfy the recovery gate. |
| `0x0007c300..0x0007cc00` | no | Workspace block alone did not satisfy the recovery gate. |

This strongly suggests that the boot recovery gate is primarily checking the
low marker page around `0x00000400`, not the applet table or AlphaWord file
records themselves. After recovery, that page starts with:

```text
0x00000400: "I am not corrupted!\0"
```

The minimized passing subset is just the non-NUL bytes
`0x00000400..0x00000413`:

```text
0x00000400: "I am not corrupted!"
```

This works because the surrounding fresh RAM is zero-filled, so the C-string
comparison still terminates at `0x00000413`.

Marker-only seeding was rejected as the emulator fix because it suppresses the
prompt without preserving the applet/AlphaWord state produced by the firmware
repair path. It remains useful only as a minimized proof of the recovery gate.

Static firmware analysis ties this directly to the full OS recovery routine:

| Runtime address | Behavior |
| --- | --- |
| `0x004264c0` | Pushes ROM string pointer `0x00448d16`. |
| `0x004264c6` | Pushes RAM marker pointer `0x00000400`. |
| `0x004264cc` | Calls `0x00436706`, a bytewise `strcmp` helper. |
| `0x004264d6` | Branches to the non-recovery path when the compare returns zero. |
| `0x00426566` | In the `Y` recovery-confirmation path, pushes ROM string pointer `0x00448d2a`. |
| `0x0042656c` | Pushes RAM marker pointer `0x00000400`. |
| `0x00426572` | Calls `0x0043672a`, a bytewise `strcpy` helper, writing the accepted marker. |

The recovery prompt resources are resolved through the firmware resource lookup
at `0x00424212`; IDs `0xdb..0xde` map to the four prompt lines, and IDs
`0xdf..0xe2` map to the restart confirmation shown after `Y`.

Two hardware details were required for this to work:

- `0xf608` is a 16-bit timer source, but firmware combines it with a high-word
  base at `0x00005d94`. The emulator advances `0xf608` and increments
  `0x00005d94` on 16-bit wrap so firmware delay arithmetic remains monotonic.
  Full OS timer setup at `0x00424df0..0x00424e02` writes `0xf602 = 0x0020`,
  `0xf604 = 0xd6d8`, then `0xf600 = 0x0019`; this makes the `0xf608` counter
  track the 32.768 kHz clock divided by `0x20 + 1`, about 993 Hz. Advancing it
  from deferred-queue polling made AlphaWord timer behavior, including cursor
  blink, run far too fast.
- Firmware `STOP` at `0x00426752` resumes at `0x00426756` after an interrupt.
  Treat it as a wakeable low-power wait, not as a reset. Resetting there caused
  repeated splash-screen cycles after the filesystem was already formatted.

Cursor rendering note: after AlphaWord reaches a blank file, headless sampling
shows the framebuffer cursor pixels are static and the firmware does not
continuously rewrite the LCD for blink. The GUI therefore overlays the visible
NEO-style blink by hiding isolated tall cursor runs during the off phase. The
mask is limited to the detected run, not the full display column, so text pixels
above or below the cursor on the same x coordinate remain visible. This is a
display-layer behavior and does not mutate emulated LCD RAM.

The validated AlphaWord blank-editor cursor dump is a narrow block at columns
`0..1`, rows `0..15`. The headless validation command writes paired PBM dumps:

```sh
cargo run -q --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=1700000000 \
  --lcd-ranges \
  --lcd-blink-pbm-prefix=/tmp/neo-cursor \
  analysis/cab/os3kneorom.os3kos
```

Expected key lines:

```text
y=000..015: 000..001
lcd_blink_pbm_on=/tmp/neo-cursor-on.pbm off=/tmp/neo-cursor-off.pbm diff_pixels=32
```

`32` changed pixels equals `2` cursor columns times `16` cursor rows. The
off-frame PBM has no lit pixels in those rows for the blank-editor state.

Validated recovery-seed generation:

```sh
cargo run -q --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --reinit-memory \
  --recovery-seed=/tmp/neo-recovery.seed \
  --steps=120000000 \
  --stop-at-resource=0xdb \
  analysis/cab/os3kneorom.os3kos
```

The command writes a 4,384-byte seed, reloads it, and verifies that the full OS
does not hit recovery prompt resource `0xdb`. The seed generator now stops when
the firmware reaches the post-recovery boot point `0x00435a26`, rather than
blindly running to the later AlphaWord editor state. This keeps GUI
`Reinit memory` responsive and normally finishes in a few seconds on the
development machine.

Validated AlphaWord flow:

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
viewport as the top-left 264x64 square-pixel crop. The lower and right-side
controller rows/columns remain in the backing buffer for firmware compatibility
but are not part of the normal visible NEO screen.

| Address | Meaning | Evidence |
| --- | --- | --- |
| `0x01008000` | Left LCD controller command port. | Small ROM writes page/column commands here; rendering this as the left half aligns the password prompt at x=0. |
| `0x01008001` | Left LCD controller data port. | Data bytes set vertical 8-pixel columns and auto-increment the controller column. |
| `0x01000000` | Right LCD controller command port. | Small ROM writes the second controller through this port. |
| `0x01000001` | Right LCD controller data port. | Data bytes render the right half of the display. |

Command `0x40..0x7f` is display-start-line state, not a column reset. Treating
it as a column reset made firmware cursor redraws write at x=0, leaving stale
cursor pixels as a vertical black trail when moving between AlphaWord lines.

LCD data-port reads must return the controller framebuffer byte, not the last
generic MMIO byte. AlphaWord cursor movement uses controller read-modify-write:
`0xe0` starts RMW mode, data reads do not advance the column, the following
write updates the same byte and advances, and `0xee` restores the original RMW
column. Without this, old cursor columns remain visible after arrow movement.

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
