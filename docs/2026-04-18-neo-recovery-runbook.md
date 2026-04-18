# AlphaSmart NEO Recovery Runbook - 2026-04-18

This note documents the exact recovery path used on the physical NEO after
custom SmartApplet experiments corrupted the writable file/applet catalog.

## Failure State

The device failed normal boot with file-system errors such as:

- `File 127 MaxSize overflow`
- `File 128 Size overflow`
- `System software detected a problem and attempted to fix File 22.`
- `Error with memmove`

Factory reset did not clear the condition. Removing the coin-cell battery did
not clear it either. The important recovery path was that the NEO could still be
booted into the Small ROM Updater and could still enumerate as direct USB
`081e:bd01`.

## Root Cause Summary

The System OS image was still flashable and executable. The persistent failure
was in the writable file/applet catalog area, not in the main OS code segment.

Evidence:

- Reflashing the stock `analysis/cab/os3kneorom.os3kos` completed, but the
  `MaxSize overflow` errors persisted.
- Small ROM accepted the NeoManager OS flash protocol, but normal SmartApplet
  commands returned `0x92` while in Small ROM.
- Once a validator-disabled OS was flashed, normal HID/direct USB came back.
- `debug-applets` then showed a dirty applet catalog with duplicate custom
  `Alpha USB` entries.
- A broad SmartApplet-area clear removed those duplicates, and after incremental
  stock applet restore the applet table and AlphaWord file attributes were
  readable again.

## Recovery Protocols That Worked

### HID to Direct Mode

From normal HID keyboard mode `081e:bd04`, the known NeoManager fallback HID
sequence still worked:

```bash
uv run --project real-check real-check switch-to-direct
uv run --project real-check real-check watch --timeout 5
```

Expected direct-mode result:

```text
vendor_id=0x081e product_id=0xbd01 mode=direct detail=NEO direct USB mode
  interface=0 alt=0 endpoints=0x82:bulk:in:max64 0x01:bulk:out:max64
```

### Small ROM OS Flash

The normal OS update protocol is:

1. Enter updater mode with `?reset` and `?Swtch`.
2. Send `0x18`, expect `0x56` to enter Small ROM.
3. Send `0x16`, expect `0x54` to clear the OS segment map.
4. Send `0x17` for each OS segment, expect `0x55`.
5. For each `0x400` byte chunk:
   - send `0x02`, expect `0x42`
   - send the raw chunk, expect `0x43`
   - send `0x0b`, expect `0x47`
6. Send `0x07`, expect `0x48` finalize.

The patched OS image used for recovery was:

```text
/tmp/os3kneorom-disable-fscheck.os3kos
```

It was derived from `analysis/cab/os3kneorom.os3kos` by replacing the entry of
the filesystem validator at file offset `0x2c832`:

```text
old: 48 e7 1f 3a
new: 4e 75 4e 71
```

That is `RTS; NOP` at runtime address `0x0043c832`. This was enough to let the
device boot to normal mode despite the corrupted catalog, making normal direct
USB repair possible. This patched OS is intentionally retained as the recovery
baseline because it proved more useful than the stock image while repairing a
damaged writable catalog.

Successful flash command:

```bash
uv run --project real-check real-check install-os-image \
  /tmp/os3kneorom-disable-fscheck.os3kos \
  --yes-flash-os
```

Successful output:

```text
validated NEO OS image path=/tmp/os3kneorom-disable-fscheck.os3kos bytes=395264 segments=3
segment address=0x00410000 length=393216 erase_kb=384
segment address=0x00406000 length=20 erase_kb=1
segment address=0x005ffc00 length=1024 erase_kb=1
flashed NEO OS bytes=395264 chunks=386 segments=3
```

### Small ROM State Caveat

When the NEO screen said `Small ROM Updater: Connected to computer via USB`,
the direct endpoint was visible, but raw updater commands `0x18` and `0x16`
returned ASCII `Error???` until the normal updater bootstrap was sent:

```text
?reset
?Swtch
```

Observed bootstrap result:

```text
reset response: timeout
switch response: 53 77 69 74 63 68 65 64  Switched
```

After `Switched`, `install-os-image` worked normally.

## Recovery Protocols That Did Not Work

The following did not fix the corruption:

- Factory reset with password `tommy`.
- Removing the coin-cell battery.
- Reflashing the stock OS alone.
- Small ROM SmartApplet commands such as list/add/clear. They returned `0x92`,
  meaning unavailable in Small ROM.
