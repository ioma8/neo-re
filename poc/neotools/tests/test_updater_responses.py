import unittest

from neotools.updater_responses import UpdaterResponse, parse_updater_response


class UpdaterResponseTests(unittest.TestCase):
    def test_parse_updater_response_decodes_status_argument_and_trailing(self) -> None:
        self.assertEqual(
            parse_updater_response(bytes.fromhex("53 00 00 01 23 00 00 77")),
            UpdaterResponse(status=0x53, argument=0x123, trailing=0),
        )

    def test_parse_updater_response_handles_chunk_header_fields(self) -> None:
        self.assertEqual(
            parse_updater_response(bytes.fromhex("4d 00 00 00 08 01 24 7a")),
            UpdaterResponse(status=0x4D, argument=8, trailing=0x0124),
        )

    def test_parse_updater_response_rejects_invalid_length_or_checksum(self) -> None:
        with self.assertRaises(ValueError):
            parse_updater_response(b"\x53")

        with self.assertRaises(ValueError):
            parse_updater_response(bytes.fromhex("53 00 00 01 23 00 00 78"))


if __name__ == "__main__":
    unittest.main()
