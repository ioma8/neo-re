# aplha-rust-native

Native Rust experiment for AlphaSmart NEO SmartApplets.

This workspace builds applet code with Cargo for `m68k-unknown-none-elf`, then packages the linked ELF into an AlphaSmart `.os3kapp` image.

## Requirements

- Rust nightly with `rust-src`
- `m68k-elf-binutils` or another m68k GNU linker

On macOS with Homebrew:

```sh
brew install m68k-elf-binutils
```

## Build Alpha USB

```sh
cd aplha-rust-native
./build.sh alpha_usb
```

Output:

```text
../exports/applets/alpha-usb-native.os3kapp
```

The wrapper runs:

1. `cargo +nightly build -p alpha-usb-applet --target m68k-unknown-none-elf -Z build-std=core,panic_abort --release`
2. `cargo +nightly run -p alpha-neo-pack -- alpha-usb <linked-elf> <output.os3kapp>`

Release mode is intentional. The tested 2026-03-25 nightly segfaulted while compiling `compiler_builtins` for this target in debug mode, but the release build succeeds.

## Structure

- `crates/alpha-neo-sdk`: `no_std` applet-side SDK with message dispatch, display traps, and USB helpers.
- `crates/alpha-neo-pack`: host-side ELF-to-OS3KApp packer.
- `applets/alpha_usb`: native Rust Alpha USB applet.

## Scope

This first iteration targets a functional native Rust Alpha USB applet. It is not byte-exact with the previous Python/generated Rust applet.

