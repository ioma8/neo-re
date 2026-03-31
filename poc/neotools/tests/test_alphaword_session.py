import unittest

from neotools.alphaword_flow import UpdaterStep
from neotools.alphaword_session import (
    build_direct_usb_full_text_session,
    build_direct_usb_preview_session,
)


class AlphaWordSessionTests(unittest.TestCase):
    def test_preview_session_bootstraps_once_and_scans_all_8_slots(self) -> None:
        session = build_direct_usb_preview_session(applet_id=0xA000)

        self.assertEqual(
            session[:4],
            [
                UpdaterStep("reset_connection", bytes.fromhex("3f ff 00 72 65 73 65 74")),
                UpdaterStep("switch_to_updater", bytes.fromhex("3f 53 77 74 63 68 00 00")),
                UpdaterStep("list_applets", bytes.fromhex("04 00 00 00 00 00 07 0b")),
                UpdaterStep("raw_file_attributes", bytes.fromhex("13 00 00 00 01 a0 00 b4")),
            ],
        )
        self.assertEqual(session[4], UpdaterStep("retrieve_file", bytes.fromhex("12 00 00 b4 01 a0 00 67")))
        self.assertEqual(session[5], UpdaterStep("retrieve_chunk", bytes.fromhex("10 00 00 00 00 00 00 10")))
        self.assertEqual(len(session), 2 + (8 * 4))
        self.assertEqual(session[-3], UpdaterStep("raw_file_attributes", bytes.fromhex("13 00 00 00 08 a0 00 bb")))
        self.assertEqual(session[-2], UpdaterStep("retrieve_file", bytes.fromhex("12 00 00 b4 08 a0 00 6e")))
        self.assertEqual(session[-1], UpdaterStep("retrieve_chunk", bytes.fromhex("10 00 00 00 00 00 00 10")))

    def test_full_text_session_bootstraps_once_and_scans_all_8_slots(self) -> None:
        session = build_direct_usb_full_text_session(applet_id=0xA000)

        self.assertEqual(session[0].kind, "reset_connection")
        self.assertEqual(session[1].kind, "switch_to_updater")
        self.assertEqual(session[2], UpdaterStep("list_applets", bytes.fromhex("04 00 00 00 00 00 07 0b")))
        self.assertEqual(session[3], UpdaterStep("raw_file_attributes", bytes.fromhex("13 00 00 00 01 a0 00 b4")))
        self.assertEqual(session[4], UpdaterStep("retrieve_file", bytes.fromhex("12 08 00 00 01 a0 00 bb")))
        self.assertEqual(session[5], UpdaterStep("retrieve_chunk", bytes.fromhex("10 00 00 00 00 00 00 10")))
        self.assertEqual(len(session), 2 + (8 * 4))
        self.assertEqual(session[-2], UpdaterStep("retrieve_file", bytes.fromhex("12 08 00 00 08 a0 00 c2")))


if __name__ == "__main__":
    unittest.main()
