import argparse

from neotools.asusbcomm import (
    build_hid_fallback_init_plan,
    build_get_mac_address_packet,
    build_set_mac_address_packet,
    classify_alpha_smart_presence,
)
from neotools.driver64_model import (
    build_driver64_cancel_active_transfer_request,
    build_driver64_config_descriptor_full_request,
    build_driver64_config_descriptor_header_request,
    build_driver64_data_transfer_request,
    build_driver64_device_descriptor_request,
    build_driver64_dispatch_map,
    build_driver64_endpoint_trigger_request,
    build_driver64_probe_sequence_plan,
    describe_driver64_internal_ioctl,
    classify_driver64_create,
    classify_driver64_device_control,
    classify_driver64_pnp_minor,
    classify_driver64_read_write,
)
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
from neotools.os3kapp_format import parse_os3kapp_image
from neotools.os3kapp_runtime import (
    build_os3kapp_entry_abi,
    describe_known_applet_command_prototype,
    describe_known_applet_payload_subcommand_prototype,
    describe_known_trap_prototype,
    decompose_os3kapp_command,
    scan_os3kapp_trap_blocks,
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


def _parse_bool(raw: str) -> bool:
    value = raw.strip().lower()
    if value == "true":
        return True
    if value == "false":
        return False
    raise ValueError(f"unsupported boolean literal: {raw}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="neotools")
    subparsers = parser.add_subparsers(dest="command", required=True)

    descriptor_parser = subparsers.add_parser("descriptor")
    descriptor_parser.add_argument("raw_hex")

    switch_packet_parser = subparsers.add_parser("switch-packet")
    switch_packet_parser.add_argument("applet_id")

    asusbcomm_presence_parser = subparsers.add_parser("asusbcomm-presence")
    asusbcomm_presence_parser.add_argument("raw_hex")

    asusbcomm_set_mac_parser = subparsers.add_parser("asusbcomm-set-mac-packet")
    asusbcomm_set_mac_parser.add_argument("source_hex")

    subparsers.add_parser("asusbcomm-get-mac-packet")

    asusbcomm_hid_fallback_parser = subparsers.add_parser("asusbcomm-hid-fallback-plan")
    asusbcomm_hid_fallback_parser.add_argument("os_major_version")

    subparsers.add_parser("driver64-dispatch-map")

    driver64_ioctl_route_parser = subparsers.add_parser("driver64-ioctl-route")
    driver64_ioctl_route_parser.add_argument("ioctl")

    driver64_create_route_parser = subparsers.add_parser("driver64-create-route")
    driver64_create_route_parser.add_argument("--state", required=True)
    driver64_create_route_parser.add_argument("--has-configuration", required=True)
    driver64_create_route_parser.add_argument("--file-name-suffix")

    driver64_read_write_route_parser = subparsers.add_parser("driver64-read-write-route")
    driver64_read_write_route_parser.add_argument("--major-function", required=True)
    driver64_read_write_route_parser.add_argument("--state", required=True)
    driver64_read_write_route_parser.add_argument("--transfer-length", required=True)
    driver64_read_write_route_parser.add_argument("--file-context-present", required=True)
    driver64_read_write_route_parser.add_argument("--endpoint-type", required=True)

    driver64_pnp_route_parser = subparsers.add_parser("driver64-pnp-route")
    driver64_pnp_route_parser.add_argument("minor")

    driver64_internal_request_parser = subparsers.add_parser("driver64-internal-request")
    driver64_internal_request_parser.add_argument(
        "kind",
        choices=[
            "device-descriptor",
            "config-header",
            "config-full",
            "endpoint-trigger",
            "data-transfer",
            "cancel-transfer",
        ],
    )
    driver64_internal_request_parser.add_argument("--total-length")
    driver64_internal_request_parser.add_argument("--direction")
    driver64_internal_request_parser.add_argument("--chunk-length")

    driver64_probe_sequence_parser = subparsers.add_parser("driver64-probe-sequence")
    driver64_probe_sequence_parser.add_argument("flags")

    driver64_internal_ioctl_name_parser = subparsers.add_parser("driver64-internal-ioctl-name")
    driver64_internal_ioctl_name_parser.add_argument("ioctl")

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

    os3kapp_image_parser = subparsers.add_parser("os3kapp-image")
    os3kapp_image_parser.add_argument("image_hex")

    os3kapp_entry_abi_parser = subparsers.add_parser("os3kapp-entry-abi")
    os3kapp_entry_abi_parser.add_argument("image_hex")

    os3kapp_command_parser = subparsers.add_parser("os3kapp-command")
    os3kapp_command_parser.add_argument("raw_command")

    os3kapp_traps_parser = subparsers.add_parser("os3kapp-traps")
    os3kapp_traps_parser.add_argument("image_hex")

    os3kapp_trap_prototype_parser = subparsers.add_parser("os3kapp-trap-prototype")
    os3kapp_trap_prototype_parser.add_argument("opcode")

    os3kapp_applet_command_parser = subparsers.add_parser("os3kapp-applet-command")
    os3kapp_applet_command_parser.add_argument("applet_name")
    os3kapp_applet_command_parser.add_argument("raw_command")

    os3kapp_payload_subcommand_parser = subparsers.add_parser("os3kapp-payload-subcommand")
    os3kapp_payload_subcommand_parser.add_argument("applet_name")
    os3kapp_payload_subcommand_parser.add_argument("parent_command")
    os3kapp_payload_subcommand_parser.add_argument("first_input_byte")

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

    if args.command == "asusbcomm-presence":
        result = classify_alpha_smart_presence(_parse_hex_bytes(args.raw_hex))
        print(
            f"descriptor_valid={result.descriptor_valid} "
            f"cached_mode={result.cached_mode} "
            f"return_code={result.return_code}"
        )
        return 0

    if args.command == "asusbcomm-set-mac-packet":
        packet = build_set_mac_address_packet(_parse_hex_bytes(args.source_hex))
        print(packet.hex(" "))
        return 0

    if args.command == "asusbcomm-get-mac-packet":
        print(build_get_mac_address_packet().hex(" "))
        return 0

    if args.command == "asusbcomm-hid-fallback-plan":
        plan = build_hid_fallback_init_plan(os_major_version=int(args.os_major_version, 0))
        for step in plan:
            if step.kind == "sleep_ms":
                print(f"{step.kind} value={step.value}")
            elif step.code is not None:
                print(f"{step.kind} code=0x{step.code:08x} payload={step.payload.hex(' ')}")
            else:
                print(f"{step.kind} payload={step.payload.hex(' ')}")
        return 0

    if args.command == "driver64-dispatch-map":
        dispatch_map = build_driver64_dispatch_map()
        print(f"create: 0x{dispatch_map.create:08x}")
        print(f"close: 0x{dispatch_map.close:08x}")
        print(f"device_control: 0x{dispatch_map.device_control:08x}")
        print(f"pnp: 0x{dispatch_map.pnp:08x}")
        print(f"power: 0x{dispatch_map.power:08x}")
        print(f"system_control: 0x{dispatch_map.system_control:08x}")
        print(f"unload: 0x{dispatch_map.unload:08x}")
        return 0

    if args.command == "driver64-ioctl-route":
        route = classify_driver64_device_control(int(args.ioctl, 0))
        if route.internal_plan is None:
            print(f"kind={route.kind} ioctl=0x{route.ioctl:08x}")
        else:
            print(
                f"kind={route.kind} ioctl=0x{route.ioctl:08x} "
                f"first=0x{route.internal_plan.first:08x} "
                f"second=0x{route.internal_plan.second:08x}"
            )
        return 0

    if args.command == "driver64-create-route":
        suffix = None if args.file_name_suffix is None else int(args.file_name_suffix, 0)
        route = classify_driver64_create(
            state=int(args.state, 0),
            has_configuration=_parse_bool(args.has_configuration),
            file_name_suffix=suffix,
        )
        endpoint_index = "none" if route.endpoint_index is None else str(route.endpoint_index)
        print(
            f"kind={route.kind} ntstatus=0x{route.ntstatus:08x} "
            f"open_count={route.increments_open_count} "
            f"cancel_timer={route.cancels_timer_if_active} "
            f"endpoint_index={endpoint_index}"
        )
        return 0

    if args.command == "driver64-read-write-route":
        route = classify_driver64_read_write(
            major_function=int(args.major_function, 0),
            state=int(args.state, 0),
            transfer_length=int(args.transfer_length, 0),
            file_context_present=_parse_bool(args.file_context_present),
            endpoint_type=int(args.endpoint_type, 0),
        )
        print(
            f"kind={route.kind} ntstatus=0x{route.ntstatus:08x} "
            f"direction={route.direction} transfer_code={route.transfer_code} "
            f"first_chunk=0x{route.first_chunk_length:x} remaining=0x{route.remaining_length:x} "
            f"ioctl=0x{route.uses_ioctl:08x} probe_fallback={route.falls_back_to_probe_sequence}"
        )
        return 0

    if args.command == "driver64-pnp-route":
        route = classify_driver64_pnp_minor(int(args.minor, 0))
        print(f"kind={route.kind} minor=0x{route.minor:02x} handler={route.handler}")
        return 0

    if args.command == "driver64-internal-request":
        if args.kind == "device-descriptor":
            request = build_driver64_device_descriptor_request()
        elif args.kind == "config-header":
            request = build_driver64_config_descriptor_header_request()
        elif args.kind == "config-full":
            request = build_driver64_config_descriptor_full_request(int(args.total_length, 0))
        elif args.kind == "data-transfer":
            request = build_driver64_data_transfer_request(
                chunk_length=int(args.chunk_length, 0),
                direction=args.direction,
            )
        elif args.kind == "cancel-transfer":
            request = build_driver64_cancel_active_transfer_request()
        else:
            request = build_driver64_endpoint_trigger_request()
        request_type = "none" if request.request_type is None else str(request.request_type)
        response_offset = (
            "none"
            if request.response_buffer_pointer_offset is None
            else f"0x{request.response_buffer_pointer_offset:02x}"
        )
        print(
            f"size=0x{request.size:x} function=0x{request.function:02x} "
            f"buffer_length=0x{request.transfer_buffer_length:x} request_type={request_type} "
            f"endpoint_offset=0x{request.endpoint_pointer_offset:02x} "
            f"response_buffer_offset={response_offset}"
        )
        return 0

    if args.command == "driver64-probe-sequence":
        plan = build_driver64_probe_sequence_plan(int(args.flags, 0))
        second = "none" if plan.second_ioctl is None else f"0x{plan.second_ioctl:08x}"
        print(f"first=0x{plan.first_ioctl:08x} second={second} flags=0x{plan.flags:08x}")
        return 0

    if args.command == "driver64-internal-ioctl-name":
        print(describe_driver64_internal_ioctl(int(args.ioctl, 0)))
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

    if args.command == "os3kapp-image":
        image = parse_os3kapp_image(_parse_hex_bytes(args.image_hex))
        print(
            f"file_size=0x{image.metadata.header.file_size:08x} "
            f"applet_id=0x{image.metadata.applet_id:04x} "
            f"applet_class=0x{image.metadata.applet_class:02x} "
            f"body_size=0x{image.metadata.header.file_size - image.header_size:x} "
            f"info_table_offset=0x{image.info_table_offset:08x}"
        )
        print(
            "body_prefix_words="
            + ",".join(f"0x{word:08x}" for word in image.body_prefix_words)
        )
        print(f"info_records={len(image.info_records)}")
        return 0

    if args.command == "os3kapp-entry-abi":
        abi = build_os3kapp_entry_abi(parse_os3kapp_image(_parse_hex_bytes(args.image_hex)))
        print(
            f"entry_offset=0x{abi.entry_offset:08x} "
            f"loader_stub_length=0x{abi.loader_stub_length:x} "
            f"init_opcode=0x{abi.init_opcode:02x} "
            f"shutdown_opcode=0x{abi.shutdown_opcode:02x} "
            f"shutdown_status=0x{abi.shutdown_status:08x}"
        )
        print(
            f"call_block_words={abi.call_block_words} "
            f"input_length_index={abi.input_length_index} "
            f"input_pointer_index={abi.input_pointer_index} "
            f"output_capacity_index={abi.output_capacity_index} "
            f"output_length_index={abi.output_length_index} "
            f"output_buffer_pointer_index={abi.output_buffer_pointer_index}"
        )
        return 0

    if args.command == "os3kapp-command":
        command = decompose_os3kapp_command(int(args.raw_command, 0))
        lifecycle_name = "none" if command.lifecycle_name is None else command.lifecycle_name
        print(
            f"raw=0x{command.raw:08x} "
            f"namespace_byte=0x{command.namespace_byte:02x} "
            f"selector_byte=0x{command.selector_byte:02x} "
            f"low_word=0x{command.low_word:04x} "
            f"custom_dispatch={command.uses_custom_dispatch} "
            f"lifecycle={lifecycle_name}"
        )
        return 0

    if args.command == "os3kapp-traps":
        image = parse_os3kapp_image(_parse_hex_bytes(args.image_hex))
        for block in scan_os3kapp_trap_blocks(image):
            print(
                f"block=0x{block.start_file_offset:04x}-0x{block.end_file_offset:04x} "
                f"count={len(block.stubs)} "
                f"first=0x{block.stubs[0].opcode:04x} "
                f"last=0x{block.stubs[-1].opcode:04x}"
            )
            for stub in block.stubs:
                inferred_name = "unknown" if stub.inferred_name is None else stub.inferred_name
                print(
                    f"offset=0x{stub.file_offset:04x} "
                    f"opcode=0x{stub.opcode:04x} "
                    f"family=0x{stub.family_byte:02x} "
                    f"selector=0x{stub.selector_byte:02x} "
                    f"name={inferred_name}"
                )
        return 0

    if args.command == "os3kapp-trap-prototype":
        prototype = describe_known_trap_prototype(int(args.opcode, 0))
        print(
            f"opcode=0x{prototype.opcode:04x} "
            f"name={prototype.name} "
            f"stack_argument_count={prototype.stack_argument_count} "
            f"return_kind={prototype.return_kind}"
        )
        print(f"notes={prototype.notes}")
        return 0

    if args.command == "os3kapp-applet-command":
        prototype = describe_known_applet_command_prototype(
            args.applet_name,
            int(args.raw_command, 0),
        )
        print(
            f"applet={prototype.applet_name} "
            f"raw_command=0x{prototype.raw_command:05x} "
            f"selector_byte=0x{prototype.selector_byte:02x} "
            f"handler={prototype.handler_name} "
            f"status_code=0x{prototype.status_code:08x} "
            f"response_word_count={prototype.response_word_count}"
        )
        print(f"notes={prototype.notes}")
        return 0

    if args.command == "os3kapp-payload-subcommand":
        prototype = describe_known_applet_payload_subcommand_prototype(
            args.applet_name,
            int(args.parent_command, 0),
            int(args.first_input_byte, 0),
        )
        status_code = "conditional" if prototype.status_code is None else f"0x{prototype.status_code:08x}"
        print(
            f"applet={prototype.applet_name} "
            f"parent_command=0x{prototype.parent_command:05x} "
            f"first_input_byte=0x{prototype.first_input_byte:02x} "
            f"status_code={status_code} "
            f"response_length={prototype.response_length}"
        )
        print(f"notes={prototype.notes}")
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
