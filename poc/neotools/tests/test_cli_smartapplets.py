import io
import unittest
from contextlib import redirect_stdout

from neotools import main


class SmartAppletCliTests(unittest.TestCase):
    def test_smartapplet_header_prints_derived_add_applet_fields(self) -> None:
        output = io.StringIO()
        header_hex = (
            "c0 ff ee ad 00 01 a0 bc 00 00 0d 90 00 01 9f a4"
            " ff 00 00 ce a0 00 01 00"
            + " 00" * (0x80 - 0x18)
            + " 00 00 20 00"
        )

        with redirect_stdout(output):
            exit_code = main(["smartapplet-header", header_hex])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "magic=0xc0ffeead file_size=0x0001a0bc base_memory_size=0x00000d90 extra_memory_size=0x00002000 argument=0x0001a0bc trailing=0x2d90",
            ],
        )

    def test_smartapplet_add_plan_from_image_prints_derived_begin_packet(self) -> None:
        output = io.StringIO()
        image_hex = (
            "c0 ff ee ad 00 00 5f e0 00 00 05 6c 00 00 00 00"
            " ff 00 00 00 a0 02 01 00"
            + " 00" * (0x80 - 0x18)
            + " 00 00 00 00"
            + " 41 42 43 44 45"
        )

        with redirect_stdout(output):
            exit_code = main(["smartapplet-add-plan-from-image", image_hex])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines()[:4],
            [
                "reset_connection: 3f ff 00 72 65 73 65 74",
                "switch_to_updater: 3f 53 77 74 63 68 00 00",
                "add_applet_begin: 06 00 00 5f e0 05 6c b6",
                "add_applet_chunk_handshake: 02 00 00 00 89 07 fb 8d",
            ],
        )

    def test_smartapplet_metadata_prints_named_header_fields(self) -> None:
        output = io.StringIO()
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
        header[0x80:0x84] = bytes.fromhex("00 00 20 00")

        with redirect_stdout(output):
            exit_code = main(["smartapplet-metadata", bytes(header).hex(" ")])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "applet_id=0xa000 version=3.4 name=AlphaWord Plus info_table_offset=0x00019fa4 applet_class=0x01 extra_memory_size=0x00002000",
            ],
        )

    def test_smartapplet_retrieve_plan_prints_expected_steps(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["smartapplet-retrieve-plan", "0xa123"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "reset_connection: 3f ff 00 72 65 73 65 74",
                "switch_to_updater: 3f 53 77 74 63 68 00 00",
                "retrieve_applet: 0f 00 00 00 00 a1 23 d3",
                "retrieve_chunk: 10 00 00 00 00 00 00 10",
            ],
        )

    def test_smartapplet_string_prints_known_resource_label(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["smartapplet-string", "0xf138"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(output.getvalue().splitlines(), ["Maximum File Size (in characters)"])

    def test_smartapplet_menu_prints_decoded_popup_menu_items(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["smartapplet-menu", "163"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "command_id=0x800e label=Startup",
                "command_id=0x800f label=Startup Lock",
                "command_id=0x8010 label=Remove",
                "command_id=0x8012 label=Get Info",
                "command_id=0x8013 label=Help",
            ],
        )

    def test_smartapplet_add_plan_prints_expected_steps(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["smartapplet-add-plan", "0x12345678", "0x9abc", "41 42 43 44 45"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "reset_connection: 3f ff 00 72 65 73 65 74",
                "switch_to_updater: 3f 53 77 74 63 68 00 00",
                "add_applet_begin: 06 12 34 56 78 9a bc 70",
                "add_applet_chunk_handshake: 02 00 00 00 05 01 4f 57",
                "add_applet_chunk_data: 41 42 43 44 45",
                "add_applet_chunk_commit: ff 00 00 00 00 00 00 ff",
                "program_applet: 0b 00 00 00 00 00 00 0b",
                "finalize_applet_update: 07 00 00 00 00 00 00 07",
            ],
        )


if __name__ == "__main__":
    unittest.main()
