from neotools.alphaword_flow import (
    UpdaterStep,
    build_direct_usb_full_text_retrieval_plan,
    build_preview_retrieval_plan,
)
from neotools.switch_packets import build_reset_preamble, build_switch_packet


def build_direct_usb_preview_session(*, applet_id: int) -> list[UpdaterStep]:
    session = [
        UpdaterStep("reset_connection", build_reset_preamble()),
        UpdaterStep("switch_to_updater", build_switch_packet(0)),
    ]
    for file_slot in range(1, 9):
        session.extend(build_preview_retrieval_plan(applet_id=applet_id, file_slot=file_slot))
    return session


def build_direct_usb_full_text_session(*, applet_id: int) -> list[UpdaterStep]:
    session = [
        UpdaterStep("reset_connection", build_reset_preamble()),
        UpdaterStep("switch_to_updater", build_switch_packet(0)),
    ]
    for file_slot in range(1, 9):
        session.extend(build_direct_usb_full_text_retrieval_plan(applet_id=applet_id, file_slot=file_slot)[2:])
    return session
