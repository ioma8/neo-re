import argparse
import sys

from real_check.client import NeoAlphaWordClient
from real_check.hid_switch import send_manager_switch_sequence
from real_check.live_usb import open_direct_usb_transport, probe_direct_usb_device, watch_alphasmart_devices
from real_check.usb_select import EndpointDescriptor, InterfaceDescriptor


def _endpoint_direction(endpoint: EndpointDescriptor) -> str:
    return "in" if endpoint.is_in else "out"


def _format_interface(interface: InterfaceDescriptor) -> str:
    endpoints = " ".join(
        f"0x{endpoint.address:02x}:{endpoint.transfer_type}:{_endpoint_direction(endpoint)}:max{endpoint.max_packet_size}"
        for endpoint in interface.endpoints
    )
    return f"  interface={interface.number} alt={interface.alternate_setting} endpoints={endpoints}"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="real-check")
    subparsers = parser.add_subparsers(dest="command")

    subparsers.add_parser("probe")
    watch_parser = subparsers.add_parser("watch")
    watch_parser.add_argument("--timeout", type=float, default=10.0)
    watch_parser.add_argument("--try-switch", action="store_true")
    subparsers.add_parser("switch-to-direct")
    subparsers.add_parser("debug-attributes")
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

    if args.command == "watch":
        observations = watch_alphasmart_devices(
            timeout_seconds=args.timeout,
            interval_seconds=0.25,
            try_switch=args.try_switch,
        )
        for observation in observations:
            switch_suffix = f" switch={observation.switch_result}" if observation.switch_result else ""
            print(
                f"vendor_id=0x{observation.vendor_id:04x} "
                f"product_id=0x{observation.product_id:04x} "
                f"mode={observation.mode.kind} "
                f"detail={observation.mode.detail}"
                f"{switch_suffix}"
            )
            for interface in observation.interfaces:
                print(_format_interface(interface))
        if not observations:
            print("No AlphaSmart USB device observed")
        return 0

    if args.command == "switch-to-direct":
        try:
            result = send_manager_switch_sequence()
        except RuntimeError as exc:
            print(str(exc), file=sys.stderr)
            return 1
        print(f"sent_manager_switch_reports={result.reports_sent}")
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

    if args.command == "debug-attributes":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            for line in client.debug_alpha_word_attributes():
                print(line)
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
