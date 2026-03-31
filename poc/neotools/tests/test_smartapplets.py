import unittest

from neotools.alphaword_flow import UpdaterStep
from neotools.smartapplets import (
    SmartAppletHeader,
    SmartAppletInfoRecord,
    SmartAppletMenuItem,
    build_add_applet_begin_command,
    build_direct_usb_add_applet_plan,
    build_direct_usb_add_applet_plan_from_image,
    build_direct_usb_retrieve_applet_plan,
    build_list_applets_command,
    build_retrieve_applet_command,
    derive_add_applet_start_fields,
    get_known_smartapplet_menu,
    parse_smartapplet_info_table,
    parse_smartapplet_metadata,
    parse_smartapplet_header,
    resolve_known_smartapplet_string,
)


class SmartAppletTests(unittest.TestCase):
    def test_parse_smartapplet_metadata_extracts_shared_0x84_record_fields(self) -> None:
        header = bytearray(0x84)
        header[0x00:0x04] = bytes.fromhex("c0 ff ee ad")
        header[0x04:0x08] = bytes.fromhex("00 01 a0 bc")
        header[0x08:0x0C] = bytes.fromhex("00 00 0d 90")
        header[0x0C:0x10] = bytes.fromhex("00 01 9f a4")
        header[0x10:0x14] = bytes.fromhex("ff 00 00 ce")
        header[0x14:0x18] = bytes.fromhex("a0 00 01 00")
        header[0x18:0x18 + len(b"AlphaWord Plus")] = b"AlphaWord Plus"
        header[0x3C] = 0x03
        header[0x3D] = 0x04
        header[0x3F] = 0x01
        copyright_text = b"Copyright (c) 2005-2012 by Renaissance Learning, Inc."
        header[0x40:0x40 + len(copyright_text)] = copyright_text
        header[0x80:0x84] = bytes.fromhex("00 00 20 00")

        metadata = parse_smartapplet_metadata(bytes(header))

        self.assertEqual(metadata.applet_id, 0xA000)
        self.assertEqual(metadata.version_major, 3)
        self.assertEqual(metadata.version_minor, 4)
        self.assertEqual(metadata.name, "AlphaWord Plus")
        self.assertEqual(metadata.info_table_offset, 0x00019FA4)
        self.assertEqual(metadata.flags_raw, 0xFF0000CE)
        self.assertEqual(metadata.applet_class, 0x01)
        self.assertEqual(metadata.extra_memory_size, 0x00002000)
        self.assertTrue(metadata.has_info_table)
        self.assertTrue(metadata.flag_high_0x10)
        self.assertFalse(metadata.flag_word_0x00010000)
        self.assertTrue(metadata.flag_high_0x40)

    def test_parse_smartapplet_info_table_decodes_variable_length_string_records(self) -> None:
        records = parse_smartapplet_info_table(
            bytes.fromhex(
                "00 01 80 02 00 12 50 61 73 73 77 6f 72 64 73 20 45 6e 61 62 6c 65 64 00"
                " 00 01 80 03 00 11 44 65 6c 65 74 65 20 61 6c 6c 20 66 69 6c 65 73 00"
                " 00 00"
            )
        )

        self.assertEqual(
            records,
            [
                SmartAppletInfoRecord(group=0x0001, key=0x8002, record_type=0x0001, payload=b"Passwords Enabled\x00", text="Passwords Enabled"),
                SmartAppletInfoRecord(group=0x0001, key=0x8003, record_type=0x0001, payload=b"Delete all files\x00", text="Delete all files"),
            ],
        )

    def test_parse_smartapplet_info_table_classifies_record_types(self) -> None:
        records = parse_smartapplet_info_table(
            bytes.fromhex(
                "01 01 80 02 00 04 00 00 00 64"
                " 01 02 80 03 00 0c 00 00 00 0a 00 00 00 14 00 00 00 1e"
                " 01 03 80 04 00 06 00 11 00 12 00 13"
                " 01 04 80 05 00 06 77 72 69 74 65 00"
                " 01 05 80 06 00 06 72 65 61 64 00 00"
                " 00 00"
            )
        )

        self.assertEqual(
            [record.record_type for record in records],
            [0x0101, 0x0102, 0x0103, 0x0104, 0x0105],
        )

    def test_parse_smartapplet_header_extracts_big_endian_fields(self) -> None:
        header = parse_smartapplet_header(
            bytes.fromhex(
                "c0 ff ee ad 00 01 a0 bc 00 00 0d 90 00 01 9f a4"
                " ff 00 00 ce a0 00 01 00"
                + "00" * (0x80 - 0x18)
                + "00 00 20 00"
            )
        )

        self.assertEqual(
            header,
            SmartAppletHeader(
                magic=0xC0FFEEAD,
                file_size=0x0001A0BC,
                base_memory_size=0x00000D90,
                payload_or_code_size=0x00019FA4,
                flags_and_version=0xFF0000CE,
                applet_id_and_version=0xA0000100,
                extra_memory_size=0x00002000,
            ),
        )

    def test_derive_add_applet_start_fields_packs_file_size_and_combined_memory_size(self) -> None:
        header = SmartAppletHeader(
            magic=0xC0FFEEAD,
            file_size=0x0001A0BC,
            base_memory_size=0x00000D90,
            payload_or_code_size=0x00019FA4,
            flags_and_version=0xFF0000CE,
            applet_id_and_version=0xA0000100,
            extra_memory_size=0x00002000,
        )

        self.assertEqual(
            derive_add_applet_start_fields(header),
            (0x0001A0BC, 0x2D90),
        )

    def test_build_list_applets_command_matches_confirmed_opcode(self) -> None:
        self.assertEqual(
            build_list_applets_command(page_offset=0, page_size=7),
            bytes.fromhex("04 00 00 00 00 00 07 0b"),
        )

    def test_direct_usb_add_applet_plan_from_image_derives_start_fields_from_header(self) -> None:
        image = bytes.fromhex(
            "c0 ff ee ad 00 00 5f e0 00 00 05 6c 00 00 00 00"
            " ff 00 00 00 a0 02 01 00"
            + "00" * (0x80 - 0x18)
            + "00 00 00 00"
        ) + b"ABCDE"

        self.assertEqual(
            build_direct_usb_add_applet_plan_from_image(image),
            [
                UpdaterStep("reset_connection", bytes.fromhex("3f ff 00 72 65 73 65 74")),
                UpdaterStep("switch_to_updater", bytes.fromhex("3f 53 77 74 63 68 00 00")),
                UpdaterStep("add_applet_begin", bytes.fromhex("06 00 00 5f e0 05 6c b6")),
                UpdaterStep("add_applet_chunk_handshake", bytes.fromhex("02 00 00 00 89 07 fb 8d")),
                UpdaterStep("add_applet_chunk_data", image),
                UpdaterStep("add_applet_chunk_commit", bytes.fromhex("ff 00 00 00 00 00 00 ff")),
                UpdaterStep("program_applet", bytes.fromhex("0b 00 00 00 00 00 00 0b")),
                UpdaterStep("finalize_applet_update", bytes.fromhex("07 00 00 00 00 00 00 07")),
            ],
        )

    def test_build_retrieve_applet_command_uses_applet_id_in_trailing_field(self) -> None:
        self.assertEqual(
            build_retrieve_applet_command(applet_id=0xA123),
            bytes.fromhex("0f 00 00 00 00 a1 23 d3"),
        )

    def test_build_add_applet_begin_command_keeps_caller_supplied_selector_fields(self) -> None:
        self.assertEqual(
            build_add_applet_begin_command(argument=0x12345678, trailing=0x9ABC),
            bytes.fromhex("06 12 34 56 78 9a bc 70"),
        )

    def test_direct_usb_retrieve_applet_plan_bootstraps_then_requests_chunks(self) -> None:
        self.assertEqual(
            build_direct_usb_retrieve_applet_plan(applet_id=0xA123),
            [
                UpdaterStep("reset_connection", bytes.fromhex("3f ff 00 72 65 73 65 74")),
                UpdaterStep("switch_to_updater", bytes.fromhex("3f 53 77 74 63 68 00 00")),
                UpdaterStep("retrieve_applet", bytes.fromhex("0f 00 00 00 00 a1 23 d3")),
                UpdaterStep("retrieve_chunk", bytes.fromhex("10 00 00 00 00 00 00 10")),
            ],
        )

    def test_direct_usb_add_applet_plan_models_handshake_program_and_finalize_steps(self) -> None:
        payload = b"ABCDE"

        self.assertEqual(
            build_direct_usb_add_applet_plan(
                start_argument=0x12345678,
                trailing=0x9ABC,
                payload=payload,
            ),
            [
                UpdaterStep("reset_connection", bytes.fromhex("3f ff 00 72 65 73 65 74")),
                UpdaterStep("switch_to_updater", bytes.fromhex("3f 53 77 74 63 68 00 00")),
                UpdaterStep("add_applet_begin", bytes.fromhex("06 12 34 56 78 9a bc 70")),
                UpdaterStep("add_applet_chunk_handshake", bytes.fromhex("02 00 00 00 05 01 4f 57")),
                UpdaterStep("add_applet_chunk_data", b"ABCDE"),
                UpdaterStep("add_applet_chunk_commit", bytes.fromhex("ff 00 00 00 00 00 00 ff")),
                UpdaterStep("program_applet", bytes.fromhex("0b 00 00 00 00 00 00 0b")),
                UpdaterStep("finalize_applet_update", bytes.fromhex("07 00 00 00 00 00 00 07")),
            ],
        )

    def test_known_smartapplet_string_map_resolves_alpha_word_file_limit_labels(self) -> None:
        self.assertEqual(resolve_known_smartapplet_string(0xF138), "Maximum File Size (in characters)")
        self.assertEqual(resolve_known_smartapplet_string(0xF139), "Minimum File Size (in characters)")

    def test_known_smartapplet_menu_map_decodes_popup_resource_163(self) -> None:
        self.assertEqual(
            get_known_smartapplet_menu(163),
            (
                SmartAppletMenuItem(command_id=0x800E, label="Startup"),
                SmartAppletMenuItem(command_id=0x800F, label="Startup Lock"),
                SmartAppletMenuItem(command_id=0x8010, label="Remove"),
                SmartAppletMenuItem(command_id=0x8012, label="Get Info"),
                SmartAppletMenuItem(command_id=0x8013, label="Help"),
            ),
        )


if __name__ == "__main__":
    unittest.main()
