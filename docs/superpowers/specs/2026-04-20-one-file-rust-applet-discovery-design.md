# One-File Rust Applet Discovery Design

Historical note: this design applied to the old `alpha-usb-rust` builder, which
has been removed after the native `aplha-rust-native` SDK replaced it. Keep this
file only as design history for the removed builder.

## Goal

Let a developer add a new Rust SmartApplet by creating only one file:

```text
alpha-usb-rust/src/applets/new_applet.rs
```

No registry, module declaration, CLI, or package code should need manual edits for that applet to become visible to:

```bash
cargo run -- list
cargo run -- build new_applet
cargo run -- build all
```

## Chosen Approach

Use a Cargo build script to scan `src/applets/*.rs` and generate a Rust module/registry file into `OUT_DIR`.

Each applet source remains normal Rust. It exposes one required item:

```rust
pub const PACKAGE: AppletPackage = AppletPackage {
    name: "new_applet",
    output_filename: "new-applet.os3kapp",
    build: build,
    validate: validate_basic,
};
```

The generated registry imports every applet file and returns all discovered `PACKAGE` constants.

## Structure

```text
alpha-usb-rust/
  build.rs
  src/
    applets/
      alpha_usb.rs
    applets.rs
    cli.rs
    compiler.rs
    lib.rs
    main.rs
    os3kapp.rs
    sdk.rs
```

`src/applets.rs` owns shared applet package types and includes generated discovery code:

```rust
include!(concat!(env!("OUT_DIR"), "/applets_generated.rs"));
```

`build.rs` generates module declarations from file stems:

```rust
#[path = "/abs/path/src/applets/alpha_usb.rs"]
pub mod alpha_usb;

pub fn all() -> &'static [AppletPackage] {
    &[alpha_usb::PACKAGE]
}
```

The generated file is not checked into git.

## Applet File Contract

Each applet file must:

1. Define the applet type implementing `NeoApplet`.
2. Define `fn build() -> AppletDefinition`.
3. Define `pub const PACKAGE: AppletPackage`.

The applet file may use shared validators:

```rust
validate: validate_basic
```

Alpha USB keeps its stricter validator:

```rust
validate: validate_alpha_usb
```

## Validation

Implementation must pass:

```bash
cargo check
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic
cargo run -- list
cargo run -- build alpha_usb
cargo run -- build all
```

Alpha USB must remain byte-exact with the Python reference:

```text
6a167dd71f52800f3608bbc4e235cb5e
```
