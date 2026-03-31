from dataclasses import dataclass


@dataclass(frozen=True)
class Driver64DispatchMap:
    create: int
    close: int
    device_control: int
    pnp: int
    power: int
    system_control: int
    unload: int


@dataclass(frozen=True)
class Driver64InternalIoctlPlan:
    first: int
    second: int


@dataclass(frozen=True)
class Driver64DeviceControlRoute:
    kind: str
    ioctl: int
    internal_plan: Driver64InternalIoctlPlan | None = None


@dataclass(frozen=True)
class Driver64CreateRoute:
    kind: str
    ntstatus: int
    increments_open_count: bool
    cancels_timer_if_active: bool
    endpoint_index: int | None


@dataclass(frozen=True)
class Driver64ReadWriteRoute:
    kind: str
    ntstatus: int
    direction: str
    transfer_code: int
    first_chunk_length: int
    remaining_length: int
    uses_ioctl: int
    falls_back_to_probe_sequence: bool


@dataclass(frozen=True)
class Driver64PnpRoute:
    kind: str
    minor: int
    handler: str


@dataclass(frozen=True)
class Driver64Internal220003Request:
    size: int
    function: int
    transfer_buffer_length: int
    request_type: int | None
    endpoint_pointer_offset: int
    response_buffer_pointer_offset: int | None


@dataclass(frozen=True)
class Driver64ProbeSequencePlan:
    first_ioctl: int
    second_ioctl: int | None
    flags: int


def build_driver64_dispatch_map() -> Driver64DispatchMap:
    return Driver64DispatchMap(
        create=0x11400,
        close=0x11528,
        device_control=0x115A4,
        pnp=0x119E4,
        power=0x12E98,
        system_control=0x13DD0,
        unload=0x12AF8,
    )


def classify_driver64_device_control(ioctl: int) -> Driver64DeviceControlRoute:
    if ioctl == 0x220000:
        return Driver64DeviceControlRoute(kind="copy_cached_block", ioctl=ioctl)
    if ioctl == 0x220004:
        return Driver64DeviceControlRoute(
            kind="internal_probe_sequence",
            ioctl=ioctl,
            internal_plan=Driver64InternalIoctlPlan(first=0x220013, second=0x220007),
        )
    if ioctl == 0x220008:
        return Driver64DeviceControlRoute(kind="file_handle_triggered_transfer", ioctl=ioctl)
    if ioctl == 0x80002000:
        return Driver64DeviceControlRoute(kind="copy_usb_device_descriptor", ioctl=ioctl)
    return Driver64DeviceControlRoute(kind="invalid_device_request", ioctl=ioctl)


def classify_driver64_create(
    *,
    state: int,
    has_configuration: bool,
    file_name_suffix: int | None,
) -> Driver64CreateRoute:
    if state != 2 or not has_configuration:
        return Driver64CreateRoute(
            kind="device_not_ready",
            ntstatus=0xC0000184,
            increments_open_count=False,
            cancels_timer_if_active=False,
            endpoint_index=None,
        )
    if file_name_suffix is None:
        return Driver64CreateRoute(
            kind="control_handle",
            ntstatus=0,
            increments_open_count=True,
            cancels_timer_if_active=True,
            endpoint_index=None,
        )
    if file_name_suffix < 0 or file_name_suffix > 5:
        return Driver64CreateRoute(
            kind="invalid_name_suffix",
            ntstatus=0xC000000D,
            increments_open_count=False,
            cancels_timer_if_active=False,
            endpoint_index=None,
        )
    return Driver64CreateRoute(
        kind="endpoint_handle",
        ntstatus=0,
        increments_open_count=True,
        cancels_timer_if_active=True,
        endpoint_index=file_name_suffix,
    )


def classify_driver64_read_write(
    *,
    major_function: int,
    state: int,
    transfer_length: int,
    file_context_present: bool,
    endpoint_type: int | None,
) -> Driver64ReadWriteRoute:
    direction = "read" if major_function == 0x03 else "write"
    transfer_code = 3 if direction == "read" else 2
    if state != 2:
        return Driver64ReadWriteRoute(
            kind="device_not_ready",
            ntstatus=0xC0000184,
            direction=direction,
            transfer_code=transfer_code,
            first_chunk_length=0,
            remaining_length=0,
            uses_ioctl=0x220003,
            falls_back_to_probe_sequence=False,
        )
    if transfer_length > 0x10000:
        return Driver64ReadWriteRoute(
            kind="invalid_transfer_length",
            ntstatus=0xC000000D,
            direction=direction,
            transfer_code=transfer_code,
            first_chunk_length=0,
            remaining_length=0,
            uses_ioctl=0x220003,
            falls_back_to_probe_sequence=False,
        )
    if endpoint_type not in (2, 3):
        return Driver64ReadWriteRoute(
            kind="invalid_endpoint",
            ntstatus=0xC0000008,
            direction=direction,
            transfer_code=transfer_code,
            first_chunk_length=0,
            remaining_length=0,
            uses_ioctl=0x220003,
            falls_back_to_probe_sequence=False,
        )
    first_chunk_length = min(transfer_length, 0x100)
    remaining_length = max(0, transfer_length - first_chunk_length)
    return Driver64ReadWriteRoute(
        kind="chunked_internal_transfer",
        ntstatus=0x103,
        direction=direction,
        transfer_code=transfer_code,
        first_chunk_length=first_chunk_length,
        remaining_length=remaining_length,
        uses_ioctl=0x220003,
        falls_back_to_probe_sequence=True,
    )


