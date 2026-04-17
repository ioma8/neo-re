# alpha-cli

Minimal terminal backup utility for AlphaSmart NEO.

Run the terminal UI:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml
```

Run the GUI:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml --bin alpha-gui
```

Behavior:

- waits for a NEO in USB HID keyboard mode
- switches it to direct USB mode with the validated `e0 e1 e2 e3 e4` HID report sequence
- initializes the direct updater protocol
- shows `Files on device`
- lists all AlphaWord slots with byte size and approximate word count
- downloads one selected slot or `All files`

Backups are written directly under:

```text
~/alpha-cli/backups/{date-time}/
```

The app saves only `.txt` files. The downloaded byte stream is converted host-side by replacing NUL bytes with spaces and CR bytes with LF bytes. The converted text length is validated against the downloaded byte length before the file is accepted.

Logs are written to:

```text
~/alpha-cli/logs/alpha-cli.log
```

The GUI writes its own log file:

```text
~/alpha-cli/logs/alpha-gui.log
```

The GUI uses `eframe`/`egui` with the lighter `glow` renderer. It shares the same protocol, USB, and backup code as the terminal UI. The desktop USB implementation is enabled for macOS, Linux, and Windows. Mobile targets compile the GUI with a clear USB-not-implemented path until a mobile USB adapter is added.

Validated GUI check targets:

```bash
cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-apple-darwin --bin alpha-gui
cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-linux-android --bin alpha-gui
```
