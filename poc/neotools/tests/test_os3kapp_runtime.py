from pathlib import Path
import unittest

from neotools.os3kapp_format import parse_os3kapp_image
from neotools.os3kapp_runtime import (
    build_minimal_smartapplet_image,
    build_os3kapp_entry_abi,
    decompose_os3kapp_command,
    describe_known_applet_command_prototype,
    describe_known_applet_payload_subcommand_prototype,
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
        self.assertEqual(image.body[-6:-4], bytes.fromhex("4e 75"))
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))
        self.assertEqual(abi.shutdown_status, 7)

    def test_build_minimal_smartapplet_image_can_emit_direct_mode_callback_experiment(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA123,
            name="Direct USB Test",
            version_major_bcd=0x01,
            version_minor_bcd=0x01,
            direct_mode_callback=0x00410B26,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.version_major, 1)
        self.assertEqual(image.metadata.version_minor, 1)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 18"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 41 0b 26"), image.body)
        self.assertEqual(image.body[-6:-4], bytes.fromhex("4e 75"))
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))

    def test_build_minimal_smartapplet_image_can_emit_direct_mode_command_handler(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA124,
            name="USB Direct",
            version_major_bcd=0x01,
            version_minor_bcd=0x02,
            direct_mode_callback=0x00410B26,
            direct_mode_command_handler=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA124)
        self.assertEqual(image.metadata.version_minor, 2)
        self.assertIn(bytes.fromhex("02 81 00 ff 00 00"), image.body)
        self.assertIn(bytes.fromhex("0c 81 00 04 00 00"), image.body)
        self.assertIn(bytes.fromhex("a0 00"), image.body)
        self.assertIn(bytes.fromhex("a0 04"), image.body)
        self.assertIn(bytes.fromhex("a0 14"), image.body)
        self.assertIn(bytes.fromhex("a0 98"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 41 0b 26"), image.body)
        self.assertIn(b"Opening direct USB...\x00", image.body)
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))

    def test_build_minimal_smartapplet_image_can_emit_stay_open_init_screen(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA124,
            name="USB Direct",
            version_major_bcd=0x01,
            version_minor_bcd=0x03,
            stay_open_on_init=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.version_minor, 3)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 18"), image.body)
        self.assertIn(bytes.fromhex("a0 00"), image.body)
        self.assertIn(bytes.fromhex("a0 04"), image.body)
        self.assertIn(bytes.fromhex("a0 14"), image.body)
        self.assertIn(bytes.fromhex("a0 98"), image.body)
        self.assertIn(bytes.fromhex("a2 5c 60 fc"), image.body)
        self.assertIn(b"USB Direct open\x00", image.body)
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))

    def test_build_minimal_smartapplet_image_can_emit_calculator_style_menu_handler(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA126,
            name="USB Calc Test",
            version_major_bcd=0x01,
            version_minor_bcd=0x04,
            calculator_style_menu=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA126)
        self.assertEqual(image.metadata.version_minor, 4)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 01"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 02"), image.body)
        self.assertIn(bytes.fromhex("72 01 20 81"), image.body)
        self.assertIn(bytes.fromhex("72 04 24 81 20 81"), image.body)
        self.assertIn(b"USB Direct open\x00", image.body)
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))

    def test_build_minimal_smartapplet_image_can_emit_draw_on_any_command_probe(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA127,
            name="USB Any Cmd",
            version_major_bcd=0x01,
            version_minor_bcd=0x05,
            draw_on_any_command=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA127)
        self.assertEqual(image.metadata.version_minor, 5)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 18"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 19"), image.body)
        self.assertIn(bytes.fromhex("66 08 72 07 20 81 4e 75"), image.body)
        self.assertIn(bytes.fromhex("a0 00"), image.body)
        self.assertIn(bytes.fromhex("a0 04"), image.body)
        self.assertIn(bytes.fromhex("70 55 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 53 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 42 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("a0 10 a0 98 a2 5c"), image.body)
        self.assertIn(bytes.fromhex("a0 98"), image.body)
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))

    def test_build_minimal_smartapplet_image_can_emit_menu_command_screen_probe(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA129,
            name="USB Menu Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x08,
            draw_on_menu_command=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA129)
        self.assertEqual(image.metadata.version_minor, 8)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 19"), image.body)
        self.assertIn(bytes.fromhex("a0 00"), image.body)
        self.assertIn(bytes.fromhex("70 55 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 53 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 42 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("a0 10 a0 98 a2 5c"), image.body)
        self.assertIn(bytes.fromhex("60 fa"), image.body)
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))

    def test_menu_command_screen_probe_can_emit_usb_event_direct_handler(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA129,
            name="USB Menu Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x08,
            draw_on_menu_command=True,
            direct_mode_callback=0x00410B26,
        )

        image = parse_os3kapp_image(raw)

        self.assertIn(bytes.fromhex("0c 80 00 00 00 20"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 21"), image.body)
        self.assertIn(bytes.fromhex("70 44 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 49 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 52 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 41 0b 26"), image.body)

    def test_menu_command_screen_probe_can_arm_system_direct_callback(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA129,
            name="USB Menu Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x09,
            draw_on_menu_command=True,
            arm_direct_on_menu=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.version_minor, 9)
        self.assertIn(bytes.fromhex("42 a7"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 42 6b b0"), image.body)
        self.assertIn(bytes.fromhex("13 fc 00 00 00 00 04 44"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 41 2c 82"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 41 09 ca"), image.body)
        self.assertIn(bytes.fromhex("48 79 00 01 11 11"), image.body)
        self.assertIn(bytes.fromhex("48 78 25 80"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 42 4f b0"), image.body)
        self.assertIn(bytes.fromhex("48 79 00 41 0b 26"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 42 4f 66"), image.body)
        self.assertIn(bytes.fromhex("4f ef 00 10"), image.body)
        self.assertIn(bytes.fromhex("70 41 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 52 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4d 2f 00 61 00"), image.body)

    def test_menu_command_screen_probe_can_emit_host_usb_message_handler(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA129,
            name="USB Menu Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x10,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.version_minor, 10)
        for command in (
            0x00000020,
            0x00000021,
            0x00000026,
            0x00010001,
            0x00020001,
            0x00010003,
            0x00010006,
            0x00020002,
            0x00020006,
            0x0002011F,
        ):
            self.assertIn(bytes.fromhex("0c 80") + command.to_bytes(4, "big"), image.body)
        self.assertIn(bytes.fromhex("70 48 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4f 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 53 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 54 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4c 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 49 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4e 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4b 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("72 11 20 81"), image.body)
        self.assertIn(bytes.fromhex("72 04 20 81"), image.body)
        self.assertIn(bytes.fromhex("22 3c 00 00 a1 29 20 81 4e 75"), image.body)

    def test_menu_command_screen_probe_can_emit_alphaword_write_metadata(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA129,
            name="USB Menu Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x11,
            flags_raw=0xFF0000CE,
            extra_memory_size=0x2000,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
            alphaword_write_metadata=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.version_minor, 11)
        self.assertEqual(image.metadata.flags_raw, 0xFF0000CE)
        self.assertEqual(image.metadata.extra_memory_size, 0x2000)
        self.assertNotEqual(image.info_table_offset, 0)
        self.assertEqual(raw[-4:], bytes.fromhex("ca fe fe ed"))
        self.assertEqual(
            [(record.group, record.key, record.text) for record in image.info_records],
            [(0x0105, 0x100B, "write")]
            + [(0xC001, key, "write") for key in range(0x8011, 0x8019)],
        )

    def test_menu_command_screen_probe_can_emit_alphaword_state_machine_probe(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA129,
            name="USB Menu Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x12,
            flags_raw=0xFF0000CE,
            base_memory_size=0x240,
            extra_memory_size=0x2000,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
            alphaword_write_metadata=True,
            alphaword_state_machine_probe=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.version_minor, 12)
        self.assertEqual(image.metadata.header.base_memory_size, 0x240)
        self.assertEqual(image.metadata.extra_memory_size, 0x2000)
        self.assertIn(bytes.fromhex("0c 80 00 03 00 01"), image.body)
        self.assertIn(bytes.fromhex("28 3c 00 00 01 43 1b bc 00 01 48 00"), image.body)
        self.assertIn(bytes.fromhex("28 3c 00 00 01 43 42 35 48 00"), image.body)
        self.assertIn(bytes.fromhex("28 3c 00 00 00 bc 42 35 48 00"), image.body)

    def test_menu_command_screen_probe_can_emit_safe_alphaword_init_command_probe(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA12B,
            name="USB Init Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x13,
            flags_raw=0xFF0000CE,
            extra_memory_size=0x2000,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
            alphaword_write_metadata=True,
            alphaword_init_command_probe=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA12B)
        self.assertEqual(image.metadata.version_minor, 13)
        self.assertIn(bytes.fromhex("0c 80 00 01 00 01"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 03 00 01"), image.body)
        self.assertIn(bytes.fromhex("70 4e 2f 00"), image.body)
        self.assertIn(bytes.fromhex("70 31 2f 00"), image.body)
        self.assertIn(bytes.fromhex("70 33 2f 00"), image.body)
        self.assertNotIn(bytes.fromhex("1b bc 00 01 48 00"), image.body)
        self.assertNotIn(bytes.fromhex("42 35 48 00"), image.body)

    def test_menu_command_screen_probe_can_emit_alphaword_init_fault_probe(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA12C,
            name="USB Fault Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x14,
            flags_raw=0xFF0000CE,
            extra_memory_size=0x2000,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
            alphaword_write_metadata=True,
            alphaword_init_fault_probe=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA12C)
        self.assertEqual(image.metadata.version_minor, 14)
        self.assertIn(bytes.fromhex("20 7c 00 58 10 01 22 10"), image.body)
        self.assertIn(bytes.fromhex("20 7c 00 58 30 01 22 10"), image.body)
        self.assertIn(bytes.fromhex("20 7c 00 58 20 01 22 10"), image.body)
        self.assertIn(bytes.fromhex("20 7c 00 58 f0 0d 22 10"), image.body)
        self.assertNotIn(bytes.fromhex("1b bc 00 01 48 00"), image.body)

    def test_menu_command_screen_probe_can_emit_silent_30001_probe(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA12D,
            name="USB Silent Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x15,
            flags_raw=0xFF0000CE,
            extra_memory_size=0x2000,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
            alphaword_write_metadata=True,
            alphaword_init_fault_probe=True,
            alphaword_silent_init_probe=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA12D)
        self.assertEqual(image.metadata.version_minor, 15)
        self.assertIn(bytes.fromhex("0c 80 00 03 00 01"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 30 01 22 10"), image.body)
        self.assertIn(bytes.fromhex("20 7c 00 58 10 01 22 10"), image.body)
        self.assertIn(bytes.fromhex("20 7c 00 58 f0 0d 22 10"), image.body)
        self.assertIn(bytes.fromhex("72 11 20 81 4e 75"), image.body)

    def test_menu_command_screen_probe_can_call_direct_callback_on_30001(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA12E,
            name="USB Switch Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x16,
            flags_raw=0xFF0000CE,
            extra_memory_size=0x2000,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
            alphaword_write_metadata=True,
            alphaword_init_fault_probe=True,
            alphaword_switch_on_init_probe=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA12E)
        self.assertEqual(image.metadata.version_minor, 16)
        self.assertIn(bytes.fromhex("0c 80 00 03 00 01"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 41 0b 26 72 11 20 81 4e 75"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 30 01 22 10"), image.body)
        self.assertIn(bytes.fromhex("20 7c 00 58 10 01 22 10"), image.body)
        self.assertIn(bytes.fromhex("20 7c 00 58 f0 0d 22 10"), image.body)

    def test_menu_command_screen_probe_can_call_hid_completion_path_on_30001(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA12F,
            name="USB HID Complete",
            version_major_bcd=0x01,
            version_minor_bcd=0x17,
            flags_raw=0xFF0000CE,
            extra_memory_size=0x2000,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
            alphaword_write_metadata=True,
            alphaword_init_fault_probe=True,
            alphaword_hid_complete_switch_probe=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA12F)
        self.assertEqual(image.metadata.version_minor, 17)
        self.assertIn(bytes.fromhex("4e b9 00 41 f9 a0"), image.body)
        self.assertIn(bytes.fromhex("13 fc 00 01 00 01 3c f9"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 44 04 4e"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 44 04 7c"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 30 01 22 10"), image.body)

    def test_menu_command_screen_probe_can_emit_alpha_usb_production_applet(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA130,
            name="Alpha USB",
            version_major_bcd=0x01,
            version_minor_bcd=0x18,
            flags_raw=0xFF0000CE,
            extra_memory_size=0x2000,
            draw_on_menu_command=True,
            host_usb_message_handler=True,
            alphaword_write_metadata=True,
            alpha_usb_production=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA130)
        self.assertEqual(image.metadata.name, "Alpha USB")
        self.assertEqual(image.metadata.version_minor, 18)
        self.assertIn(bytes.fromhex("13 fc 00 01 00 01 3c f9"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 44 04 4e"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 44 04 7c"), image.body)
        self.assertIn(bytes.fromhex("72 04 20 81 4e 75"), image.body)
        self.assertIn(bytes.fromhex("72 11 20 81 4e 75"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 10 01 22 10"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 30 01 22 10"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 f0 0d 22 10"), image.body)

    def test_build_minimal_smartapplet_image_can_emit_command_fault_probe(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA128,
            name="USB Cmd Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x06,
            command_fault_probe=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.applet_id, 0xA128)
        self.assertEqual(image.metadata.version_minor, 6)
        self.assertIn(bytes.fromhex("20 2f 00 04"), image.body)
        self.assertIn(bytes.fromhex("02 81 00 00 ff ff"), image.body)
        self.assertIn(bytes.fromhex("00 81 00 58 00 00"), image.body)
        self.assertIn(bytes.fromhex("22 10"), image.body)
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))

    def test_build_minimal_smartapplet_image_can_emit_post_shutdown_command_fault_probe(self) -> None:
        raw = build_minimal_smartapplet_image(
            applet_id=0xA128,
            name="USB Cmd Probe",
            version_major_bcd=0x01,
            version_minor_bcd=0x07,
            command_fault_after_shutdown_probe=True,
        )

        image = parse_os3kapp_image(raw)

        self.assertEqual(image.metadata.version_minor, 7)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 19 66 06 72 07 20 81 4e 75"), image.body)
        self.assertIn(bytes.fromhex("00 81 00 58 00 00"), image.body)
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))

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
        a378_stub = next(stub for block in trap_blocks for stub in block.stubs if stub.opcode == 0xA378)
        self.assertEqual(a378_stub.inferred_name, "render_formatted_pending_text")

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
        cursor = describe_known_trap_prototype(0xA008)
        row_span = describe_known_trap_prototype(0xA020)
        text = describe_known_trap_prototype(0xA014)
        key = describe_known_trap_prototype(0xA094)
        flush = describe_known_trap_prototype(0xA098)
        render = describe_known_trap_prototype(0xA0F0)
        slot = describe_known_trap_prototype(0xA0F4)
        allowed_key = describe_known_trap_prototype(0xA0F8)
        input_char = describe_known_trap_prototype(0xA14C)
        builder = describe_known_trap_prototype(0xA190)
        metric = describe_known_trap_prototype(0xA1C8)
        commit = describe_known_trap_prototype(0xA1CC)
        file_name = describe_known_trap_prototype(0xA1D4)
        iterator = describe_known_trap_prototype(0xA1E0)
        slot_map = describe_known_trap_prototype(0xA1EC)
        chooser_builder = describe_known_trap_prototype(0xA1F8)
        chooser_row_break = describe_known_trap_prototype(0xA1FC)
        chooser_row = describe_known_trap_prototype(0xA200)
        chooser_begin = describe_known_trap_prototype(0xA208)
        chooser_event = describe_known_trap_prototype(0xA20C)
        chooser_selector = describe_known_trap_prototype(0xA210)
        chooser_value = describe_known_trap_prototype(0xA214)
        edit_commit = describe_known_trap_prototype(0xA2BC)
        finalize_context = describe_known_trap_prototype(0xA2C0)
        begin_replacement = describe_known_trap_prototype(0xA2CC)
        replacement_status = describe_known_trap_prototype(0xA2D0)
        reset_search = describe_known_trap_prototype(0xA2D4)
        char_stream = describe_known_trap_prototype(0xA2D8)
        switch_file = describe_known_trap_prototype(0xA2DC)
        workspace_status = describe_known_trap_prototype(0xA2EC)
        init_empty = describe_known_trap_prototype(0xA2FC)
        shared_available = describe_known_trap_prototype(0xA364)
        render_formatted = describe_known_trap_prototype(0xA378)
        format_pending = describe_known_trap_prototype(0xA380)
        shared_state = describe_known_trap_prototype(0xA36C)
        shared_disabled = describe_known_trap_prototype(0xA388)
        pending_length = describe_known_trap_prototype(0xA398)
        shared = describe_known_trap_prototype(0xA390)

        self.assertEqual(layout.name, "set_text_row_column_width")
        self.assertEqual(layout.stack_argument_count, 3)
        self.assertEqual(layout.return_kind, "none")
        self.assertEqual(cursor.name, "get_text_row_col")
        self.assertEqual(cursor.stack_argument_count, 2)
        self.assertEqual(row_span.name, "prepare_text_row_span")
        self.assertEqual(row_span.stack_argument_count, 3)
        self.assertEqual(text.stack_argument_count, 1)
        self.assertEqual(text.return_kind, "none")
        self.assertEqual(key.stack_argument_count, 0)
        self.assertEqual(key.return_kind, "value")
        self.assertEqual(flush.name, "flush_text_frame")
        self.assertEqual(flush.stack_argument_count, 0)
        self.assertEqual(render.name, "render_wrapped_text_block")
        self.assertEqual(render.stack_argument_count, 5)
        self.assertEqual(slot.name, "define_text_layout_slot")
        self.assertEqual(slot.stack_argument_count, 6)
        self.assertEqual(allowed_key.name, "register_allowed_key")
        self.assertEqual(allowed_key.stack_argument_count, 1)
        self.assertEqual(input_char.name, "read_text_input_char")
        self.assertEqual(input_char.return_kind, "value")
        self.assertEqual(builder.name, "begin_output_builder")
        self.assertEqual(builder.stack_argument_count, 3)
        self.assertEqual(builder.return_kind, "none")
        self.assertEqual(metric.name, "query_object_metric")
        self.assertEqual(commit.name, "commit_editable_buffer")
        self.assertEqual(commit.return_kind, "none")
        self.assertEqual(file_name.name, "assign_current_file_name_from_pending_text")
        self.assertEqual(file_name.return_kind, "none")
        self.assertEqual(iterator.name, "query_advanced_file_iterator_ordinal")
        self.assertEqual(iterator.return_kind, "value")
        self.assertEqual(slot_map.name, "sync_current_slot_map_entry")
        self.assertEqual(chooser_builder.name, "begin_chooser_row_builder")
        self.assertEqual(chooser_builder.stack_argument_count, 0)
        self.assertEqual(chooser_row_break.name, "advance_current_output_row")
        self.assertEqual(chooser_row_break.return_kind, "value")
        self.assertEqual(chooser_row.name, "append_current_chooser_row")
        self.assertEqual(chooser_row.return_kind, "none")
        self.assertEqual(chooser_begin.name, "begin_chooser_input_session")
        self.assertEqual(chooser_event.name, "read_chooser_event_code")
        self.assertEqual(chooser_event.return_kind, "value")
        self.assertEqual(chooser_selector.name, "read_chooser_action_selector")
        self.assertEqual(chooser_value.name, "read_chooser_selection_value")
        self.assertEqual(edit_commit.name, "commit_current_file_edit_session")
        self.assertEqual(finalize_context.name, "finalize_current_file_context")
        self.assertEqual(begin_replacement.name, "begin_current_replacement")
        self.assertEqual(replacement_status.name, "query_current_replacement_status")
        self.assertEqual(reset_search.name, "reset_current_search_state")
        self.assertEqual(char_stream.name, "read_next_char_stream_unit")
        self.assertEqual(switch_file.name, "switch_to_current_file_context")
        self.assertEqual(workspace_status.name, "query_current_workspace_file_status")
        self.assertEqual(init_empty.name, "initialize_empty_workspace_file")
        self.assertEqual(shared_available.name, "query_active_service_available")
        self.assertEqual(shared_available.return_kind, "value")
        self.assertEqual(render_formatted.name, "render_formatted_pending_text")
        self.assertEqual(render_formatted.return_kind, "none")
        self.assertEqual(format_pending.name, "format_pending_text")
        self.assertEqual(shared_state.name, "query_active_service_status")
        self.assertEqual(shared_disabled.name, "query_active_service_disabled_state")
        self.assertEqual(pending_length.name, "query_pending_text_length")
        self.assertEqual(pending_length.return_kind, "value")
        self.assertEqual(shared.name, "shared_runtime_a390")

    def test_alphaquiz_command_prototype_exposes_namespace_dispatch_contract(self) -> None:
        title = describe_known_applet_command_prototype("alphaquiz", 0x60001)
        delta = describe_known_applet_command_prototype("alphaquiz", 0x6000D)
        redraw = describe_known_applet_command_prototype("alphaquiz", 0x60010)
        help_prompt = describe_known_applet_command_prototype("alphaquiz", 0x60020)

        self.assertEqual(title.selector_byte, 0x06)
        self.assertEqual(title.handler_name, "HandleAlphaQuizNamespace6Commands")
        self.assertEqual(title.status_code, 0x11)
        self.assertIn("copies up to 0x27 input bytes", title.notes)
        self.assertEqual(delta.selector_byte, 0x06)
        self.assertEqual(delta.response_word_count, 1)
        self.assertIn("writes one 32-bit value", delta.notes)
        self.assertEqual(redraw.status_code, 0)
        self.assertIn("redraw", redraw.notes)
        self.assertEqual(help_prompt.status_code, 0)
        self.assertIn("only when the first input byte is ASCII 'H'", help_prompt.notes)

    def test_alphawordplus_command_prototype_exposes_namespace_dispatch_contract(self) -> None:
        namespace1_init = describe_known_applet_command_prototype("alphawordplus", 0x10001)
        current_slot = describe_known_applet_command_prototype("alphawordplus", 0x10004)
        namespace2_stream = describe_known_applet_command_prototype("alphawordplus", 0x20002)
        namespace4_payload = describe_known_applet_command_prototype("alphawordplus", 0x40002)
        namespace7_payload = describe_known_applet_command_prototype("alphawordplus", 0x70002)

        self.assertEqual(namespace1_init.selector_byte, 0x01)
        self.assertEqual(namespace1_init.handler_name, "HandleAlphaWordNamespace1Commands")
        self.assertEqual(namespace1_init.status_code, 0x11)
        self.assertIn("resets the namespace-1 command-stream state", namespace1_init.notes)
        self.assertEqual(current_slot.handler_name, "HandleAlphaWordNamespace1Commands")
        self.assertIn("currently selected AlphaWord slot number", current_slot.notes)
        self.assertEqual(namespace2_stream.handler_name, "HandleAlphaWordNamespace2Commands")
        self.assertIn("incoming transferred bytes", namespace2_stream.notes)
        self.assertEqual(namespace4_payload.handler_name, "HandleAlphaWordNamespace4Commands")
        self.assertIn("byte-payload helper", namespace4_payload.notes)
        self.assertEqual(namespace7_payload.handler_name, "HandleAlphaWordNamespace7Commands")
        self.assertIn("encoded transfer machinery", namespace7_payload.notes)

    def test_alphaquiz_payload_subcommand_prototype_exposes_status_and_response_shape(self) -> None:
        ready = describe_known_applet_payload_subcommand_prototype("alphaquiz", 0x50002, 0x1D)
        helper = describe_known_applet_payload_subcommand_prototype("alphaquiz", 0x40002, 0x1A)
        fallback = describe_known_applet_payload_subcommand_prototype("alphaquiz", 0x50005, 0x00)

        self.assertEqual(ready.status_code, 4)
        self.assertEqual(ready.response_length, 2)
        self.assertIn("0x5d 0x02", ready.notes)
        self.assertIsNone(helper.status_code)
        self.assertEqual(helper.response_length, -1)
        self.assertIn("status 4 only when the helper returns a nonzero output length", helper.notes)
        self.assertEqual(fallback.status_code, 4)
        self.assertEqual(fallback.response_length, 1)
        self.assertIn("| 0x80", fallback.notes)


if __name__ == "__main__":
    unittest.main()
