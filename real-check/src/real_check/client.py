import hashlib
from dataclasses import dataclass
from typing import Protocol

from neotools.alphaword_attributes import parse_file_attributes_record
from neotools.smartapplets import (
    build_add_applet_begin_command,
    build_finalize_applet_update_command,
    build_list_applets_command,
    build_program_applet_command,
    build_retrieve_applet_command,
    build_retrieve_chunk_command,
    derive_add_applet_start_fields,
    parse_smartapplet_header,
    parse_smartapplet_metadata,
)
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


@dataclass(frozen=True)
class AlphaWordFileVerification:
    slot: int
    reported_length: int
    bytes_read: int
    sum16: int
    sha256: str


@dataclass(frozen=True)
class SmartAppletEntry:
    applet_id: int
    version_major: int
    version_minor: int
    name: str
    file_size: int
    applet_class: int


class NeoAlphaWordClient:
    def __init__(self, transport: NeoTransport, *, alphaword_applet_id: int = 0xA000) -> None:
        self._transport = transport
        self._alphaword_applet_id = alphaword_applet_id
        self._updater_entered = False

    def assume_updater_mode(self) -> None:
        self._updater_entered = True

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

    def list_smart_applets(self) -> list[SmartAppletEntry]:
        self.enter_updater_mode()
        entries: list[SmartAppletEntry] = []
        page_offset = 0
        page_size = 7

        while True:
            self._transport.write(build_list_applets_command(page_offset=page_offset, page_size=page_size))
            response = parse_updater_response(self._transport.read_exact(8, timeout_ms=600))
            if response.status == 0x90:
                break
            if response.status != 0x44:
                raise ValueError(f"unexpected applet list status 0x{response.status:02x}")
            if response.argument == 0:
                break
            if response.argument % 0x84 != 0:
                raise ValueError("applet list payload length is not a multiple of 0x84")

            payload = self._transport.read_exact(response.argument, timeout_ms=1000)
            if (sum(payload) & 0xFFFF) != response.trailing:
                raise ValueError("applet list payload checksum mismatch")

            records = [payload[offset : offset + 0x84] for offset in range(0, len(payload), 0x84)]
            for record in records:
                metadata = parse_smartapplet_metadata(record)
                entries.append(
                    SmartAppletEntry(
                        applet_id=metadata.applet_id,
                        version_major=metadata.version_major,
                        version_minor=metadata.version_minor,
                        name=metadata.name,
                        file_size=metadata.header.file_size,
                        applet_class=metadata.applet_class,
                    )
                )
            if len(records) < page_size:
                break
            page_offset += len(records)
        return entries

    def debug_smart_applet_records(self) -> list[str]:
        self.enter_updater_mode()
        lines: list[str] = []
        page_offset = 0
        page_size = 7
        row = 0

        while True:
            self._transport.write(build_list_applets_command(page_offset=page_offset, page_size=page_size))
            response = parse_updater_response(self._transport.read_exact(8, timeout_ms=600))
            lines.append(
                f"page_offset={page_offset} status=0x{response.status:02x} "
                f"argument={response.argument} trailing=0x{response.trailing:04x}"
            )
            if response.status == 0x90 or response.argument == 0:
                break
            if response.status != 0x44:
                break
            payload = self._transport.read_exact(response.argument, timeout_ms=1000)
            lines.append(f"payload_sum16=0x{sum(payload) & 0xffff:04x}")
            records = [payload[offset : offset + 0x84] for offset in range(0, len(payload), 0x84)]
            for record in records:
                metadata = parse_smartapplet_metadata(record)
                lines.append(
                    f"row={row} applet_id=0x{metadata.applet_id:04x} name={metadata.name} "
                    f"record={record.hex(' ')}"
                )
                row += 1
            if len(records) < page_size:
                break
            page_offset += len(records)
        return lines

    def _retrieve_alpha_word_file(self, *, slot: int, requested_length: int) -> tuple[bytes, int]:
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
        return bytes(payload), start.argument

    def download_alpha_word_file(self, *, slot: int, requested_length: int = 0x80000) -> bytes:
        payload, _reported_length = self._retrieve_alpha_word_file(slot=slot, requested_length=requested_length)
        return payload

    def download_smart_applet(self, *, applet_id: int) -> bytes:
        self.enter_updater_mode()
        self._transport.write(build_retrieve_applet_command(applet_id=applet_id))
        start = parse_updater_response(self._transport.read_exact(8, timeout_ms=10000))
        if start.status != 0x53:
            raise ValueError(f"unexpected applet retrieve start status 0x{start.status:02x}")

        remaining = start.argument
        payload = bytearray()
        while remaining > 0:
            self._transport.write(build_retrieve_chunk_command())
            chunk = parse_updater_response(self._transport.read_exact(8, timeout_ms=1000))
            if chunk.status != 0x4D:
                raise ValueError(f"unexpected applet retrieve chunk status 0x{chunk.status:02x}")
            chunk_payload = self._transport.read_exact(chunk.argument, timeout_ms=1000)
            if (sum(chunk_payload) & 0xFFFF) != chunk.trailing:
                raise ValueError("applet chunk payload checksum mismatch")
            payload.extend(chunk_payload)
            remaining -= len(chunk_payload)
        return bytes(payload)

    def install_smart_applet(self, image: bytes) -> SmartAppletEntry:
        if len(image) < 0x84:
            raise ValueError("SmartApplet image is shorter than its header")
        metadata = parse_smartapplet_metadata(image[:0x84])
        if metadata.header.file_size != len(image):
            raise ValueError("SmartApplet image length does not match header file size")

        self.enter_updater_mode()
        start_argument, trailing = derive_add_applet_start_fields(parse_smartapplet_header(image[:0x84]))
        self._transport.write(build_add_applet_begin_command(argument=start_argument, trailing=trailing))
        self._require_status(0x46, timeout_ms=2000, operation="add-applet begin")

        for offset in range(0, len(image), 0x400):
            chunk = image[offset : offset + 0x400]
            self._transport.write(
                build_updater_command(
                    command=0x02,
                    argument=len(chunk),
                    trailing=sum(chunk) & 0xFFFF,
                )
            )
            self._require_status(0x42, timeout_ms=2000, operation="add-applet chunk handshake")
            self._transport.write(chunk)
            self._require_status(0x43, timeout_ms=5000, operation="add-applet chunk commit")
            self._transport.write(build_program_applet_command())
            self._require_status(0x47, timeout_ms=10000, operation="program applet")

        self._transport.write(build_finalize_applet_update_command())
        self._require_status(0x48, timeout_ms=10000, operation="finalize applet update")
        return SmartAppletEntry(
            applet_id=metadata.applet_id,
            version_major=metadata.version_major,
            version_minor=metadata.version_minor,
            name=metadata.name,
            file_size=metadata.header.file_size,
            applet_class=metadata.applet_class,
        )

    def remove_smart_applet_by_index(self, *, index: int) -> None:
        self.enter_updater_mode()
        self._transport.write(build_updater_command(command=0x05, argument=5, trailing=index))
        self._require_status(0x45, timeout_ms=1000, operation="remove applet")

    def clear_smart_applet_area(self) -> None:
        self.enter_updater_mode()
        self._transport.write(build_updater_command(command=0x11, argument=0, trailing=0))
        self._require_status(0x4F, timeout_ms=90000, operation="clear SmartApplet area")

    def restart_device(self) -> None:
        self.enter_updater_mode()
        self._transport.write(build_updater_command(command=0x08, argument=0, trailing=0))
        self._require_status(0x52, timeout_ms=1000, operation="restart device")

    def _require_status(self, expected_status: int, *, timeout_ms: int, operation: str) -> None:
        response = parse_updater_response(self._transport.read_exact(8, timeout_ms=timeout_ms))
        if response.status != expected_status:
            raise ValueError(
                f"unexpected {operation} status 0x{response.status:02x}, expected 0x{expected_status:02x}"
            )

    def verify_alpha_word_file(self, *, slot: int, requested_length: int = 0x80000) -> AlphaWordFileVerification:
        payload, reported_length = self._retrieve_alpha_word_file(slot=slot, requested_length=requested_length)
        return AlphaWordFileVerification(
            slot=slot,
            reported_length=reported_length,
            bytes_read=len(payload),
            sum16=sum(payload) & 0xFFFF,
            sha256=hashlib.sha256(payload).hexdigest(),
        )

    def close(self) -> None:
        self._transport.close()
