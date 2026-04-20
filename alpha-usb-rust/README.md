# Alpha USB Rust

This is a small Rust prototype for writing AlphaSmart NEO SmartApplets through a
high-level SDK-style API. The applet authoring surface is intentionally simple:
the applet implements typed message hooks and uses safe contexts for screen,
event, and USB actions.

The current applet is `Alpha USB`, matching the validated Python-generated
production applet byte-for-byte.

## Applet Source

The applet definition lives in `src/alpha_usb.rs`:

```rust
use crate::sdk::{
    define, AppletDefinition, AppletId, AppletManifest, NeoApplet, Status, UiContext, UsbContext,
    Version,
};

pub struct AlphaUsb;

impl NeoApplet for AlphaUsb {
    const MANIFEST: AppletManifest = AppletManifest::alpha_usb_bridge(
        AppletId(0xA130),
        "Alpha USB",
        Version::decimal(1, 20),
    );

    fn on_focus(&self, ctx: &mut UiContext) {
        ctx.screen().clear();
        ctx.screen().write_lines(
            2,
            [
                "Now connect the NEO",
                "to your computer or",
                "smartphone via USB.",
            ],
        );
        ctx.events().idle_forever();
    }

    fn on_usb_plug(&self, ctx: &mut UsbContext) {
        ctx.usb().complete_hid_to_direct();
        ctx.usb().mark_direct_connected();
        ctx.status(Status::USB_HANDLED);
    }
}

```

No applet source code needs to write raw bytes, A-line traps, or direct ROM
addresses. The `UiContext` exposes screen/event actions; the `UsbContext`
intentionally does not expose screen drawing, which prevents unsafe USB callback
UI work.

Applet authors also do not need to guess SmartApplet header flags or memory
fields. Use named manifest presets:

```rust
AppletManifest::basic(AppletId(0xA140), "Hello", Version::decimal(1, 0));
AppletManifest::alpha_usb_bridge(AppletId(0xA130), "Alpha USB", Version::decimal(1, 20));
```

The preset owns the low-level flags, allocation fields, and metadata records.

Additional message hooks can be added by implementing more trait methods:

```rust
fn on_key(&self, ctx: &mut KeyContext) {
    ctx.when_key(Key::Esc, |ctx| {
        ctx.status(Status::EXIT);
    });
    ctx.status(Status::UNHANDLED);
}
```

The hook records runtime actions into a compiler IR. That keeps applet source
readable while the backend still emits the exact NEO-compatible 68k bytecode.
Known return statuses are exposed as named constants such as `Status::OK`,
`Status::UNHANDLED`, `Status::EXIT`, and `Status::USB_HANDLED`; `Status::raw`
remains available for reverse-engineered statuses that are not named yet.

## Build

```bash
cargo run -- --output ../exports/alpha-usb-rust.os3kapp
```

The generated file is ignored by git under `exports/`.

## Validate Against Existing Python Tooling

```bash
uv run --project poc/neotools neotools build-benign-smartapplet \
  --output exports/alpha-usb-python-reference.os3kapp \
  --applet-id 0xa130 \
  --name "Alpha USB" \
  --draw-on-menu-command \
  --host-usb-message-handler \
  --alphaword-write-metadata \
  --alpha-usb-production

md5 exports/alpha-usb-python-reference.os3kapp exports/alpha-usb-rust.os3kapp
cmp exports/alpha-usb-python-reference.os3kapp exports/alpha-usb-rust.os3kapp
```

Expected current MD5:

```text
6a167dd71f52800f3608bbc4e235cb5e
```
