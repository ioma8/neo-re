import argparse

from real_check.client import NeoAlphaWordClient
from real_check.live_usb import open_direct_usb_transport, probe_direct_usb_device


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="real-check")
    subparsers = parser.add_subparsers(dest="command")

    subparsers.add_parser("probe")
    subparsers.add_parser("list")

    get_parser = subparsers.add_parser("get")
    get_parser.add_argument("slot")
    get_parser.add_argument("--output")

    try:
        args = parser.parse_args(argv)
    except SystemExit as exc:
        return int(exc.code)

    if args.command == "probe":
        result = probe_direct_usb_device()
        print(
            f"vendor_id=0x{result.vendor_id:04x} "
            f"product_id=0x{result.product_id:04x} "
            f"interface={result.interface_number} "
            f"out_ep=0x{result.out_endpoint:02x} "
            f"in_ep=0x{result.in_endpoint:02x}"
        )
        return 0

    if args.command == "list":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            for entry in client.list_alpha_word_files():
                print(
                    f"slot={entry.slot} name={entry.name} "
                    f"file_length={entry.file_length} reserved_length={entry.reserved_length}"
                )
        finally:
            client.close()
        return 0

    if args.command == "get":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            payload = client.download_alpha_word_file(slot=int(args.slot, 0))
        finally:
            client.close()
        if args.output:
            with open(args.output, "wb") as handle:
                handle.write(payload)
        else:
            print(payload.hex(" "))
        return 0

    parser.print_help()
    return 0
