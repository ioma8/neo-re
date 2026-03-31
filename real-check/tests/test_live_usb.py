import unittest

from real_check.live_usb import build_interface_descriptors


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


if __name__ == "__main__":
    unittest.main()
