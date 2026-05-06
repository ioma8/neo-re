#!/usr/bin/env python3
"""
Fix AlphaWord file MaxSize on a connected AlphaSmart NEO.

This script uses the proven NeoManager send flow to update file attributes
with correct MaxSize (reserved_length = 51200 instead of 512).

The protocol: for each file slot, sends BOTH attributes AND file data
via the updater protocol, matching what NeoManager does during restore.

Usage:
  uv run --project real-check python3 scripts/fix_file_sizes.py
"""

import sys
import time
import hashlib

sys.path.insert(0, "poc/neotools/src")
sys.path.insert(0, "real-check/src")

from neotools.updater_packets import (
    build_updater_command,
    build_raw_file_attributes_command,
    build_retrieve_file_command,
)
from neotools.updater_responses import parse_updater_response
from neotools.switch_packets import build_reset_preamble, build_switch_packet
from neotools.alphaword_attributes import parse_file_attributes_record
from real_check.live_usb import open_direct_usb_transport

CORRECT_RESERVED_LENGTH = 51200  # 0xC800
ALPHAWORD_APPLET_ID = 0xA000


def _checksum16(payload: bytes) -> int:
    return sum(payload) & 0xFFFF


def modify_attribute_record(record: bytes, new_reserved_length: int) -> bytes:
    """Modify the reserved_length field (bytes 24-27, big-endian) in a 40-byte record."""
    if len(record) != 0x28:
        raise ValueError(f"attribute record must be 0x28 bytes, got {len(record)}")
    return record[:0x18] + new_reserved_length.to_bytes(4, "big") + record[0x1C:]


def require_status(transport, expected_status, timeout_ms, context):
    """Read an 8-byte response and verify status code."""
    response = parse_updater_response(transport.read_exact(8, timeout_ms=timeout_ms))
    if response.status != expected_status:
        raise ValueError(
            f"{context}: expected 0x{expected_status:02x}, got 0x{response.status:02x} "
            f"(arg=0x{response.argument:08x} trail=0x{response.trailing:04x})"
        )
    return response


def send_file_record(transport, file_slot, record, payload):
    """Send one AlphaWord file record using the NeoManager send flow."""
    # Enter updater mode fresh
    transport.write(build_reset_preamble())
    transport.write(build_switch_packet(0))
    switch_resp = transport.read_exact(8, timeout_ms=3000)
    if parse_updater_response(switch_resp).status != 0x53:  # 'S' = Switched
        # Try reading 'Switched' text response
        pass

    # === Phase 1: Write attributes ===
    transport.write(build_updater_command(command=0x1D, argument=file_slot, trailing=ALPHAWORD_APPLET_ID))
    require_status(transport, 0x5B, 3000, f"slot {file_slot} begin-attr")

    transport.write(build_updater_command(command=0x02, argument=0x28, trailing=_checksum16(record)))
    require_status(transport, 0x42, 3000, f"slot {file_slot} attr-handshake")

    transport.write(record)
    # After sending data, try to read response
    time.sleep(0.1)
    try:
        resp = parse_updater_response(transport.read_exact(8, timeout_ms=1500))
        print(f"    after-attr-data: status=0x{resp.status:02x}", end="")
        if resp.status == 0x5C:
            print(" (auto-finish!)")
            # Attributes done, skip to file data phase
            pass
        elif resp.status == 0x43:
            print(" (commit), sending 0x1E...")
            transport.write(build_updater_command(command=0x1E, argument=file_slot, trailing=ALPHAWORD_APPLET_ID))
            resp2 = parse_updater_response(transport.read_exact(8, timeout_ms=3000))
            print(f"    after-0x1E: status=0x{resp2.status:02x}")
        else:
            print()
    except Exception as e:
        print(f"    no auto-response: {e}")
        # Try 0xFF probe
        transport.write(build_updater_command(command=0xFF, argument=0, trailing=0))
        resp = parse_updater_response(transport.read_exact(8, timeout_ms=3000))
        print(f"    after-0xFF: status=0x{resp.status:02x}")
        
        transport.write(build_updater_command(command=0x1E, argument=file_slot, trailing=ALPHAWORD_APPLET_ID))
        try:
            resp = parse_updater_response(transport.read_exact(8, timeout_ms=3000))
            print(f"    after-0x1E: status=0x{resp.status:02x}")
        except Exception as e2:
            print(f"    0x1E error: {e2}")

    # === Phase 2: Write file data ===
    if len(payload) > 0:
        cmd = build_updater_command(
            command=0x14,
            argument=(file_slot << 24) | len(payload),
            trailing=ALPHAWORD_APPLET_ID,
        )
        transport.write(cmd)
        require_status(transport, 0x50, 5000, f"slot {file_slot} begin-file")

        for offset in range(0, len(payload), 0x400):
            chunk = payload[offset:offset + 0x400]
            transport.write(
                build_updater_command(command=0x02, argument=len(chunk), trailing=_checksum16(chunk))
            )
            require_status(transport, 0x42, 3000, f"slot {file_slot} chunk-handshake")
            
            transport.write(chunk)
            # Try reading auto-response after data
            time.sleep(0.05)
            try:
                chunk_resp = parse_updater_response(transport.read_exact(8, timeout_ms=1000))
                if chunk_resp.status != 0x43:
                    pass  # Unexpected but continue
            except:
                # Send 0xFF probe
                transport.write(build_updater_command(command=0xFF, argument=0, trailing=0))
                require_status(transport, 0x43, 3000, f"slot {file_slot} chunk-commit")

        # Finish file data
        transport.write(build_updater_command(command=0x15, argument=0, trailing=0))
        try:
            resp = parse_updater_response(transport.read_exact(8, timeout_ms=3000))
            if resp.status == 0x51:
                print(f"  slot {file_slot}: file data OK")
            else:
                print(f"  slot {file_slot}: file-finish=0x{resp.status:02x}")
        except Exception as e:
            print(f"  slot {file_slot}: file-finish error: {e}")
    else:
        print(f"  slot {file_slot}: no file data to send")


