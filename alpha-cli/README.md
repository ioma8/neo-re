# alpha-cli

Minimal terminal backup utility for AlphaSmart NEO.

Run the terminal UI:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml
```

Run the GUI:

```bash
cargo run --manifest-path alpha-cli/Cargo.toml --bin alpha-gui
```

Behavior:

- waits for a NEO in USB HID keyboard mode
- switches it to direct USB mode with the validated `e0 e1 e2 e3 e4` HID report sequence
- initializes the direct updater protocol
- shows `Files on device`
- lists all AlphaWord slots with byte size and approximate word count
- downloads one selected slot or `All files`

On desktop, backups are written directly under:

```text
~/alpha-cli/backups/{date-time}/
```

On Android, backups are written to the public Documents tree:

```text
/sdcard/Documents/alpha-cli/backups/{date-time}/
```

For Android 11 and newer, the app declares `MANAGE_EXTERNAL_STORAGE` and opens the system "All files access" settings page if that privilege has not been granted yet. Enable access for Alpha GUI, return to the app, and retry the backup. On older Android versions, the app requests `WRITE_EXTERNAL_STORAGE` at runtime.

The app saves only `.txt` files. The downloaded byte stream is converted host-side by replacing NUL bytes with spaces and CR bytes with LF bytes. The converted text length is validated against the downloaded byte length before the file is accepted.

Logs are written to:

```text
~/alpha-cli/logs/alpha-cli.log
```

The GUI writes its own log file:

```text
~/alpha-cli/logs/alpha-gui.log
```

The GUI uses `eframe`/`egui` with the lighter `glow` renderer. It shares the same protocol, USB, and backup code as the terminal UI.

USB support:

- macOS, Linux, Windows: desktop USB backend through `rusb`
- Android: native USB Host backend through Android `UsbManager` over JNI for direct mode
- other targets: compile with a clear USB-not-implemented path

Android startup path:

The NEO starts in `081e:bd04` HID boot-keyboard mode. Desktop OSes let the app send the validated `e0 e1 e2 e3 e4` HID output-report sequence to switch it into `081e:bd01` direct mode. Stock Android does not: AOSP `UsbHostManager` deny-lists HID boot mouse/keyboard devices before they enter the `UsbManager.getDeviceList()` map. On the tested Pixel, the NEO appears in Android's input stack as `AlphaSmart, Inc. AlphaSmart` with `vendor=0x081e product=0xbd04`, but not in `UsbManager`; `/dev/hidraw0`, `/dev/input/event4`, and `/dev/bus/usb/*` are also not writable by a normal app.

The production workaround is the `Alpha USB` SmartApplet. Launch `Alpha USB` on the NEO first, then connect the NEO to Android by USB. The applet invokes the validated ROM HID-completion path from the device side and re-enumerates as `081e:bd01`, so the Android backend can request normal USB Host permission and use the same direct-mode bulk protocol as the desktop backend.

The Android GUI still detects plain HID keyboard mode through `InputDevice` and reports it explicitly instead of spinning forever. If that appears, disconnect USB, launch `Alpha USB` on the NEO, and reconnect USB.

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
