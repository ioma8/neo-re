from dataclasses import dataclass


@dataclass(frozen=True)
class USBDeviceDescriptor:
    bcd_usb: int
    device_class: int
    device_subclass: int
    device_protocol: int
    max_packet_size_0: int
    vendor_id: int
    product_id: int
    bcd_device: int
    manufacturer_index: int
    product_index: int
    serial_number_index: int
    num_configurations: int


DIRECT_NEO_VENDOR_ID = 0x081E
DIRECT_NEO_PRODUCT_ID = 0xBD01


def parse_usb_device_descriptor(raw: bytes) -> USBDeviceDescriptor:
    if len(raw) != 18:
        raise ValueError("USB device descriptor must be exactly 18 bytes")

    if raw[0] != 18 or raw[1] != 1:
        raise ValueError("invalid USB device descriptor header")

    return USBDeviceDescriptor(
        bcd_usb=int.from_bytes(raw[2:4], "little"),
        device_class=raw[4],
        device_subclass=raw[5],
        device_protocol=raw[6],
        max_packet_size_0=raw[7],
        vendor_id=int.from_bytes(raw[8:10], "little"),
        product_id=int.from_bytes(raw[10:12], "little"),
        bcd_device=int.from_bytes(raw[12:14], "little"),
        manufacturer_index=raw[14],
        product_index=raw[15],
        serial_number_index=raw[16],
        num_configurations=raw[17],
    )


def is_direct_neo_descriptor(descriptor: USBDeviceDescriptor) -> bool:
    return (
        descriptor.vendor_id == DIRECT_NEO_VENDOR_ID
        and descriptor.product_id == DIRECT_NEO_PRODUCT_ID
    )
