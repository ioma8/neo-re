from dataclasses import dataclass


@dataclass(frozen=True)
class AlphaWordFileAttributes:
    raw: bytes
    value_0x18: int
    file_length: int


def parse_file_attributes_record(record: bytes) -> AlphaWordFileAttributes:
    if len(record) != 0x28:
        raise ValueError("AlphaWord file attributes record must be exactly 0x28 bytes")

    return AlphaWordFileAttributes(
        raw=record,
        value_0x18=int.from_bytes(record[0x18:0x1C], "big"),
        file_length=int.from_bytes(record[0x1C:0x20], "big"),
    )
