from neotools.alphaword_attributes import parse_file_attributes_record
from neotools.alphaword_flow import UpdaterStep
from neotools.switch_packets import build_reset_preamble, build_switch_packet
from neotools.updater_packets import build_updater_command


def _checksum16(payload: bytes) -> int:
    return sum(payload) & 0xFFFF


def build_put_raw_file_attributes_plan(*, file_slot: int, applet_id: int, record: bytes) -> list[UpdaterStep]:
    return [
        UpdaterStep(
            "put_raw_file_attributes_begin",
            build_updater_command(command=0x1D, argument=file_slot, trailing=applet_id),
        ),
        UpdaterStep("put_raw_file_attributes_data", record),
        UpdaterStep(
            "put_raw_file_attributes_finish",
            build_updater_command(command=0x1E, argument=file_slot, trailing=applet_id),
        ),
    ]


def build_put_file_plan(*, file_slot: int, applet_id: int, payload: bytes) -> list[UpdaterStep]:
    file_length = len(payload)
    plan = [
        UpdaterStep(
            "put_file_begin",
            build_updater_command(
                command=0x14,
                argument=(file_slot << 24) | file_length,
                trailing=applet_id,
            ),
        ),
    ]
    for offset in range(0, file_length, 0x400):
        chunk = payload[offset : offset + 0x400]
        plan.append(
            UpdaterStep(
                "put_file_chunk_handshake",
                build_updater_command(
                    command=0x02,
                    argument=len(chunk),
                    trailing=_checksum16(chunk),
                ),
            )
        )
        plan.append(UpdaterStep("put_file_chunk_data", chunk))
    plan.append(UpdaterStep("put_file_finish", build_updater_command(command=0x15, argument=0, trailing=0)))
    return plan


def build_direct_usb_send_file_record(
    *,
    file_slot: int,
    applet_id: int,
    record: bytes,
    payload: bytes,
) -> list[UpdaterStep]:
    attributes = parse_file_attributes_record(record)
    if len(payload) != attributes.file_length:
        raise ValueError("payload length must match AlphaWord attribute file_length")
    return [
        UpdaterStep("reset_connection", build_reset_preamble()),
        UpdaterStep("switch_to_updater", build_switch_packet(0)),
        *build_put_raw_file_attributes_plan(file_slot=file_slot, applet_id=applet_id, record=record),
        *build_put_file_plan(
            file_slot=file_slot,
            applet_id=applet_id,
            payload=payload,
        ),
    ]
