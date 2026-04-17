import contextlib
import io
import unittest
from unittest import mock

from real_check import main
from real_check.client import AlphaWordFileEntry
from real_check.hid_switch import ManagerSwitchResult
from real_check.live_usb import AlphaSmartDeviceMode, ObservedAlphaSmartDevice
from real_check.usb_select import EndpointDescriptor, InterfaceDescriptor


class CLITests(unittest.TestCase):
    def test_probe_parser_accepts_descriptor_dump_mode(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["--help"])

        self.assertEqual(exit_code, 0)
        self.assertIn("probe", stdout.getvalue())
        self.assertIn("watch", stdout.getvalue())
        self.assertIn("switch-to-direct", stdout.getvalue())
        self.assertIn("debug-attributes", stdout.getvalue())
        self.assertIn("list", stdout.getvalue())
        self.assertIn("get", stdout.getvalue())

    @mock.patch("real_check.probe_direct_usb_device")
    def test_probe_command_prints_selection(self, probe_direct_usb_device: mock.Mock) -> None:
        probe_direct_usb_device.return_value.vendor_id = 0x081E
        probe_direct_usb_device.return_value.product_id = 0xBD01
        probe_direct_usb_device.return_value.interface_number = 1
        probe_direct_usb_device.return_value.out_endpoint = 0x02
        probe_direct_usb_device.return_value.in_endpoint = 0x83
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["probe"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "vendor_id=0x081e product_id=0xbd01 interface=1 out_ep=0x02 in_ep=0x83\n",
        )

    @mock.patch("real_check.watch_alphasmart_devices")
    def test_watch_command_prints_observed_modes(self, watch_alphasmart_devices: mock.Mock) -> None:
        watch_alphasmart_devices.return_value = [
            ObservedAlphaSmartDevice(
                vendor_id=0x081E,
                product_id=0xBD04,
                mode=AlphaSmartDeviceMode("keyboard", "AlphaSmart HID keyboard mode; no direct USB OUT endpoint"),
                interfaces=[
                    InterfaceDescriptor(
                        number=0,
                        alternate_setting=0,
                        endpoints=[EndpointDescriptor(address=0x82, transfer_type="interrupt", max_packet_size=64)],
                    )
                ],
            )
        ]
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["watch", "--timeout", "0.1"])

        self.assertEqual(exit_code, 0)
        watch_alphasmart_devices.assert_called_once_with(timeout_seconds=0.1, interval_seconds=0.25, try_switch=False)
        self.assertEqual(
            stdout.getvalue(),
            "vendor_id=0x081e product_id=0xbd04 mode=keyboard detail=AlphaSmart HID keyboard mode; no direct USB OUT endpoint\n"
            "  interface=0 alt=0 endpoints=0x82:interrupt:in:max64\n",
        )

    @mock.patch("real_check.watch_alphasmart_devices")
    def test_watch_command_can_enable_switch_attempt(self, watch_alphasmart_devices: mock.Mock) -> None:
        watch_alphasmart_devices.return_value = []
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["watch", "--timeout", "0.1", "--try-switch"])

        self.assertEqual(exit_code, 0)
        watch_alphasmart_devices.assert_called_once_with(timeout_seconds=0.1, interval_seconds=0.25, try_switch=True)

    @mock.patch("real_check.watch_alphasmart_devices")
    def test_watch_command_prints_switch_result(self, watch_alphasmart_devices: mock.Mock) -> None:
        watch_alphasmart_devices.return_value = [
            ObservedAlphaSmartDevice(
                vendor_id=0x081E,
                product_id=0xBD01,
                mode=AlphaSmartDeviceMode("direct", "NEO direct USB mode"),
                interfaces=[],
                switch_result="Switched",
            )
        ]
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["watch", "--try-switch"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "vendor_id=0x081e product_id=0xbd01 mode=direct detail=NEO direct USB mode switch=Switched\n",
        )

    @mock.patch("real_check.open_direct_usb_transport")
    @mock.patch("real_check.NeoAlphaWordClient")
    def test_list_command_prints_entries(self, client_cls: mock.Mock, open_transport: mock.Mock) -> None:
        client = client_cls.return_value
        client.list_alpha_word_files.return_value = [
            AlphaWordFileEntry(slot=1, name="FILE1", file_length=5, reserved_length=0x28)
        ]
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["list"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "slot=1 name=FILE1 file_length=5 reserved_length=40\n",
        )
        open_transport.assert_called_once()
        client.close.assert_called_once()

    @mock.patch("real_check.open_direct_usb_transport")
    @mock.patch("real_check.NeoAlphaWordClient")
    def test_debug_attributes_command_prints_raw_trace(self, client_cls: mock.Mock, open_transport: mock.Mock) -> None:
        client = client_cls.return_value
        client.debug_alpha_word_attributes.return_value = ["write reset 3f ff 00 72 65 73 65 74", "slot 1 empty"]
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["debug-attributes"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "write reset 3f ff 00 72 65 73 65 74\nslot 1 empty\n")
        open_transport.assert_called_once()
        client.debug_alpha_word_attributes.assert_called_once()
        client.close.assert_called_once()

    @mock.patch("real_check.open_direct_usb_transport")
    @mock.patch("real_check.NeoAlphaWordClient")
    def test_get_command_prints_payload_hex(self, client_cls: mock.Mock, open_transport: mock.Mock) -> None:
        client = client_cls.return_value
        client.download_alpha_word_file.return_value = b"ABC"
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["get", "2"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "41 42 43\n")
        open_transport.assert_called_once()
        client.download_alpha_word_file.assert_called_once_with(slot=2)
        client.close.assert_called_once()

    @mock.patch("real_check.send_manager_switch_sequence", return_value=ManagerSwitchResult(reports_sent=6))
    def test_switch_to_direct_command_prints_sent_report_count(self, send_manager_switch_sequence: mock.Mock) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["switch-to-direct"])

        self.assertEqual(exit_code, 0)
        send_manager_switch_sequence.assert_called_once()
        self.assertEqual(stdout.getvalue(), "sent_manager_switch_reports=6\n")

    @mock.patch("real_check.send_manager_switch_sequence", side_effect=RuntimeError("USB HID GET_REPORT failed"))
    def test_switch_to_direct_command_prints_runtime_error_without_traceback(
        self, send_manager_switch_sequence: mock.Mock
    ) -> None:
        stderr = io.StringIO()

        with contextlib.redirect_stderr(stderr):
            exit_code = main(["switch-to-direct"])

        self.assertEqual(exit_code, 1)
        send_manager_switch_sequence.assert_called_once()
        self.assertEqual(stderr.getvalue(), "USB HID GET_REPORT failed\n")


if __name__ == "__main__":
    unittest.main()
