import unittest

from neotools.updater_packets import (
    build_raw_file_attributes_command,
    build_retrieve_file_command,
    build_updater_command,
)


class UpdaterPacketTests(unittest.TestCase):
    def test_build_updater_command_encodes_big_endian_fields(self) -> None:
        self.assertEqual(
            build_updater_command(command=0x13, argument=0x12345678, trailing=0x9ABC),
            bytes.fromhex("13 12 34 56 78 9a bc 7d"),
        )

    def test_build_updater_command_uses_low_byte_sum_checksum(self) -> None:
        packet = build_updater_command(command=0x12, argument=0x00000102, trailing=0x0304)

        self.assertEqual(packet[:-1], bytes.fromhex("12 00 00 01 02 03 04"))
        self.assertEqual(packet[-1], sum(packet[:-1]) & 0xFF)

    def test_build_updater_command_rejects_values_out_of_range(self) -> None:
        with self.assertRaises(ValueError):
            build_updater_command(command=0x100, argument=0, trailing=0)

        with self.assertRaises(ValueError):
            build_updater_command(command=0, argument=0x1_0000_0000, trailing=0)

        with self.assertRaises(ValueError):
            build_updater_command(command=0, argument=0, trailing=0x1_0000)

    def test_build_raw_file_attributes_command_includes_applet_id_in_trailing_field(self) -> None:
        self.assertEqual(
            build_raw_file_attributes_command(file_slot=0x12, applet_id=0xA000),
            bytes.fromhex("13 00 00 00 12 a0 00 c5"),
        )

    def test_build_retrieve_file_command_uses_requested_length_and_applet_id(self) -> None:
        self.assertEqual(
            build_retrieve_file_command(
                file_slot=0x12,
                applet_id=0xA000,
                requested_length=0x80000,
                alternate_mode=False,
            ),
            bytes.fromhex("12 08 00 00 12 a0 00 cc"),
        )
        self.assertEqual(
            build_retrieve_file_command(
                file_slot=0x12,
                applet_id=0xA000,
                requested_length=0x80000,
                alternate_mode=True,
            ),
            bytes.fromhex("1c 08 00 00 12 a0 00 d6"),
        )


if __name__ == "__main__":
    unittest.main()
