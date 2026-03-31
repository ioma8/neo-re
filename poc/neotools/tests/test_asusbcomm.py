import unittest

from neotools.asusbcomm import (
    AsUSBCommGetMacOutcome,
    AsUSBCommPresenceResult,
    AsUSBCommReadOutcome,
    AsUSBCommReadState,
    AsUSBCommSwitchResult,
    build_get_mac_address_packet,
    build_set_mac_address_packet,
    classify_alpha_smart_presence,
    interpret_switch_transaction,
    interpret_get_mac_address_transaction,
    interpret_set_mac_address_transaction,
    simulate_read_data,
)


class AsUSBCommPresenceTests(unittest.TestCase):
    def test_classify_presence_returns_direct_mode_when_bcd_device_is_one(self) -> None:
        result = classify_alpha_smart_presence(
            bytes.fromhex("12 01 00 02 00 00 00 40 1e 08 01 bd 01 00 01 02 03 01")
        )

        self.assertEqual(
            result,
            AsUSBCommPresenceResult(
                descriptor_valid=True,
                cached_mode=1,
                return_code=1,
            ),
        )

    def test_classify_presence_returns_three_when_bcd_device_is_two(self) -> None:
        result = classify_alpha_smart_presence(
            bytes.fromhex("12 01 00 02 00 00 00 40 1e 08 01 bd 02 00 01 02 03 01")
        )

        self.assertEqual(result.return_code, 3)
        self.assertEqual(result.cached_mode, 3)

    def test_classify_presence_returns_two_for_other_matching_descriptors(self) -> None:
        result = classify_alpha_smart_presence(
            bytes.fromhex("12 01 00 02 00 00 00 40 1e 08 01 bd 03 00 01 02 03 01")
        )

        self.assertEqual(result.return_code, 2)
        self.assertEqual(result.cached_mode, 2)

    def test_classify_presence_rejects_non_direct_neo_descriptor(self) -> None:
        result = classify_alpha_smart_presence(
            bytes.fromhex("12 01 00 02 00 00 00 40 1e 08 00 01 01 00 01 02 03 01")
        )

        self.assertFalse(result.descriptor_valid)
        self.assertEqual(result.cached_mode, 0)
        self.assertEqual(result.return_code, 0)


class AsUSBCommSwitchTests(unittest.TestCase):
    def test_interpret_switch_transaction_returns_three_for_invalid_handle(self) -> None:
        self.assertEqual(
            interpret_switch_transaction(handle_valid=False),
            AsUSBCommSwitchResult.INVALID_HANDLE,
        )

    def test_interpret_switch_transaction_returns_one_on_transport_failure(self) -> None:
        self.assertEqual(
            interpret_switch_transaction(handle_valid=True, write_ok=False),
            AsUSBCommSwitchResult.TRANSPORT_FAILURE,
        )
        self.assertEqual(
            interpret_switch_transaction(handle_valid=True, write_ok=True, read_ok=False),
            AsUSBCommSwitchResult.TRANSPORT_FAILURE,
        )

    def test_interpret_switch_transaction_returns_five_on_short_read(self) -> None:
        self.assertEqual(
            interpret_switch_transaction(
                handle_valid=True,
                write_ok=True,
                read_ok=True,
                response=b"short",
            ),
            AsUSBCommSwitchResult.SHORT_RESPONSE,
        )

    def test_interpret_switch_transaction_maps_known_textual_responses(self) -> None:
        self.assertEqual(
            interpret_switch_transaction(
                handle_valid=True,
                write_ok=True,
                read_ok=True,
                response=b"Switched",
            ),
            AsUSBCommSwitchResult.SWITCHED,
        )
        self.assertEqual(
            interpret_switch_transaction(
                handle_valid=True,
                write_ok=True,
                read_ok=True,
                response=b"NoSwitch",
            ),
            AsUSBCommSwitchResult.NO_SWITCH,
        )
        self.assertEqual(
            interpret_switch_transaction(
                handle_valid=True,
                write_ok=True,
                read_ok=True,
                response=b"NoApplet",
            ),
            AsUSBCommSwitchResult.NO_APPLET,
        )

    def test_interpret_switch_transaction_returns_three_for_unknown_8_byte_reply(self) -> None:
        self.assertEqual(
            interpret_switch_transaction(
                handle_valid=True,
                write_ok=True,
                read_ok=True,
                response=b"Unknown!",
            ),
            AsUSBCommSwitchResult.UNKNOWN_RESPONSE,
        )


