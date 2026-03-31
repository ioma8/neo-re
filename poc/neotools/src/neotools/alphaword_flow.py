from dataclasses import dataclass

from neotools.switch_packets import build_reset_preamble, build_switch_packet
from neotools.updater_packets import (
    build_raw_file_attributes_command,
    build_retrieve_file_command,
    build_updater_command,
)


@dataclass(frozen=True)
class UpdaterStep:
    kind: str
    packet: bytes


def _build_list_applets_command(*, page_offset: int = 0, page_size: int = 7) -> bytes:
    return build_updater_command(command=0x04, argument=page_offset, trailing=page_size)


def _build_retrieve_chunk_command() -> bytes:
    return build_updater_command(command=0x10, argument=0, trailing=0)


def build_full_text_retrieval_plan(*, applet_id: int, file_slot: int) -> list[UpdaterStep]:
    return [
        UpdaterStep("list_applets", _build_list_applets_command()),
        UpdaterStep(
            "raw_file_attributes",
            build_raw_file_attributes_command(file_slot=file_slot, applet_id=applet_id),
        ),
        UpdaterStep(
            "retrieve_file",
            build_retrieve_file_command(
                file_slot=file_slot,
                applet_id=applet_id,
                requested_length=0x80000,
                alternate_mode=False,
            ),
        ),
        UpdaterStep("retrieve_chunk", _build_retrieve_chunk_command()),
    ]


def build_preview_retrieval_plan(*, applet_id: int, file_slot: int) -> list[UpdaterStep]:
    return [
        UpdaterStep("list_applets", _build_list_applets_command()),
        UpdaterStep(
            "raw_file_attributes",
            build_raw_file_attributes_command(file_slot=file_slot, applet_id=applet_id),
        ),
        UpdaterStep(
            "retrieve_file",
            build_retrieve_file_command(
                file_slot=file_slot,
                applet_id=applet_id,
                requested_length=0xB4,
                alternate_mode=False,
            ),
        ),
        UpdaterStep("retrieve_chunk", _build_retrieve_chunk_command()),
    ]


def build_direct_usb_full_text_retrieval_plan(*, applet_id: int, file_slot: int) -> list[UpdaterStep]:
    return [
        UpdaterStep("reset_connection", build_reset_preamble()),
        UpdaterStep("switch_to_updater", build_switch_packet(0)),
        *build_full_text_retrieval_plan(applet_id=applet_id, file_slot=file_slot),
    ]
