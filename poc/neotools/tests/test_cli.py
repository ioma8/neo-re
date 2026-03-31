import contextlib
import io
import unittest

from neotools import main


class CLITests(unittest.TestCase):
    def test_descriptor_command_parses_and_classifies_direct_neo(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["descriptor", "12 01 00 02 00 00 00 40 1e 08 01 bd 00 01 01 02 03 01"]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "vendor_id=0x081e product_id=0xbd01 direct_neo=True\n",
        )

    def test_switch_packet_command_prints_hex_bytes(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["switch-packet", "0x1234"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "3f 53 77 74 63 68 12 34\n")

    def test_asusbcomm_presence_command_prints_cached_mode_and_return_code(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["asusbcomm-presence", "12 01 00 02 00 00 00 40 1e 08 01 bd 02 00 01 02 03 01"]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "descriptor_valid=True cached_mode=3 return_code=3\n")

    def test_asusbcomm_set_mac_packet_command_prints_hex_bytes(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["asusbcomm-set-mac-packet", "00 00 aa bb cc dd ee ff"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "20 aa bb cc dd ee ff 1b\n")

    def test_asusbcomm_get_mac_packet_command_prints_hex_bytes(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["asusbcomm-get-mac-packet"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "00 00 00 00 00 00 00 00\n")

    def test_asusbcomm_hid_fallback_plan_command_prints_newer_windows_sequence(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["asusbcomm-hid-fallback-plan", "5"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "\n".join(
                [
                    "device_io_control code=0x000b0040 payload=00 00 00 00",
                    "device_io_control code=0x000b0008 payload=05 00 00 00",
                    "device_io_control code=0x000b0008 payload=02 00 00 00",
                    "device_io_control code=0x000b0008 payload=04 00 00 00",
                    "device_io_control code=0x000b0008 payload=01 00 00 00",
                    "device_io_control code=0x000b0008 payload=06 00 00 00",
                    "device_io_control code=0x000b0008 payload=07 00 00 00",
                    "sleep_ms value=2000",
                    "",
                ]
            ),
        )

    def test_alphaword_plan_command_prints_retrieval_sequence(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["alphaword-plan", "0xa000", "0x12"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "\n".join(
                [
                    "list_applets: 04 00 00 00 00 00 07 0b",
                    "raw_file_attributes: 13 00 00 00 12 a0 00 c5",
                    "retrieve_file: 12 08 00 00 12 a0 00 cc",
                    "retrieve_chunk: 10 00 00 00 00 00 00 10",
                    "",
                ]
            ),
        )


if __name__ == "__main__":
    unittest.main()
