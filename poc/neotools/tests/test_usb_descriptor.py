import unittest

from neotools.usb_descriptor import (
    USBDeviceDescriptor,
    is_direct_neo_descriptor,
    parse_usb_device_descriptor,
)


class USBDeviceDescriptorTests(unittest.TestCase):
    def test_parse_standard_usb_device_descriptor(self) -> None:
        raw = bytes.fromhex(
            "12 01 00 02 00 00 00 40 1e 08 01 bd 00 01 01 02 03 01"
        )

        descriptor = parse_usb_device_descriptor(raw)

        self.assertEqual(
            descriptor,
            USBDeviceDescriptor(
                bcd_usb=0x0200,
                device_class=0,
                device_subclass=0,
                device_protocol=0,
                max_packet_size_0=0x40,
                vendor_id=0x081E,
                product_id=0xBD01,
                bcd_device=0x0100,
                manufacturer_index=1,
                product_index=2,
                serial_number_index=3,
                num_configurations=1,
            ),
        )

    def test_parse_usb_device_descriptor_rejects_invalid_length(self) -> None:
        with self.assertRaises(ValueError):
            parse_usb_device_descriptor(b"\x12\x01")

    def test_parse_usb_device_descriptor_rejects_wrong_descriptor_header(self) -> None:
        with self.assertRaises(ValueError):
            parse_usb_device_descriptor(bytes.fromhex("11 02") + b"\x00" * 16)

    def test_is_direct_neo_descriptor_matches_vid_pid_pair(self) -> None:
        self.assertTrue(
            is_direct_neo_descriptor(
                USBDeviceDescriptor(
                    bcd_usb=0x0200,
                    device_class=0,
                    device_subclass=0,
                    device_protocol=0,
                    max_packet_size_0=0x40,
                    vendor_id=0x081E,
                    product_id=0xBD01,
                    bcd_device=0x0100,
                    manufacturer_index=1,
                    product_index=2,
                    serial_number_index=3,
                    num_configurations=1,
                )
            )
        )

    def test_is_direct_neo_descriptor_rejects_other_product_ids(self) -> None:
        self.assertFalse(
            is_direct_neo_descriptor(
                USBDeviceDescriptor(
                    bcd_usb=0x0200,
                    device_class=0,
                    device_subclass=0,
                    device_protocol=0,
                    max_packet_size_0=0x40,
                    vendor_id=0x081E,
                    product_id=0x0100,
                    bcd_device=0x0100,
                    manufacturer_index=1,
                    product_index=2,
                    serial_number_index=3,
                    num_configurations=1,
                )
            )
        )


if __name__ == "__main__":
    unittest.main()
