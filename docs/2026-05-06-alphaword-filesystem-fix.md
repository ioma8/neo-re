# AlphaWord Filesystem Repair Record

Date: 2026-05-06
Device: AlphaSmart NEO (System 3.15)
Issue: AlphaWord files limited to ~501 characters, unable to create new files

## Root Cause

The NEO's filesystem recovery/format path creates all 8 AlphaWord file slots
with MaxSize=512 (0x200) instead of the factory default 51200 (0xC800).
This is confirmed in both the firmware binary (os3kneorom.os3kos) and the
NM ACF1 configuration file which specifies:
- Maximum File size: 51200 characters
- Minimum File size: 512 characters

The format routine creates files with the Minimum size (512) instead of
dividing the available storage evenly.

## What Did NOT Work

### 1. USB Updater Attribute Write (0x1D/0x1E)
The NeoManager protocol `UpdaterPutRawFileAttributes` uses commands
0x1D (begin) and 0x1E (finish) to write file attribute records. On this
firmware version, 0x1E returns status 0x8F and crashes the Small ROM
MiniMonitor, requiring a full power cycle. The attribute write is simply
not supported on System 3.15.

### 2. Emulator Testing
The headless emulator (alpha-emu) crashes with "address error vector=3
at 0x00000019" when running the direct-memory-write fixer applet, due to
different memory initialization compared to the physical device.

## What Fixed It

### File Fixer SmartApplet (v5 - Direct Memory Write)
Built using the Betawise-derived C SDK (smartapplets/file_fixer_bw/).

ID: 0xa137
Name: File Fixer
Size: 504 bytes

The applet:
1. Reads the file table pointer from firmware RAM at 0x00000fda
2. Iterates 8 file records (each 0x48 = 72 bytes)
3. Patches the MaxSize field (offset 0x08, big-endian uint32) from
   512 (0x200) to 51200 (0xC800)
4. Displays "fixed!" and exits

This works because:
- The real NEO handles byte-level memory access correctly
- Writing MaxSize=51200 removes the artificial limit
- File writes via USB (0x14/0x15 protocol) then use the corrected size

### USB File Data Write (0x14/0x15)
The updater file write protocol works reliably:
- 0x14: begin file data transfer → returns 0x50
- 0x02: chunk handshake → returns 0x42
- Data bytes (up to 0x400 per chunk)
- 0xFF: commit → returns 0x43
- 0x15: finish → returns 0x8F (data IS persisted despite non-0x51 code)

Tested: 2400 bytes written and read back successfully.

## Key Finding: reserved_length != MaxSize

The USB attribute record (0x28 bytes, returned by command 0x13) has a
field at offset 0x18-0x1B labeled "reserved_length". This field is NOT
the internal MaxSize from the file record. File writes of 2400 bytes
succeeded even though reserved_length showed 512.

## Current Device State

### Installed Applets
- System (0x0000)
- AlphaWord Plus (0xa000)
- Calculator (0xa002)
- Control Panel (0xa007)
- Alpha USB (0xa130)
- Forth Mini (0xa131)
- WriteOrDie (0xa133)
- Snake (0xa135)
- File Fixer (0xa137)
- Neo Fonts: Small, Medium, Large, Very Large, Extra Large

### Filesystem
All 8 AlphaWord slots operational. File writes work at any size.