def classify_driver64_pnp_minor(minor: int) -> Driver64PnpRoute:
    if minor == 0x00:
        return Driver64PnpRoute(kind="start_device", minor=minor, handler="StartDeviceAndLoadUsbDescriptors")
    if minor == 0x01:
        return Driver64PnpRoute(kind="query_remove", minor=minor, handler="forward_after_wait")
    if minor == 0x02:
        return Driver64PnpRoute(kind="remove_device", minor=minor, handler="HandleRemoveDevice")
    if minor == 0x03:
        return Driver64PnpRoute(kind="cancel_remove", minor=minor, handler="HandleCancelRemoveDevice")
    if minor == 0x04:
        return Driver64PnpRoute(kind="stop_device", minor=minor, handler="HandleStopDevice")
    if minor == 0x05:
        return Driver64PnpRoute(kind="query_stop", minor=minor, handler="forward_after_wait")
    if minor == 0x06:
        return Driver64PnpRoute(kind="cancel_stop", minor=minor, handler="HandleCancelStopDevice")
    if minor == 0x09:
        return Driver64PnpRoute(kind="query_capabilities", minor=minor, handler="HandleQueryCapabilities")
    if minor == 0x17:
        return Driver64PnpRoute(kind="surprise_removal", minor=minor, handler="HandleSurpriseRemoval")
    return Driver64PnpRoute(kind="pass_through", minor=minor, handler="IofCallDriver")


def build_driver64_device_descriptor_request() -> Driver64Internal220003Request:
    return Driver64Internal220003Request(
        size=0x88,
        function=0x0B,
        transfer_buffer_length=0x12,
        request_type=1,
        endpoint_pointer_offset=0x18,
        response_buffer_pointer_offset=0x14,
    )


def build_driver64_config_descriptor_header_request() -> Driver64Internal220003Request:
    return Driver64Internal220003Request(
        size=0x88,
        function=0x0B,
        transfer_buffer_length=9,
        request_type=2,
        endpoint_pointer_offset=0x18,
        response_buffer_pointer_offset=0x14,
    )


def build_driver64_config_descriptor_full_request(total_length: int) -> Driver64Internal220003Request:
    return Driver64Internal220003Request(
        size=0x88,
        function=0x0B,
        transfer_buffer_length=total_length,
        request_type=2,
        endpoint_pointer_offset=0x18,
        response_buffer_pointer_offset=0x14,
    )


def build_driver64_endpoint_trigger_request() -> Driver64Internal220003Request:
    return Driver64Internal220003Request(
        size=0x28,
        function=0x1E,
        transfer_buffer_length=0,
        request_type=None,
        endpoint_pointer_offset=0x18,
        response_buffer_pointer_offset=None,
    )


def build_driver64_data_transfer_request(
    *,
    chunk_length: int,
    direction: str,
) -> Driver64Internal220003Request:
    request_type = 3 if direction == "read" else 2
    return Driver64Internal220003Request(
        size=0x80,
        function=9,
        transfer_buffer_length=chunk_length,
        request_type=request_type,
        endpoint_pointer_offset=0x18,
        response_buffer_pointer_offset=None,
    )


def build_driver64_cancel_active_transfer_request() -> Driver64Internal220003Request:
    return Driver64Internal220003Request(
        size=0x28,
        function=2,
        transfer_buffer_length=0,
        request_type=None,
        endpoint_pointer_offset=0x18,
        response_buffer_pointer_offset=None,
    )


def build_driver64_probe_sequence_plan(flags: int) -> Driver64ProbeSequencePlan:
    second_ioctl = None
    if (flags & 1) == 0 and (flags & 2) != 0:
        second_ioctl = 0x220007
    return Driver64ProbeSequencePlan(
        first_ioctl=0x220013,
        second_ioctl=second_ioctl,
        flags=flags,
    )


def describe_driver64_internal_ioctl(ioctl: int) -> str:
    if ioctl == 0x220003:
        return "IOCTL_INTERNAL_USB_SUBMIT_URB"
    if ioctl == 0x220007:
        return "IOCTL_INTERNAL_USB_RESET_PORT"
    if ioctl == 0x220013:
        return "IOCTL_INTERNAL_USB_GET_PORT_STATUS"
    return "unknown"


def describe_driver64_urb_function(function: int) -> str:
    if function == 2:
        return "URB_FUNCTION_ABORT_PIPE"
    if function == 9:
        return "URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER"
    if function == 0x0B:
        return "URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE"
    if function == 0x1E:
        return "URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL (inference)"
    return "unknown"
