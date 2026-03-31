import contextlib
import io
import unittest

from neotools import main


class CLIAlphaWordSessionTests(unittest.TestCase):
    def test_direct_usb_preview_session_command_prints_8_slot_scan(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["direct-usb-alphaword-session", "preview", "0xa000"])

        self.assertEqual(exit_code, 0)
        lines = stdout.getvalue().splitlines()
        self.assertEqual(lines[0], "reset_connection: 3f ff 00 72 65 73 65 74")
        self.assertEqual(lines[1], "switch_to_updater: 3f 53 77 74 63 68 00 00")
        self.assertEqual(lines[2], "list_applets: 04 00 00 00 00 00 07 0b")
        self.assertEqual(lines[3], "raw_file_attributes: 13 00 00 00 01 a0 00 b4")
        self.assertEqual(lines[-3], "raw_file_attributes: 13 00 00 00 08 a0 00 bb")
        self.assertEqual(lines[-2], "retrieve_file: 12 00 00 b4 08 a0 00 6e")
        self.assertEqual(lines[-1], "retrieve_chunk: 10 00 00 00 00 00 00 10")
        self.assertEqual(len(lines), 34)


if __name__ == "__main__":
    unittest.main()
