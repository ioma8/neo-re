from collections.abc import Iterable
from dataclasses import dataclass

from neotools.alphaword_flow import (
    build_full_text_retrieval_plan,
    build_preview_retrieval_plan,
)
from neotools.updater_packets import build_retrieve_file_command, build_updater_command
from neotools.switch_packets import build_reset_preamble, build_switch_packet


@dataclass(frozen=True)
class GetPrintFlowStep:
    kind: str
    detail: str | None = None
    file_slot: int | None = None
    packet: bytes | None = None
    status: int | None = None


@dataclass(frozen=True)
class GetPrintFlowResult:
    steps: list[GetPrintFlowStep]
    return_code: int


@dataclass(frozen=True)
class UpdaterErrorLogEntry:
    message: str
    operation: str
    source: str
    reserved: int
    source_line: int
    error_code: int
    response_byte: int
    detail: str | None


def classify_retrieve_opcode(opcode: int) -> str:
    if opcode == 0x12:
        return "interactive_retrieve"
    if opcode == 0x1C:
        return "save_archive_retrieve"
    raise ValueError(f"unknown retrieve opcode: 0x{opcode:02x}")


def make_updater_error_log_entry(
    *,
    message: str,
    operation: str,
    source: str,
    source_line: int,
    error_code: int,
    response_byte: int,
    detail: str | None,
) -> UpdaterErrorLogEntry:
    return UpdaterErrorLogEntry(
        message=message,
        operation=operation,
        source=source,
        reserved=0,
        source_line=source_line,
        error_code=error_code,
        response_byte=response_byte,
        detail=detail,
    )


def get_retrieved_text_replacements() -> dict[str, str]:
    return {
        "cr": "\r",
        "crlf": "\r\n",
        "space": " ",
    }


def _packet_step(kind: str, packet: bytes, *, file_slot: int) -> GetPrintFlowStep:
    return GetPrintFlowStep(kind=kind, packet=packet, file_slot=file_slot)


def _bootstrap_steps() -> list[GetPrintFlowStep]:
    return [
        GetPrintFlowStep(kind="reset_connection", packet=build_reset_preamble()),
        GetPrintFlowStep(kind="switch_to_updater", packet=build_switch_packet(0)),
    ]


def build_alpha_word_tree_notification_flow(
    *, notification_code: int, tree_action: int | None = None
) -> list[GetPrintFlowStep]:
    flow = [
        GetPrintFlowStep(kind="controller", detail="HandleAlphaWordTreeNotifications"),
        GetPrintFlowStep(kind="tree_control_id", status=0x8A8A),
        GetPrintFlowStep(kind="tree_notification_code", status=notification_code),
    ]

    if notification_code == -2:
        flow.append(GetPrintFlowStep(kind="dispatch", detail="ToggleAlphaWordTreeCheckByMouse"))
        return flow
    if notification_code == -3:
        flow.append(GetPrintFlowStep(kind="dispatch", detail="HandleAlphaWordTreeDoubleClick"))
        flow.append(GetPrintFlowStep(kind="dispatch", detail="OpenSelectedAlphaWordSlotDialog"))
        return flow
    if notification_code == -0x19C:
        flow.append(GetPrintFlowStep(kind="dispatch", detail="ToggleAlphaWordTreeCheckByKeyboard"))
        return flow
    if notification_code == -0x192:
        flow.append(GetPrintFlowStep(kind="dispatch", detail="UpdateAlphaWordButtonsForTreeSelection"))
        return flow
    if notification_code == -0x195:
        flow.append(GetPrintFlowStep(kind="dispatch", detail="HandleAlphaWordTreeExpandStateChange"))
        if tree_action is not None:
            flow.append(GetPrintFlowStep(kind="tree_action", status=tree_action))
            if tree_action == 2:
                flow.append(GetPrintFlowStep(kind="dispatch", detail="RefreshAlphaWordPreviewCacheForTreeItem"))
        return flow

    raise ValueError(f"unsupported AlphaWord tree notification code: {notification_code}")


