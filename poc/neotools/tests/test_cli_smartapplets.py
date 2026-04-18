import io
from pathlib import Path
import tempfile
import unittest
from contextlib import redirect_stdout

from neotools import main
from neotools.os3kapp_format import parse_os3kapp_image


FIXTURE_DIR = Path("/Users/jakubkolcar/customs/neo-re/analysis/cab")


def _drawn_ascii_from_moveq_text(body: bytes) -> bytes:
    return bytes(
        body[index + 1]
        for index in range(len(body) - 3)
        if body[index] == 0x70 and 0x20 <= body[index + 1] <= 0x7E
        and body[index + 2 : index + 4] == bytes.fromhex("2f 00")
    )


class SmartAppletCliTests(unittest.TestCase):
    def test_build_benign_smartapplet_writes_menu_visible_image(self) -> None:
        output = io.StringIO()

        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "direct-usb-test.os3kapp"
            with redirect_stdout(output):
                exit_code = main(["build-benign-smartapplet", "--output", str(output_path)])

            raw = output_path.read_bytes()

        image = parse_os3kapp_image(raw)

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA123)
        self.assertEqual(image.metadata.name, "Direct USB Test")
        self.assertEqual(image.entry_offset, 0x94)
        self.assertEqual(image.body_prefix_words, (0x94, 0, 1, 2))
        self.assertEqual(image.body[-6:-4], bytes.fromhex("4e 75"))
        self.assertEqual(image.body[-4:], bytes.fromhex("ca fe fe ed"))
        self.assertIn("applet_id=0xa123 name=Direct USB Test\n", output.getvalue())

    def test_build_benign_smartapplet_can_emit_direct_callback_experiment(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "direct-usb-test.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--direct-mode-callback",
                    "0x00410b26",
                ]
            )

            image = parse_os3kapp_image(output_path.read_bytes())

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.version_minor, 1)
        self.assertIn(bytes.fromhex("4e b9 00 41 0b 26"), image.body)

    def test_build_benign_smartapplet_can_emit_direct_command_handler_experiment(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-direct.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa124",
                    "--name",
                    "USB Direct",
                    "--direct-mode-callback",
                    "0x00410b26",
                    "--direct-mode-command-handler",
                ]
            )

            image = parse_os3kapp_image(output_path.read_bytes())

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA124)
        self.assertEqual(image.metadata.name, "USB Direct")
        self.assertEqual(image.metadata.version_minor, 2)
        self.assertIn(bytes.fromhex("0c 81 00 04 00 00"), image.body)
        self.assertIn(b"Opening direct USB...\x00", image.body)

    def test_build_benign_smartapplet_can_emit_menu_command_screen_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-menu.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa129",
                    "--name",
                    "USB Menu Probe",
                    "--draw-on-menu-command",
                ]
            )

            image = parse_os3kapp_image(output_path.read_bytes())

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA129)
        self.assertEqual(image.metadata.name, "USB Menu Probe")
        self.assertEqual(image.metadata.version_minor, 8)
        self.assertIn(bytes.fromhex("70 55 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 53 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 42 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("a0 10 a0 98 a2 5c"), image.body)

    def test_build_benign_smartapplet_can_emit_direct_arm_menu_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-menu.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa129",
                    "--name",
                    "USB Menu Probe",
                    "--draw-on-menu-command",
                    "--arm-direct-on-menu",
                ]
            )

            image = parse_os3kapp_image(output_path.read_bytes())

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA129)
        self.assertEqual(image.metadata.name, "USB Menu Probe")
        self.assertEqual(image.metadata.version_minor, 9)
        self.assertIn(bytes.fromhex("4e b9 00 42 4f 66"), image.body)
        self.assertIn(bytes.fromhex("48 79 00 41 0b 26"), image.body)
        self.assertIn(bytes.fromhex("70 41 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 52 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4d 2f 00 61 00"), image.body)

    def test_build_benign_smartapplet_can_emit_host_usb_message_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-menu.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa129",
                    "--name",
                    "USB Menu Probe",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                ]
            )

            image = parse_os3kapp_image(output_path.read_bytes())

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA129)
        self.assertEqual(image.metadata.name, "USB Menu Probe")
        self.assertEqual(image.metadata.version_minor, 10)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 20"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 21"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 00 00 26"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 01 00 01"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 02 00 01"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 01 00 03"), image.body)
        self.assertIn(bytes.fromhex("0c 80 00 02 00 02"), image.body)
        self.assertIn(bytes.fromhex("70 48 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4f 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 53 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 54 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4c 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 49 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4e 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("70 4b 2f 00 61 00"), image.body)
        self.assertIn(bytes.fromhex("22 3c 00 00 a1 29 20 81 4e 75"), image.body)

    def test_build_benign_smartapplet_can_emit_alphaword_write_metadata_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-menu.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa129",
                    "--name",
                    "USB Menu Probe",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                    "--alphaword-write-metadata",
                ]
            )

            raw = output_path.read_bytes()
            image = parse_os3kapp_image(raw)

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.version_minor, 11)
        self.assertEqual(image.metadata.flags_raw, 0xFF0000CE)
        self.assertEqual(image.metadata.extra_memory_size, 0x2000)
        self.assertEqual(raw[-4:], bytes.fromhex("ca fe fe ed"))
        self.assertEqual(len(image.info_records), 9)
        self.assertIn((0xC001, 0x8011, "write"), [(r.group, r.key, r.text) for r in image.info_records])
        self.assertIn((0xC001, 0x8018, "write"), [(r.group, r.key, r.text) for r in image.info_records])

    def test_build_benign_smartapplet_can_emit_alphaword_state_machine_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-menu.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa129",
                    "--name",
                    "USB Menu Probe",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                    "--alphaword-write-metadata",
                    "--alphaword-state-machine-probe",
                ]
            )

            raw = output_path.read_bytes()
            image = parse_os3kapp_image(raw)

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.version_minor, 12)
        self.assertEqual(image.metadata.header.base_memory_size, 0x240)
        self.assertEqual(image.metadata.extra_memory_size, 0x2000)
        self.assertIn(bytes.fromhex("0c 80 00 03 00 01"), image.body)
        self.assertIn(bytes.fromhex("28 3c 00 00 01 43 1b bc 00 01 48 00"), image.body)
        self.assertEqual(raw[-4:], bytes.fromhex("ca fe fe ed"))

    def test_build_benign_smartapplet_can_emit_safe_alphaword_init_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-init.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa12b",
                    "--name",
                    "USB Init Probe",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                    "--alphaword-write-metadata",
                    "--alphaword-init-command-probe",
                ]
            )

            raw = output_path.read_bytes()
            image = parse_os3kapp_image(raw)

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA12B)
        self.assertEqual(image.metadata.version_minor, 13)
        self.assertEqual(image.metadata.extra_memory_size, 0x2000)
        self.assertIn(bytes.fromhex("0c 80 00 03 00 01"), image.body)
        self.assertNotIn(bytes.fromhex("1b bc 00 01 48 00"), image.body)

    def test_build_benign_smartapplet_can_emit_alphaword_init_fault_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-fault.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa12c",
                    "--name",
                    "USB Fault Probe",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                    "--alphaword-write-metadata",
                    "--alphaword-init-fault-probe",
                ]
            )

            raw = output_path.read_bytes()
            image = parse_os3kapp_image(raw)

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA12C)
        self.assertEqual(image.metadata.version_minor, 14)
        self.assertIn(bytes.fromhex("20 7c 00 58 10 01 22 10"), image.body)

    def test_build_benign_smartapplet_can_emit_silent_alphaword_init_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-silent.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa12d",
                    "--name",
                    "USB Silent Probe",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                    "--alphaword-write-metadata",
                    "--alphaword-init-fault-probe",
                    "--alphaword-silent-init-probe",
                ]
            )

            raw = output_path.read_bytes()
            image = parse_os3kapp_image(raw)

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA12D)
        self.assertEqual(image.metadata.version_minor, 15)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 30 01 22 10"), image.body)
        self.assertIn(bytes.fromhex("20 7c 00 58 f0 0d 22 10"), image.body)

    def test_build_benign_smartapplet_can_emit_switch_on_init_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-switch.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa12e",
                    "--name",
                    "USB Switch Probe",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                    "--alphaword-write-metadata",
                    "--alphaword-init-fault-probe",
                    "--alphaword-switch-on-init-probe",
                ]
            )

            raw = output_path.read_bytes()
            image = parse_os3kapp_image(raw)

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA12E)
        self.assertEqual(image.metadata.version_minor, 16)
        self.assertEqual(image.metadata.extra_memory_size, 0x2000)
        self.assertIn(bytes.fromhex("4e b9 00 41 0b 26 72 11 20 81 4e 75"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 30 01 22 10"), image.body)

    def test_build_benign_smartapplet_can_emit_hid_completion_switch_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "usb-hid-complete.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa12f",
                    "--name",
                    "USB HID Complete",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                    "--alphaword-write-metadata",
                    "--alphaword-init-fault-probe",
                    "--alphaword-hid-complete-switch-probe",
                ]
            )

            image = parse_os3kapp_image(output_path.read_bytes())

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA12F)
        self.assertEqual(image.metadata.version_minor, 17)
        self.assertIn(bytes.fromhex("13 fc 00 01 00 01 3c f9"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 44 04 4e"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 44 04 7c"), image.body)

    def test_build_benign_smartapplet_can_emit_alpha_usb_production_applet(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "alpha-usb.os3kapp"
            exit_code = main(
                [
                    "build-benign-smartapplet",
                    "--output",
                    str(output_path),
                    "--applet-id",
                    "0xa130",
                    "--name",
                    "Alpha USB",
                    "--draw-on-menu-command",
                    "--host-usb-message-handler",
                    "--alphaword-write-metadata",
                    "--alpha-usb-production",
                ]
            )

            image = parse_os3kapp_image(output_path.read_bytes())

        self.assertEqual(exit_code, 0)
        self.assertEqual(image.metadata.applet_id, 0xA130)
        self.assertEqual(image.metadata.name, "Alpha USB")
        self.assertEqual(image.metadata.version_minor, 19)
        drawn_text = _drawn_ascii_from_moveq_text(image.body)
        self.assertIn(b"Now connect the NEO", drawn_text)
        self.assertIn(b"to your computer or", drawn_text)
        self.assertIn(b"smartphone via USB.", drawn_text)
        self.assertIn(bytes.fromhex("13 fc 00 01 00 01 3c f9"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 44 04 4e"), image.body)
        self.assertIn(bytes.fromhex("4e b9 00 44 04 7c"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 30 01 22 10"), image.body)
        self.assertNotIn(bytes.fromhex("20 7c 00 58 f0 0d 22 10"), image.body)

    def test_smartapplet_header_prints_derived_add_applet_fields(self) -> None:
        output = io.StringIO()
        header_hex = (
            "c0 ff ee ad 00 01 a0 bc 00 00 0d 90 00 01 9f a4"
            " ff 00 00 ce a0 00 01 00"
            + " 00" * (0x80 - 0x18)
            + " 00 00 20 00"
        )

        with redirect_stdout(output):
            exit_code = main(["smartapplet-header", header_hex])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "magic=0xc0ffeead file_size=0x0001a0bc base_memory_size=0x00000d90 extra_memory_size=0x00002000 argument=0x0001a0bc trailing=0x2d90",
            ],
        )

    def test_smartapplet_add_plan_from_image_prints_derived_begin_packet(self) -> None:
        output = io.StringIO()
        image_hex = (
            "c0 ff ee ad 00 00 5f e0 00 00 05 6c 00 00 00 00"
            " ff 00 00 00 a0 02 01 00"
            + " 00" * (0x80 - 0x18)
            + " 00 00 00 00"
            + " 41 42 43 44 45"
        )

        with redirect_stdout(output):
            exit_code = main(["smartapplet-add-plan-from-image", image_hex])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines()[:4],
            [
                "reset_connection: 3f ff 00 72 65 73 65 74",
                "switch_to_updater: 3f 53 77 74 63 68 00 00",
                "add_applet_begin: 06 00 00 5f e0 05 6c b6",
                "add_applet_chunk_handshake: 02 00 00 00 89 07 fb 8d",
            ],
        )

    def test_smartapplet_metadata_prints_named_header_fields(self) -> None:
        output = io.StringIO()
        header = bytearray(0x84)
        header[0x00:0x04] = bytes.fromhex("c0 ff ee ad")
        header[0x04:0x08] = bytes.fromhex("00 01 a0 bc")
        header[0x08:0x0C] = bytes.fromhex("00 00 0d 90")
        header[0x0C:0x10] = bytes.fromhex("00 01 9f a4")
        header[0x10:0x14] = bytes.fromhex("ff 00 00 ce")
        header[0x14:0x18] = bytes.fromhex("a0 00 01 00")
        header[0x18:0x18 + len(b"AlphaWord Plus")] = b"AlphaWord Plus"
        header[0x3C] = 0x03
        header[0x3D] = 0x04
        header[0x3F] = 0x01
        header[0x80:0x84] = bytes.fromhex("00 00 20 00")

        with redirect_stdout(output):
            exit_code = main(["smartapplet-metadata", bytes(header).hex(" ")])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "applet_id=0xa000 version=3.4 name=AlphaWord Plus info_table_offset=0x00019fa4 applet_class=0x01 extra_memory_size=0x00002000",
            ],
        )

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

    def test_smartapplet_string_prints_known_resource_label(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["smartapplet-string", "0xf138"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(output.getvalue().splitlines(), ["Maximum File Size (in characters)"])

    def test_smartapplet_menu_prints_decoded_popup_menu_items(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["smartapplet-menu", "163"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "command_id=0x800e label=Startup",
                "command_id=0x800f label=Startup Lock",
                "command_id=0x8010 label=Remove",
                "command_id=0x8012 label=Get Info",
                "command_id=0x8013 label=Help",
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

    def test_os3kapp_image_prints_full_container_breakdown(self) -> None:
        output = io.StringIO()
        image_hex = (
            "c0 ff ee ad 00 00 00 a0 00 00 00 10 00 00 00 94"
            " ff 00 00 31 a0 02 01 00"
            + " 00" * (0x3F - 0x18)
            + " 01"
            + " 00" * (0x80 - 0x40)
            + " 00 00 00 00"
            + " 00 00 00 94 00 00 00 00 00 00 00 01 00 00 00 02"
            + " aa" * 0x0c
        )

        with redirect_stdout(output):
            exit_code = main(["os3kapp-image", image_hex])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "file_size=0x000000a0 applet_id=0xa002 applet_class=0x01 body_size=0x1c info_table_offset=0x00000094",
                "body_prefix_words=0x00000094,0x00000000,0x00000001,0x00000002",
                "info_records=0",
            ],
        )

    def test_os3kapp_entry_abi_prints_recovered_runtime_contract(self) -> None:
        output = io.StringIO()
        image_hex = (
            "c0 ff ee ad 00 00 00 a0 00 00 00 10 00 00 00 94"
            " ff 00 00 31 a0 02 01 00"
            + " 00" * (0x3F - 0x18)
            + " 01"
            + " 00" * (0x80 - 0x40)
            + " 00 00 00 00"
            + " 00 00 00 a0 00 00 00 00 00 00 00 01 00 00 00 02"
            + " aa" * 0x0c
        )

        with redirect_stdout(output):
            exit_code = main(["os3kapp-entry-abi", image_hex])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "entry_offset=0x000000a0 loader_stub_length=0xc init_opcode=0x18 shutdown_opcode=0x19 shutdown_status=0x00000007",
                "call_block_words=5 input_length_index=0 input_pointer_index=1 output_capacity_index=2 output_length_index=3 output_buffer_pointer_index=4",
            ],
        )

    def test_os3kapp_command_prints_namespace_and_selector_bytes(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["os3kapp-command", "0x12040000"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "raw=0x12040000 namespace_byte=0x12 selector_byte=0x04 low_word=0x0000 custom_dispatch=True lifecycle=none",
            ],
        )

    def test_os3kapp_traps_prints_dense_import_blocks(self) -> None:
        output = io.StringIO()
        image_hex = (FIXTURE_DIR / "calculator.os3kapp").read_bytes().hex(" ")

        with redirect_stdout(output):
            exit_code = main(["os3kapp-traps", image_hex])

        self.assertEqual(exit_code, 0)
        lines = output.getvalue().splitlines()
        self.assertEqual(lines[0], "block=0x34ce-0x34ee count=16 first=0xa000 last=0xa03c")
        self.assertIn("offset=0x34ce opcode=0xa000 family=0xa0 selector=0x00 name=clear_text_screen", lines)
        self.assertIn("offset=0x3666 opcode=0xa378 family=0xa3 selector=0x78 name=render_formatted_pending_text", lines)

    def test_os3kapp_trap_prototype_prints_known_stack_signature(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["os3kapp-trap-prototype", "0xa004"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "opcode=0xa004 name=set_text_row_column_width stack_argument_count=3 return_kind=none",
                "notes=row/column/width layout primitive inferred from calculator menu loop",
            ],
        )

    def test_os3kapp_applet_command_prints_known_alphaquiz_dispatch_contract(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["os3kapp-applet-command", "alphaquiz", "0x60001"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "applet=alphaquiz raw_command=0x60001 selector_byte=0x06 handler=HandleAlphaQuizNamespace6Commands status_code=0x00000011 response_word_count=0",
                "notes=copies up to 0x27 input bytes into the applet-global title buffer and NUL-terminates it",
            ],
        )

    def test_os3kapp_payload_subcommand_prints_known_alphaquiz_byte_protocol_entry(self) -> None:
        output = io.StringIO()

        with redirect_stdout(output):
            exit_code = main(["os3kapp-payload-subcommand", "alphaquiz", "0x50002", "0x1d"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "applet=alphaquiz parent_command=0x50002 first_input_byte=0x1d status_code=0x00000004 response_length=2",
                "notes=clears the UI and writes the fixed two-byte reply 0x5d 0x02",
            ],
        )


if __name__ == "__main__":
    unittest.main()
