import unittest

from neotools.alphaword_attributes import AlphaWordFileAttributes, parse_file_attributes_record


class AlphaWordAttributesTests(unittest.TestCase):
    def test_parse_file_attributes_record_extracts_confirmed_big_endian_fields(self) -> None:
        record = bytes.fromhex(
            "00 01 02 03 04 05 06 07"
            " 08 09 0a 0b 0c 0d 0e 0f"
            " 10 11 12 13 14 15 16 17"
            " 00 00 01 23 00 00 45 67"
            " aa bb cc dd ee ff 10 20"
        )

        self.assertEqual(
            parse_file_attributes_record(record),
            AlphaWordFileAttributes(
                raw=record,
                value_0x18=0x123,
                file_length=0x4567,
            ),
        )

    def test_parse_file_attributes_record_requires_0x28_bytes(self) -> None:
        with self.assertRaises(ValueError):
            parse_file_attributes_record(b"\x00" * 0x27)


if __name__ == "__main__":
    unittest.main()