def build_direct_usb_preview_refresh_flow(
    *, applet_id: int, file_slots: Iterable[int] = range(1, 9)
) -> list[GetPrintFlowStep]:
    flow = [
        GetPrintFlowStep(kind="controller", detail="HandleAlphaWordTreeExpandStateChange"),
        GetPrintFlowStep(kind="controller", detail="RefreshAlphaWordPreviewCacheForTreeItem"),
    ]
    flow.extend(_bootstrap_steps())
    for file_slot in file_slots:
        for step in build_preview_retrieval_plan(applet_id=applet_id, file_slot=file_slot):
            flow.append(_packet_step(step.kind, step.packet, file_slot=file_slot))
        flow.append(GetPrintFlowStep(kind="cache_preview_text", file_slot=file_slot, status=1))
        flow.append(GetPrintFlowStep(kind="cache_preview_size", file_slot=file_slot, status=1))
    return flow


def build_direct_usb_full_get_print_flow(
    *,
    applet_id: int,
    file_slots: Iterable[int] = range(1, 9),
    selected_only: bool = False,
    single_slot: bool = False,
) -> list[GetPrintFlowStep]:
    if selected_only and single_slot:
        raise ValueError("selected_only and single_slot are mutually exclusive")

    if single_slot:
        controller = "RetrieveSingleAlphaWordSlotForDevice"
    elif selected_only:
        controller = "RetrieveSelectedAlphaWordSlotsForDevice"
    else:
        controller = "RetrieveAllAlphaWordSlotsForDevice"

    flow = [
        GetPrintFlowStep(kind="controller", detail="ExecuteGetPrintAlphaWordFlow"),
        GetPrintFlowStep(kind="controller", detail=controller),
    ]
    flow.extend(_bootstrap_steps())
    for file_slot in file_slots:
        for step in build_full_text_retrieval_plan(applet_id=applet_id, file_slot=file_slot):
            flow.append(_packet_step(step.kind, step.packet, file_slot=file_slot))
        flow.append(GetPrintFlowStep(kind="get_file_name", file_slot=file_slot))
        flow.append(GetPrintFlowStep(kind="cache_full_text", file_slot=file_slot, status=2))
        flow.append(GetPrintFlowStep(kind="cache_full_size", file_slot=file_slot, status=2))
        flow.append(GetPrintFlowStep(kind="cache_file_name", file_slot=file_slot, status=2))
    return flow


def build_retrieve_all_alpha_word_slots_for_device_flow(
    *,
    applet_id: int,
    file_slots: Iterable[int] = range(1, 9),
    initial_statuses: dict[int, int] | None = None,
    fail_slot: int | None = None,
    cancel_after_slot: int | None = None,
    device_ref_valid: bool = True,
) -> GetPrintFlowResult:
    flow = [
        GetPrintFlowStep(kind="controller", detail="RetrieveAllAlphaWordSlotsForDevice"),
        GetPrintFlowStep(kind="load_global_empty_string"),
        GetPrintFlowStep(kind="prepare_retrieval_loop"),
    ]
    if not device_ref_valid:
        flow.append(GetPrintFlowStep(kind="return_invalid_device", status=0x51))
        return GetPrintFlowResult(steps=flow, return_code=0x51)

    statuses = dict(initial_statuses or {})
    for zero_based_index, file_slot in enumerate(file_slots):
        slot_index = zero_based_index
        current_status = statuses.get(file_slot, 0)
        flow.append(GetPrintFlowStep(kind="format_progress_text", file_slot=file_slot))
        flow.append(GetPrintFlowStep(kind="set_progress_dialog_text", file_slot=file_slot))
        flow.append(GetPrintFlowStep(kind="read_slot_status", file_slot=file_slot, status=current_status))
        if current_status == 2:
            flow.append(GetPrintFlowStep(kind="skip_slot_already_loaded", file_slot=file_slot, status=2))
            continue

        flow.append(GetPrintFlowStep(kind="remember_previous_status", file_slot=file_slot, status=current_status))
        for step in build_full_text_retrieval_plan(applet_id=applet_id, file_slot=file_slot):
            flow.append(_packet_step(step.kind, step.packet, file_slot=file_slot))

        if fail_slot == file_slot:
            flow.append(
                GetPrintFlowStep(
                    kind="restore_previous_status_on_error",
                    file_slot=file_slot,
                    status=current_status,
                )
            )
            return GetPrintFlowResult(steps=flow, return_code=0x2A)

        flow.append(GetPrintFlowStep(kind="cache_full_text", file_slot=file_slot, status=slot_index))
        flow.append(GetPrintFlowStep(kind="cache_full_size", file_slot=file_slot, status=slot_index))
        flow.append(GetPrintFlowStep(kind="get_file_name", file_slot=file_slot))
        flow.append(GetPrintFlowStep(kind="cache_file_name", file_slot=file_slot, status=slot_index))
        flow.append(GetPrintFlowStep(kind="set_slot_status_loaded", file_slot=file_slot, status=2))
        statuses[file_slot] = 2

        cancel_code = 0x33 if cancel_after_slot == file_slot else 0
        flow.append(
            GetPrintFlowStep(
                kind="read_progress_dialog_cancel_code",
                file_slot=file_slot,
                status=cancel_code,
            )
        )
        if cancel_code == 0x33:
            flow.append(GetPrintFlowStep(kind="return_cancelled", file_slot=file_slot, status=0x29))
            return GetPrintFlowResult(steps=flow, return_code=0x29)

    return GetPrintFlowResult(steps=flow, return_code=0)


