from collections import defaultdict
from dataclasses import dataclass


ALPHAWORDPLUS_ENCODE_TABLE_OFFSET = 0x1488C
ALPHAWORDPLUS_DECODE_TABLE_OFFSET = 0x1498C
ALPHAWORDPLUS_TABLE_LENGTH = 0x100


@dataclass(frozen=True)
class AlphaWordPlusCodec:
    encode_table: bytes
    decode_table: bytes

    def encode_byte(self, value: int) -> int:
        return self.encode_table[value & 0xFF]

    def decode_byte(self, value: int) -> int:
        return self.decode_table[value & 0xFF]

    def encode_bytes(self, payload: bytes) -> bytes:
        return bytes(self.encode_byte(byte) for byte in payload)

    def decode_bytes(self, payload: bytes) -> bytes:
        return bytes(self.decode_byte(byte) for byte in payload)

    def encode_inverse_match_count(self) -> int:
        return sum(self.decode_byte(self.encode_byte(value)) == value for value in range(0x100))

    def decode_inverse_match_count(self) -> int:
        return sum(self.encode_byte(self.decode_byte(value)) == value for value in range(0x100))

    def source_aliases_for_encoded_byte(self, encoded_value: int) -> tuple[int, ...]:
        encoded_value &= 0xFF
        aliases = [index for index, table_value in enumerate(self.encode_table) if table_value == encoded_value]
        return tuple(aliases)


def extract_alphawordplus_codec_from_image(raw: bytes) -> AlphaWordPlusCodec:
    decode_end = ALPHAWORDPLUS_DECODE_TABLE_OFFSET + ALPHAWORDPLUS_TABLE_LENGTH
    if len(raw) < decode_end:
        raise ValueError("AlphaWordPlus image is too short to contain both translation tables")

    return AlphaWordPlusCodec(
        encode_table=raw[ALPHAWORDPLUS_ENCODE_TABLE_OFFSET : ALPHAWORDPLUS_ENCODE_TABLE_OFFSET + ALPHAWORDPLUS_TABLE_LENGTH],
        decode_table=raw[ALPHAWORDPLUS_DECODE_TABLE_OFFSET:decode_end],
    )

