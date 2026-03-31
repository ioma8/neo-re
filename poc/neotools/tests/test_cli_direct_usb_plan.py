import contextlib
import io
import unittest

from neotools import main


class CLIDirectUsbPlanTests(unittest.TestCase):
    def test_direct_usb_alphaword_plan_prints_bootstrap_and_retrieve_sequence(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["direct-usb-alphaword-plan", "0xa000", "0x12"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "\n".join(
                [
                    "reset_connection: 3f ff 00 72 65 73 65 74",
                    "switch_to_updater: 3f 53 77 74 63 68 00 00",
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
