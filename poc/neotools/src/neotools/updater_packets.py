def _require_range(name: str, value: int, upper_bound: int) -> None:
    if not 0 <= value <= upper_bound:
        raise ValueError(f"{name} out of range")


def build_updater_command(*, command: int, argument: int, trailing: int) -> bytes:
    _require_range("command", command, 0xFF)
    _require_range("argument", argument, 0xFFFF_FFFF)
    _require_range("trailing", trailing, 0xFFFF)

    packet_without_checksum = bytes([command]) + argument.to_bytes(4, "big") + trailing.to_bytes(2, "big")
    checksum = sum(packet_without_checksum) & 0xFF
    return packet_without_checksum + bytes([checksum])


def build_raw_file_attributes_command(*, file_slot: int, applet_id: int) -> bytes:
    _require_range("file_slot", file_slot, 0xFF)
    _require_range("applet_id", applet_id, 0xFFFF)
    return build_updater_command(command=0x13, argument=file_slot, trailing=applet_id)


def build_retrieve_file_command(
    *,
    file_slot: int,
    applet_id: int,
    requested_length: int,
    alternate_mode: bool,
) -> bytes:
    _require_range("file_slot", file_slot, 0xFF)
    _require_range("applet_id", applet_id, 0xFFFF)
    _require_range("requested_length", requested_length, 0xFF_FFFF)
    command = 0x1C if alternate_mode else 0x12
    selector = (requested_length << 8) | file_slot
    return build_updater_command(command=command, argument=selector, trailing=applet_id)
