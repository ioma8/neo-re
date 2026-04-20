# alpha-emu

Desktop AlphaSmart NEO SmartApplet emulator.

The emulator loads a real `.os3kapp` package and runs selected SmartApplet
machine code through the `m68000` interpreter plus a small emulated NEO OS
surface. `NeoSystem` owns firmware-like behavior such as the SmartApplets menu
and USB attach screens; `applet_runner` executes the applet machine code; and
`os_shims` implements the NEO services that interpreted applet code calls.

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
- persistent applet sessions across `A25C` yields
- emulated SmartApplets menu with Up/Down/Enter navigation
- `Open applet` file dialog for choosing another `.os3kapp`
- minimal NEO display, keyboard, and event traps used by `Alpha USB` and stock
  `Calculator`
- simulated USB attach screen and direct-mode transition
- simple `eframe/egui` desktop UI
- stock Calculator opens, renders its initial screen, and accepts emulated key
  input without interpreter traps

Not implemented yet:

- complete NEO OS trap surface
- exact Calculator key/display fidelity
- full filesystem or AlphaWord storage
- real USB hardware access

## Runtime Notes

NEO applets call OS services through local A-line trap stubs such as `A004`,
`A010`, `A094`, and `A25C`. Those stubs are contiguous trap words, not normal
inline calls. The real OS handler behaves like a subroutine return: after
handling the trap, it resumes at the JSR/BSR return address from the applet
stack. The emulator must therefore pop that return address for every handled
A-line trap. Otherwise execution falls through the import table, which was the
root cause of the original Calculator blank-screen behavior.
