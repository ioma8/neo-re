from dataclasses import dataclass
from enum import IntEnum

from neotools.usb_descriptor import DIRECT_NEO_PRODUCT_ID, DIRECT_NEO_VENDOR_ID
from neotools.updater_packets import build_updater_command


class AsUSBCommSwitchResult(IntEnum):
    SWITCHED = 0
    TRANSPORT_FAILURE = 1
    NO_APPLET = 2
    INVALID_HANDLE = 3
    UNKNOWN_RESPONSE = 3
    NO_SWITCH = 4
    SHORT_RESPONSE = 5


@dataclass(frozen=True)
class AsUSBCommPresenceResult:
    descriptor_valid: bool
    cached_mode: int
    return_code: int


@dataclass(frozen=True)
class AsUSBCommReadState:
    pending: bytes = b""


@dataclass(frozen=True)
class AsUSBCommReadOutcome:
    return_code: int
    data: bytes
    bytes_read: int
    state: AsUSBCommReadState


@dataclass(frozen=True)
class AsUSBCommGetMacOutcome:
    return_code: int
    mac_bytes: bytes | None
    collected_payload_length: int


def classify_alpha_smart_presence(raw_descriptor: bytes) -> AsUSBCommPresenceResult:
    if len(raw_descriptor) != 18:
        raise ValueError("presence probe descriptor must be exactly 18 bytes")

    vendor_id = int.from_bytes(raw_descriptor[8:10], "little")
    product_id = int.from_bytes(raw_descriptor[10:12], "little")
    if vendor_id != DIRECT_NEO_VENDOR_ID or product_id != DIRECT_NEO_PRODUCT_ID:
        return AsUSBCommPresenceResult(descriptor_valid=False, cached_mode=0, return_code=0)

    bcd_device = int.from_bytes(raw_descriptor[12:14], "little")
    if bcd_device == 1:
        return AsUSBCommPresenceResult(descriptor_valid=True, cached_mode=1, return_code=1)
    if bcd_device == 2:
        return AsUSBCommPresenceResult(descriptor_valid=True, cached_mode=3, return_code=3)
    return AsUSBCommPresenceResult(descriptor_valid=True, cached_mode=2, return_code=2)


def interpret_switch_transaction(
    *,
    handle_valid: bool,
    write_ok: bool = True,
    read_ok: bool = True,
    response: bytes = b"Switched",
) -> AsUSBCommSwitchResult:
    if not handle_valid:
        return AsUSBCommSwitchResult.INVALID_HANDLE
    if not write_ok or not read_ok:
        return AsUSBCommSwitchResult.TRANSPORT_FAILURE
    if len(response) != 8:
        return AsUSBCommSwitchResult.SHORT_RESPONSE
    if response == b"Switched":
        return AsUSBCommSwitchResult.SWITCHED
    if response == b"NoSwitch":
        return AsUSBCommSwitchResult.NO_SWITCH
    if response == b"NoApplet":
        return AsUSBCommSwitchResult.NO_APPLET
    return AsUSBCommSwitchResult.UNKNOWN_RESPONSE


def simulate_read_data(
    *,
    handle_valid: bool,
    max_length: int,
    min_required: int,
    timeout_ms: int,
    start_tick: int,
    refill_chunks: list[bytes | None],
    state: AsUSBCommReadState | None = None,
    timeout_ticks: list[int] | None = None,
) -> AsUSBCommReadOutcome:
    pending = bytearray(b"" if state is None else state.pending)
    output = bytearray()
    bytes_read = 0
    tick_index = 0
    refill_index = 0

    if not handle_valid:
        return AsUSBCommReadOutcome(3, b"", 0, AsUSBCommReadState(bytes(pending)))
    if max_length < min_required:
        return AsUSBCommReadOutcome(0x0B, b"", 0, AsUSBCommReadState(bytes(pending)))

    while True:
        if not pending:
            if refill_index >= len(refill_chunks) or refill_chunks[refill_index] is None:
                return AsUSBCommReadOutcome(1, bytes(output), bytes_read, AsUSBCommReadState())
            chunk = refill_chunks[refill_index]
            refill_index += 1
            if not 0 <= len(chunk) <= 8:
                raise ValueError("refill chunk must contain between 0 and 8 bytes")
            pending.extend(chunk)

        while pending and bytes_read < max_length:
            output.append(pending.pop(0))
            bytes_read += 1

        if bytes_read >= min_required:
            return AsUSBCommReadOutcome(
                0,
                bytes(output),
                bytes_read,
                AsUSBCommReadState(bytes(pending)),
            )

        current_tick = start_tick if timeout_ticks is None or tick_index >= len(timeout_ticks) else timeout_ticks[tick_index]
        tick_index += 1
        if start_tick + timeout_ms < current_tick:
            return AsUSBCommReadOutcome(
                0x0C,
                bytes(output),
                bytes_read,
                AsUSBCommReadState(bytes(pending)),
            )


def build_set_mac_address_packet(source: bytes) -> bytes:
    if len(source) < 8:
        raise ValueError("source must contain at least 8 bytes")
    return build_updater_command(
        command=0x20,
        argument=int.from_bytes(source[2:6], "big"),
        trailing=int.from_bytes(source[6:8], "big"),
    )


def interpret_set_mac_address_transaction(
    *,
    write_return_code: int,
    read_return_code: int,
    bytes_read: int,
    response: bytes,
) -> int:
    if write_return_code != 0:
        return write_return_code
    if read_return_code == 0 and bytes_read == 8 and response[:1] != b" ":
        return -1
    return read_return_code


def build_get_mac_address_packet() -> bytes:
    return build_updater_command(command=0x00, argument=0, trailing=0)


def interpret_get_mac_address_transaction(
    *,
    write_return_code: int,
    bytes_written: int,
    header_read_return_code: int,
    header_response: bytes,
    payload_read_return_codes: list[int],
    payload_blocks: list[bytes],
) -> AsUSBCommGetMacOutcome:
    if write_return_code != 0:
        return AsUSBCommGetMacOutcome(write_return_code, None, 0)
    if bytes_written != 8:
        return AsUSBCommGetMacOutcome(-1, None, 0)
    if header_read_return_code != 0 or not header_response.startswith(b"@"):
        return AsUSBCommGetMacOutcome(write_return_code, None, 0)

    payload = bytearray()
    collected_length = 0
    for read_return_code, block in zip(payload_read_return_codes, payload_blocks):
        if read_return_code != 0:
            return AsUSBCommGetMacOutcome(write_return_code, None, collected_length)
        if len(block) != 8:
            raise ValueError("payload blocks must be exactly 8 bytes")
        payload.extend(block)
        collected_length += 8
        if collected_length == 64:
            return AsUSBCommGetMacOutcome(write_return_code, bytes(payload[-8:]), collected_length)

    return AsUSBCommGetMacOutcome(write_return_code, None, collected_length)
