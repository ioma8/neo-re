# Alpha Emulator Headless Usage

This is a command map for `alpha-emu` headless runs. It focuses on repeatable
firmware and SmartApplet checks from the repo root.

Use this base form:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  [options] \
  analysis/cab/os3kneorom.os3kos
```

If no firmware path is provided, the emulator defaults to
`../analysis/cab/smallos3kneorom.os3kos` relative to `alpha-emu`.

## Output

Every normal headless run ends with one summary line:

```text
pc=0x00435a26 ssp=0x0007ffba steps=119976278 cycles=1247089312 elapsed_ms=42090 achieved_hz=29628757 target_hz=33000000 stopped=false stop_at=false exception=none
```

Important fields:

| Field | Meaning |
| --- | --- |
| `pc` | Current firmware program counter. |
| `ssp` | Supervisor stack pointer. |
| `steps` | Interpreted instruction count. |
| `cycles` | Emulated 68000 cycle count reported by the interpreter. |
| `achieved_hz` | Host execution speed, not the emulated device clock. |
| `target_hz` | Real-time target clock, fixed at 33 MHz for NEO full-speed runs. |
| `stopped` | Whether the emulated CPU is currently in STOP state. |
| `stop_at` | Whether a requested stop condition was reached. |
| `exception` | Last unhandled CPU exception, or `none`. |

## Basic Boot Runs

Small ROM default boot:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- --headless --steps=200000
```

Small ROM explicit path:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=200000 \
  analysis/cab/smallos3kneorom.os3kos
```

Full System 3 boot:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  analysis/cab/os3kneorom.os3kos
```

Run until a known PC is reached:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=250000000 \
  --stop-at-pc=0x00435a26 \
  analysis/cab/os3kneorom.os3kos
```

Stop at the second time a PC is reached:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=5000000 \
  --stop-at-pc=0x00426752 \
  --stop-at-pc-hit=2 \
  analysis/cab/os3kneorom.os3kos
```

## LCD Inspection

Print a coarse ASCII rendering:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  --lcd-ascii \
  analysis/cab/os3kneorom.os3kos
```

Write a PBM image:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  --lcd-pbm=/tmp/neo-lcd.pbm \
  analysis/cab/os3kneorom.os3kos
```

List occupied x ranges by LCD row. This is useful for crop, cursor, and layout
debugging:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  --lcd-ranges \
  analysis/cab/os3kneorom.os3kos
```

## Traces and MMIO

Verbose mode uses the slower traced interpreter path and prints recent
registers, debug words, MMIO accesses, and instruction trace lines:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --verbose \
  --steps=200000 \
  analysis/cab/smallos3kneorom.os3kos
```

Keep verbose runs short. The trace buffers are intentionally bounded so command
output stays usable.

## Keyboard Input

Type text at an instruction step:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=1700000000 \
  --type-at=1400000000:hello \
  --lcd-ascii \
  analysis/cab/os3kneorom.os3kos
```

Press a named key at a step:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=20000000 \
  --key-at=9000000:enter \
  analysis/cab/smallos3kneorom.os3kos
```

Hold a key over a step range:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=30000000 \
  --hold-key=5000000-9000000:down \
  analysis/cab/os3kneorom.os3kos
```

Use exact matrix code by hex:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=20000000 \
  --key-at=9000000:0x69 \
  analysis/cab/os3kneorom.os3kos
```

Supported named keys:

```text
enter return up down left right esc escape tab backspace
applets send find print spell-check spellcheck clear-file clearfile
file1 file-1 file2 file-2 file3 file-3 file4 file-4
file5 file-5 file6 file-6 file7 file-7 file8 file-8
```

`--key-all-rows-at=STEP:KEY` forces the key visible on any scanned row. Use it
only for early boot or discovery probes; normal full-System text entry should
prefer `--type-at` or `--key-at`.

## Boot Chords

Boot full System 3 directly into the SmartApplets list:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --boot-left-shift-tab \
  --steps=18000000 \
  --stop-at-resource=0x6b \
  analysis/cab/os3kneorom.os3kos
```

Boot with custom all-row matrix keys held briefly:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --boot-keys=0x0e,0x0c \
  --steps=18000000 \
  analysis/cab/os3kneorom.os3kos
```

Boot with exact-row matrix keys held longer:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --boot-keys-exact=0x0e,0x0c \
  --steps=18000000 \
  analysis/cab/os3kneorom.os3kos
```

## Resource Stops

Stop when firmware resolves a resource string ID:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  --stop-at-resource=0xdb \
  analysis/cab/os3kneorom.os3kos
```

Useful resource IDs:

| ID | Use |
| --- | --- |
| `0x6b` | SmartApplets menu/help resource path. |
| `0xdb` | Full System recovery prompt: `An unexpected data change occurred.` |

## Recovery Seed