def build_retrieve_full_alpha_word_text_flow(
    *,
    transport_mode: int,
    applet_id: int,
    file_slot: int,
    retrieved_length: int,
    alternate_selector: int = 0,
    temp_open_success: bool = True,
) -> GetPrintFlowResult:
    flow = [
        GetPrintFlowStep(kind="controller", detail="RetrieveFullAlphaWordText"),
        GetPrintFlowStep(kind="dispatch_thread_pump"),
        GetPrintFlowStep(kind="reset_retrieved_text_workspace"),
        GetPrintFlowStep(kind="open_temporary_retrieved_text_file"),
        GetPrintFlowStep(kind="build_retrieved_text_temp_path_base"),
    ]
    if temp_open_success:
        flow.extend(
            [
                GetPrintFlowStep(kind="open_retrieved_text_sink_for_write"),
                GetPrintFlowStep(kind="seed_retrieved_text_sink"),
                GetPrintFlowStep(kind="reopen_retrieved_text_sink_for_read"),
            ]
        )

    if temp_open_success and transport_mode in (2, 5):
        flow.append(GetPrintFlowStep(kind="transport", detail="DirectUsbRetrieveAppletFileData"))
        for step in build_full_text_retrieval_plan(applet_id=applet_id, file_slot=file_slot):
            flow.append(_packet_step(step.kind, step.packet, file_slot=file_slot))
        flow.append(GetPrintFlowStep(kind="close_file_descriptor"))
        flow.extend(
            [
                GetPrintFlowStep(kind="load_retrieved_text_file_as_cstring"),
                GetPrintFlowStep(kind="read_retrieved_text_sink", status=retrieved_length),
                GetPrintFlowStep(kind="normalize_retrieved_text_bytes"),
                GetPrintFlowStep(kind="delete_temporary_retrieved_text_file"),
                GetPrintFlowStep(kind="assign_output_cstring"),
            ]
        )
    elif temp_open_success and transport_mode == 3:
        flow.append(
            GetPrintFlowStep(
                kind="transport",
                detail="AlternateTransportRetrieveAppletFileData",
                status=alternate_selector,
            )
        )
        flow.append(GetPrintFlowStep(kind="close_file_descriptor"))
        flow.extend(
            [
                GetPrintFlowStep(kind="load_retrieved_text_file_as_cstring"),
                GetPrintFlowStep(kind="read_retrieved_text_sink", status=retrieved_length),
                GetPrintFlowStep(kind="normalize_retrieved_text_bytes"),
                GetPrintFlowStep(kind="delete_temporary_retrieved_text_file"),
                GetPrintFlowStep(kind="assign_output_cstring"),
            ]
        )

    flow.append(GetPrintFlowStep(kind="finalize_retrieved_text_workspace", status=0))
    return GetPrintFlowResult(steps=flow, return_code=0)


