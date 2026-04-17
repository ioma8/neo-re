import unittest
from unittest import mock

from real_check.live_usb import (
    LiveUsbTransport,
    build_interface_descriptors,
    classify_alphasmart_device,
    try_switch_to_updater,
    watch_alphasmart_devices,
)
from real_check.usb_select import EndpointDescriptor, EndpointSelection, InterfaceDescriptor


class FakeEndpoint:
    def __init__(self, address: int, attributes: int, max_packet_size: int) -> None:
        self.bEndpointAddress = address
        self.bmAttributes = attributes
        self.wMaxPacketSize = max_packet_size


class FakeInterface:
    def __init__(self, number: int, alternate_setting: int, endpoints: list[FakeEndpoint]) -> None:
        self.bInterfaceNumber = number
        self.bAlternateSetting = alternate_setting
        self._endpoints = endpoints

    def __iter__(self):
        return iter(self._endpoints)


class FakeConfiguration:
    def __init__(self, interfaces: list[FakeInterface]) -> None:
        self._interfaces = interfaces

    def __iter__(self):
        return iter(self._interfaces)


class FakeUsbDevice:
    def __init__(self) -> None:
        self.writes: list[tuple[int, bytes]] = []

    def write(self, endpoint: int, payload: bytes) -> None:
        self.writes.append((endpoint, payload))

    def read(self, endpoint: int, length: int, *, timeout: int):
        if endpoint != 0x83:
            raise AssertionError(f"unexpected endpoint 0x{endpoint:02x}")
        if length != 8:
            raise AssertionError(f"unexpected length {length}")
        if timeout != 600:
            raise AssertionError(f"unexpected timeout {timeout}")
        return b"Switched"

    def attach_kernel_driver(self, interface: int) -> None:
        pass


class ChunkedReadUsbDevice:
    def __init__(self, chunks: list[bytes]) -> None:
        self.chunks = list(chunks)
        self.reads: list[tuple[int, int, int]] = []

    def read(self, endpoint: int, length: int, *, timeout: int):
        self.reads.append((endpoint, length, timeout))
        if not self.chunks:
            raise AssertionError("unexpected read")
        return self.chunks.pop(0)

    def attach_kernel_driver(self, interface: int) -> None:
        pass


class FakeObservedUsbDevice:
    def __init__(self, vendor_id: int, product_id: int, configuration: FakeConfiguration) -> None:
        self.idVendor = vendor_id
        self.idProduct = product_id
        self._configuration = configuration

    def get_active_configuration(self) -> FakeConfiguration:
        return self._configuration


class BuildInterfaceDescriptorsTests(unittest.TestCase):
    def test_build_interface_descriptors_maps_bulk_and_interrupt_attributes(self) -> None:
        config = FakeConfiguration(
            [
                FakeInterface(
                    number=2,
                    alternate_setting=1,
                    endpoints=[
                        FakeEndpoint(address=0x81, attributes=0x03, max_packet_size=8),
                        FakeEndpoint(address=0x02, attributes=0x02, max_packet_size=64),
                    ],
                )
            ]
        )

        interfaces = build_interface_descriptors(config)

        self.assertEqual(len(interfaces), 1)
        self.assertEqual(interfaces[0].number, 2)
        self.assertEqual(interfaces[0].alternate_setting, 1)
        self.assertEqual(interfaces[0].endpoints[0].transfer_type, "interrupt")
        self.assertEqual(interfaces[0].endpoints[1].transfer_type, "bulk")


class ClassifyAlphaSmartDeviceTests(unittest.TestCase):
    def test_classifies_bd01_as_direct_usb_mode(self) -> None:
        mode = classify_alphasmart_device(
            vendor_id=0x081E,
            product_id=0xBD01,
            interfaces=[],
        )

        self.assertEqual(mode.kind, "direct")
        self.assertEqual(mode.detail, "NEO direct USB mode")

    def test_classifies_bd04_hid_keyboard_mode(self) -> None:
        mode = classify_alphasmart_device(
            vendor_id=0x081E,
            product_id=0xBD04,
            interfaces=[
                InterfaceDescriptor(
                    number=0,
                    alternate_setting=0,
                    endpoints=[EndpointDescriptor(address=0x82, transfer_type="interrupt", max_packet_size=64)],
                )
            ],
        )

        self.assertEqual(mode.kind, "keyboard")
        self.assertEqual(mode.detail, "AlphaSmart HID keyboard mode; no direct USB OUT endpoint")

    def test_classifies_other_alphasmart_pid_as_unknown(self) -> None:
        mode = classify_alphasmart_device(
            vendor_id=0x081E,
            product_id=0x1234,
            interfaces=[],
        )

        self.assertEqual(mode.kind, "unknown")
        self.assertEqual(mode.detail, "unknown AlphaSmart USB mode")


