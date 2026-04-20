# alpha-emu

Desktop AlphaSmart NEO SmartApplet emulator.

The first slice loads a real `.os3kapp` package and runs the validated
`Alpha USB` applet through the `m68000` interpreter plus a small emulated NEO OS
surface. It starts at an emulated SmartApplets menu, supports opening the
selected applet with Enter, and supports the simulated USB attach event.

## Run

Build the applet package first if needed:

```sh
cd ../aplha-rust-native
./build.sh alpha_usb
```

Run the emulator:

```sh
cd ../alpha-emu
cargo +nightly run -- ../exports/applets/alpha-usb-native.os3kapp
```

If no path is passed, the emulator defaults to:

```text
../exports/applets/alpha-usb-native.os3kapp
```

## Current Scope

Implemented:

- `.os3kapp` metadata and image loading
- `m68000`-based applet execution
- emulated SmartApplets menu with Up/Down/Enter navigation
- `Open applet` file dialog for choosing another `.os3kapp`
- minimal NEO display traps used by `Alpha USB`
- simulated USB attach screen and direct-mode transition
- simple `eframe/egui` desktop UI

Not implemented yet:

- stock Calculator execution
- complete NEO OS trap surface
- full filesystem or AlphaWord storage
- real USB hardware access
