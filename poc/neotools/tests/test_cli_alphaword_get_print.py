import contextlib
import io
import unittest

from neotools import main


class CLIAlphaWordGetPrintFlowTests(unittest.TestCase):
    def test_updater_retrieve_applet_file_data_flow_command_prints_root_retrieve_sequence(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                [
                    "updater-retrieve-applet-file-data-flow",
                    "0x12",
                    "0xa000",
                    "2",
                    "0x80000",
                    "--reported-total-length",
                    "0x30",
                    "--chunk-lengths",
                    "0x10",
                    "0x20",
                    "--chunk-checksums",
                    "0x10",
                    "0x20",
                ]
            )

        self.assertEqual(exit_code, 0)
        lines = stdout.getvalue().splitlines()
        self.assertEqual(lines[0], "controller: UpdaterRetrieveAppletFileData")
        self.assertEqual(lines[4], "send_start_command slot=2: 12 08 00 00 02 a0 00 bc")
        self.assertIn("send_chunk_command: 10 00 00 00 00 00 00 10", lines)
        self.assertEqual(lines[-1], "return_code: 0x00000000")

    def test_retrieve_full_alpha_word_text_flow_command_prints_direct_usb_stack(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                [
                    "retrieve-full-alphaword-text-flow",
                    "2",
                    "0xa000",
                    "2",
                    "--retrieved-length",
                    "0x1234",
                ]
            )

        self.assertEqual(exit_code, 0)
        lines = stdout.getvalue().splitlines()
        self.assertEqual(lines[0], "controller: RetrieveFullAlphaWordText")
        self.assertIn("transport: DirectUsbRetrieveAppletFileData", lines)
        self.assertIn("retrieve_file slot=2: 12 08 00 00 02 a0 00 bc", lines)
        self.assertEqual(lines[-1], "return_code: 0x00")

    def test_retrieve_all_alpha_word_slots_flow_command_prints_skip_and_return_code(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                [
                    "retrieve-all-alphaword-slots-flow",
                    "0xa000",
                    "--file-slots",
                    "1",
                    "2",
                    "3",
                    "--initial-statuses",
                    "1=2",
                    "2=0",
                    "3=2",
                ]
            )

        self.assertEqual(exit_code, 0)
        lines = stdout.getvalue().splitlines()
        self.assertEqual(lines[0], "controller: RetrieveAllAlphaWordSlotsForDevice")
        self.assertIn("read_slot_status slot=1 status=2", lines)
        self.assertIn("skip_slot_already_loaded slot=1 status=2", lines)
        self.assertIn("retrieve_file slot=2: 12 08 00 00 02 a0 00 bc", lines)
        self.assertEqual(lines[-1], "return_code: 0x00")

    def test_get_print_preview_flow_command_prints_controller_and_cache_steps(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["alphaword-get-print-flow", "preview-all", "0xa000", "1", "2"])

        self.assertEqual(exit_code, 0)
        lines = stdout.getvalue().splitlines()
        self.assertEqual(lines[0], "controller: RefreshAlphaWordPreviewCacheForTreeItem")
        self.assertEqual(lines[1], "reset_connection: 3f ff 00 72 65 73 65 74")
        self.assertEqual(lines[2], "switch_to_updater: 3f 53 77 74 63 68 00 00")
        self.assertEqual(lines[3], "list_applets slot=1: 04 00 00 00 00 00 07 0b")
        self.assertEqual(lines[7], "cache_preview_text slot=1 status=1")
        self.assertEqual(lines[-1], "cache_preview_size slot=2 status=1")

    def test_get_print_full_selected_flow_command_prints_selected_controller(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["alphaword-get-print-flow", "full-selected", "0xa000", "2", "4"])

        self.assertEqual(exit_code, 0)
        lines = stdout.getvalue().splitlines()
        self.assertEqual(lines[0], "controller: ExecuteGetPrintAlphaWordFlow")
        self.assertEqual(lines[1], "controller: RetrieveSelectedAlphaWordSlotsForDevice")
        self.assertIn("retrieve_file slot=2: 12 08 00 00 02 a0 00 bc", lines)
        self.assertIn("get_file_name slot=4", lines)


if __name__ == "__main__":
    unittest.main()
