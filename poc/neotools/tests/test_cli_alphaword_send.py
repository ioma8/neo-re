import contextlib
import io
import unittest

from neotools import main


class CLIAlphaWordSendTests(unittest.TestCase):
    def test_direct_usb_alphaword_send_record_prints_expected_sequence(self) -> None:
        stdout = io.StringIO()
        record = (
            "00 01 02 03 04 05 06 07 "
            "08 09 0a 0b 0c 0d 0e 0f "
            "10 11 12 13 14 15 16 17 "
            "00 00 01 23 00 00 00 05 "
            "aa bb cc dd ee ff 10 20"
        )

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                [
                    "direct-usb-alphaword-send-record",
                    "0xa000",
                    "0x01",
                    record,
                    "41 42 43 44 45",
                ]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "\n".join(
                [
                    "reset_connection: 3f ff 00 72 65 73 65 74",
                    "switch_to_updater: 3f 53 77 74 63 68 00 00",
                    "put_raw_file_attributes_begin: 1d 00 00 00 01 a0 00 be",
                    "put_raw_file_attributes_handshake: 02 00 00 00 28 06 68 98",
                    f"put_raw_file_attributes_data: {record.lower()}",
                    "put_raw_file_attributes_commit: ff 00 00 00 00 00 00 ff",
                    "put_raw_file_attributes_finish: 1e 00 00 00 01 a0 00 bf",
                    "put_file_begin: 14 01 00 00 05 a0 00 ba",
                    "put_file_chunk_handshake: 02 00 00 00 05 01 4f 57",
                    "put_file_chunk_data: 41 42 43 44 45",
                    "put_file_chunk_commit: ff 00 00 00 00 00 00 ff",
                    "put_file_finish: 15 00 00 00 00 00 00 15",
                    "",
                ]
            ),
        )


if __name__ == "__main__":
    unittest.main()
