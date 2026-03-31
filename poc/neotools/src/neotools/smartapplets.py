from dataclasses import dataclass

from neotools.alphaword_flow import UpdaterStep
from neotools.switch_packets import build_reset_preamble, build_switch_packet
from neotools.updater_packets import build_updater_command


@dataclass(frozen=True)
class SmartAppletHeader:
    magic: int
    file_size: int
    base_memory_size: int
    payload_or_code_size: int
    flags_and_version: int
    applet_id_and_version: int
    extra_memory_size: int


def _checksum16(payload: bytes) -> int:
    return sum(payload) & 0xFFFF


def _read_be32(raw: bytes, offset: int) -> int:
    return int.from_bytes(raw[offset : offset + 4], byteorder="big")


def parse_smartapplet_header(raw: bytes) -> SmartAppletHeader:
    if len(raw) != 0x84:
        raise ValueError("SmartApplet header must be exactly 0x84 bytes")

    return SmartAppletHeader(
        magic=_read_be32(raw, 0x00),
        file_size=_read_be32(raw, 0x04),
        base_memory_size=_read_be32(raw, 0x08),
        payload_or_code_size=_read_be32(raw, 0x0C),
        flags_and_version=_read_be32(raw, 0x10),
        applet_id_and_version=_read_be32(raw, 0x14),
        extra_memory_size=_read_be32(raw, 0x80),
    )


def derive_add_applet_start_fields(header: SmartAppletHeader) -> tuple[int, int]:
    combined_memory_size = (header.base_memory_size + header.extra_memory_size) & 0xFFFFFFFF
    argument = (header.file_size | ((combined_memory_size & 0xFFFF0000) << 8)) & 0xFFFFFFFF
    trailing = combined_memory_size & 0xFFFF
    return argument, trailing


def build_list_applets_command(*, page_offset: int = 0, page_size: int = 7) -> bytes:
    return build_updater_command(command=0x04, argument=page_offset, trailing=page_size)


def build_retrieve_applet_command(*, applet_id: int) -> bytes:
    return build_updater_command(command=0x0F, argument=0, trailing=applet_id)


def build_retrieve_chunk_command() -> bytes:
    return build_updater_command(command=0x10, argument=0, trailing=0)


def build_add_applet_begin_command(*, argument: int, trailing: int) -> bytes:
    return build_updater_command(command=0x06, argument=argument, trailing=trailing)


def build_program_applet_command() -> bytes:
    return build_updater_command(command=0x0B, argument=0, trailing=0)


def build_finalize_applet_update_command() -> bytes:
    return build_updater_command(command=0x07, argument=0, trailing=0)


def build_direct_usb_retrieve_applet_plan(*, applet_id: int) -> list[UpdaterStep]:
    return [
        UpdaterStep("reset_connection", build_reset_preamble()),
        UpdaterStep("switch_to_updater", build_switch_packet(0)),
        UpdaterStep("retrieve_applet", build_retrieve_applet_command(applet_id=applet_id)),
        UpdaterStep("retrieve_chunk", build_retrieve_chunk_command()),
    ]


def build_direct_usb_add_applet_plan_from_image(image: bytes) -> list[UpdaterStep]:
    header = parse_smartapplet_header(image[:0x84])
    argument, trailing = derive_add_applet_start_fields(header)
    return build_direct_usb_add_applet_plan(start_argument=argument, trailing=trailing, payload=image)


def build_direct_usb_add_applet_plan(*, start_argument: int, trailing: int, payload: bytes) -> list[UpdaterStep]:
    return [
        UpdaterStep("reset_connection", build_reset_preamble()),
        UpdaterStep("switch_to_updater", build_switch_packet(0)),
        UpdaterStep(
            "add_applet_begin",
            build_add_applet_begin_command(argument=start_argument, trailing=trailing),
        ),
        UpdaterStep(
            "add_applet_chunk_handshake",
            build_updater_command(command=0x02, argument=len(payload), trailing=_checksum16(payload)),
        ),
        UpdaterStep("add_applet_chunk_data", payload),
        UpdaterStep("add_applet_chunk_commit", build_updater_command(command=0xFF, argument=0, trailing=0)),
        UpdaterStep("program_applet", build_program_applet_command()),
        UpdaterStep("finalize_applet_update", build_finalize_applet_update_command()),
    ]
