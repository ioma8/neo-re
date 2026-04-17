import unittest

from real_check.hid_switch import AlreadyDirectMode, MANAGER_SWITCH_SEQUENCE, MacIOHidBackend, send_manager_switch_sequence


class FakeHidBackend:
    def __init__(self, *, already_direct: bool = False) -> None:
        self.opened = False
        self.closed = False
        self.already_direct = already_direct
        self.writes: list[bytes] = []

    def open_alphasmart_keyboard(self):
        if self.already_direct:
            raise AlreadyDirectMode("already direct")
        self.opened = True
        return self

    def write_output_report(self, handle, report: bytes) -> int:
        if handle is not self:
            raise AssertionError("unexpected handle")
        self.writes.append(report)
        return len(report)

    def close(self, handle) -> None:
        if handle is not self:
            raise AssertionError("unexpected handle")
        self.closed = True


class SendManagerSwitchSequenceTests(unittest.TestCase):
    def test_uses_recovered_legacy_switch_sequence_that_works_on_connected_neo(self) -> None:
        self.assertEqual(MANAGER_SWITCH_SEQUENCE, (0xE0, 0xE1, 0xE2, 0xE3, 0xE4))

    def test_sends_recovered_led_output_report_sequence(self) -> None:
        backend = FakeHidBackend()

        result = send_manager_switch_sequence(backend=backend, delay_seconds=0)

        self.assertEqual(result.reports_sent, len(MANAGER_SWITCH_SEQUENCE))
        self.assertTrue(backend.opened)
        self.assertTrue(backend.closed)
        self.assertEqual(
            backend.writes,
            [bytes([0x00, value]) for value in MANAGER_SWITCH_SEQUENCE],
        )

    def test_returns_zero_reports_when_device_is_already_direct(self) -> None:
        result = send_manager_switch_sequence(backend=FakeHidBackend(already_direct=True), delay_seconds=0)

        self.assertEqual(result.reports_sent, 0)

    def test_mac_iokit_backend_strips_hidapi_report_id_before_set_report(self) -> None:
        self.assertEqual(MacIOHidBackend.output_report_payload(bytes([0x00, 0x05])), bytes([0x05]))

    def test_mac_iokit_backend_rejects_unexpected_report_id(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected report ID 0"):
            MacIOHidBackend.output_report_payload(bytes([0x01, 0x05]))


if __name__ == "__main__":
    unittest.main()
