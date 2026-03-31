import unittest

from neotools.updater_packets import build_updater_command

from real_check.client import AlphaWordFileEntry, NeoAlphaWordClient


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

    def test_close_closes_transport(self) -> None:
        transport = FakeTransport([])
        client = NeoAlphaWordClient(transport)

        client.close()

        self.assertTrue(transport.closed)


if __name__ == "__main__":
    unittest.main()
