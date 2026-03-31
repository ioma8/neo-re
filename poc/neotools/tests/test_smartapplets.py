import unittest

from neotools.alphaword_flow import UpdaterStep
from neotools.smartapplets import (
    SmartAppletHeader,
    build_add_applet_begin_command,
    build_direct_usb_add_applet_plan,
    build_direct_usb_add_applet_plan_from_image,
    build_direct_usb_retrieve_applet_plan,
    build_list_applets_command,
    build_retrieve_applet_command,
    derive_add_applet_start_fields,
    parse_smartapplet_header,
)


class SmartAppletTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
