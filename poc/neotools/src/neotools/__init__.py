import argparse

from neotools.switch_packets import build_switch_packet
from neotools.usb_descriptor import (
    is_direct_neo_descriptor,
    parse_usb_device_descriptor,
)


def _parse_hex_bytes(raw: str) -> bytes:
    compact = raw.replace(" ", "")
    return bytes.fromhex(compact)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="neotools")
    subparsers = parser.add_subparsers(dest="command", required=True)

    descriptor_parser = subparsers.add_parser("descriptor")
    descriptor_parser.add_argument("raw_hex")

    switch_packet_parser = subparsers.add_parser("switch-packet")
    switch_packet_parser.add_argument("applet_id")

    args = parser.parse_args(argv)

    if args.command == "descriptor":
        descriptor = parse_usb_device_descriptor(_parse_hex_bytes(args.raw_hex))
        print(
            f"vendor_id=0x{descriptor.vendor_id:04x} "
            f"product_id=0x{descriptor.product_id:04x} "
            f"direct_neo={is_direct_neo_descriptor(descriptor)}"
        )
        return 0

    if args.command == "switch-packet":
        applet_id = int(args.applet_id, 0)
        packet = build_switch_packet(applet_id)
        print(packet.hex(" "))
        return 0

    raise AssertionError(f"unhandled command: {args.command}")
