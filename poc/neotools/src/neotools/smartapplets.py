from neotools.alphaword_flow import UpdaterStep
from neotools.switch_packets import build_reset_preamble, build_switch_packet
from neotools.updater_packets import build_updater_command


def _checksum16(payload: bytes) -> int:
    return sum(payload) & 0xFFFF


def build_list_applets_command(*, page_offset: int = 0, page_size: int = 7) -> bytes:
    return build_updater_command(command=0x04, argument=page_offset, trailing=page_size)


def build_retrieve_applet_command(*, applet_id: int) -> bytes:
    return build_updater_command(command=0x0F, argument=0, trailing=applet_id)


def build_retrieve_chunk_command() -> bytes:
    return build_updater_command(command=0x10, argument=0, trailing=0)


def build_add_applet_begin_command(*, argument: int, trailing: int) -> bytes:
    return build_updater_command(command=0x06, argument=argument, trailing=trailing)


def build_program_applet_command() -> bytes:
    return build_updater_command(command=0x0B, argument=0, trailing=0)


def build_finalize_applet_update_command() -> bytes:
    return build_updater_command(command=0x07, argument=0, trailing=0)


def build_direct_usb_retrieve_applet_plan(*, applet_id: int) -> list[UpdaterStep]:
    return [
        UpdaterStep("reset_connection", build_reset_preamble()),
        UpdaterStep("switch_to_updater", build_switch_packet(0)),
        UpdaterStep("retrieve_applet", build_retrieve_applet_command(applet_id=applet_id)),
        UpdaterStep("retrieve_chunk", build_retrieve_chunk_command()),
    ]


def build_direct_usb_add_applet_plan(*, start_argument: int, trailing: int, payload: bytes) -> list[UpdaterStep]:
    return [
        UpdaterStep("reset_connection", build_reset_preamble()),
        UpdaterStep("switch_to_updater", build_switch_packet(0)),
        UpdaterStep(
            "add_applet_begin",
            build_add_applet_begin_command(argument=start_argument, trailing=trailing),
        ),
        UpdaterStep(
            "add_applet_chunk_handshake",
            build_updater_command(command=0x02, argument=len(payload), trailing=_checksum16(payload)),
        ),
        UpdaterStep("add_applet_chunk_data", payload),
        UpdaterStep("add_applet_chunk_commit", build_updater_command(command=0xFF, argument=0, trailing=0)),
        UpdaterStep("program_applet", build_program_applet_command()),
        UpdaterStep("finalize_applet_update", build_finalize_applet_update_command()),
    ]
