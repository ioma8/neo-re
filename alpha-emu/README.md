# alpha-emu

Small, firmware-first AlphaSmart NEO emulator experiment.

Current scope is deliberately narrow: load NEO firmware images, run bounded m68k
instruction slices, and log MMIO accesses for hardware-mapping research.
SmartApplet trap shims and direct applet execution were removed from this crate;
the emulator should add hardware devices under the firmware, not reimplement NEO
OS calls in Rust.

## Run

```sh
cd alpha-emu
cargo run
```

The default firmware is:

```text
../analysis/cab/smallos3kneorom.os3kos
```

To boot another Small ROM-compatible image:

```sh
cargo run -- ../analysis/cab/smallos3kneorom.os3kos
```

Headless full System 3 firmware boot:

```sh
cargo run -- --headless --steps=200000 ../analysis/cab/os3kneorom.os3kos
```

Normal headless execution uses the fast interpreter path. `--verbose` switches
to traced execution and prints MMIO plus instruction trace data.

Headless supports:

- scheduled input: `--type-at`, `--key-at`, `--hold-key`, `--key-all-rows-at`
- immediate input: `--type-now`, `--key-now`
- boot chords: `--boot-left-shift-tab`, `--boot-keys`, `--boot-keys-exact`
- LCD outputs: `--lcd-ascii`, `--lcd-visible-ascii`, `--lcd-bits`,
  `--lcd-bits-path`, `--lcd-pbm`, `--lcd-visible-pbm`, `--lcd-blink-pbm-prefix`,
  `--lcd-ranges`, `--lcd-dump-dir`, `--lcd-ocr`
- validation helpers: `--launch-forth-mini`, `--launch-calculator`,
  `--validate-alpha-usb-native`, `--validate-forth-mini`

The full flag reference and verified command patterns are in
[`docs/2026-04-22-alpha-emu-headless-usage.md`](../docs/2026-04-22-alpha-emu-headless-usage.md).

Example Forth Mini debug run:

```sh
cargo run -- --headless \
  --launch-forth-mini \
  --type-now=1 \
  --key-now=enter \
  --lcd-ocr \
  --lcd-dump-dir=/tmp/alpha-emu-forth \
  ../analysis/cab/os3kneorom.os3kos
```

The desktop UI shows:

- emulated NEO LCD active viewport, cropped to 256x64 square pixels from the 320x128 controller buffer
- reset-vector boot state
- current running/stopped state

Realtime GUI execution is cycle-paced against a 33 MHz DragonBall VZ target,
matching the commonly reported AlphaSmart NEO/NEO2 CPU clock. GUI repaint is
capped to a 16 ms cadence, so the display updates at no more than about 60 FPS
while the interpreter advances by elapsed emulated CPU cycles.

The GUI samples and logs actual emulator throughput once per second. Run with
`RUST_LOG=alpha_emu=info cargo run -- ../analysis/cab/os3kneorom.os3kos` to see
`target_hz` and `achieved_hz` in the terminal. On the current development
machine, normal optimized `cargo run` measured about 373 MHz on the full System
3 boot workload, well above the real 33 MHz target. The crate still uses an
optimized dev profile so plain `cargo run` stays useful for emulator work.

For faster hardware probing without the UI:

```sh
cargo run -- --headless --steps=2000000
```

Headless output includes `cycles`, `elapsed_ms`, `achieved_hz`, and
`target_hz=33000000`:

```sh
cargo run --release -- --headless --steps=2000000 ../analysis/cab/os3kneorom.os3kos
```

CPU backend microbenchmarks are available with:

```sh
cargo run --release --bin cpu_bench
```

These compare the current `m68000` crate against built-in slice memory and a
minimal custom `MemoryAccess`; both exceed 33 MHz by a wide margin on simple
NOP, branch, and RAM read/write workloads.

Normal boot/open does not hold any synthetic keys. The GUI includes a separate
`Reboot Small ROM with activating key chord` button for the special updater
entry path. That button briefly presents the Small ROM entry key chord to the
firmware and then releases it. It does not type the `ernie` password; the
firmware waits for normal keyboard input at the password prompt.

For the full System 3 image, `Boot into SmartApplets list` holds the documented
left-shift + tab boot chord at reset. Headless validation stops at the
SmartApplets menu resource with:

```sh
cargo run -- --headless --boot-left-shift-tab \
  --steps=18000000 --stop-at-resource=0x6b \
  ../analysis/cab/os3kneorom.os3kos
```

## Current Hardware Map

Detailed notes are in
[`docs/2026-04-20-alpha-emu-memory-map.md`](../docs/2026-04-20-alpha-emu-memory-map.md).

Validated from the Small ROM boot path:

- `0x00000000`: reset-vector mirror of the Small ROM image
- `0x00400000`: executable Small ROM mapping; reset PC is `0x0040042a`
- `0x0000f000..0x00010000`: DragonBall-style internal register window
- `0xfffff000..0xffffffff`: sign-extended alias of the `0x0000f000` register window
- `0x01008000..0x01008002`: left LCD controller command/data byte ports
- `0x01000000..0x01000002`: right LCD controller command/data byte ports
- `0xfffff419` / `0x0000f419`: active-low keyboard matrix input byte
- `0xfffff411` / `0x0000f411`: observed Small ROM keyboard row-select byte

The current emulation preserves MMIO register byte state, logs reads/writes, and
lets the Small ROM pass CPU setup, LCD initialization/clear, and keyboard-matrix
polling without a bus error. Labelled PC keyboard input covers the firmware
keyboard matrix entries currently mapped in `src/keyboard.rs`, including the
Small ROM password keys proven by firmware table `0x004053ee`: `e`, `r`, `n`,
and `i`.

The LCD model implements the page/column/data behavior needed by the Small ROM
boot path. It is intentionally minimal: unsupported controller commands are
ignored until firmware execution reaches a path that needs them.

## Validation

```sh
cargo check
cargo test
```
