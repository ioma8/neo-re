import unittest

from neotools.smartapplets import build_list_applets_command
from neotools.updater_packets import build_updater_command

from real_check.client import AlphaWordFileEntry, AlphaWordFileVerification, NeoAlphaWordClient, SmartAppletEntry


def _build_response(status: int, argument: int, trailing: int) -> bytes:
    packet = bytes([status]) + argument.to_bytes(4, "big") + trailing.to_bytes(2, "big")
    checksum = sum(packet) & 0xFF
    return packet + bytes([checksum])


class FakeTransport:
    def __init__(self, reads: list[bytes]) -> None:
        self.reads = list(reads)
        self.writes: list[bytes] = []
        self.closed = False

    def write(self, payload: bytes) -> None:
        self.writes.append(payload)

    def read_exact(self, length: int, *, timeout_ms: int) -> bytes:
        self.assert_timeout(timeout_ms)
        if not self.reads:
            raise AssertionError("unexpected read")
        payload = self.reads.pop(0)
        if len(payload) != length:
            raise AssertionError(f"expected read length {length}, got {len(payload)}")
        return payload

    def close(self) -> None:
        self.closed = True

    @staticmethod
    def assert_timeout(timeout_ms: int) -> None:
        if timeout_ms <= 0:
            raise AssertionError("timeout must be positive")


