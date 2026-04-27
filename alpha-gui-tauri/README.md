# AlphaGUI Tauri

React + Tailwind + Tauri desktop manager for AlphaSmart NEO devices.

## Development

```bash
npm --prefix alpha-gui-tauri install
npm --prefix alpha-gui-tauri run tauri:dev
```

Debug builds show a connection-screen button that opens the tabbed UI without a physical device.

## Validation

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
cargo clippy --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml --all-targets -- -D warnings
```

The Tauri backend uses `alpha-core` directly. It does not shell out to Python, `uv`, or `real-check`.