class TrySwitchToUpdaterTests(unittest.TestCase):
    @mock.patch("real_check.live_usb.usb.util.release_interface")
    @mock.patch("real_check.live_usb._prepare_device")
    def test_switch_attempt_sends_reset_and_updater_switch_only(
        self,
        prepare_device: mock.Mock,
        release_interface: mock.Mock,
    ) -> None:
        device = FakeUsbDevice()

        result = try_switch_to_updater(
            device,
            interfaces=[
                InterfaceDescriptor(
                    number=1,
                    alternate_setting=0,
                    endpoints=[
                        EndpointDescriptor(address=0x02, transfer_type="bulk", max_packet_size=64),
                        EndpointDescriptor(address=0x83, transfer_type="bulk", max_packet_size=64),
                    ],
                )
            ],
        )

        self.assertEqual(result, "Switched")
        prepare_device.assert_called_once()
        self.assertEqual(
            device.writes,
            [
                (0x02, b"?\xff\x00reset"),
                (0x02, b"?Swtch\x00\x00"),
            ],
        )


class LiveUsbTransportTests(unittest.TestCase):
    def test_read_exact_accumulates_short_usb_reads(self) -> None:
        device = ChunkedReadUsbDevice([b"12345678", b"abcdefgh", b"ABCDEFGH", b"87654321", b"abcdefgh"])
        transport = LiveUsbTransport(
            device,
            EndpointSelection(
                interface_number=0,
                alternate_setting=0,
                out_endpoint=EndpointDescriptor(address=0x01, transfer_type="bulk", max_packet_size=64),
                in_endpoint=EndpointDescriptor(address=0x82, transfer_type="bulk", max_packet_size=64),
            ),
        )

        data = transport.read_exact(40, timeout_ms=600)

        self.assertEqual(data, b"12345678abcdefghABCDEFGH87654321abcdefgh")
        self.assertEqual(
            device.reads,
            [
                (0x82, 40, 600),
                (0x82, 32, 600),
                (0x82, 24, 600),
                (0x82, 16, 600),
                (0x82, 8, 600),
            ],
        )


class WatchAlphaSmartDevicesTests(unittest.TestCase):
    @mock.patch("real_check.live_usb.time.sleep")
    @mock.patch("real_check.live_usb.time.monotonic", side_effect=[0.0, 0.0, 0.0, 1.0])
    @mock.patch("real_check.live_usb.try_switch_to_updater", return_value="Switched")
    @mock.patch("real_check.live_usb.usb.core.find")
    def test_try_switch_records_result_once_for_direct_endpoint_pair(
        self,
        find: mock.Mock,
        try_switch_to_updater_mock: mock.Mock,
        monotonic: mock.Mock,
        sleep: mock.Mock,
    ) -> None:
        device = FakeObservedUsbDevice(
            0x081E,
            0xBD01,
            FakeConfiguration(
                [
                    FakeInterface(
                        number=1,
                        alternate_setting=0,
                        endpoints=[
                            FakeEndpoint(address=0x02, attributes=0x02, max_packet_size=64),
                            FakeEndpoint(address=0x83, attributes=0x02, max_packet_size=64),
                        ],
                    )
                ]
            ),
        )
        find.return_value = [device]

        observations = watch_alphasmart_devices(timeout_seconds=0.0, interval_seconds=0.25, try_switch=True)

        self.assertEqual(len(observations), 1)
        self.assertEqual(observations[0].switch_result, "Switched")
        try_switch_to_updater_mock.assert_called_once()


if __name__ == "__main__":
    unittest.main()
