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

- reset-vector boot state
- current PC/SSP/step count
- recent m68k instruction trace
- MMIO reads/writes observed while the firmware runs

## Validation

```sh
cargo +nightly check
cargo +nightly test
```
