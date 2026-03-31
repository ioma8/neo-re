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
