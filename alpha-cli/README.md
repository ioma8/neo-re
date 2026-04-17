# alpha-cli

Minimal terminal backup utility for AlphaSmart NEO.

Run:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml
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
