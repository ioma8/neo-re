# AlphaSmart NEO Keyboard Layout Findings

Date: 2026-04-24

## Scope

This note documents how the stock NEO firmware handles text-entry keyboard
layouts such as QWERTY and Dvorak, with the goal of determining whether new
layouts can be added by SmartApplet alone or require an OS patch.

## Firmware Artifacts

The relevant implementation is in the full System firmware image:

- `analysis/cab/os3kneorom.os3kos`

The Small ROM image and host-side Windows binaries do not contain the layout
strings or layout logic:

- `analysis/cab/smallos3kneorom.os3kos`
- `analysis/cab/neomanager.exe`
- `analysis/cab/asusbcomm.dll`
- `analysis/cab/ashubcomm.dll`

## User-Visible Layout Strings

The full OS contains these strings:

- `To change key layout, type 1, 2, 3 or 4.`
- `1: QWERTY (default)   2: Dvorak`
- `3: Right (one hand)   4: Left (one hand)`
- `Key layout changed to QWERTY.`
- `Key layout changed to Dvorak.`
- `Key layout changed to Right (one hand).`
- `Key layout changed to Left (one hand).`

It also contains a compact name block:

- `Layout:`
- `QWERTY`
- `Dvorak`
- `Right`
- `Left`

These strings are in the full OS package near file offsets `0x35b3e`,
`0x35bf7`, `0x35c15`, `0x35c3d`, and `0x35c90`.

## Layout Selection State

The active layout is stored in RAM byte:

- `0x00005d36`

Relevant behavior:

- `0x00414176` stores a new layout value into `0x5d36`, but only if the value
  is `<= 3`.
- `0x00413b96..0x00413ba2` validates the stored layout and forces it back to
  `3` if it is out of range.

Observed meaning:

- `3` = pass-through mode, used by the stock default QWERTY behavior
- `0..2` = three alternate remap-table slots used for the non-default layouts

The exact `0/1/2` assignment for `Dvorak` vs `Right` vs `Left` is not fully
proven yet, but the firmware clearly supports only these four total layout
states.

## Layout Application Path

The key routine is:

- `0x00423d8a`

Behavior:

1. It converts a raw matrix key byte through the logical-key decode table at
   `0x0044c37b`.
2. It checks layout selector byte `0x5d36`.
3. If `0x5d36 == 3`, it returns the logical key unchanged.
4. Otherwise it indexes the layout transform table at `0x0044c3fb`.

The transform table shape is:

- 3 columns
- one row per logical key
- lookup form: `mapped = table[logical * 3 + layout]`

This is already documented in
[`2026-04-21-keyboard-matrix-map.md`](./2026-04-21-keyboard-matrix-map.md).

Important file-offset note:

- runtime `0x0044c37b` corresponds to file offset `0x0003c37b`
- runtime `0x0044c3fb` corresponds to file offset `0x0003c3fb`

That offset difference exists because the full OS package is mapped with its
header at runtime.

Correction: the older `0x00413d7a` hypothesis was wrong. That routine is part
of another byte/word transformation path and is not the stock keyboard-layout
selector/remap helper.

## Later Character Tables

There are also helper routines for these tables:

- `0x00423df4` -> `0x0044c288`
- `0x00423e06` -> `0x0044c2d9`
- `0x00423dcc` -> `0x0044c32a`

These looked promising as possible normal/shift/option text-output tables, but
one direct AlphaWord typing probe disproved the simple model:

- stock raw key `0x5c` types `1` in AlphaWord
- patching `0x44c288[0x38]` in the OS image did not change that output

So these tables are not the direct AlphaWord unshifted typing table in the
simple `logical -> output byte` sense for stock key `1`. Nearby code around
`0x0041d000` also shows these helpers feeding keyboard-layout view/UI builders,
so they should currently be treated as UI-facing or at least not yet proven to
be the live AlphaWord text-entry path.

