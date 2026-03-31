import unittest

from neotools.alphaword_flow import UpdaterStep
from neotools.alphaword_send import (
    build_direct_usb_send_file_record,
    build_put_file_plan,
    build_put_raw_file_attributes_plan,
)


class AlphaWordSendTests(unittest.TestCase):
    def test_put_raw_file_attributes_plan_matches_confirmed_handshake(self) -> None:
        record = bytes.fromhex(
            "00 01 02 03 04 05 06 07"
            " 08 09 0a 0b 0c 0d 0e 0f"
            " 10 11 12 13 14 15 16 17"
            " 00 00 01 23 00 00 45 67"
            " aa bb cc dd ee ff 10 20"
        )

        self.assertEqual(
            build_put_raw_file_attributes_plan(file_slot=0x01, applet_id=0xA000, record=record),
            [
                UpdaterStep("put_raw_file_attributes_begin", bytes.fromhex("1d 00 00 00 01 a0 00 be")),
                UpdaterStep("put_raw_file_attributes_data", record),
                UpdaterStep("put_raw_file_attributes_finish", bytes.fromhex("1e 00 00 00 01 a0 00 bf")),
            ],
        )

    def test_put_file_plan_models_start_chunk_and_finish_commands(self) -> None:
        self.assertEqual(
            build_put_file_plan(file_slot=0x01, applet_id=0xA000, payload=b"ABCDE"),
            [
                UpdaterStep("put_file_begin", bytes.fromhex("14 01 00 00 05 a0 00 ba")),
                UpdaterStep("put_file_chunk_handshake", bytes.fromhex("02 00 00 00 05 01 4f 57")),
                UpdaterStep("put_file_chunk_data", b"ABCDE"),
                UpdaterStep("put_file_finish", bytes.fromhex("15 00 00 00 00 00 00 15")),
            ],
        )

    def test_put_file_plan_splits_payload_into_up_to_0x400_byte_chunks(self) -> None:
        payload = b"A" * 0x401

        plan = build_put_file_plan(file_slot=0x01, applet_id=0xA000, payload=payload)

        self.assertEqual(plan[0], UpdaterStep("put_file_begin", bytes.fromhex("14 01 00 04 01 a0 00 ba")))
        self.assertEqual(plan[1], UpdaterStep("put_file_chunk_handshake", bytes.fromhex("02 00 00 04 00 04 00 0a")))
        self.assertEqual(plan[2], UpdaterStep("put_file_chunk_data", b"A" * 0x400))
        self.assertEqual(plan[3], UpdaterStep("put_file_chunk_handshake", bytes.fromhex("02 00 00 00 01 00 41 44")))
        self.assertEqual(plan[4], UpdaterStep("put_file_chunk_data", b"A"))
        self.assertEqual(plan[5], UpdaterStep("put_file_finish", bytes.fromhex("15 00 00 00 00 00 00 15")))

    def test_direct_usb_send_file_record_bootstraps_once_then_sends_attributes_and_file(self) -> None:
        record = bytes.fromhex(
            "00 01 02 03 04 05 06 07"
            " 08 09 0a 0b 0c 0d 0e 0f"
            " 10 11 12 13 14 15 16 17"
            " 00 00 01 23 00 00 00 05"
            " aa bb cc dd ee ff 10 20"
        )

        self.assertEqual(
            build_direct_usb_send_file_record(
                file_slot=0x01,
                applet_id=0xA000,
                record=record,
                payload=b"ABCDE",
            ),
            [
                UpdaterStep("reset_connection", bytes.fromhex("3f ff 00 72 65 73 65 74")),
                UpdaterStep("switch_to_updater", bytes.fromhex("3f 53 77 74 63 68 00 00")),
                UpdaterStep("put_raw_file_attributes_begin", bytes.fromhex("1d 00 00 00 01 a0 00 be")),
                UpdaterStep("put_raw_file_attributes_data", record),
                UpdaterStep("put_raw_file_attributes_finish", bytes.fromhex("1e 00 00 00 01 a0 00 bf")),
                UpdaterStep("put_file_begin", bytes.fromhex("14 01 00 00 05 a0 00 ba")),
                UpdaterStep("put_file_chunk_handshake", bytes.fromhex("02 00 00 00 05 01 4f 57")),
                UpdaterStep("put_file_chunk_data", b"ABCDE"),
                UpdaterStep("put_file_finish", bytes.fromhex("15 00 00 00 00 00 00 15")),
            ],
        )


if __name__ == "__main__":
    unittest.main()