- NeoManager-style rest-of-ROM erase using `0x17 address=0x005ffc00 trailing=0`
  as a general fix. It accepted the erase, but later full final-tail programming
  failed with status `0x8c`.

The following was specifically unsafe:

- A custom SmartApplet USB attach callback that tried to draw a message and idle
  inside the `0x30001` USB attach path. That callback must return status to the
  System USB dispatcher. Blocking there correlated with finalize/checksum
  trouble and the later file-catalog boot failures.

## Why The Next Alpha USB Version Broke The System

The last working production applet is `Alpha USB` `0xa130` version `1.20`.
It does only two visible things:

- menu-open command `0x19`: draw brief instructions, then idle in normal applet
  UI context
- USB attach command `0x30001`: run the proven ROM HID-completion path, call the
  direct-mode status callback, set status `0x11`, and return immediately

The next experimental version attempted to improve the post-switch screen text.
That crossed an ownership boundary. It tried to draw or idle from inside the
`0x30001` USB attach callback, which is not normal applet UI context. It is a
System USB dispatcher callback. The dispatcher expects the applet to write a
status word and return quickly.

The observed failure chain was:

1. A v1.21-style applet install showed a finalize-response checksum mismatch.
2. The device later booted with `File 127 MaxSize overflow` and related file
   catalog errors.
3. After patched-OS recovery, `debug-applets` showed multiple duplicate
   `Alpha USB` records in the applet catalog.
4. Clearing the SmartApplet area removed the duplicates, but the broad restore
   flow did not complete; incremental one-at-a-time applet install was required.

The most defensible root cause is not that the v1.21 code directly overwrote a
known file record. The stronger conclusion is that the install/finalize process
was interrupted or left internally inconsistent after unsafe applet behavior in
the USB attach path. That produced duplicate applet metadata and a damaged
writable catalog, which then tripped the OS boot-time filesystem validator.

Important distinction:

- Drawing instructions in `0x19` is safe; this is the normal applet screen.
- Returning status from `0x30001` is safe; this is the USB attach callback.
- Drawing, flushing, waiting, or idling from `0x30001` is unsafe; the System owns
  that call stack and is waiting for a quick status result.

## Future Breakage Prevention

Rules for any future custom SmartApplet that touches USB:

1. Keep USB callbacks non-blocking.
   - For `0x30001` and related attach/init commands, do the minimum hardware
     action, write status, and return.
   - Do not call display traps, flush traps, keyboard loops, or `A25C` idle loops
     from USB callback context.

2. Keep UI and USB separated.
   - Draw instructions only in the menu-open `0x19` path.
   - Treat `0x30001` as an interrupt-like dispatcher callback, not as an applet
     screen.

3. Install only one custom applet candidate at a time.
   - Before install: `real-check applets`.
   - Install the candidate.
   - Immediately verify: `real-check applets`.
   - Restart and verify HID/direct mode before further writes.

4. Never continue after an install checksum/finalize anomaly.
   - If install reports a checksum mismatch, timeout, or unexpected status, stop.
   - Do not install another applet over it.
   - Re-read `real-check applets` and document the table before taking action.

5. Prefer version monotonicity and unique test ids.
   - Stable production applet: `0xa130` version `1.20`.
   - Experimental probes should use separate ids (`0xa12x`) unless replacing the
     production applet is the exact test.

6. Use the patched validator-disabled OS as the recovery baseline.
   - It lets a catalog-damaged device boot far enough to use normal direct USB
     repair commands.
   - Do not flash stock OS as a cleanup step while applet/file catalog work is
     still in progress.

The prevention rule can be summarized as: the applet may prepare the user in
`0x19`, and it may trigger direct USB in `0x30001`, but it must not attempt to
own the screen or event loop from the USB attach callback.

## Applet Catalog Repair That Worked

After patched OS boot, the NEO returned to HID mode and could be switched to
direct mode:

```bash
uv run --project real-check real-check switch-to-direct
```

Read-only inspection first showed the dirty catalog:

```bash
uv run --project real-check real-check debug-applets
```

It contained stock applets plus four duplicate custom `Alpha USB` entries
(`applet_id=0xa130`).

A broad restore attempt using `restore-stock-applets --restart` cleared the
SmartApplet area but did not complete the restore; the device displayed:

```text
Bus Error Accessing: 0x240000A
Next Instruction At: 0x43C5C4
```