## UI / Status Consumers

Firmware UI code also assumes exactly four layout states.

Confirmed readers of `0x5d36` include:

- `0x0040890e..0x0040899c`
- `0x0041418a..0x00414210`

These branches select one of the built-in layout names/resources and do not
show any extensible registration mechanism.

## SmartApplet Feasibility

A SmartApplet can likely:

- switch among the existing built-in layouts by writing or invoking the same
  selector path
- implement its own private remap logic while running inside that applet
- possibly patch RAM temporarily for one boot/session if a safe hook is found

A SmartApplet cannot cleanly add a fifth global layout for AlphaWord and the
system text-entry path, because the stock firmware hard-codes:

- valid selector range `0..3`
- exactly 3 alternate remap columns
- UI branches for exactly 4 visible layouts
- built-in strings/resources for the stock set only

## Live AlphaWord Character Path

The stock layout selector at `0x00423d8a` is only the first stage. The live
AlphaWord text-entry path has a later printable-character pipeline that is
separate from the `0x44c3fb` layout-remap table and also separate from the
special-key dispatcher in `0x00418272`.

Confirmed from emulator traces on stock `os3kneorom.os3kos`:

- raw matrix key `0x5c` is the stock physical `1` key in AlphaWord
- unshifted `1` reaches AlphaWord as applet message `0x20` with parameter
  `0x31`
- shifted `1` first reaches AlphaWord as applet message `0x21` with parameter
  `0x0408`, then later as applet message `0x20` with parameter `0x21`

That proves two distinct runtime paths:

- `0x21` key-event dispatch path:
  `0x004183a6..0x004183ae -> 0x00418754..0x0041878a -> 0x00417ae6 -> 0x00417b14`
- `0x20` printable-char dispatch path:
  `0x00434eb4/0x00434eb8..0x00434ee8 -> 0x00417d00 -> 0x00417acc -> 0x00417b14`

For shifted printable output such as `!`, the printable path gains an extra
pre-stage:

- shifted printable pre-stage:
  `0x00435a20..0x00435a5a -> 0x00434eb8..0x00434ee8`

The important result is that the final printable byte is not fetched directly
from `0x44c288` at send time. The printable sender reads low RAM state instead:

- byte `0x00000433`: current printable character byte
- byte `0x00000434`: printable-char pending flag
- byte `0x00000435`: companion pending/ack flag

The sender block at `0x00434eb8..0x00434ee8` does:

- call `0x00435a20` first
- if needed, call `0x00424ecc`
- load byte `0x433` into `d0`
- forward it through `0x00426bb0`
- clear `0x434` and `0x435`

The same `0x433/0x434/0x435` consume-and-clear pattern also appears in the
nearby companion block `0x004359ba..0x004359d8`, which strongly indicates this
low-RAM trio is the real printable-character latch used by the live firmware
text-entry pipeline.

This resolves the earlier contradiction:

- the one-column-per-layout table at `0x44c3fb` is real, but not sufficient to
  explain shifted output
- the `0x44c288/0x44c2d9/0x44c32a` helper-table hypothesis was too direct
- the firmware can still produce exact shifted printable output because the
  later printable-char pipeline and RAM latches are separate from the initial
  logical-key remap stage

## Conclusion

Adding a new system-wide keyboard layout requires an OS patch, not just a
SmartApplet.

The minimum OS-patch surface is:

1. expand the layout selector range beyond `3`
2. relocate or expand the remap table beyond 3 alternate columns
3. update UI/resource selection code that assumes 4 total layouts
4. update the layout-switching paths used by the firmware UI and keyboard
   command handling

## Validation Note

The in-place patch strategy was validated on a real device with a Czech-patched
OS image derived from the stock `os3kneorom.os3kos` by replacing the `dvorak`
slot. The patched image flashed successfully through `real-check
install-os-image` after switching the device to direct USB mode, and the device
booted normally afterward.
