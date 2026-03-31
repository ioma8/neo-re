from dataclasses import dataclass


@dataclass(frozen=True)
class AlphaWordFileAttributes:
    raw: bytes
    name: str
    name_field: bytes
    reserved_length: int
    file_length: int
    trailing_bytes: bytes


def parse_file_attributes_record(record: bytes) -> AlphaWordFileAttributes:
    if len(record) != 0x28:
        raise ValueError("AlphaWord file attributes record must be exactly 0x28 bytes")

    name_field = record[:0x18]
    nul_offset = name_field.find(b"\x00")
    if nul_offset == -1:
        nul_offset = len(name_field)

    return AlphaWordFileAttributes(
        raw=record,
        name=name_field[:nul_offset].decode("latin-1"),
        name_field=name_field,
        reserved_length=int.from_bytes(record[0x18:0x1C], "big"),
        file_length=int.from_bytes(record[0x1C:0x20], "big"),
        trailing_bytes=record[0x20:0x28],
    )
