import contextlib
import io
import unittest

from neotools import main


class CLITests(unittest.TestCase):
    def test_descriptor_command_parses_and_classifies_direct_neo(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["descriptor", "12 01 00 02 00 00 00 40 1e 08 01 bd 00 01 01 02 03 01"]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "vendor_id=0x081e product_id=0xbd01 direct_neo=True\n",
        )

    def test_switch_packet_command_prints_hex_bytes(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["switch-packet", "0x1234"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "3f 53 77 74 63 68 12 34\n")

    def test_asusbcomm_presence_command_prints_cached_mode_and_return_code(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["asusbcomm-presence", "12 01 00 02 00 00 00 40 1e 08 01 bd 02 00 01 02 03 01"]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "descriptor_valid=True cached_mode=3 return_code=3\n")

    def test_asusbcomm_set_mac_packet_command_prints_hex_bytes(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["asusbcomm-set-mac-packet", "00 00 aa bb cc dd ee ff"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "20 aa bb cc dd ee ff 1b\n")

    def test_asusbcomm_get_mac_packet_command_prints_hex_bytes(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["asusbcomm-get-mac-packet"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "00 00 00 00 00 00 00 00\n")

    def test_asusbcomm_hid_fallback_plan_command_prints_newer_windows_sequence(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["asusbcomm-hid-fallback-plan", "5"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "\n".join(
                [
                    "device_io_control code=0x000b0040 payload=00 00 00 00",
                    "device_io_control code=0x000b0008 payload=05 00 00 00",
                    "device_io_control code=0x000b0008 payload=02 00 00 00",
                    "device_io_control code=0x000b0008 payload=04 00 00 00",
                    "device_io_control code=0x000b0008 payload=01 00 00 00",
                    "device_io_control code=0x000b0008 payload=06 00 00 00",
                    "device_io_control code=0x000b0008 payload=07 00 00 00",
                    "sleep_ms value=2000",
                    "",
                ]
            ),
        )

    def test_driver64_dispatch_map_command_prints_major_handlers(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["driver64-dispatch-map"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "\n".join(
                [
                    "create: 0x00011400",
                    "close: 0x00011528",
                    "device_control: 0x000115a4",
                    "pnp: 0x000119e4",
                    "power: 0x00012e98",
                    "system_control: 0x00013dd0",
                    "unload: 0x00012af8",
                    "",
                ]
            ),
        )

    def test_driver64_ioctl_route_command_prints_internal_probe_sequence(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["driver64-ioctl-route", "0x220004"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "kind=internal_probe_sequence ioctl=0x00220004 first=0x00220013 second=0x00220007\n",
        )

    def test_driver64_create_route_command_prints_endpoint_open_behavior(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                [
                    "driver64-create-route",
                    "--state",
                    "2",
                    "--has-configuration",
                    "true",
                    "--file-name-suffix",
                    "5",
                ]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "kind=endpoint_handle ntstatus=0x00000000 open_count=True cancel_timer=True endpoint_index=5\n",
        )

    def test_driver64_read_write_route_command_prints_chunked_transfer_behavior(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                [
                    "driver64-read-write-route",
                    "--major-function",
                    "0x03",
                    "--state",
                    "2",
                    "--transfer-length",
                    "0x180",
                    "--file-context-present",
                    "true",
                    "--endpoint-type",
                    "2",
                ]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "kind=chunked_internal_transfer ntstatus=0x00000103 direction=read transfer_code=3 "
            "first_chunk=0x100 remaining=0x80 ioctl=0x00220003 probe_fallback=True\n",
        )

    def test_driver64_pnp_route_command_prints_remove_handler(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["driver64-pnp-route", "0x02"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "kind=remove_device minor=0x02 handler=HandleRemoveDevice\n",
        )

    def test_driver64_internal_request_command_prints_device_descriptor_layout(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["driver64-internal-request", "device-descriptor"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "size=0x88 function=0x0b buffer_length=0x12 request_type=1 "
            "endpoint_offset=0x18 response_buffer_offset=0x14\n",
        )

    def test_driver64_probe_sequence_command_prints_optional_second_ioctl(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["driver64-probe-sequence", "0x02"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "first=0x00220013 second=0x00220007 flags=0x00000002\n",
        )

    def test_driver64_internal_request_command_prints_data_transfer_layout(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(
                ["driver64-internal-request", "data-transfer", "--direction", "read", "--chunk-length", "0x100"]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "size=0x80 function=0x09 buffer_length=0x100 request_type=3 "
            "endpoint_offset=0x18 response_buffer_offset=none\n",
        )

    def test_driver64_internal_request_command_prints_cancel_transfer_layout(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["driver64-internal-request", "cancel-transfer"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "size=0x28 function=0x02 buffer_length=0x0 request_type=none "
            "endpoint_offset=0x18 response_buffer_offset=none\n",
        )

    def test_driver64_internal_ioctl_name_command_prints_usb_name(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["driver64-internal-ioctl-name", "0x220013"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "IOCTL_INTERNAL_USB_GET_PORT_STATUS\n")

    def test_alphaword_plan_command_prints_retrieval_sequence(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["alphaword-plan", "0xa000", "0x12"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            stdout.getvalue(),
            "\n".join(
                [
                    "list_applets: 04 00 00 00 00 00 07 0b",
                    "raw_file_attributes: 13 00 00 00 12 a0 00 c5",
                    "retrieve_file: 12 08 00 00 12 a0 00 cc",
                    "retrieve_chunk: 10 00 00 00 00 00 00 10",
                    "",
                ]
            ),
        )


if __name__ == "__main__":
    unittest.main()
