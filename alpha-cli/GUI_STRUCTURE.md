# AlphaGUI Structure

AlphaGUI is a tabbed `egui` manager for AlphaSmart NEO. The UI starts with a
connection gate and shows the manager tabs only after direct USB mode is
available.

## Backend Boundary

All desktop and Android manager operations are implemented in Rust inside
`alpha-cli`.

- USB mode detection and HID-to-direct switching use `usb` / `usb_android`.
- AlphaWord inventory and downloads use `NeoClient::list_files()` and
  `NeoClient::download_file()`.
- SmartApplet inventory, backup, install, and area clear use Rust updater
  packets through `NeoClient`.
- OS image flashing uses Rust updater packets through `NeoClient`.
- The GUI must not call `uv`, `real-check`, Python scripts, or repo-relative
  helper commands.

## Tabs

- `Dashboard`: lists AlphaWord files and offers per-file or full file backup.
- `SmartApplets`: lists installed and bundled applets, installs Alpha USB, adds
  desktop-selected applet files, and performs install-only or clear/reinstall
  workflows.
- `OS Operations`: backs up the whole device and flashes the bundled validated
  NEO system image after confirmation.
- `About`: project summary, version, links, NEO-only validation note, and risk
  warning.

## Background Tasks

Every task opens a fresh direct USB client, emits structured progress, and sends
one final status event.

- Refresh inventory reads AlphaWord slots and SmartApplet records.
- Install applet reads the `.os3kapp` image, validates its header, uploads it in
  0x400-byte chunks, finalizes, and refreshes inventory.
- Clear/reinstall clears the SmartApplet area, then uploads all checked applet
  images.
- Full backup downloads raw AlphaWord payloads, text exports, installed
  SmartApplet binaries, and writes a manifest.
- OS flash parses the bundled `.os3kos` segment table, enters Small ROM mode,
  erases segments, uploads chunks, and finalizes.

## Mobile Notes

Stock Android cannot open the NEO while it is still HID boot-keyboard mode.
Users must run Alpha USB on the NEO first, then connect the phone. Once direct
USB is visible, Android uses the same Rust manager flow through the JNI USB
transport.
