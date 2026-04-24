# Alpha Emulator Headless Usage

All commands below are run from the repo root.

Base form:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  [options] \
  analysis/cab/os3kneorom.os3kos
```

If no firmware path is given, the default is `../analysis/cab/smallos3kneorom.os3kos`
relative to `alpha-emu`.

## Summary Output

Normal headless runs end with:

```text
pc=0x... ssp=0x... steps=... cycles=... elapsed_ms=... achieved_hz=... target_hz=33000000 stopped=... stop_at=... exception=...
```

Fields:

| Field | Meaning |
| --- | --- |
| `pc` | current program counter |
| `ssp` | supervisor stack pointer |
| `steps` | interpreted instruction count |
| `cycles` | emulated 68000 cycle count |
| `elapsed_ms` | host wall-clock runtime |
| `achieved_hz` | host-side throughput, not device clock |
| `target_hz` | pacing target, fixed at 33 MHz |
| `stopped` | CPU STOP state |
| `stop_at` | result of `--stop-at-pc` or `--stop-at-resource`; otherwise `n/a` |
| `exception` | last unhandled exception or `none` |

## Boot Control

Basic boot:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- --headless --steps=200000
```

Full OS boot:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  analysis/cab/os3kneorom.os3kos
```

Boot with an all-rows key chord:

```sh
--boot-keys=0x0e,0x0c
```

Boot with exact row visibility instead of all-rows visibility:

```sh
--boot-keys-exact=0x0e,0x0c
```

Boot directly into SmartApplets:

```sh
--boot-left-shift-tab
```

This is equivalent to holding raw keys `0x0e,0x0c` during reset.

## Stop Conditions

Stop before a PC:

```sh
--stop-at-pc=0x00426752
```

Stop on the Nth hit of a PC:

```sh
--stop-at-pc=0x00426752 --stop-at-pc-hit=2
```

Stop before a firmware resource lookup:

```sh
--stop-at-resource=0x006b
```

## Scripted Input

The scheduler is instruction-step based.

Text at a step:

```sh
--type-at=31000000:7+2
```

Key at a step:

```sh
--key-at=33000000:enter
```

Hold a key over a step range:

```sh
--hold-key=28100000-30000000:esc
```

Force a key visible on any selected row:

```sh
--key-all-rows-at=STEP:KEY
```

Immediate text after scripted boot/launch:

```sh
--type-now=hello
```

Immediate keys after scripted boot/launch:

```sh
--key-now=enter,backspace
```

Key names:

```text
enter return up down left right esc escape tab backspace
applets send find print spell-check spellcheck clear-file clearfile
file1 file-1 file2 file-2 file3 file-3 file4 file-4
file5 file-5 file6 file-6 file7 file-7 file8 file-8
0xNN
```

Semantics:

| Flag | Injection mode |
| --- | --- |
| `--type-at` | per-character debug tap profile |
| `--key-at` | debug tap profile |
| `--key-all-rows-at` | debug tap profile, forced visible on all rows |
| `--hold-key` | exact press/release at scheduled steps |
| `--type-now` | same as `--type-at`, applied after scheduled headless stepping |
| `--key-now` | same as `--key-at`, applied after scheduled headless stepping |

`--type-at` is for normal printable input. `--key-at` is for control/navigation
keys and raw matrix tests.

## LCD Output

Full LCD coarse ASCII:

```sh
--lcd-ascii
```

Visible 264x64 area ASCII:

```sh
--lcd-visible-ascii
```

Full LCD bits to stdout:

```sh
--lcd-bits
```

Full LCD bits to file:

```sh
--lcd-bits-path=/tmp/lcd-bits.txt
```

Full LCD PBM:

```sh
--lcd-pbm=/tmp/lcd-full.pbm
```

Visible LCD PBM:

```sh
--lcd-visible-pbm=/tmp/lcd-visible.pbm
```

Blink pair PBMs with cursor-on and cursor-off renderings:

```sh
--lcd-blink-pbm-prefix=/tmp/lcd-blink
```

Occupied x-ranges per row:

```sh
--lcd-ranges
```

Sample LCD hash/diff over time:

```sh
--sample-lcd-after=INTERVAL_STEPS:COUNT
```

Complete dump directory:

```sh
--lcd-dump-dir=/tmp/alpha-emu-lcd
```

This writes:

- `lcd-full-bits.txt`
- `lcd-full.pbm`
- `lcd-visible.pbm`
- `lcd-visible-ocr.pbm`
- `lcd-ocr.txt` when OCR succeeds

OCR:

```sh
--lcd-ocr
--lcd-ocr-scale=8
```

Recommended order:

1. Use the emulator text-capture path first.
2. Use bitmap OCR only when no emulator text layer is available.

For text-heavy applet and firmware screens, prefer:

- `--lcd-ocr`
- `--lcd-dump-dir=...` and inspect `lcd-ocr.txt`

These use the emulator-captured text layer when present and are the primary
debugging path. Bitmap OCR is fallback only.

Text extraction behavior:

| Screen type | OCR source |
| --- | --- |
| trap-rendered text screens | emulator-captured text layer |
| raw pixel screens | visible LCD bitmap passed to `tesseract` |

## Debug Modes

Verbose traced execution:

```sh
--verbose
```

This switches from the fast interpreter path to traced execution and prints:

- registers
- debug words
- MMIO log
- recent instruction trace

Keep verbose runs short.

Matrix visibility scan from a given state:

```sh
--scan-matrix-visibility-at=STEP
```

Special key scan:

```sh
--scan-special-keys-at=STEP
```

Key-map validation:

```sh
--validate-key-map-at=STEP
```

## Memory Overlay / Capture

Rebuild and save recovery seed:

```sh
--reinit-memory
```

Use an explicit recovery seed path:

```sh
--recovery-seed=/tmp/full-system-recovery.seed
```

Overlay raw memory bytes into the current session:

```sh
--load-memory=/tmp/memory.bin
```

Dump memory before execution:

```sh
--dump-memory-start=/tmp/memory-start.bin
```

Dump memory after execution:

```sh
--dump-memory=/tmp/memory-end.bin
```

## Built-In Validation / Launch Helpers

Launch Forth Mini through direct validation context:

```sh
--launch-forth-mini
```

Launch stock Calculator through direct validation context:

```sh
--launch-calculator
```

These require the full OS image.

Built-in validators:

```sh
--validate-alpha-usb-native
--validate-forth-mini
```

`--validate-forth-mini` boots the full system, enters SmartApplets, launches
Forth Mini through the menu, sends a fixed evaluation sequence, and fails if the
LCD does not change or an exception occurs.

## Verified Calculator Workflow

This is the currently verified full-System headless workflow for stock
Calculator interaction:

1. Boot full System with `--boot-left-shift-tab`.
2. Navigate SmartApplets with two `down` presses and `enter`.
3. Dismiss Calculator help by holding `esc`.
4. Send expression with `--type-at`.
5. Evaluate with `enter`.
6. Read result via `--lcd-ocr`.

Example:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --boot-left-shift-tab \
  --steps=37000000 \
  --key-at=25100000:down \
  --key-at=26100000:down \
  --key-at=27100000:enter \
  --hold-key=28100000-30000000:esc \
  --type-at=31000000:7+2 \
  --key-at=33000000:enter \
  --lcd-ocr \
  analysis/cab/os3kneorom.os3kos
```

Verified expressions:

- `8-5 -> 3`
- `9/3 -> 3`
- `7+2 -> 9`
- `2x3 -> 6`

Calculator renders multiplication as `*` in OCR output even when the scripted
input used `x`.

## Verified Forth Mini Workflow

Direct applet launch:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --launch-forth-mini \
  --type-now=1 \
  --key-now=enter \
  --lcd-ocr \
  analysis/cab/os3kneorom.os3kos
```

Menu-path validation:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --validate-forth-mini \
  analysis/cab/os3kneorom.os3kos
```

When `--launch-forth-mini` is active:

- `--type-now` and `--type-at` dispatch Forth Mini `0x20` Char messages directly
- `--key-now` dispatches Forth Mini `0x21` Key messages directly

This is specific to Forth Mini debug launch. It does not apply to normal stock
firmware text entry.
