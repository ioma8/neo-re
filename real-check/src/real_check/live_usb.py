from dataclasses import dataclass
import time

import usb.core
import usb.util

from real_check.client import NeoAlphaWordClient
from real_check.usb_select import EndpointDescriptor, EndpointSelection, InterfaceDescriptor, select_direct_usb_endpoints


VENDOR_ID = 0x081E
PRODUCT_ID = 0xBD01
KEYBOARD_PRODUCT_ID = 0xBD04


@dataclass(frozen=True)
class ProbeResult:
    vendor_id: int
    product_id: int
    interface_number: int
    out_endpoint: int
    in_endpoint: int


@dataclass(frozen=True)
class AlphaSmartDeviceMode:
    kind: str
    detail: str


@dataclass(frozen=True)
class ObservedAlphaSmartDevice:
    vendor_id: int
    product_id: int
    mode: AlphaSmartDeviceMode
    interfaces: list[InterfaceDescriptor]
    switch_result: str | None = None


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


def classify_alphasmart_device(
    *,
    vendor_id: int,
    product_id: int,
    interfaces: list[InterfaceDescriptor],
) -> AlphaSmartDeviceMode:
    if vendor_id != VENDOR_ID:
        return AlphaSmartDeviceMode("other", "not an AlphaSmart USB device")
    if product_id == PRODUCT_ID:
        return AlphaSmartDeviceMode("direct", "NEO direct USB mode")
    if product_id == KEYBOARD_PRODUCT_ID and all(endpoint.is_in for interface in interfaces for endpoint in interface.endpoints):
        return AlphaSmartDeviceMode("keyboard", "AlphaSmart HID keyboard mode; no direct USB OUT endpoint")
    return AlphaSmartDeviceMode("unknown", "unknown AlphaSmart USB mode")


def _open_device() -> usb.core.Device:
    device = usb.core.find(idVendor=VENDOR_ID, idProduct=PRODUCT_ID)
    if device is None:
        keyboard = usb.core.find(idVendor=VENDOR_ID, idProduct=KEYBOARD_PRODUCT_ID)
        if keyboard is not None:
            raise ValueError("AlphaSmart found in HID keyboard mode (081e:bd04), not NEO direct USB mode (081e:bd01)")
        raise ValueError("AlphaSmart NEO direct USB device not found")
    return device


def _observe_device(device: usb.core.Device, *, try_switch: bool = False) -> ObservedAlphaSmartDevice:
    configuration = device.get_active_configuration()
    interfaces = build_interface_descriptors(configuration)
    switch_result = None
    if try_switch:
        try:
            select_direct_usb_endpoints(interfaces)
            switch_result = try_switch_to_updater(device, interfaces=interfaces)
        except (ValueError, usb.core.USBError):
            switch_result = None
    return ObservedAlphaSmartDevice(
        vendor_id=device.idVendor,
        product_id=device.idProduct,
        mode=classify_alphasmart_device(
            vendor_id=device.idVendor,
            product_id=device.idProduct,
            interfaces=interfaces,
        ),
        interfaces=interfaces,
        switch_result=switch_result,
    )


def try_switch_to_updater(device: usb.core.Device, *, interfaces: list[InterfaceDescriptor]) -> str:
    selection = select_direct_usb_endpoints(interfaces)
    _prepare_device(device, selection)
    transport = LiveUsbTransport(device, selection)
    client = NeoAlphaWordClient(transport)
    try:
        client.enter_updater_mode()
        return "Switched"
    finally:
        client.close()


def watch_alphasmart_devices(
    *,
    timeout_seconds: float,
    interval_seconds: float = 0.25,
    try_switch: bool = False,
) -> list[ObservedAlphaSmartDevice]:
    deadline = time.monotonic() + timeout_seconds
    observed: dict[tuple[int, int], ObservedAlphaSmartDevice] = {}
    attempted_switches: set[tuple[int, int]] = set()
    while time.monotonic() <= deadline:
        for device in usb.core.find(find_all=True, idVendor=VENDOR_ID):
            key = (device.idVendor, device.idProduct)
            should_try_switch = try_switch and key not in attempted_switches
            try:
                observation = _observe_device(device, try_switch=should_try_switch)
            except usb.core.USBError:
                continue
            if should_try_switch:
                attempted_switches.add(key)
            previous = observed.get(key)
            if previous is not None and previous.switch_result and observation.switch_result is None:
                observation = ObservedAlphaSmartDevice(
                    vendor_id=observation.vendor_id,
                    product_id=observation.product_id,
                    mode=observation.mode,
                    interfaces=observation.interfaces,
                    switch_result=previous.switch_result,
                )
            observed[(observation.vendor_id, observation.product_id)] = observation
        if not try_switch and observed:
            break
        time.sleep(interval_seconds)
    return list(observed.values())


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
