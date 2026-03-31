from pathlib import Path
import unittest

from neotools.os3kapp_format import parse_os3kapp_image
from neotools.os3kapp_runtime import (
    build_minimal_smartapplet_image,
    build_os3kapp_entry_abi,
    decompose_os3kapp_command,
    describe_known_trap_prototype,
    scan_os3kapp_trap_blocks,
)


FIXTURE_DIR = Path("/Users/jakubkolcar/customs/neo-re/analysis/cab")


class Os3kAppRuntimeTests(unittest.TestCase):
    def test_build_minimal_smartapplet_image_emits_parseable_container_and_stub(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA123,
            name="Custom Test Applet",
            version_major_bcd=0x01,
            version_minor_bcd=0x00,
        )

        image = parse_os3kapp_image(raw)
        abi = build_os3kapp_entry_abi(image)

        self.assertEqual(image.metadata.applet_id, 0xA123)
        self.assertEqual(image.entry_offset, 0x94)
        self.assertEqual(image.loader_stub, b"")
        self.assertEqual(image.body_prefix_words, (0x94, 0, 1, 2))
        self.assertEqual(image.body[0x10:0x14], bytes.fromhex("20 6f 00 0c"))
        self.assertEqual(image.body[0x16:0x1a], bytes.fromhex("20 2f 00 04"))
        self.assertEqual(image.body[-2:], bytes.fromhex("4e 75"))
        self.assertEqual(abi.shutdown_status, 7)

    def test_calculator_entry_abi_matches_recovered_dispatch_contract(self) -> None:
        image = parse_os3kapp_image((FIXTURE_DIR / "calculator.os3kapp").read_bytes())

        abi = build_os3kapp_entry_abi(image)

        self.assertEqual(abi.entry_offset, 0x168)
        self.assertEqual(abi.loader_stub_length, 0x168 - 0x94)
        self.assertEqual(abi.init_opcode, 0x18)
        self.assertEqual(abi.shutdown_opcode, 0x19)
        self.assertEqual(abi.shutdown_status, 7)
        self.assertEqual(abi.call_block_words, 5)
        self.assertEqual(abi.input_length_index, 0)
        self.assertEqual(abi.input_pointer_index, 1)
        self.assertEqual(abi.output_capacity_index, 2)
        self.assertEqual(abi.output_length_index, 3)
        self.assertEqual(abi.output_buffer_pointer_index, 4)

    def test_command_decomposer_exposes_outer_namespace_bytes(self) -> None:
        command = decompose_os3kapp_command(0x12040000)

        self.assertEqual(command.namespace_byte, 0x12)
        self.assertEqual(command.selector_byte, 0x04)
        self.assertEqual(command.low_word, 0x0000)
        self.assertTrue(command.uses_custom_dispatch)

    def test_lifecycle_command_decomposer_recognizes_init_and_shutdown(self) -> None:
        init_command = decompose_os3kapp_command(0x18)
        shutdown_command = decompose_os3kapp_command(0x19)

        self.assertFalse(init_command.uses_custom_dispatch)
        self.assertEqual(init_command.lifecycle_name, "initialize")
        self.assertEqual(shutdown_command.lifecycle_name, "shutdown")

    def test_minimal_custom_applet_has_no_imported_trap_blocks(self) -> None:
        image = parse_os3kapp_image(
            build_minimal_smartapplet_image(
                applet_id=0xA123,
                name="Custom Test Applet",
                version_major_bcd=0x01,
                version_minor_bcd=0x00,
            )
        )

        self.assertEqual(scan_os3kapp_trap_blocks(image), ())

    def test_calculator_imports_dense_a_line_trap_blocks(self) -> None:
        image = parse_os3kapp_image((FIXTURE_DIR / "calculator.os3kapp").read_bytes())

        trap_blocks = scan_os3kapp_trap_blocks(image)

        self.assertEqual(trap_blocks[0].start_file_offset, 0x34CE)
        self.assertEqual(trap_blocks[0].stubs[0].opcode, 0xA000)
        self.assertEqual(trap_blocks[0].stubs[0].inferred_name, "clear_text_screen")
        self.assertEqual(trap_blocks[-1].stubs[-1].opcode, 0xA3B0)
        a368_stub = next(stub for block in trap_blocks for stub in block.stubs if stub.opcode == 0xA368)
        self.assertEqual(a368_stub.file_offset, 0x365E)
        self.assertEqual(a368_stub.inferred_name, "calculator_runtime_init_slot_a")

    def test_cross_sample_runtime_import_pattern_matches_other_shipped_applets(self) -> None:
        alphaquiz = parse_os3kapp_image((FIXTURE_DIR / "alphaquiz.os3kapp").read_bytes())
        spellcheck = parse_os3kapp_image((FIXTURE_DIR / "spellcheck_small_usa.os3kapp").read_bytes())
        neofont = parse_os3kapp_image((FIXTURE_DIR / "neofontmedium.os3kapp").read_bytes())

        alphaquiz_blocks = scan_os3kapp_trap_blocks(alphaquiz)
        spellcheck_blocks = scan_os3kapp_trap_blocks(spellcheck)

        self.assertEqual(alphaquiz.body_prefix_words, (0x0E20, 0, 1, 2))
        self.assertEqual(alphaquiz_blocks[0].stubs[0].opcode, 0xA000)
        self.assertEqual(alphaquiz_blocks[1].stubs[-1].opcode, 0xA308)
        self.assertEqual(alphaquiz_blocks[2].stubs[-1].opcode, 0xA3B0)
        self.assertEqual(spellcheck_blocks[0].stubs[0].opcode, 0xA000)
        self.assertEqual(spellcheck_blocks[1].stubs[-1].opcode, 0xA308)
        self.assertGreaterEqual(spellcheck_blocks[2].stubs[-1].opcode, 0xA3B0)
        self.assertEqual(scan_os3kapp_trap_blocks(neofont), ())

    def test_known_trap_prototype_exposes_stack_arg_shape(self) -> None:
        layout = describe_known_trap_prototype(0xA004)
        text = describe_known_trap_prototype(0xA014)
        key = describe_known_trap_prototype(0xA094)

        self.assertEqual(layout.name, "set_text_row_column_width")
        self.assertEqual(layout.stack_argument_count, 3)
        self.assertEqual(layout.return_kind, "none")
        self.assertEqual(text.stack_argument_count, 1)
        self.assertEqual(text.return_kind, "none")
        self.assertEqual(key.stack_argument_count, 0)
        self.assertEqual(key.return_kind, "value")


if __name__ == "__main__":
    unittest.main()
