import argparse

from neotools.alphaword_flow import (
    build_direct_usb_full_text_retrieval_plan,
    build_full_text_retrieval_plan,
)
from neotools.alphaword_attributes import parse_file_attributes_record
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
            f"value_0x18=0x{attributes.value_0x18:08x} "
            f"file_length=0x{attributes.file_length:08x}"
        )
        return 0

    raise AssertionError(f"unhandled command: {args.command}")
