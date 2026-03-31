from pathlib import Path
import unittest

from neotools.os3kapp_format import Os3kAppHeaderFields, build_os3kapp_image, parse_os3kapp_image


FIXTURE_DIR = Path("/Users/jakubkolcar/customs/neo-re/analysis/cab")


class Os3kAppFormatTests(unittest.TestCase):
    def test_build_os3kapp_image_roundtrips_structural_fields(self) -> None:
        payload = bytes.fromhex("00 00 00 94 00 00 00 00 00 00 00 01 00 00 00 02 aa bb cc dd")
        info_table = bytes.fromhex("00 01 80 00 00 05 48 65 6c 6c 6f 00 00 00")
        raw = build_os3kapp_image(
            header_fields=Os3kAppHeaderFields(
                magic=0xC0FFEEAD,
                base_memory_size=0x10,
                flags_raw=0xFF000031,
                applet_id_and_version=0xA0020100,
                name="Calculator",
                version_major_bcd=0x03,
                version_minor_bcd=0x00,
                version_build_byte=0x00,
                applet_class=0x01,
                copyright="Example Copyright",
                extra_memory_size=0,
            ),
            payload=payload,
            info_table_bytes=info_table,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.header.file_size, len(raw))
        self.assertEqual(image.entry_offset, 0x94)
        self.assertEqual(image.payload, payload + info_table)
        self.assertEqual(image.body, payload)
        self.assertEqual(image.info_table_offset, 0x84 + len(payload))
        self.assertEqual(image.info_records[0].text, "Hello")

    def test_parse_neofontmedium_image_maps_header_and_body_sections(self) -> None:
        image = parse_os3kapp_image((FIXTURE_DIR / "neofontmedium.os3kapp").read_bytes())

        self.assertEqual(image.metadata.header.file_size, 0x1108)
        self.assertEqual(image.header_size, 0x84)
        self.assertEqual(image.body_prefix_words, (0x94, 0, 1, 2))
        self.assertEqual(image.entry_offset, 0x94)
        self.assertEqual(image.payload, (FIXTURE_DIR / "neofontmedium.os3kapp").read_bytes()[0x84:])
        self.assertEqual(image.loader_stub, b"")
        self.assertEqual(len(image.body), 0x1108 - 0x84)
        self.assertEqual(image.info_table_bytes, b"")
        self.assertEqual(image.body[:8], bytes.fromhex("00 00 00 94 00 00 00 00"))

    def test_parse_spellcheck_image_splits_trailing_info_table(self) -> None:
        image = parse_os3kapp_image((FIXTURE_DIR / "spellcheck_small_usa.os3kapp").read_bytes())

        self.assertEqual(image.metadata.header.file_size, 0x2DAC0)
        self.assertEqual(image.body_prefix_words, (0x33B2, 0, 1, 2))
        self.assertEqual(image.entry_offset, 0x33B2)
        self.assertEqual(len(image.loader_stub), 0x33B2 - 0x94)
        self.assertEqual(len(image.body), 0x2DA58 - 0x84)
        self.assertEqual(image.info_table_offset, 0x2DA58)
        self.assertGreater(len(image.info_records), 0)
        self.assertEqual(image.info_records[0].text, "Allow adding words to dictionary")

    def test_parse_alphaquiz_image_keeps_full_tail_when_no_info_table_exists(self) -> None:
        image = parse_os3kapp_image((FIXTURE_DIR / "alphaquiz.os3kapp").read_bytes())

        self.assertEqual(image.metadata.header.file_size, 0xC2A4)
        self.assertEqual(image.body_prefix_words, (0x0E20, 0, 1, 2))
        self.assertEqual(image.entry_offset, 0x0E20)
        self.assertEqual(len(image.loader_stub), 0x0E20 - 0x94)
        self.assertEqual(image.info_table_offset, 0)
        self.assertEqual(image.info_records, ())
        self.assertEqual(len(image.body), 0xC2A4 - 0x84)


if __name__ == "__main__":
    unittest.main()
