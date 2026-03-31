import argparse

from neotools.alphaword_flow import (
    build_direct_usb_full_text_retrieval_plan,
    build_full_text_retrieval_plan,
)
from neotools.alphaword_attributes import parse_file_attributes_record
from neotools.alphaword_get_print import (
    build_retrieve_all_alpha_word_slots_for_device_flow,
    build_direct_usb_full_get_print_flow,
    build_direct_usb_preview_refresh_flow,
    build_retrieve_full_alpha_word_text_flow,
    build_updater_retrieve_applet_file_data_flow,
)
from neotools.alphaword_session import (
    build_direct_usb_full_text_session,
    build_direct_usb_preview_session,
)
from neotools.alphaword_send import build_direct_usb_send_file_record
from neotools.smartapplets import (
    build_direct_usb_add_applet_plan,
    build_direct_usb_add_applet_plan_from_image,
    build_direct_usb_retrieve_applet_plan,
    derive_add_applet_start_fields,
    get_known_smartapplet_menu,
    parse_smartapplet_metadata,
    parse_smartapplet_header,
    resolve_known_smartapplet_string,
)
from neotools.switch_packets import build_switch_packet
from neotools.updater_packets import build_updater_command
from neotools.updater_responses import parse_updater_response
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

    updater_packet_parser = subparsers.add_parser("updater-packet")
    updater_packet_parser.add_argument("command_byte")
    updater_packet_parser.add_argument("argument")
    updater_packet_parser.add_argument("trailing")

    alphaword_plan_parser = subparsers.add_parser("alphaword-plan")
    alphaword_plan_parser.add_argument("applet_id")
    alphaword_plan_parser.add_argument("file_slot")

    direct_usb_plan_parser = subparsers.add_parser("direct-usb-alphaword-plan")
    direct_usb_plan_parser.add_argument("applet_id")
    direct_usb_plan_parser.add_argument("file_slot")

    direct_usb_session_parser = subparsers.add_parser("direct-usb-alphaword-session")
    direct_usb_session_parser.add_argument("mode", choices=["preview", "full"])
    direct_usb_session_parser.add_argument("applet_id")

    get_print_flow_parser = subparsers.add_parser("alphaword-get-print-flow")
    get_print_flow_parser.add_argument(
        "mode",
        choices=["preview-all", "full-all", "full-selected", "full-single"],
    )
    get_print_flow_parser.add_argument("applet_id")
    get_print_flow_parser.add_argument("file_slots", nargs="*")

    retrieve_all_flow_parser = subparsers.add_parser("retrieve-all-alphaword-slots-flow")
    retrieve_all_flow_parser.add_argument("applet_id")
    retrieve_all_flow_parser.add_argument("--file-slots", nargs="*", default=[])
    retrieve_all_flow_parser.add_argument("--initial-statuses", nargs="*", default=[])
    retrieve_all_flow_parser.add_argument("--fail-slot")
    retrieve_all_flow_parser.add_argument("--cancel-after-slot")

    retrieve_full_text_parser = subparsers.add_parser("retrieve-full-alphaword-text-flow")
    retrieve_full_text_parser.add_argument("transport_mode")
    retrieve_full_text_parser.add_argument("applet_id")
    retrieve_full_text_parser.add_argument("file_slot")
    retrieve_full_text_parser.add_argument("--retrieved-length", default="0")
    retrieve_full_text_parser.add_argument("--alternate-selector", default="0")
    retrieve_full_text_parser.add_argument("--temp-open-success", choices=["true", "false"], default="true")

    updater_retrieve_parser = subparsers.add_parser("updater-retrieve-applet-file-data-flow")
    updater_retrieve_parser.add_argument("command_selector")
    updater_retrieve_parser.add_argument("applet_id")
    updater_retrieve_parser.add_argument("file_slot")
    updater_retrieve_parser.add_argument("requested_length")
    updater_retrieve_parser.add_argument("--reported-total-length", required=True)
    updater_retrieve_parser.add_argument("--chunk-lengths", nargs="*", default=[])
    updater_retrieve_parser.add_argument("--chunk-checksums", nargs="*", default=[])
    updater_retrieve_parser.add_argument("--computed-chunk-checksums", nargs="*", default=[])
    updater_retrieve_parser.add_argument("--progress-current", default="0")
    updater_retrieve_parser.add_argument("--progress-total", default="0")
    updater_retrieve_parser.add_argument("--start-status", default="0x53")
    updater_retrieve_parser.add_argument("--chunk-status", default="0x4d")

    direct_usb_send_parser = subparsers.add_parser("direct-usb-alphaword-send-record")
    direct_usb_send_parser.add_argument("applet_id")
    direct_usb_send_parser.add_argument("file_slot")
    direct_usb_send_parser.add_argument("record_hex")
    direct_usb_send_parser.add_argument("payload_hex")

    smartapplet_retrieve_parser = subparsers.add_parser("smartapplet-retrieve-plan")
    smartapplet_retrieve_parser.add_argument("applet_id")

    smartapplet_add_parser = subparsers.add_parser("smartapplet-add-plan")
    smartapplet_add_parser.add_argument("start_argument")
    smartapplet_add_parser.add_argument("trailing")
    smartapplet_add_parser.add_argument("payload_hex")

    smartapplet_add_from_image_parser = subparsers.add_parser("smartapplet-add-plan-from-image")
    smartapplet_add_from_image_parser.add_argument("image_hex")

    smartapplet_header_parser = subparsers.add_parser("smartapplet-header")
    smartapplet_header_parser.add_argument("header_hex")

    smartapplet_metadata_parser = subparsers.add_parser("smartapplet-metadata")
    smartapplet_metadata_parser.add_argument("header_hex")

    smartapplet_string_parser = subparsers.add_parser("smartapplet-string")
    smartapplet_string_parser.add_argument("resource_id")

    smartapplet_menu_parser = subparsers.add_parser("smartapplet-menu")
    smartapplet_menu_parser.add_argument("resource_id")

    decode_response_parser = subparsers.add_parser("decode-updater-response")
    decode_response_parser.add_argument("raw_hex")

    parse_attributes_parser = subparsers.add_parser("parse-alphaword-attributes")
    parse_attributes_parser.add_argument("raw_hex")

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

    if args.command == "updater-packet":
        packet = build_updater_command(
            command=int(args.command_byte, 0),
            argument=int(args.argument, 0),
            trailing=int(args.trailing, 0),
        )
        print(packet.hex(" "))
        return 0

    if args.command == "alphaword-plan":
        plan = build_full_text_retrieval_plan(
            applet_id=int(args.applet_id, 0),
            file_slot=int(args.file_slot, 0),
        )
        for step in plan:
            print(f"{step.kind}: {step.packet.hex(' ')}")
        return 0

    if args.command == "direct-usb-alphaword-plan":
        plan = build_direct_usb_full_text_retrieval_plan(
            applet_id=int(args.applet_id, 0),
            file_slot=int(args.file_slot, 0),
        )
        for step in plan:
            print(f"{step.kind}: {step.packet.hex(' ')}")
        return 0

    if args.command == "direct-usb-alphaword-session":
        if args.mode == "preview":
            session = build_direct_usb_preview_session(applet_id=int(args.applet_id, 0))
        else:
            session = build_direct_usb_full_text_session(applet_id=int(args.applet_id, 0))
        for step in session:
            print(f"{step.kind}: {step.packet.hex(' ')}")
        return 0

    if args.command == "alphaword-get-print-flow":
        slots = [int(raw, 0) for raw in args.file_slots] or list(range(1, 9))
        if args.mode == "preview-all":
            flow = build_direct_usb_preview_refresh_flow(
                applet_id=int(args.applet_id, 0),
                file_slots=slots,
            )
        else:
            flow = build_direct_usb_full_get_print_flow(
                applet_id=int(args.applet_id, 0),
                file_slots=slots,
                selected_only=args.mode == "full-selected",
                single_slot=args.mode == "full-single",
            )
        for step in flow:
            if step.packet is not None:
                if step.file_slot is None:
                    print(f"{step.kind}: {step.packet.hex(' ')}")
                else:
                    print(f"{step.kind} slot={step.file_slot}: {step.packet.hex(' ')}")
            elif step.status is not None:
                print(f"{step.kind} slot={step.file_slot} status={step.status}")
            else:
                if step.file_slot is None:
                    print(f"{step.kind}: {step.detail}")
                else:
                    print(f"{step.kind} slot={step.file_slot}")
        return 0

    if args.command == "retrieve-all-alphaword-slots-flow":
        slots = [int(raw, 0) for raw in args.file_slots] or list(range(1, 9))
        initial_statuses: dict[int, int] = {}
        for raw in args.initial_statuses:
            slot_raw, status_raw = raw.split("=", 1)
            initial_statuses[int(slot_raw, 0)] = int(status_raw, 0)
        result = build_retrieve_all_alpha_word_slots_for_device_flow(
            applet_id=int(args.applet_id, 0),
            file_slots=slots,
            initial_statuses=initial_statuses,
            fail_slot=int(args.fail_slot, 0) if args.fail_slot is not None else None,
            cancel_after_slot=int(args.cancel_after_slot, 0)
            if args.cancel_after_slot is not None
            else None,
        )
        for step in result.steps:
            if step.packet is not None:
                print(f"{step.kind} slot={step.file_slot}: {step.packet.hex(' ')}")
            elif step.status is not None:
                if step.file_slot is None:
                    print(f"{step.kind} status=0x{step.status:02x}")
                else:
                    print(f"{step.kind} slot={step.file_slot} status={step.status}")
            else:
                print(f"{step.kind}: {step.detail}")
        print(f"return_code: 0x{result.return_code:02x}")
        return 0

    if args.command == "retrieve-full-alphaword-text-flow":
        result = build_retrieve_full_alpha_word_text_flow(
            transport_mode=int(args.transport_mode, 0),
            applet_id=int(args.applet_id, 0),
            file_slot=int(args.file_slot, 0),
            retrieved_length=int(args.retrieved_length, 0),
            alternate_selector=int(args.alternate_selector, 0),
            temp_open_success=args.temp_open_success == "true",
        )
        for step in result.steps:
            if step.packet is not None:
                print(f"{step.kind} slot={step.file_slot}: {step.packet.hex(' ')}")
            elif step.status is not None:
                if step.kind == "transport" and step.detail is not None:
                    print(f"{step.kind}: {step.detail} selector={step.status}")
                elif step.file_slot is None:
                    print(f"{step.kind} status=0x{step.status:02x}")
                else:
                    print(f"{step.kind} slot={step.file_slot} status={step.status}")
            else:
                print(f"{step.kind}: {step.detail}")
        print(f"return_code: 0x{result.return_code:02x}")
        return 0

    if args.command == "updater-retrieve-applet-file-data-flow":
        result = build_updater_retrieve_applet_file_data_flow(
            command_selector=int(args.command_selector, 0),
            applet_id=int(args.applet_id, 0),
            file_slot=int(args.file_slot, 0),
            requested_length=int(args.requested_length, 0),
            reported_total_length=int(args.reported_total_length, 0),
            chunk_lengths=[int(raw, 0) for raw in args.chunk_lengths],
            chunk_checksums=[int(raw, 0) for raw in args.chunk_checksums],
            computed_chunk_checksums=[int(raw, 0) for raw in args.computed_chunk_checksums]
            if args.computed_chunk_checksums
            else None,
            progress_current=int(args.progress_current, 0),
            progress_total=int(args.progress_total, 0),
            start_status=int(args.start_status, 0),
            chunk_status=int(args.chunk_status, 0),
        )
        for step in result.steps:
            if step.packet is not None:
                if step.file_slot is None:
                    print(f"{step.kind}: {step.packet.hex(' ')}")
                else:
                    print(f"{step.kind} slot={step.file_slot}: {step.packet.hex(' ')}")
            elif step.status is not None:
                if step.file_slot is None:
                    print(f"{step.kind} status=0x{step.status:08x}")
                else:
                    print(f"{step.kind} slot={step.file_slot} status={step.status}")
            else:
                print(f"{step.kind}: {step.detail}")
        print(f"return_code: 0x{result.return_code & 0xffffffff:08x}")
        return 0

    if args.command == "direct-usb-alphaword-send-record":
        plan = build_direct_usb_send_file_record(
            applet_id=int(args.applet_id, 0),
            file_slot=int(args.file_slot, 0),
            record=_parse_hex_bytes(args.record_hex),
            payload=_parse_hex_bytes(args.payload_hex),
        )
        for step in plan:
            if isinstance(step.packet, bytes):
                print(f"{step.kind}: {step.packet.hex(' ')}")
            else:
                raise AssertionError("unexpected non-bytes packet payload")
        return 0

    if args.command == "smartapplet-retrieve-plan":
        plan = build_direct_usb_retrieve_applet_plan(applet_id=int(args.applet_id, 0))
        for step in plan:
            print(f"{step.kind}: {step.packet.hex(' ')}")
        return 0

    if args.command == "smartapplet-add-plan":
        plan = build_direct_usb_add_applet_plan(
            start_argument=int(args.start_argument, 0),
            trailing=int(args.trailing, 0),
            payload=_parse_hex_bytes(args.payload_hex),
        )
        for step in plan:
            print(f"{step.kind}: {step.packet.hex(' ')}")
        return 0

    if args.command == "smartapplet-add-plan-from-image":
        plan = build_direct_usb_add_applet_plan_from_image(_parse_hex_bytes(args.image_hex))
        for step in plan:
            print(f"{step.kind}: {step.packet.hex(' ')}")
        return 0

    if args.command == "smartapplet-header":
        header = parse_smartapplet_header(_parse_hex_bytes(args.header_hex))
        argument, trailing = derive_add_applet_start_fields(header)
        print(
            f"magic=0x{header.magic:08x} "
            f"file_size=0x{header.file_size:08x} "
            f"base_memory_size=0x{header.base_memory_size:08x} "
            f"extra_memory_size=0x{header.extra_memory_size:08x} "
            f"argument=0x{argument:08x} "
            f"trailing=0x{trailing:04x}"
        )
        return 0

    if args.command == "smartapplet-metadata":
        metadata = parse_smartapplet_metadata(_parse_hex_bytes(args.header_hex))
        print(
            f"applet_id=0x{metadata.applet_id:04x} "
            f"version={metadata.version_major}.{metadata.version_minor} "
            f"name={metadata.name} "
            f"info_table_offset=0x{metadata.info_table_offset:08x} "
            f"applet_class=0x{metadata.applet_class:02x} "
            f"extra_memory_size=0x{metadata.extra_memory_size:08x}"
        )
        return 0

    if args.command == "smartapplet-string":
        print(resolve_known_smartapplet_string(int(args.resource_id, 0)))
        return 0

    if args.command == "smartapplet-menu":
        for item in get_known_smartapplet_menu(int(args.resource_id, 0)):
            print(f"command_id=0x{item.command_id:04x} label={item.label}")
        return 0

    if args.command == "decode-updater-response":
        response = parse_updater_response(_parse_hex_bytes(args.raw_hex))
        print(
            f"status=0x{response.status:02x} "
            f"argument=0x{response.argument:08x} "
            f"trailing=0x{response.trailing:04x}"
        )
        return 0

    if args.command == "parse-alphaword-attributes":
        attributes = parse_file_attributes_record(_parse_hex_bytes(args.raw_hex))
        print(
            f"name={attributes.name} "
            f"reserved_length=0x{attributes.reserved_length:08x} "
            f"file_length=0x{attributes.file_length:08x} "
            f"trailing={attributes.trailing_bytes.hex(' ')}"
        )
        return 0

    raise AssertionError(f"unhandled command: {args.command}")
