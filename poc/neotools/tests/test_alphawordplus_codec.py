from pathlib import Path
from unittest import TestCase

from neotools.alphawordplus_codec import (
    AlphaWordPlusCodec,
    extract_alphawordplus_codec_from_image,
)


class AlphaWordPlusCodecTests(TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        raw = Path("analysis/cab/alphawordplus.os3kapp").read_bytes()
        cls.codec = extract_alphawordplus_codec_from_image(raw)

    def test_extracts_both_256_byte_tables(self) -> None:
        self.assertIsInstance(self.codec, AlphaWordPlusCodec)
        self.assertEqual(len(self.codec.encode_table), 256)
        self.assertEqual(len(self.codec.decode_table), 256)

    def test_printable_ascii_is_identity_in_both_directions(self) -> None:
        payload = bytes(range(0x20, 0x7F))
        self.assertEqual(self.codec.encode_bytes(payload), payload)
        self.assertEqual(self.codec.decode_bytes(payload), payload)

    def test_known_write_side_control_mappings_match_decompile(self) -> None:
        self.assertEqual(self.codec.encode_byte(0x00), 0xE7)
        self.assertEqual(self.codec.encode_byte(0x01), 0xFC)
        self.assertEqual(self.codec.encode_byte(0x02), 0xD6)
        self.assertEqual(self.codec.encode_byte(0x1D), 0xAC)
        self.assertEqual(self.codec.encode_byte(0x1E), 0x00)
        self.assertEqual(self.codec.encode_byte(0xBC), 0xB6)
        self.assertEqual(self.codec.encode_byte(0xBD), 0xBD)
        self.assertEqual(self.codec.encode_byte(0xC0), 0xF8)

    def test_known_read_side_control_mappings_match_decompile(self) -> None:
        self.assertEqual(self.codec.decode_byte(0x80), 0x19)
        self.assertEqual(self.codec.decode_byte(0x8C), 0x04)
        self.assertEqual(self.codec.decode_byte(0x9D), 0xFD)
        self.assertEqual(self.codec.decode_byte(0xB6), 0xBC)
        self.assertEqual(self.codec.decode_byte(0xBE), 0x1C)
        self.assertEqual(self.codec.decode_byte(0xFC), 0x01)

    def test_inverse_region_count_matches_current_table_shape(self) -> None:
        self.assertEqual(self.codec.encode_inverse_match_count(), 212)
        self.assertEqual(self.codec.decode_inverse_match_count(), 212)

    def test_reports_known_aliases_for_encoded_zero(self) -> None:
        self.assertEqual(
            self.codec.source_aliases_for_encoded_byte(0x00),
            (0x1E, 0x1F, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xC1, 0xD9, 0xEC, 0xFA, 0xFE, 0xFF),
        )
