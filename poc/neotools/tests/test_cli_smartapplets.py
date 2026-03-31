import io
import unittest
from contextlib import redirect_stdout

from neotools import main


class SmartAppletCliTests(unittest.TestCase):
    def test_smartapplet_retrieve_plan_prints_expected_steps(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["smartapplet-retrieve-plan", "0xa123"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "reset_connection: 3f ff 00 72 65 73 65 74",
                "switch_to_updater: 3f 53 77 74 63 68 00 00",
                "retrieve_applet: 0f 00 00 00 00 a1 23 d3",
                "retrieve_chunk: 10 00 00 00 00 00 00 10",
            ],
        )

    def test_smartapplet_add_plan_prints_expected_steps(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["smartapplet-add-plan", "0x12345678", "0x9abc", "41 42 43 44 45"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "reset_connection: 3f ff 00 72 65 73 65 74",
                "switch_to_updater: 3f 53 77 74 63 68 00 00",
                "add_applet_begin: 06 12 34 56 78 9a bc 70",
                "add_applet_chunk_handshake: 02 00 00 00 05 01 4f 57",
                "add_applet_chunk_data: 41 42 43 44 45",
                "add_applet_chunk_commit: ff 00 00 00 00 00 00 ff",
                "program_applet: 0b 00 00 00 00 00 00 0b",
                "finalize_applet_update: 07 00 00 00 00 00 00 07",
            ],
        )


if __name__ == "__main__":
    unittest.main()
