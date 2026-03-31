import contextlib
import io
import unittest
from unittest import mock

from real_check import main
from real_check.client import AlphaWordFileEntry


class CLITests(unittest.TestCase):
    def test_probe_parser_accepts_descriptor_dump_mode(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["--help"])

        self.assertEqual(exit_code, 0)
        self.assertIn("probe", stdout.getvalue())
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


if __name__ == "__main__":
    unittest.main()
