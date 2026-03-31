from dataclasses import dataclass


@dataclass(frozen=True)
class EndpointDescriptor:
    address: int
    transfer_type: str
    max_packet_size: int

    @property
    def is_in(self) -> bool:
        return (self.address & 0x80) != 0

    @property
    def is_out(self) -> bool:
        return not self.is_in


@dataclass(frozen=True)
class InterfaceDescriptor:
    number: int
    alternate_setting: int
    endpoints: list[EndpointDescriptor]


@dataclass(frozen=True)
class EndpointSelection:
    interface_number: int
    alternate_setting: int
    out_endpoint: EndpointDescriptor
    in_endpoint: EndpointDescriptor


def _find_endpoint_pair(
    endpoints: list[EndpointDescriptor],
    *,
    transfer_type: str,
) -> tuple[EndpointDescriptor, EndpointDescriptor] | None:
    out_endpoint = next((endpoint for endpoint in endpoints if endpoint.transfer_type == transfer_type and endpoint.is_out), None)
    in_endpoint = next((endpoint for endpoint in endpoints if endpoint.transfer_type == transfer_type and endpoint.is_in), None)
    if out_endpoint is None or in_endpoint is None:
        return None
    return out_endpoint, in_endpoint


def select_direct_usb_endpoints(interfaces: list[InterfaceDescriptor]) -> EndpointSelection:
    for transfer_type in ("bulk", "interrupt"):
        for interface in interfaces:
            pair = _find_endpoint_pair(interface.endpoints, transfer_type=transfer_type)
            if pair is None:
                continue
            out_endpoint, in_endpoint = pair
            return EndpointSelection(
                interface_number=interface.number,
                alternate_setting=interface.alternate_setting,
                out_endpoint=out_endpoint,
                in_endpoint=in_endpoint,
            )
    raise ValueError("No usable direct USB endpoint pair found")
