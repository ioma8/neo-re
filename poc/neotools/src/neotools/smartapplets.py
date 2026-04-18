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
    applet_id_and_header: int
    applet_id: int
    header_version: int
    file_count: int
    extra_memory_size: int


@dataclass(frozen=True)
class SmartAppletMetadata:
    header: SmartAppletHeader
    applet_id: int
    version_major: int
    version_minor: int
    name: str
    copyright: str
    info_table_offset: int
    flags_raw: int
    applet_class: int
    extra_memory_size: int
    has_info_table: bool
    flag_high_0x10: bool
    flag_word_0x00010000: bool
    flag_high_0x40: bool


@dataclass(frozen=True)
class SmartAppletInfoRecord:
    group: int
    key: int
    record_type: int
    payload: bytes
    text: str | None


@dataclass(frozen=True)
class SmartAppletMenuItem:
    command_id: int
    label: str


KNOWN_SMARTAPPLET_STRING_LABELS = {
    0xF138: "Maximum File Size (in characters)",
    0xF139: "Minimum File Size (in characters)",
}


KNOWN_SMARTAPPLET_MENU_RESOURCES = {
    163: (
        SmartAppletMenuItem(command_id=0x800E, label="Startup"),
        SmartAppletMenuItem(command_id=0x800F, label="Startup Lock"),
        SmartAppletMenuItem(command_id=0x8010, label="Remove"),
        SmartAppletMenuItem(command_id=0x8012, label="Get Info"),
        SmartAppletMenuItem(command_id=0x8013, label="Help"),
    ),
    208: (
        SmartAppletMenuItem(command_id=0x801D, label="Startup"),
        SmartAppletMenuItem(command_id=0x801E, label="Startup Lock"),
        SmartAppletMenuItem(command_id=0x801F, label="Get Info"),
        SmartAppletMenuItem(command_id=0x8020, label="Help"),
    ),
    219: (
        SmartAppletMenuItem(command_id=0x8021, label="Undo"),
        SmartAppletMenuItem(command_id=0x8022, label="Cut"),
        SmartAppletMenuItem(command_id=0x8023, label="Copy"),
        SmartAppletMenuItem(command_id=0x8024, label="Paste"),
        SmartAppletMenuItem(command_id=0x8025, label="Delete"),
        SmartAppletMenuItem(command_id=0x8026, label="Select All"),
    ),
}


def _checksum16(payload: bytes) -> int:
    return sum(payload) & 0xFFFF


def _read_be32(raw: bytes, offset: int) -> int:
    return int.from_bytes(raw[offset : offset + 4], byteorder="big")


def _decode_bcd(byte_value: int) -> int:
    return (byte_value & 0x0F) + ((byte_value >> 4) & 0x0F) * 10


def _read_c_string(raw: bytes, offset: int, size: int) -> str:
    return raw[offset : offset + size].split(b"\x00", 1)[0].decode("ascii")


def parse_smartapplet_header(raw: bytes) -> SmartAppletHeader:
    if len(raw) != 0x84:
        raise ValueError("SmartApplet header must be exactly 0x84 bytes")

    return SmartAppletHeader(
        magic=_read_be32(raw, 0x00),
        file_size=_read_be32(raw, 0x04),
        base_memory_size=_read_be32(raw, 0x08),
        payload_or_code_size=_read_be32(raw, 0x0C),
        flags_and_version=_read_be32(raw, 0x10),
        applet_id_and_header=_read_be32(raw, 0x14),
        applet_id=int.from_bytes(raw[0x14:0x16], byteorder="big"),
        header_version=raw[0x16],
        file_count=raw[0x17],
        extra_memory_size=_read_be32(raw, 0x80),
    )


def parse_smartapplet_metadata(raw: bytes) -> SmartAppletMetadata:
    header = parse_smartapplet_header(raw)
    return SmartAppletMetadata(
        header=header,
        applet_id=header.applet_id,
        version_major=_decode_bcd(raw[0x3C]),
        version_minor=_decode_bcd(raw[0x3D]),
        name=_read_c_string(raw, 0x18, 0x28),
        copyright=_read_c_string(raw, 0x40, 0x40),
        info_table_offset=header.payload_or_code_size,
        flags_raw=header.flags_and_version,
        applet_class=raw[0x3F],
        extra_memory_size=header.extra_memory_size,
        has_info_table=header.payload_or_code_size != 0,
        flag_high_0x10=(header.flags_and_version & 0x10000000) != 0,
        flag_word_0x00010000=(header.flags_and_version & 0x00010000) != 0,
        flag_high_0x40=(header.flags_and_version & 0x40000000) != 0,
    )


def parse_smartapplet_info_table(raw: bytes) -> list[SmartAppletInfoRecord]:
    records: list[SmartAppletInfoRecord] = []
    offset = 0

    while offset + 6 <= len(raw):
        group = int.from_bytes(raw[offset : offset + 2], byteorder="big")
        if group == 0:
            break

        key = int.from_bytes(raw[offset + 2 : offset + 4], byteorder="big")
        payload_length = int.from_bytes(raw[offset + 4 : offset + 6], byteorder="big")
        payload_start = offset + 6
        payload_end = payload_start + payload_length
        if payload_end > len(raw):
            raise ValueError("SmartApplet info record extends past end of table")

        payload = raw[payload_start:payload_end]
        try:
            text = payload.rstrip(b"\x00").decode("ascii")
        except UnicodeDecodeError:
            text = None
        records.append(
            SmartAppletInfoRecord(
                group=group,
                key=key,
                record_type=group,
                payload=payload,
                text=text,
            )
        )
        offset = payload_end + (payload_length & 1)

    return records


def resolve_known_smartapplet_string(resource_id: int) -> str:
    return KNOWN_SMARTAPPLET_STRING_LABELS[resource_id]


def get_known_smartapplet_menu(resource_id: int) -> tuple[SmartAppletMenuItem, ...]:
    return KNOWN_SMARTAPPLET_MENU_RESOURCES[resource_id]


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
