import unittest

from neotools.alphaword_flow import (
    UpdaterStep,
    build_direct_usb_full_text_retrieval_plan,
    build_full_text_retrieval_plan,
    build_preview_retrieval_plan,
)


class AlphaWordFlowTests(unittest.TestCase):
    def test_full_text_plan_starts_with_list_applets_and_slot_metadata(self) -> None:
        plan = build_full_text_retrieval_plan(applet_id=0xA000, file_slot=0x12)

        self.assertEqual(
            plan[:3],
                [
                    UpdaterStep("list_applets", bytes.fromhex("04 00 00 00 00 00 07 0b")),
                    UpdaterStep("raw_file_attributes", bytes.fromhex("13 00 00 00 12 a0 00 c5")),
                    UpdaterStep("retrieve_file", bytes.fromhex("12 08 00 00 12 a0 00 cc")),
                ],
        )

    def test_full_text_plan_uses_chunk_pull_after_initial_retrieve(self) -> None:
        plan = build_full_text_retrieval_plan(applet_id=0xA000, file_slot=0x12)

        self.assertEqual(plan[3], UpdaterStep("retrieve_chunk", bytes.fromhex("10 00 00 00 00 00 00 10")))

    def test_preview_plan_uses_same_wire_sequence_but_marks_preview_mode(self) -> None:
        plan = build_preview_retrieval_plan(applet_id=0xA000, file_slot=0x12)

        self.assertEqual(plan[0].kind, "list_applets")
        self.assertEqual(plan[1].kind, "raw_file_attributes")
        self.assertEqual(plan[2].kind, "retrieve_file")
        self.assertEqual(plan[2].packet, bytes.fromhex("12 00 00 b4 12 a0 00 78"))
        self.assertEqual(plan[3].kind, "retrieve_chunk")

    def test_direct_usb_plan_bootstraps_transport_before_updater_commands(self) -> None:
        plan = build_direct_usb_full_text_retrieval_plan(applet_id=0xA000, file_slot=0x12)

        self.assertEqual(
            plan[:2],
            [
                UpdaterStep("reset_connection", bytes.fromhex("3f ff 00 72 65 73 65 74")),
                UpdaterStep("switch_to_updater", bytes.fromhex("3f 53 77 74 63 68 00 00")),
            ],
        )
        self.assertEqual(plan[2:], build_full_text_retrieval_plan(applet_id=0xA000, file_slot=0x12))


if __name__ == "__main__":
    unittest.main()
