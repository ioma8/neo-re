import contextlib
import io
import unittest

from neotools import main


class CLIUpdaterPacketTests(unittest.TestCase):
    def test_updater_packet_command_prints_hex_bytes(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["updater-packet", "0x13", "0x12345678", "0x9abc"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "13 12 34 56 78 9a bc 7d\n")


if __name__ == "__main__":
    unittest.main()