The full System image can need a low-memory recovery seed. Generate it from the
firmware recovery path:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --reinit-memory \
  analysis/cab/os3kneorom.os3kos
```

Use an explicit seed path:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --reinit-memory \
  --recovery-seed=/tmp/neo-recovery.seed \
  analysis/cab/os3kneorom.os3kos
```

Load an existing seed:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --recovery-seed=/tmp/neo-recovery.seed \
  --steps=120000000 \
  --stop-at-resource=0xdb \
  analysis/cab/os3kneorom.os3kos
```

Expected result with a good seed:

```text
recovery_seed_loaded=/tmp/neo-recovery.seed
... stop_at=false exception=none
```

Expected result with no seed:

```text
... pc=0x00424212 ... stop_at=true exception=none
```

The default seed path is:

```text
alpha-emu/state/full-system-recovery.seed
```

That directory is ignored by git.

## Memory Snapshots

Dump memory immediately after session creation and optional seed load:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --dump-memory-start=/tmp/neo-start.bin \
  --steps=0 \
  analysis/cab/os3kneorom.os3kos
```

Dump memory after a run:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  --dump-memory=/tmp/neo-after.bin \
  analysis/cab/os3kneorom.os3kos
```

Load an 8 MiB memory image before running:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --load-memory=/tmp/neo-after.bin \
  --steps=120000000 \
  analysis/cab/os3kneorom.os3kos
```

Snapshot diffing workflow:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --dump-memory-start=/tmp/pre.bin \
  --steps=120000000 \
  --dump-memory=/tmp/post.bin \
  analysis/cab/os3kneorom.os3kos
cmp -l /tmp/pre.bin /tmp/post.bin | sed -n '1,40p'
```

## Applet Validation Shortcuts

Validate Alpha USB native applet callback shape:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --validate-alpha-usb-native \
  analysis/cab/os3kneorom.os3kos
```

Validate Forth Mini applet callback shape:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --validate-forth-mini \
  analysis/cab/os3kneorom.os3kos
```

These shortcuts call applet message handlers directly through the validation
harness. They are fast ABI and crash checks; they are not full end-user UI
scripts through the SmartApplets menu.

## Matrix Discovery

Scan special keys for menu/help resource behavior:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --scan-special-keys-at=120000000 \
  analysis/cab/os3kneorom.os3kos
```

Check whether every known matrix cell becomes visible to firmware scanning:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --scan-matrix-visibility-at=120000000 \
  analysis/cab/os3kneorom.os3kos
```

Validate current key mapping assumptions:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --validate-key-map-at=120000000 \
  analysis/cab/os3kneorom.os3kos
```

## Scenario Recipes

Detect whether a fresh full-System boot needs recovery:

```sh
rm -f /tmp/missing.seed
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --recovery-seed=/tmp/missing.seed \
  --steps=120000000 \
  --stop-at-resource=0xdb \
  analysis/cab/os3kneorom.os3kos
```

Generate a recovery seed and verify the prompt is skipped:

```sh
rm -f /tmp/neo-recovery.seed
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --reinit-memory \
  --recovery-seed=/tmp/neo-recovery.seed \
  --steps=120000000 \
  --stop-at-resource=0xdb \
  analysis/cab/os3kneorom.os3kos
```

Boot to SmartApplets menu and save an LCD image:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --boot-left-shift-tab \
  --steps=18000000 \
  --lcd-pbm=/tmp/smartapplets.pbm \
  analysis/cab/os3kneorom.os3kos
```

Run enough full-System firmware for AlphaWord and type text:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=1700000000 \
  --type-at=1400000000:hello \
  --lcd-pbm=/tmp/alphaword-hello.pbm \
  --lcd-ascii \
  analysis/cab/os3kneorom.os3kos
```

Exercise arrow cursor movement after typing:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=1700200000 \
  --type-at=1400000000:hello \
  --key-at=1500000000:enter \
  --type-at=1500100000:world \
  --key-at=1600000000:up \
  --key-at=1600100000:down \
  --lcd-pbm=/tmp/cursor-move.pbm \
  analysis/cab/os3kneorom.os3kos
```

Capture a short MMIO trace around an early boot path:

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --verbose \
  --steps=100000 \
  analysis/cab/smallos3kneorom.os3kos
```

Validate Forth Mini ABI after rebuilding applets:

```sh
cd aplha-rust-native
./build.sh forth_mini
cd ..
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --validate-forth-mini \
  analysis/cab/os3kneorom.os3kos
```

## Notes

- Large `--steps` values are normal for full-System user-flow tests.
- `--verbose`, `--stop-at-pc`, and `--stop-at-resource` can use slower traced
  execution paths.
- Prefer `cargo run --manifest-path alpha-emu/Cargo.toml -- ...` from the repo
  root when writing docs or scripts.
- Prefer `--lcd-pbm` for visual regression artifacts and `--lcd-ascii` for
  quick terminal checks.
