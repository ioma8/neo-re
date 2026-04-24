# layout-patcher

Precise in-place patcher for AlphaSmart NEO full-OS keyboard layouts.

## Scope

This tool patches the stock full OS image:

- `analysis/cab/os3kneorom.os3kos`

It replaces one existing non-default layout slot:

- `dvorak`
- `right`
- `left`

Supported replacement layouts:

- `czech`
- `polish`

The tool keeps the binary size unchanged and rewrites only known in-place
layout-table and string slots.

## Usage

```sh
uv run --project layout-patcher layout-patcher \
  --input analysis/cab/os3kneorom.os3kos \
  --output /tmp/os3kneorom-czech.os3kos \
  --replace dvorak \
  --with czech
```

## Notes

- replacement names are ASCII only
- names are cropped when a fixed firmware slot is shorter than the replacement
- the current `czech` replacement is an ASCII QWERTZ-style fallback
- the current `polish` replacement is an ASCII fallback of the common Polish
  programmer base layer, so it keeps the stock QWERTY letter layer

## Tests

```sh
uv run --project layout-patcher pytest layout-patcher/tests -q
```

## Emulator Smoke Test

```sh
cargo run --manifest-path alpha-emu/Cargo.toml -- \
  --headless \
  --steps=120000000 \
  /tmp/os3kneorom-czech.os3kos
```

## Real Device Flash

Switch the device to direct USB mode, then flash the patched OS:

```sh
uv run --project real-check real-check switch-to-direct && \
uv run --project real-check real-check install-os-image \
  /tmp/os3kneorom-czech.os3kos \
  --yes-flash-os
```
