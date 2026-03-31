import contextlib
import io
import unittest

from neotools import main


class CLIDecodeTests(unittest.TestCase):
    def test_decode_updater_response_prints_fields(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["decode-updater-response", "53 00 00 00 05 00 00 58"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "status=0x53 argument=0x00000005 trailing=0x0000\n")

    def test_parse_alphaword_attributes_prints_confirmed_fields(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                [
                    "parse-alphaword-attributes",
                    (
                        "00 01 02 03 04 05 06 07 "
                        "08 09 0a 0b 0c 0d 0e 0f "
                        "10 11 12 13 14 15 16 17 "
                        "00 00 01 23 00 00 45 67 "
                        "aa bb cc dd ee ff 10 20"
                    ),
                ]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "value_0x18=0x00000123 file_length=0x00004567\n",
        )


if __name__ == "__main__":
    unittest.main()
