import unittest

from neotools.alphaword_attributes import AlphaWordFileAttributes, parse_file_attributes_record


class AlphaWordAttributesTests(unittest.TestCase):
    def test_parse_file_attributes_record_extracts_layout_fields(self) -> None:
        record = (
            b"FILE1\0"
            + bytes.fromhex("01 02 03 04 05 06 07 08 09 0a 0b 0c 00 00 00 00 00 00")
            + bytes.fromhex("00 00 01 23 00 00 45 67 aa bb cc dd ee ff 10 20")
        )

        self.assertEqual(
            parse_file_attributes_record(record),
            AlphaWordFileAttributes(
                raw=record,
                name="FILE1",
                name_field=record[:0x18],
                reserved_length=0x123,
                file_length=0x4567,
                trailing_bytes=bytes.fromhex("aa bb cc dd ee ff 10 20"),
            ),
        )

    def test_parse_file_attributes_record_stops_name_at_first_nul(self) -> None:
        record = (
            b"AB\0CD"
            + (b"\x00" * (0x18 - 5))
            + bytes.fromhex("00 00 00 09 00 00 00 0a 11 22 33 44 55 66 77 88")
        )

        attributes = parse_file_attributes_record(record)

        self.assertEqual(attributes.name, "AB")
        self.assertEqual(attributes.name_field, record[:0x18])
        self.assertEqual(attributes.reserved_length, 9)
        self.assertEqual(attributes.file_length, 10)
        self.assertEqual(attributes.trailing_bytes, bytes.fromhex("11 22 33 44 55 66 77 88"))

    def test_parse_file_attributes_record_requires_0x28_bytes(self) -> None:
        with self.assertRaises(ValueError):
            parse_file_attributes_record(b"\x00" * 0x27)


if __name__ == "__main__":
    unittest.main()
