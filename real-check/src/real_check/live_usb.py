from dataclasses import dataclass

import usb.core
import usb.util

from real_check.usb_select import EndpointDescriptor, EndpointSelection, InterfaceDescriptor, select_direct_usb_endpoints


VENDOR_ID = 0x081E
PRODUCT_ID = 0xBD01


@dataclass(frozen=True)
class ProbeResult:
    vendor_id: int
    product_id: int
    interface_number: int
    out_endpoint: int
    in_endpoint: int


class LiveUsbTransport:
    def __init__(self, device: usb.core.Device, selection: EndpointSelection) -> None:
        self._device = device
        self._selection = selection
        self._closed = False

    def write(self, payload: bytes) -> None:
        self._device.write(self._selection.out_endpoint.address, payload)

    def read_exact(self, length: int, *, timeout_ms: int) -> bytes:
        data = self._device.read(self._selection.in_endpoint.address, length, timeout=timeout_ms)
        return bytes(data)

    def close(self) -> None:
        if self._closed:
            return
        usb.util.release_interface(self._device, self._selection.interface_number)
        try:
            self._device.attach_kernel_driver(self._selection.interface_number)
        except (NotImplementedError, usb.core.USBError):
            pass
        self._closed = True


def _transfer_type_name(attributes: int) -> str:
    transfer_type = attributes & 0x03
    if transfer_type == 0x02:
        return "bulk"
    if transfer_type == 0x03:
        return "interrupt"
    if transfer_type == 0x01:
        return "isochronous"
    return "control"


def build_interface_descriptors(configuration) -> list[InterfaceDescriptor]:
    interfaces: list[InterfaceDescriptor] = []
    for interface in configuration:
        interfaces.append(
            InterfaceDescriptor(
                number=interface.bInterfaceNumber,
                alternate_setting=interface.bAlternateSetting,
                endpoints=[
                    EndpointDescriptor(
                        address=endpoint.bEndpointAddress,
                        transfer_type=_transfer_type_name(endpoint.bmAttributes),
                        max_packet_size=endpoint.wMaxPacketSize,
                    )
                    for endpoint in interface
                ],
            )
        )
    return interfaces


def _open_device() -> usb.core.Device:
    device = usb.core.find(idVendor=VENDOR_ID, idProduct=PRODUCT_ID)
    if device is None:
        raise ValueError("AlphaSmart NEO USB device not found")
    return device


def _prepare_device(device: usb.core.Device, selection: EndpointSelection) -> None:
    device.set_configuration()
    try:
        if device.is_kernel_driver_active(selection.interface_number):
            device.detach_kernel_driver(selection.interface_number)
    except (NotImplementedError, usb.core.USBError):
        pass
    usb.util.claim_interface(device, selection.interface_number)
    try:
        device.set_interface_altsetting(
            interface=selection.interface_number,
            alternate_setting=selection.alternate_setting,
        )
    except usb.core.USBError:
        pass


def probe_direct_usb_device() -> ProbeResult:
    device = _open_device()
    configuration = device.get_active_configuration()
    selection = select_direct_usb_endpoints(build_interface_descriptors(configuration))
    return ProbeResult(
        vendor_id=device.idVendor,
        product_id=device.idProduct,
        interface_number=selection.interface_number,
        out_endpoint=selection.out_endpoint.address,
        in_endpoint=selection.in_endpoint.address,
    )


def open_direct_usb_transport() -> LiveUsbTransport:
    device = _open_device()
    configuration = device.get_active_configuration()
    selection = select_direct_usb_endpoints(build_interface_descriptors(configuration))
    _prepare_device(device, selection)
    return LiveUsbTransport(device, selection)
