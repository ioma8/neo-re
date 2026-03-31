from dataclasses import dataclass

from neotools.smartapplets import SmartAppletInfoRecord, SmartAppletMetadata, parse_smartapplet_info_table, parse_smartapplet_metadata


@dataclass(frozen=True)
class Os3kAppHeaderFields:
    magic: int
    base_memory_size: int
    flags_raw: int
    applet_id_and_version: int
    name: str
    version_major_bcd: int
    version_minor_bcd: int
    version_build_byte: int
    applet_class: int
    copyright: str
    extra_memory_size: int


@dataclass(frozen=True)
class Os3kAppImage:
    metadata: SmartAppletMetadata
    header_size: int
    body_prefix_words: tuple[int, int, int, int]
    payload: bytes
    entry_offset: int
    loader_stub: bytes
    body: bytes
    info_table_offset: int
    info_table_bytes: bytes
    info_records: tuple[SmartAppletInfoRecord, ...]


def build_os3kapp_image(
    *,
    header_fields: Os3kAppHeaderFields,
    payload: bytes,
    info_table_bytes: bytes = b"",
    info_table_offset: int | None = None,
) -> bytes:
    if info_table_offset is None:
        info_table_offset = 0 if not info_table_bytes else 0x84 + len(payload)

    file_size = 0x84 + len(payload) + len(info_table_bytes)
    if info_table_offset != 0 and info_table_offset != 0x84 + len(payload):
        raise ValueError("OS3KApp info table offset must either be zero or start immediately after the payload")

    header = bytearray(0x84)
    header[0x00:0x04] = header_fields.magic.to_bytes(4, "big")
    header[0x04:0x08] = file_size.to_bytes(4, "big")
    header[0x08:0x0C] = header_fields.base_memory_size.to_bytes(4, "big")
    header[0x0C:0x10] = info_table_offset.to_bytes(4, "big")
    header[0x10:0x14] = header_fields.flags_raw.to_bytes(4, "big")
    header[0x14:0x18] = header_fields.applet_id_and_version.to_bytes(4, "big")
    header[0x18:0x40] = header_fields.name.encode("ascii")[:0x28].ljust(0x28, b"\x00")
    header[0x3C] = header_fields.version_major_bcd & 0xFF
    header[0x3D] = header_fields.version_minor_bcd & 0xFF
    header[0x3E] = header_fields.version_build_byte & 0xFF
    header[0x3F] = header_fields.applet_class & 0xFF
    header[0x40:0x80] = header_fields.copyright.encode("ascii")[:0x40].ljust(0x40, b"\x00")
    header[0x80:0x84] = header_fields.extra_memory_size.to_bytes(4, "big")
    return bytes(header) + payload + info_table_bytes


def parse_os3kapp_image(raw: bytes) -> Os3kAppImage:
    if len(raw) < 0x94:
        raise ValueError("OS3KApp image is too short")

    metadata = parse_smartapplet_metadata(raw[:0x84])
    header_size = 0x84

    if metadata.header.file_size != len(raw):
        raise ValueError("OS3KApp header file size does not match actual image length")

    info_table_offset = metadata.info_table_offset
    if info_table_offset != 0 and not (header_size <= info_table_offset <= len(raw)):
        raise ValueError("OS3KApp info table offset is outside the image")

    payload = raw[header_size:]
    entry_offset = int.from_bytes(raw[0x84:0x88], "big")
    body_end = info_table_offset if info_table_offset else len(raw)
    body = raw[header_size:body_end]
    info_table_bytes = raw[info_table_offset:] if info_table_offset else b""
    if info_table_bytes:
        try:
            info_records = tuple(parse_smartapplet_info_table(info_table_bytes))
        except ValueError:
            info_records = ()
    else:
        info_records = ()

    return Os3kAppImage(
        metadata=metadata,
        header_size=header_size,
        body_prefix_words=(
            entry_offset,
            int.from_bytes(raw[0x88:0x8C], "big"),
            int.from_bytes(raw[0x8C:0x90], "big"),
            int.from_bytes(raw[0x90:0x94], "big"),
        ),
        payload=payload,
        entry_offset=entry_offset,
        loader_stub=raw[0x94:entry_offset] if entry_offset > 0x94 else b"",
        body=body,
        info_table_offset=info_table_offset,
        info_table_bytes=info_table_bytes,
        info_records=info_records,
    )
