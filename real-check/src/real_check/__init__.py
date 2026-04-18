import argparse
from pathlib import Path
import sys

from neotools.smartapplets import parse_smartapplet_metadata

from real_check.client import NeoAlphaWordClient, parse_neo_os_segments
from real_check.hid_switch import send_hid_output_report_sequence, send_manager_switch_sequence
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


def _parse_hid_sequence(chunks: list[str]) -> tuple[int, ...]:
    tokens = " ".join(chunks).replace(",", " ").split()
    if not tokens:
        raise ValueError("empty HID output-report sequence")
    values = []
    for token in tokens:
        value = int(token, 16)
        if value < 0 or value > 0xFF:
            raise ValueError(f"invalid HID report byte: 0x{value:x}")
        values.append(value)
    return tuple(values)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="real-check")
    subparsers = parser.add_subparsers(dest="command")

    subparsers.add_parser("probe")
    watch_parser = subparsers.add_parser("watch")
    watch_parser.add_argument("--timeout", type=float, default=10.0)
    watch_parser.add_argument("--try-switch", action="store_true")
    subparsers.add_parser("switch-to-direct")
    switch_sequence_parser = subparsers.add_parser("switch-hid-sequence")
    switch_sequence_parser.add_argument("sequence", nargs="+")
    switch_sequence_parser.add_argument("--delay", type=float, default=2.0)
    switch_sequence_parser.add_argument("--wait", type=float, default=5.0)
    subparsers.add_parser("debug-attributes")
    subparsers.add_parser("list")
    subparsers.add_parser("applets")
    subparsers.add_parser("debug-applets")
    dump_applet_parser = subparsers.add_parser("dump-applet")
    dump_applet_parser.add_argument("applet_id")
    dump_applet_parser.add_argument("--output", required=True)
    install_applet_parser = subparsers.add_parser("install-applet")
    install_applet_parser.add_argument("path")
    install_applet_parser.add_argument(
        "--assume-updater",
        action="store_true",
        help="skip ?Swtch bootstrap when the NEO is already on the SmartApplet loading/updater screen",
    )
    remove_applet_parser = subparsers.add_parser("remove-applet-index")
    remove_applet_parser.add_argument("index")
    subparsers.add_parser("clear-applet-area")
    restore_stock_parser = subparsers.add_parser("restore-stock-applets")
    restore_stock_parser.add_argument("--backup-dir", required=True)
    restore_stock_parser.add_argument(
        "--yes",
        action="store_true",
        help="required because this clears the SmartApplet area before restoring backups",
    )
    restore_stock_parser.add_argument(
        "--include-system",
        action="store_true",
        help="also install applet id 0x0000; normally unsafe and unnecessary",
    )
    restore_stock_parser.add_argument(
        "--skip",
        action="append",
        default=[],
        help="applet id to skip, for example 0xa017; may be repeated",
    )
    restore_stock_parser.add_argument("--restart", action="store_true")
    install_os_parser = subparsers.add_parser("install-os-image")
    install_os_parser.add_argument("path")
    install_os_parser.add_argument(
        "--yes-flash-os",
        action="store_true",
        help="required because this erases and rewrites the NEO System OS flash segments",
    )
    install_os_parser.add_argument(
        "--reformat-rest-of-rom",
        action="store_true",
        help="use NeoManager's length-0 erase for the 0x005ffc00 segment before rewriting the OS tail",
    )
    subparsers.add_parser("restart-device")

    verify_get_parser = subparsers.add_parser("verify-get")
    verify_get_parser.add_argument("slot")

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

    if args.command == "switch-hid-sequence":
        try:
            sequence = _parse_hid_sequence(args.sequence)
            result = send_hid_output_report_sequence(
                sequence,
                delay_seconds=args.delay,
                wait_for_direct_seconds=args.wait,
            )
        except (RuntimeError, ValueError) as exc:
            print(str(exc), file=sys.stderr)
            return 1
        print(f"sent_hid_reports={result.reports_sent}")
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

    if args.command == "applets":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            for entry in client.list_smart_applets():
                print(
                    f"applet_id=0x{entry.applet_id:04x} version={entry.version_major}.{entry.version_minor} "
                    f"name={entry.name} file_size={entry.file_size} applet_class=0x{entry.applet_class:02x}"
                )
        finally:
            client.close()
        return 0

    if args.command == "debug-applets":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            for line in client.debug_smart_applet_records():
                print(line)
        finally:
            client.close()
        return 0

    if args.command == "dump-applet":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            payload = client.download_smart_applet(applet_id=int(args.applet_id, 0))
        finally:
            client.close()
        with open(args.output, "wb") as handle:
            handle.write(payload)
        print(f"applet_id=0x{int(args.applet_id, 0):04x} bytes_read={len(payload)} output={args.output}")
        return 0

    if args.command == "install-applet":
        with open(args.path, "rb") as handle:
            image = handle.read()
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            if args.assume_updater:
                client.assume_updater_mode()
            entry = client.install_smart_applet(image)
        finally:
            client.close()
        print(
            f"installed applet_id=0x{entry.applet_id:04x} "
            f"version={entry.version_major}.{entry.version_minor} "
            f"name={entry.name} file_size={entry.file_size} "
            f"applet_class=0x{entry.applet_class:02x}"
        )
        return 0

    if args.command == "remove-applet-index":
        index = int(args.index, 0)
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            client.remove_smart_applet_by_index(index=index)
        finally:
            client.close()
        print(f"removed applet_index={index}")
        return 0

    if args.command == "clear-applet-area":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            client.clear_smart_applet_area()
        finally:
            client.close()
        print("cleared SmartApplet area")
        return 0

    if args.command == "restore-stock-applets":
        if not args.yes:
            print("refusing to clear SmartApplet area without --yes", file=sys.stderr)
            return 2
        backup_dir = Path(args.backup_dir)
        images = sorted(backup_dir.glob("*.os3kapp"))
        if not images:
            print(f"no .os3kapp files found in {backup_dir}", file=sys.stderr)
            return 1
        skipped_ids = {int(value, 0) for value in args.skip}
        selected: list[tuple[Path, bytes]] = []
        for image_path in images:
            image = image_path.read_bytes()
            metadata = parse_smartapplet_metadata(image[:0x84])
            if metadata.applet_id == 0 and not args.include_system:
                print(f"skip applet_id=0x0000 name={metadata.name} path={image_path}")
                continue
            if metadata.applet_id in skipped_ids:
                print(f"skip applet_id=0x{metadata.applet_id:04x} name={metadata.name} path={image_path}")
                continue
            selected.append((image_path, image))
        if not selected:
            print("no applets selected for restore", file=sys.stderr)
            return 1

        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            client.clear_smart_applet_area()
            print("cleared SmartApplet area")
            for image_path, image in selected:
                entry = client.install_smart_applet(image)
                print(
                    f"installed applet_id=0x{entry.applet_id:04x} "
                    f"version={entry.version_major}.{entry.version_minor} "
                    f"name={entry.name} file_size={entry.file_size} path={image_path}"
                )
            if args.restart:
                client.restart_device()
                print("sent restart command")
        finally:
            client.close()
        return 0

    if args.command == "install-os-image":
        if not args.yes_flash_os:
            print("refusing to flash OS image without --yes-flash-os", file=sys.stderr)
            return 2
        image_path = Path(args.path)
        image = image_path.read_bytes()
        segments = parse_neo_os_segments(image)
        print(
            f"validated NEO OS image path={image_path} bytes={len(image)} "
            f"segments={len(segments)}"
        )
        for segment in segments:
            erase_kb = (segment.length + 0x3FF) >> 10
            if args.reformat_rest_of_rom and segment.address == 0x005FFC00:
                erase_kb = 0
            print(f"segment address=0x{segment.address:08x} length={segment.length} erase_kb={erase_kb}")
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            summary = client.install_neo_os_image(image, reformat_rest_of_rom=args.reformat_rest_of_rom)
        finally:
            client.close()
        print(
            f"flashed NEO OS bytes={summary.bytes_written} "
            f"chunks={summary.chunks_written} segments={len(summary.segments)}"
        )
        return 0

    if args.command == "restart-device":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            client.restart_device()
        finally:
            client.close()
        print("sent restart command")
        return 0

    if args.command == "verify-get":
        transport = open_direct_usb_transport()
        client = NeoAlphaWordClient(transport)
        try:
            verification = client.verify_alpha_word_file(slot=int(args.slot, 0))
        finally:
            client.close()
        print(
            f"slot={verification.slot} reported_length={verification.reported_length} "
            f"bytes_read={verification.bytes_read} "
            f"sum16=0x{verification.sum16:04x} sha256={verification.sha256}"
        )
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