class ClientTests(unittest.TestCase):
    def test_bootstrap_updater_writes_reset_and_switch_sequence(self) -> None:
        transport = FakeTransport([b"Switched"])
        client = NeoAlphaWordClient(transport)

        client.enter_updater_mode()

        self.assertEqual(
            transport.writes,
            [
                b"?\xff\x00reset",
                b"?Swtch\x00\x00",
            ],
        )

    def test_list_alpha_word_files_reads_only_data_bearing_slots(self) -> None:
        slot_one_record = (
            b"FILE1\x00" + (b"\x00" * 18) + (0x28).to_bytes(4, "big") + (0x123).to_bytes(4, "big") + b"\x11" * 8
        )
        transport = FakeTransport(
            [
                b"Switched",
                _build_response(0x5A, 0x28, sum(slot_one_record) & 0xFFFF),
                slot_one_record,
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
            ]
        )
        client = NeoAlphaWordClient(transport)

        entries = client.list_alpha_word_files()

        self.assertEqual(
            entries,
            [AlphaWordFileEntry(slot=1, name="FILE1", file_length=0x123, reserved_length=0x28)],
        )
        self.assertEqual(transport.writes[2], build_updater_command(command=0x13, argument=1, trailing=0xA000))

    def test_debug_alpha_word_attributes_reports_raw_headers_payload_and_checksums(self) -> None:
        slot_one_record = (
            b"FILE1\x00" + (b"\x00" * 18) + (0x28).to_bytes(4, "big") + (0x123).to_bytes(4, "big") + b"\x11" * 8
        )
        transport = FakeTransport(
            [
                b"Switched",
                _build_response(0x5A, 0x28, sum(slot_one_record) & 0xFFFF),
                slot_one_record,
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
                _build_response(0x90, 0, 0),
            ]
        )
        client = NeoAlphaWordClient(transport)

        lines = client.debug_alpha_word_attributes()

        self.assertEqual(lines[0], "write reset 3f ff 00 72 65 73 65 74")
        self.assertEqual(lines[1], "write switch 3f 53 77 74 63 68 00 00")
        self.assertEqual(lines[2], "switch response 53 77 69 74 63 68 65 64 Switched")
        self.assertIn("slot 1 status=0x5a argument=40 trailing=0x0225", lines)
        self.assertIn("slot 1 sum16=0x0225 sum8=0x25 trailing=0x0225", lines)

    def test_debug_alpha_word_attributes_reports_unparseable_headers(self) -> None:
        bad_header = bytes.fromhex("5a 00 00 00 28 00 00 00")
        transport = FakeTransport([b"Switched", bad_header])
        client = NeoAlphaWordClient(transport)

        lines = client.debug_alpha_word_attributes()

        self.assertIn("slot 1 header 5a 00 00 00 28 00 00 00", lines)
        self.assertIn("slot 1 header_parse_error updater response checksum mismatch", lines)

    def test_download_alpha_word_file_reassembles_chunked_payload(self) -> None:
        transport = FakeTransport(
            [
                b"Switched",
                _build_response(0x53, 5, 0),
                _build_response(0x4D, 3, sum(b"ABC") & 0xFFFF),
                b"ABC",
                _build_response(0x4D, 2, sum(b"DE") & 0xFFFF),
                b"DE",
            ]
        )
        client = NeoAlphaWordClient(transport)

        payload = client.download_alpha_word_file(slot=2)

        self.assertEqual(payload, b"ABCDE")
        self.assertEqual(
            transport.writes,
            [
                b"?\xff\x00reset",
                b"?Swtch\x00\x00",
                build_updater_command(command=0x12, argument=(0x80000 << 8) | 2, trailing=0xA000),
                build_updater_command(command=0x10, argument=0, trailing=0),
                build_updater_command(command=0x10, argument=0, trailing=0),
            ],
        )

    def test_list_smart_applets_reads_metadata_records_without_retrieving_binaries(self) -> None:
        applet_record = bytearray(0x84)
        applet_record[0x00:0x04] = bytes.fromhex("c0 ff ee ad")
        applet_record[0x04:0x08] = (0x1234).to_bytes(4, "big")
        applet_record[0x08:0x0C] = (0x200).to_bytes(4, "big")
        applet_record[0x0C:0x10] = (0x1000).to_bytes(4, "big")
        applet_record[0x10:0x14] = (0xFF0000CE).to_bytes(4, "big")
        applet_record[0x14:0x18] = bytes.fromhex("a0 00 01 00")
        applet_record[0x18:0x18 + len(b"AlphaWord Plus")] = b"AlphaWord Plus"
        applet_record[0x3C] = 0x03
        applet_record[0x3D] = 0x04
        applet_record[0x3F] = 0x01
        applet_record[0x80:0x84] = (0x20).to_bytes(4, "big")
        payload = bytes(applet_record)
        transport = FakeTransport(
            [
                b"Switched",
                _build_response(0x44, len(payload), sum(payload) & 0xFFFF),
                payload,
            ]
        )
        client = NeoAlphaWordClient(transport)

        entries = client.list_smart_applets()

        self.assertEqual(
            entries,
            [
                SmartAppletEntry(
                    applet_id=0xA000,
                    version_major=3,
                    version_minor=4,
                    name="AlphaWord Plus",
                    file_size=0x1234,
                    applet_class=0x01,
                )
            ],
        )
        self.assertEqual(transport.writes[2], build_list_applets_command(page_offset=0, page_size=7))

    def test_verify_alpha_word_file_reports_hashes_without_exposing_payload(self) -> None:
        transport = FakeTransport(
            [
                b"Switched",
                _build_response(0x53, 5, 0),
                _build_response(0x4D, 5, sum(b"ABCDE") & 0xFFFF),
                b"ABCDE",
            ]
        )
        client = NeoAlphaWordClient(transport)

        verification = client.verify_alpha_word_file(slot=2)

        self.assertEqual(
            verification,
            AlphaWordFileVerification(
                slot=2,
                reported_length=5,
                bytes_read=5,
                sum16=0x014F,
                sha256="f0393febe8baaa55e32f7be2a7cc180bf34e52137d99e056c817a9c07b8f239a",
            ),
        )

    def test_close_closes_transport(self) -> None:
        transport = FakeTransport([])
        client = NeoAlphaWordClient(transport)

        client.close()

        self.assertTrue(transport.closed)


if __name__ == "__main__":
    unittest.main()
