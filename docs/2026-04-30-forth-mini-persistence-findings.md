# Forth Mini Persistence Findings

This note records the working one-file persistence path for `smartapplets/forth_mini_bw`.

## Working storage path

- applet-owned runtime file handle is `1`
- file open/write/read path is:
  - `SYS_A2DC()`
  - `if (SYS_A2EC() == 0) SYS_A2FC()`
  - `SYS_A190(handle, 0, MODE_READ|MODE_WRITE)`
  - `FileReadBuffer(...)` or `FileWriteBuffer(...)`
  - `FileClose()`

For this applet, that path is stable in the emulator when `handle == 1`.

## What did not work

- using `0x8011` as the runtime handle crashed
- using `0x11` did not produce durable relaunch persistence
- repeatedly calling commit-style traps such as `A2BC` / `A2C0` in the applet lifecycle was not safe in the validated path
- replaying persisted source text on target was not reliable

## Root cause of the replay crash

The file I/O itself was not the problem. The failure was the relaunch-time source replay.

Two target-specific issues mattered:

1. `-fshort-enums` made `ForthResult` byte-aligned.
2. GCC then emitted odd stack slots for some struct-return call sites.

On 68000, later longword copies from those odd addresses raise address errors.

That is why the applet could save source text successfully, but crash when recompiling it after relaunch.

## Working fix

`forth_mini_bw` now persists a binary `ForthMachine` snapshot in its single file.

Stored format:

- 4-byte magic: `FMN1`
- raw `ForthMachine` bytes

Load behavior:

- read the whole snapshot buffer
- verify `FMN1`
- `memcpy` the stored machine state back into the in-memory machine

Save behavior:

- fill the snapshot header
- copy the current `ForthMachine`
- write the whole snapshot buffer to file handle `1`

This avoids on-device replay entirely and gives reliable relaunch persistence in the headless emulator.

## Validation

Validated with:

```sh
./scripts/build-smartapplet.sh forth_mini_bw
```

Successful relaunch proof:

```text
forth_mini_relaunch:

8 sq . 64
forth_mini_validation=ok ... exception=none
```