def get_device_state():
    """Check if NEO is connected and in what mode."""
    from real_check.live_usb import watch_alphasmart_devices
    devices = watch_alphasmart_devices(timeout_seconds=5)
    return devices


def main():
    print("=" * 60)
    print("AlphaWord File Size Fix Tool - v3")
    print("=" * 60)
    print()

    # Check device state
    print("Checking device...")
    devices = get_device_state()
    if not devices:
        print("ERROR: No AlphaSmart NEO detected!")
        print("Please connect the NEO via USB and ensure it's powered on.")
        print()
        print("If the NEO was just power-cycled, it may be in keyboard mode (081e:bd04).")
        print("Run 'uv run --project real-check real-check switch-to-direct' first,")
        print("then run this script again.")
        return 1
    
    for d in devices:
        print(f"  {d.vendor_id:04x}:{d.product_id:04x} mode={d.mode.kind}: {d.mode.detail}")
    
    direct_devices = [d for d in devices if d.product_id == 0xBD01]
    if not direct_devices:
        print("ERROR: NEO is not in direct USB mode!")
        print("Run: uv run --project real-check real-check switch-to-direct")
        print("Then run this script again.")
        return 1

    # Step 1: Read all file records and data
    print("\nStep 1: Reading current file records...")
    transport = open_direct_usb_transport()
    try:
        # Enter updater mode
        transport.write(build_reset_preamble())
        transport.write(build_switch_packet(0))
        transport.read_exact(8, timeout_ms=3000)

        all_records = {}
        for slot in range(1, 9):
            # Read attributes
            transport.write(build_raw_file_attributes_command(file_slot=slot, applet_id=ALPHAWORD_APPLET_ID))
            resp = parse_updater_response(transport.read_exact(8, timeout_ms=3000))
            if resp.status == 0x90:
                print(f"  Slot {slot}: empty (status 0x90)")
                continue
            if resp.status != 0x5A:
                print(f"  Slot {slot}: error status 0x{resp.status:02x}")
                continue
            
            record = transport.read_exact(resp.argument, timeout_ms=3000)
            if len(record) != 0x28:
                print(f"  Slot {slot}: bad record length {len(record)}")
                continue
            
            file_length = int.from_bytes(record[0x1C:0x20], "big")
            
            # Read file data if present
            payload = b""
            if file_length > 0:
                transport.write(
                    build_retrieve_file_command(
                        file_slot=slot,
                        applet_id=ALPHAWORD_APPLET_ID,
                        requested_length=file_length,
                        alternate_mode=False,
                    )
                )
                start_resp = parse_updater_response(transport.read_exact(8, timeout_ms=10000))
                if start_resp.status == 0x53:
                    remaining = min(start_resp.argument, file_length)
                    data = bytearray()
                    while remaining > 0:
                        transport.write(build_updater_command(command=0x10, argument=0, trailing=0))
                        chunk_resp = parse_updater_response(transport.read_exact(8, timeout_ms=3000))
                        if chunk_resp.status != 0x4D:
                            break
                        chunk = transport.read_exact(chunk_resp.argument, timeout_ms=3000)
                        data.extend(chunk)
                        remaining -= len(chunk)
                    payload = bytes(data)

            attr = parse_file_attributes_record(record)
            print(f"  Slot {slot}: name='{attr.name}' reserved={attr.reserved_length} "
                  f"file_len={attr.file_length} data={len(payload)} bytes")
            all_records[slot] = (record, payload)
    finally:
        transport.close()

    if not all_records:
        print("ERROR: No file records found!")
        return 1

    # Check which need fixing
    needs_fix = {}
    for slot, (record, payload) in all_records.items():
        reserved = int.from_bytes(record[0x18:0x1C], "big")
        if reserved != CORRECT_RESERVED_LENGTH:
            needs_fix[slot] = (record, payload)

    if not needs_fix:
        print("\nAll slots already have correct reserved_length (51200). Nothing to do.")
        return 0

    print(f"\nStep 2: {len(needs_fix)} slot(s) need fixing")
    for slot in sorted(needs_fix.keys()):
        record, payload = needs_fix[slot]
        attr = parse_file_attributes_record(record)
        print(f"  Slot {slot}: reserved {attr.reserved_length} -> {CORRECT_RESERVED_LENGTH} "
              f"({attr.file_length} bytes data)")

    # Confirm
    print("\nThis will MODIFY the NEO's persistent file records.")
    print("Current file contents will be preserved and rewritten.")
    print("Proceed? (y/N): ", end="")
    sys.stdout.flush()
    answer = sys.stdin.readline().strip().lower()
    if answer != "y":
        print("Aborted.")
        return 1

    # Step 3: Send modified records
    print("\nStep 3: Sending modified records...")
    successes = 0
    failures = 0

    for slot in sorted(needs_fix.keys()):
        record, payload = needs_fix[slot]
        new_record = modify_attribute_record(record, CORRECT_RESERVED_LENGTH)
        attr = parse_file_attributes_record(new_record)
        
        print(f"\n  --- Slot {slot}: '{attr.name}' reserved={attr.reserved_length} "
              f"file_length={attr.file_length} data={len(payload)} bytes ---")

        transport = open_direct_usb_transport()
        try:
            send_file_record(transport, slot, new_record, payload)
            successes += 1
        except Exception as e:
            print(f"  FAILED: {e}")
            failures += 1
        finally:
            transport.close()
        
        time.sleep(0.5)

    # Step 4: Verify
    print("\nStep 4: Verifying...")
    time.sleep(1)
    
    transport = open_direct_usb_transport()
    try:
        transport.write(build_reset_preamble())
        transport.write(build_switch_packet(0))
        transport.read_exact(8, timeout_ms=3000)

        all_ok = True
        for slot in range(1, 9):
            transport.write(build_raw_file_attributes_command(file_slot=slot, applet_id=ALPHAWORD_APPLET_ID))
            try:
                resp = parse_updater_response(transport.read_exact(8, timeout_ms=3000))
                if resp.status == 0x5A:
                    vrecord = transport.read_exact(resp.argument, timeout_ms=3000)
                    vreserved = int.from_bytes(vrecord[0x18:0x1C], "big")
                    vfile_len = int.from_bytes(vrecord[0x1C:0x20], "big")
                    status = "OK" if vreserved == CORRECT_RESERVED_LENGTH else "MISMATCH"
                    if status != "OK":
                        all_ok = False
                    print(f"  Slot {slot}: reserved={vreserved} file_len={vfile_len} [{status}]")
                elif resp.status == 0x90:
                    print(f"  Slot {slot}: (empty)")
            except Exception as e:
                print(f"  Slot {slot}: verify error: {e}")
                all_ok = False
    finally:
        transport.close()

    print(f"\nResults: {successes} succeeded, {failures} failed")
    if all_ok:
        print("SUCCESS: All file slots now have correct reserved_length (51200).")
        print("The MaxSize limit is now 51200 characters per file.")
        print("Restart the NEO to ensure changes take full effect.")
        return 0
    else:
        print("SOME SLOTS STILL HAVE INCORRECT VALUES.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
