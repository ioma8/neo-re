# alpha-cli

Terminal and egui desktop manager for AlphaSmart NEO.

Run the terminal UI:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml
```

Run the GUI:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml --bin alpha-gui
```

GUI behavior:

- starts on a connection screen and shows tabs only after direct USB is available
- desktop automatically switches HID `081e:bd04` to direct USB `081e:bd01` with the validated `e0 e1 e2 e3 e4` sequence
- mobile explains the Alpha USB prerequisite when the device is still in HID keyboard mode
- desktop and Android direct-mode builds use the Rust USB backend directly; no Python helper process is used
- uses a tabbed flow inspired by the saved HTML references:
  - `Dashboard`: lists AlphaWord files and backs up all files or individual slots
  - `SmartApplets`: shows bundled stock applets, installed applets, and the Alpha USB install action
  - `OS Operations`: backs up everything and exposes the validated bundled system-image flash action
  - `About`: project info, version, resources, NEO-only validation note, and brick-risk warning
- embeds stock applets from `exports/smartapplet-backups/20260425-forth-clean-reflash`
- embeds Alpha USB from `exports/applets/alpha-usb-native.os3kapp`
- embeds the validated NEO OS image from `analysis/cab/os3kneorom.os3kos`
- shows structured progress for connection switching, inventory refresh, file backup, applet install/reflash, and OS flashing
- requires confirmation before clearing/reinstalling applets or flashing OS images

Full-device backup includes:

- raw AlphaWord slot dumps
- text-converted AlphaWord exports
- dumped installed SmartApplets
- a backup manifest summarizing slots and applets

The terminal UI remains the simpler AlphaWord backup flow.

On desktop, one-slot AlphaWord backups are written under:

```text
~/alpha-cli/backups/{date-time}/
```

Full-device backups are written under:

```text
~/alpha-cli/device-backups/{date-time}/
```

On Android, backups are written to the public Documents tree:

```text
/sdcard/Documents/alpha-cli/backups/{date-time}/
```

For Android 11 and newer, the app declares `MANAGE_EXTERNAL_STORAGE` and opens the system "All files access" settings page if that privilege has not been granted yet. Enable access for Alpha GUI, return to the app, and retry the backup. On older Android versions, the app requests `WRITE_EXTERNAL_STORAGE` at runtime.

The text export path converts downloaded bytes host-side by replacing NUL bytes
with spaces and CR bytes with LF bytes. The converted text length is validated
against the downloaded byte length before the file is accepted.

Logs are written to:

```text
~/alpha-cli/logs/alpha-cli.log
```

The GUI writes its own log file:

```text
~/alpha-cli/logs/alpha-gui.log
```

The GUI uses `eframe`/`egui` with the lighter `glow` renderer. Manager flows use
the Rust direct USB backend in this crate; they do not shell out to Python or
`real-check`.

USB support:

- macOS, Linux, Windows: desktop USB backend through `rusb`
- Android: native USB Host backend through Android `UsbManager` over JNI for direct mode
- other targets: compile with a clear USB-not-implemented path

Android startup path:

The NEO starts in `081e:bd04` HID boot-keyboard mode. Desktop OSes let the app send the validated `e0 e1 e2 e3 e4` HID output-report sequence to switch it into `081e:bd01` direct mode. Stock Android does not: AOSP `UsbHostManager` deny-lists HID boot mouse/keyboard devices before they enter the `UsbManager.getDeviceList()` map. On the tested Pixel, the NEO appears in Android's input stack as `AlphaSmart, Inc. AlphaSmart` with `vendor=0x081e product=0xbd04`, but not in `UsbManager`; `/dev/hidraw0`, `/dev/input/event4`, and `/dev/bus/usb/*` are also not writable by a normal app.

The production workaround is the `Alpha USB` SmartApplet. Launch `Alpha USB` on the NEO first, then connect the NEO to Android by USB. The applet invokes the validated ROM HID-completion path from the device side and re-enumerates as `081e:bd01`, so the Android backend can request normal USB Host permission and use the same direct-mode bulk protocol as the desktop backend.

The Android GUI still detects plain HID keyboard mode through `InputDevice` and reports it explicitly instead of spinning forever. If that appears, disconnect USB, launch `Alpha USB` on the NEO, and reconnect USB.

Validated Android result:

- physical Android device with USB Host/OTG
- physical AlphaSmart NEO with `Alpha USB` `0xa130` version `1.20`
- user launches `Alpha USB` on the NEO before connecting USB
- NEO re-enumerates as direct USB `081e:bd01`
- Android GUI opens the direct-mode USB device through `UsbManager`
- Android GUI backs up AlphaWord files successfully to the public Documents tree

This is the proven no-root, no-proxy, no-typing-fallback Android backup path.

APK packaging must declare USB Host and storage access, for example:

```xml
<uses-feature android:name="android.hardware.usb.host" android:required="true" />
<uses-permission android:name="android.permission.MANAGE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="29" />
```

Validated GUI check targets:

```bash
cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-apple-darwin --bin alpha-gui
cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-linux-android --bin alpha-gui
```

Build the Android debug APK:

```bash
ANDROID_NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358" cargo apk build --manifest-path alpha-cli/Cargo.toml --lib
```

The debug APK is written to:

```text
alpha-cli/target/debug/apk/alpha-gui.apk
```