def build_updater_retrieve_applet_file_data_flow(
    *,
    command_selector: int,
    applet_id: int,
    file_slot: int,
    requested_length: int,
    reported_total_length: int,
    chunk_lengths: list[int],
    chunk_checksums: list[int],
    computed_chunk_checksums: list[int] | None = None,
    progress_current: int = 0,
    progress_total: int = 0,
    start_status: int = 0x53,
    chunk_status: int = 0x4D,
) -> GetPrintFlowResult:
    remaining_chunk_lengths = list(chunk_lengths)
    remaining_expected_checksums = list(chunk_checksums)
    computed_checksums = list(computed_chunk_checksums) if computed_chunk_checksums is not None else list(chunk_checksums)
    capped_total = min(reported_total_length, requested_length)
    flow = [
        GetPrintFlowStep(kind="controller", detail="UpdaterRetrieveAppletFileData"),
        GetPrintFlowStep(kind="reset_last_updater_error"),
    ]

    if progress_total == 0:
        flow.append(GetPrintFlowStep(kind="format_progress_caption", file_slot=file_slot))
        flow.append(GetPrintFlowStep(kind="set_progress_percent", status=0))
    else:
        flow.append(
            GetPrintFlowStep(
                kind="set_progress_percent",
                status=(progress_current * 100) // progress_total,
            )
        )

    start_packet = build_retrieve_file_command(
        file_slot=file_slot,
        applet_id=applet_id,
        requested_length=requested_length,
        alternate_mode=command_selector == 0x1C,
    )
    flow.append(GetPrintFlowStep(kind="send_start_command", file_slot=file_slot, packet=start_packet))
    flow.append(GetPrintFlowStep(kind="check_start_status_byte", status=start_status))
    if start_status != 0x53:
        flow.append(GetPrintFlowStep(kind="set_last_updater_error_code", status=0x12A))
        flow.append(GetPrintFlowStep(kind="close_progress_scope"))
        return GetPrintFlowResult(steps=flow, return_code=-1)

    flow.append(GetPrintFlowStep(kind="store_reported_total_length", status=reported_total_length))
    flow.append(GetPrintFlowStep(kind="cap_total_length_to_requested", status=capped_total))

    remaining = capped_total
    while True:
        if remaining == 0:
            if progress_total == 0:
                flow.append(GetPrintFlowStep(kind="set_progress_percent", status=100))
                flow.append(GetPrintFlowStep(kind="close_progress_scope"))
            else:
                flow.append(
                    GetPrintFlowStep(
                        kind="set_progress_percent",
                        status=((progress_current + capped_total) * 100) // progress_total,
                    )
                )
            return GetPrintFlowResult(steps=flow, return_code=0)

        if progress_total == 0:
            progress = ((capped_total - remaining) * 100) // capped_total if capped_total else 100
        else:
            progress = ((progress_current + capped_total - remaining) * 100) // progress_total
        flow.append(GetPrintFlowStep(kind="set_progress_percent", status=progress))

        flow.append(
            GetPrintFlowStep(
                kind="send_chunk_command",
                packet=build_updater_command(command=0x10, argument=0, trailing=0),
            )
        )
        flow.append(GetPrintFlowStep(kind="check_chunk_status_byte", status=chunk_status))
        if chunk_status != 0x4D:
            flow.append(GetPrintFlowStep(kind="set_last_updater_error_code", status=0x12A))
            flow.append(GetPrintFlowStep(kind="close_progress_scope"))
            return GetPrintFlowResult(steps=flow, return_code=-1)

        if not remaining_chunk_lengths:
            flow.append(GetPrintFlowStep(kind="set_last_updater_error_code", status=0x12A))
            flow.append(GetPrintFlowStep(kind="close_progress_scope"))
            return GetPrintFlowResult(steps=flow, return_code=-1)

        chunk_length = remaining_chunk_lengths.pop(0)
        expected_checksum = remaining_expected_checksums.pop(0)
        computed_checksum = computed_checksums.pop(0)
        remaining = max(0, remaining - chunk_length)

        flow.append(GetPrintFlowStep(kind="read_chunk_payload", status=chunk_length))
        flow.append(GetPrintFlowStep(kind="write_chunk_to_sink", status=chunk_length))
        flow.append(GetPrintFlowStep(kind="accumulate_chunk_checksum", status=computed_checksum))
        flow.append(GetPrintFlowStep(kind="verify_chunk_checksum", status=expected_checksum))

        if computed_checksum != expected_checksum:
            flow.append(GetPrintFlowStep(kind="set_last_updater_error_code", status=0x105))
            flow.append(GetPrintFlowStep(kind="close_progress_scope"))
            return GetPrintFlowResult(steps=flow, return_code=-1)
