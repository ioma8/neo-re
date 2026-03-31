from dataclasses import dataclass


@dataclass(frozen=True)
class UpdaterResponse:
    status: int
    argument: int
    trailing: int


def parse_updater_response(packet: bytes) -> UpdaterResponse:
    if len(packet) != 8:
        raise ValueError("updater response must be exactly 8 bytes")

    expected_checksum = sum(packet[:-1]) & 0xFF
    if packet[-1] != expected_checksum:
        raise ValueError("updater response checksum mismatch")

    return UpdaterResponse(
        status=packet[0],
        argument=int.from_bytes(packet[1:5], "big"),
        trailing=int.from_bytes(packet[5:7], "big"),
    )
