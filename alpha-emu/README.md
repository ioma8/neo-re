# alpha-emu

Small, firmware-first AlphaSmart NEO emulator experiment.

Current scope is deliberately narrow: load the Small ROM image, initialize the
m68k CPU from its reset vectors, run bounded instruction slices, and log MMIO
accesses for hardware-mapping research. SmartApplet trap shims and direct
applet execution were removed from this crate; the next emulator work should
add hardware devices under the firmware, not reimplement NEO OS calls in Rust.

## Run

```sh
cd alpha-emu
cargo +nightly run
```

The default firmware is:

```text
../analysis/cab/smallos3kneorom.os3kos
```

To boot another Small ROM-compatible image:

```sh
cargo +nightly run -- ../analysis/cab/smallos3kneorom.os3kos
```

The desktop UI shows:

- emulated 320x128 LCD pixels
- reset-vector boot state
- current PC/SSP/step count
- recent m68k instruction trace
- MMIO reads/writes observed while the firmware runs

For faster hardware probing without the UI:

```sh
cargo +nightly run -- --headless --steps=2000000
```

To inject the proven Small ROM password key sequence:

```sh
cargo +nightly run -- --headless --type-password --steps=6000000
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
polling without a bus error. Labelled PC keyboard input is currently limited to
the Small ROM password keys proven by firmware table `0x004053ee`: `e`, `r`,
`n`, and `i`.

The LCD model implements the page/column/data behavior needed by the Small ROM
boot path. It is intentionally minimal: unsupported controller commands are
ignored until firmware execution reaches a path that needs them.

## Validation

```sh
cargo +nightly check
cargo +nightly test
```