class AsUSBCommReadTests(unittest.TestCase):
    def test_simulate_read_data_rejects_invalid_handle(self) -> None:
        result = simulate_read_data(
            handle_valid=False,
            max_length=8,
            min_required=0,
            timeout_ms=0,
            start_tick=100,
            refill_chunks=[],
        )

        self.assertEqual(
            result,
            AsUSBCommReadOutcome(
                return_code=3,
                data=b"",
                bytes_read=0,
                state=AsUSBCommReadState(),
            ),
        )

    def test_simulate_read_data_rejects_minimum_larger_than_maximum(self) -> None:
        result = simulate_read_data(
            handle_valid=True,
            max_length=7,
            min_required=8,
            timeout_ms=100,
            start_tick=100,
            refill_chunks=[],
        )

        self.assertEqual(result.return_code, 0x0B)
        self.assertEqual(result.bytes_read, 0)

    def test_simulate_read_data_drains_existing_stage_before_refilling(self) -> None:
        result = simulate_read_data(
            handle_valid=True,
            max_length=6,
            min_required=6,
            timeout_ms=100,
            start_tick=100,
            state=AsUSBCommReadState(pending=b"ABC"),
            refill_chunks=[b"DEFGH"],
        )

        self.assertEqual(result.return_code, 0)
        self.assertEqual(result.data, b"ABCDEF")
        self.assertEqual(result.bytes_read, 6)
        self.assertEqual(result.state.pending, b"GH")

    def test_simulate_read_data_returns_one_on_refill_failure(self) -> None:
        result = simulate_read_data(
            handle_valid=True,
            max_length=8,
            min_required=1,
            timeout_ms=100,
            start_tick=100,
            refill_chunks=[None],
        )

        self.assertEqual(result.return_code, 1)
        self.assertEqual(result.bytes_read, 0)

    def test_simulate_read_data_returns_timeout_after_partial_read(self) -> None:
        result = simulate_read_data(
            handle_valid=True,
            max_length=8,
            min_required=8,
            timeout_ms=5,
            start_tick=100,
            refill_chunks=[b"ABC"],
            timeout_ticks=[106],
        )

        self.assertEqual(result.return_code, 0x0C)
        self.assertEqual(result.data, b"ABC")
        self.assertEqual(result.bytes_read, 3)


class AsUSBCommMacTests(unittest.TestCase):
    def test_build_set_mac_address_packet_uses_bytes_two_through_seven(self) -> None:
        packet = build_set_mac_address_packet(bytes.fromhex("00 00 aa bb cc dd ee ff"))

        self.assertEqual(packet, bytes.fromhex("20 aa bb cc dd ee ff 1b"))

    def test_interpret_set_mac_address_transaction_returns_negative_one_on_wrong_ack_opcode(self) -> None:
        result = interpret_set_mac_address_transaction(
            write_return_code=0,
            read_return_code=0,
            bytes_read=8,
            response=b"@0000000",
        )

        self.assertEqual(result, -1)

    def test_build_get_mac_address_packet_is_zero_opcode_command(self) -> None:
        self.assertEqual(build_get_mac_address_packet(), bytes.fromhex("00 00 00 00 00 00 00 00"))

    def test_interpret_get_mac_address_transaction_requires_eight_written_bytes(self) -> None:
        result = interpret_get_mac_address_transaction(
            write_return_code=0,
            bytes_written=7,
            header_read_return_code=0,
            header_response=b"@0000000",
            payload_read_return_codes=[],
            payload_blocks=[],
        )

        self.assertEqual(result.return_code, -1)
        self.assertIsNone(result.mac_bytes)

    def test_interpret_get_mac_address_transaction_extracts_last_eight_payload_bytes(self) -> None:
        result = interpret_get_mac_address_transaction(
            write_return_code=0,
            bytes_written=8,
            header_read_return_code=0,
            header_response=b"@0000000",
            payload_read_return_codes=[0] * 8,
            payload_blocks=[
                b"BLOCK001",
                b"BLOCK002",
                b"BLOCK003",
                b"BLOCK004",
                b"BLOCK005",
                b"BLOCK006",
                b"BLOCK007",
                b"MACBYTES",
            ],
        )

        self.assertEqual(
            result,
            AsUSBCommGetMacOutcome(
                return_code=0,
                mac_bytes=b"MACBYTES",
                collected_payload_length=64,
            ),
        )

    def test_interpret_get_mac_address_transaction_keeps_zero_return_code_on_late_payload_failure(self) -> None:
        result = interpret_get_mac_address_transaction(
            write_return_code=0,
            bytes_written=8,
            header_read_return_code=0,
            header_response=b"@0000000",
            payload_read_return_codes=[0, 0, 0, 1],
            payload_blocks=[
                b"BLOCK001",
                b"BLOCK002",
                b"BLOCK003",
                b"BLOCK004",
            ],
        )

        self.assertEqual(result.return_code, 0)
        self.assertIsNone(result.mac_bytes)
        self.assertEqual(result.collected_payload_length, 24)


if __name__ == "__main__":
    unittest.main()
