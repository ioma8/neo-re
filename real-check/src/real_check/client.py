from dataclasses import dataclass
from typing import Protocol

from neotools.alphaword_attributes import parse_file_attributes_record
from neotools.switch_packets import SwitchResponse, build_reset_preamble, build_switch_packet, parse_switch_response
from neotools.updater_packets import (
    build_raw_file_attributes_command,
    build_retrieve_file_command,
    build_updater_command,
)
from neotools.updater_responses import parse_updater_response


class NeoTransport(Protocol):
    def write(self, payload: bytes) -> None: ...

    def read_exact(self, length: int, *, timeout_ms: int) -> bytes: ...

    def close(self) -> None: ...


@dataclass(frozen=True)
class AlphaWordFileEntry:
    slot: int
    name: str
    file_length: int
    reserved_length: int


class NeoAlphaWordClient:
    def __init__(self, transport: NeoTransport, *, alphaword_applet_id: int = 0xA000) -> None:
        self._transport = transport
        self._alphaword_applet_id = alphaword_applet_id
        self._updater_entered = False

    def enter_updater_mode(self) -> None:
        if self._updater_entered:
            return
        self._transport.write(build_reset_preamble())
        self._transport.write(build_switch_packet(0))
        response = parse_switch_response(self._transport.read_exact(8, timeout_ms=600))
        if response is not SwitchResponse.SWITCHED:
            raise ValueError(f"unexpected switch response: {response.value}")
        self._updater_entered = True

    def list_alpha_word_files(self) -> list[AlphaWordFileEntry]:
        self.enter_updater_mode()
        entries: list[AlphaWordFileEntry] = []
        for slot in range(1, 9):
            self._transport.write(
                build_raw_file_attributes_command(file_slot=slot, applet_id=self._alphaword_applet_id)
            )
            response = parse_updater_response(self._transport.read_exact(8, timeout_ms=600))
            if response.status == 0x90:
                continue
            if response.status != 0x5A:
                raise ValueError(f"unexpected attributes status 0x{response.status:02x}")
            payload = self._transport.read_exact(response.argument, timeout_ms=600)
            if (sum(payload) & 0xFFFF) != response.trailing:
                raise ValueError("attribute payload checksum mismatch")
            attributes = parse_file_attributes_record(payload)
            entries.append(
                AlphaWordFileEntry(
                    slot=slot,
                    name=attributes.name,
                    file_length=attributes.file_length,
                    reserved_length=attributes.reserved_length,
                )
            )
        return entries

    def debug_alpha_word_attributes(self) -> list[str]:
        lines: list[str] = []
        reset = build_reset_preamble()
        switch = build_switch_packet(0)
        lines.append(f"write reset {reset.hex(' ')}")
        self._transport.write(reset)
        lines.append(f"write switch {switch.hex(' ')}")
        self._transport.write(switch)
        switch_response = self._transport.read_exact(8, timeout_ms=1000)
        lines.append(f"switch response {switch_response.hex(' ')} {parse_switch_response(switch_response).value}")
        self._updater_entered = True

        for slot in range(1, 9):
            command = build_raw_file_attributes_command(file_slot=slot, applet_id=self._alphaword_applet_id)
            lines.append(f"slot {slot} command {command.hex(' ')}")
            self._transport.write(command)
            header_raw = self._transport.read_exact(8, timeout_ms=1000)
            lines.append(f"slot {slot} header {header_raw.hex(' ')}")
            try:
                response = parse_updater_response(header_raw)
            except ValueError as exc:
                lines.append(f"slot {slot} header_parse_error {exc}")
                break
            lines.append(
                f"slot {slot} status=0x{response.status:02x} "
                f"argument={response.argument} trailing=0x{response.trailing:04x}"
            )
            if response.status != 0x5A:
                continue
            payload = self._transport.read_exact(response.argument, timeout_ms=1000)
            lines.append(f"slot {slot} payload {payload.hex(' ')}")
            lines.append(
                f"slot {slot} sum16=0x{sum(payload) & 0xffff:04x} "
                f"sum8=0x{sum(payload) & 0xff:02x} trailing=0x{response.trailing:04x}"
            )
        return lines

    def download_alpha_word_file(self, *, slot: int, requested_length: int = 0x80000) -> bytes:
        self.enter_updater_mode()
        self._transport.write(
            build_retrieve_file_command(
                file_slot=slot,
                applet_id=self._alphaword_applet_id,
                requested_length=requested_length,
                alternate_mode=False,
            )
        )
        start = parse_updater_response(self._transport.read_exact(8, timeout_ms=10000))
        if start.status != 0x53:
            raise ValueError(f"unexpected retrieve start status 0x{start.status:02x}")

        remaining = min(start.argument, requested_length)
        payload = bytearray()
        while remaining > 0:
            self._transport.write(build_updater_command(command=0x10, argument=0, trailing=0))
            chunk = parse_updater_response(self._transport.read_exact(8, timeout_ms=600))
            if chunk.status != 0x4D:
                raise ValueError(f"unexpected retrieve chunk status 0x{chunk.status:02x}")
            chunk_payload = self._transport.read_exact(chunk.argument, timeout_ms=600)
            if (sum(chunk_payload) & 0xFFFF) != chunk.trailing:
                raise ValueError("chunk payload checksum mismatch")
            payload.extend(chunk_payload)
            remaining -= len(chunk_payload)
        return bytes(payload)

    def close(self) -> None:
        self._transport.close()
