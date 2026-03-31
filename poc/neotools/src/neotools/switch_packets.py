from enum import Enum


class SwitchResponse(Enum):
    SWITCHED = "Switched"
    NO_SWITCH = "NoSwitch"
    NO_APPLET = "NoApplet"


def build_reset_preamble() -> bytes:
    return b"?\xff\x00reset"


def build_switch_packet(applet_id: int) -> bytes:
    if not 0 <= applet_id <= 0xFFFF:
        raise ValueError("applet_id must fit in 16 bits")

    return b"?Swtch" + applet_id.to_bytes(2, "big")


def parse_switch_response(packet: bytes) -> SwitchResponse:
    if len(packet) != 8:
        raise ValueError("switch response must be exactly 8 bytes")

    return SwitchResponse(packet.decode("ascii"))
