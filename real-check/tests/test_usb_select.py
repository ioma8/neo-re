import unittest

from real_check.usb_select import EndpointDescriptor, InterfaceDescriptor, select_direct_usb_endpoints


class UsbEndpointSelectionTests(unittest.TestCase):
    def test_select_prefers_bulk_in_and_bulk_out_endpoints(self) -> None:
        interface = InterfaceDescriptor(
            number=1,
            alternate_setting=0,
            endpoints=[
                EndpointDescriptor(address=0x81, transfer_type="interrupt", max_packet_size=8),
                EndpointDescriptor(address=0x02, transfer_type="bulk", max_packet_size=64),
                EndpointDescriptor(address=0x83, transfer_type="bulk", max_packet_size=64),
            ],
        )

        selection = select_direct_usb_endpoints([interface])

        self.assertEqual(selection.interface_number, 1)
        self.assertEqual(selection.out_endpoint.address, 0x02)
        self.assertEqual(selection.in_endpoint.address, 0x83)

    def test_select_falls_back_to_interrupt_pair_when_bulk_pair_missing(self) -> None:
        interface = InterfaceDescriptor(
            number=3,
            alternate_setting=1,
            endpoints=[
                EndpointDescriptor(address=0x81, transfer_type="interrupt", max_packet_size=8),
                EndpointDescriptor(address=0x02, transfer_type="interrupt", max_packet_size=8),
            ],
        )

        selection = select_direct_usb_endpoints([interface])

        self.assertEqual(selection.interface_number, 3)
        self.assertEqual(selection.out_endpoint.address, 0x02)
        self.assertEqual(selection.in_endpoint.address, 0x81)

    def test_select_rejects_interfaces_without_bidirectional_candidate(self) -> None:
        interface = InterfaceDescriptor(
            number=0,
            alternate_setting=0,
            endpoints=[EndpointDescriptor(address=0x81, transfer_type="bulk", max_packet_size=64)],
        )

        with self.assertRaisesRegex(ValueError, "No usable"):
            select_direct_usb_endpoints([interface])


if __name__ == "__main__":
    unittest.main()
