from unittest import TestCase

from neotools.alphawordplus_runtime import (
    describe_namespace2_command,
    describe_namespace2_control_selector,
    describe_namespace2_stream_state,
)


class AlphaWordPlusRuntimeTests(TestCase):
    def test_namespace2_command_descriptions_cover_main_edit_path(self) -> None:
        command = describe_namespace2_command(0x20002)
        self.assertEqual(command.name, "write_payload")
        self.assertIn("typed", command.summary)

    def test_namespace2_control_selector_descriptions_cover_known_values(self) -> None:
        selector = describe_namespace2_control_selector(0x84)
        self.assertEqual(selector.name, "begin_chunked_readback")
        self.assertIn("readback", selector.summary)

        selector = describe_namespace2_control_selector(0x87)
        self.assertEqual(selector.name, "validate_span_reaches_file_end")
        self.assertIn("file size", selector.summary)

        selector = describe_namespace2_control_selector(0x90)
        self.assertEqual(selector.name, "query_current_span_end")
        self.assertIn("end position", selector.summary)

        selector = describe_namespace2_control_selector(0x88)
        self.assertEqual(selector.name, "query_current_slot")

    def test_namespace2_stream_state_descriptions_cover_observed_values(self) -> None:
        self.assertEqual(describe_namespace2_stream_state(0).name, "idle")
        self.assertEqual(describe_namespace2_stream_state(3).name, "chunked_readback")
        self.assertEqual(describe_namespace2_stream_state(6).name, "control_selector")
