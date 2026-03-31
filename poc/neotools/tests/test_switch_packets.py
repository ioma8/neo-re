import unittest

from neotools.switch_packets import (
    SwitchResponse,
    build_reset_preamble,
    build_switch_packet,
    parse_switch_response,
)


class SwitchPacketTests(unittest.TestCase):
    def test_build_reset_preamble_matches_recovered_bytes(self) -> None:
        self.assertEqual(build_reset_preamble(), b"?\xff\x00reset")

    def test_build_switch_packet_uses_big_endian_applet_id(self) -> None:
        self.assertEqual(build_switch_packet(0x1234), b"?Swtch\x124")

    def test_parse_known_switch_responses(self) -> None:
        self.assertEqual(parse_switch_response(b"Switched"), SwitchResponse.SWITCHED)
        self.assertEqual(parse_switch_response(b"NoSwitch"), SwitchResponse.NO_SWITCH)
        self.assertEqual(parse_switch_response(b"NoApplet"), SwitchResponse.NO_APPLET)

    def test_build_switch_packet_rejects_out_of_range_applet_id(self) -> None:
        with self.assertRaises(ValueError):
            build_switch_packet(-1)

        with self.assertRaises(ValueError):
            build_switch_packet(0x1_0000)

    def test_parse_switch_response_rejects_invalid_packets(self) -> None:
        with self.assertRaises(ValueError):
            parse_switch_response(b"short")

        with self.assertRaises(ValueError):
            parse_switch_response(b"Unknown!")


if __name__ == "__main__":
    unittest.main()