After reflashing the patched OS again, read-only applet listing showed only
System:

```text
applet_id=0x0000 version=3.15 name=System file_size=401408 applet_class=0x01
```

The successful repair was to install the stock applets one at a time and verify
the table after each install. Thesaurus was intentionally skipped.

Successful restore order:

```bash
uv run --project real-check real-check install-applet analysis/device-dumps/applets/A000-AlphaWord_Plus.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/AF00-Neo_Font_Small_6_lines.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/AF75-Neo_Font_Medium_5_lines.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/AF02-Neo_Font_Large_4_lines.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/AF73-Neo_Font_Very_Large_3_lines.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/AF03-Neo_Font_Extra_Large_2_lines.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/A004-KeyWords.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/A007-Control_Panel.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/A006-Beamer.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/A001-AlphaQuiz.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/A002-Calculator.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/A027-Text2Speech_Updater.os3kapp
uv run --project real-check real-check install-applet analysis/device-dumps/applets/A005-SpellCheck_Large_USA.os3kapp
```

Validation after each install:

```bash
uv run --project real-check real-check applets
```

Final applet list:

```text
applet_id=0x0000 version=3.15 name=System file_size=401408 applet_class=0x01
applet_id=0xa000 version=3.4 name=AlphaWord Plus file_size=106684 applet_class=0x01
applet_id=0xaf00 version=1.0 name=Neo Font - Small (6 lines) file_size=4164 applet_class=0x01
applet_id=0xaf75 version=1.0 name=Neo Font - Medium (5 lines) file_size=4360 applet_class=0x01
applet_id=0xaf02 version=1.0 name=Neo Font - Large (4 lines) file_size=4264 applet_class=0x01
applet_id=0xaf73 version=1.0 name=Neo Font - Very Large (3 lines) file_size=9392 applet_class=0x01
applet_id=0xaf03 version=1.0 name=Neo Font - Extra Large (2 lines) file_size=13152 applet_class=0x01
applet_id=0xa004 version=3.6 name=KeyWords file_size=126888 applet_class=0x01
applet_id=0xa007 version=1.0 name=Control Panel file_size=27412 applet_class=0x01
applet_id=0xa006 version=1.0 name=Beamer file_size=32580 applet_class=0x01
applet_id=0xa001 version=3.1 name=AlphaQuiz file_size=49828 applet_class=0x01
applet_id=0xa002 version=3.0 name=Calculator file_size=24544 applet_class=0x01
applet_id=0xa027 version=1.4 name=Text2Speech Updater file_size=11460 applet_class=0x01
applet_id=0xa005 version=1.0 name=SpellCheck Large USA file_size=357312 applet_class=0x01
```

Then restart:

```bash
uv run --project real-check real-check restart-device
```

The NEO came back as HID keyboard mode:

```text
vendor_id=0x081e product_id=0xbd04 mode=keyboard detail=AlphaSmart HID keyboard mode; no direct USB OUT endpoint
  interface=0 alt=0 endpoints=0x82:interrupt:in:max64
```

Switching back to direct and reading AlphaWord attributes then succeeded:

```bash
uv run --project real-check real-check switch-to-direct
uv run --project real-check real-check list
```

Final AlphaWord file attributes:

```text
slot=1 name=File 1 file_length=512 reserved_length=512
slot=2 name=File 2 file_length=512 reserved_length=512
slot=3 name=File 3 file_length=512 reserved_length=512
slot=4 name=File 4 file_length=512 reserved_length=512
slot=5 name=File 5 file_length=512 reserved_length=512
slot=6 name=File 6 file_length=512 reserved_length=512
slot=7 name=File 7 file_length=512 reserved_length=512
slot=8 name=File 8 file_length=512 reserved_length=512
```

## Current Status And Caution

The device was recovered to a usable normal HID/direct USB state, with stock
applets restored except Thesaurus and with valid AlphaWord file attributes.

The final executed recovery state intentionally keeps the patched
validator-disabled OS image. Do not flash the original stock OS as a cleanup
step unless there is a separate reason to restore strict stock behavior.

Useful post-recovery checks:

```bash
uv run --project real-check real-check watch --timeout 12
uv run --project real-check real-check switch-to-direct
uv run --project real-check real-check applets
uv run --project real-check real-check list
```

Do not run the broad `restore-stock-applets --restart` flow on a fragile device
again. The safer recovery pattern is one install at a time, with `applets`
validation after each install.
