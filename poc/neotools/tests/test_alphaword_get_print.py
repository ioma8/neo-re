import unittest

from neotools.alphaword_get_print import (
    GetPrintFlowStep,
    UpdaterErrorLogEntry,
    build_retrieve_all_alpha_word_slots_for_device_flow,
    build_direct_usb_full_get_print_flow,
    build_direct_usb_preview_refresh_flow,
    build_retrieve_full_alpha_word_text_flow,
    build_updater_retrieve_applet_file_data_flow,
    classify_retrieve_opcode,
    get_retrieved_text_replacements,
    make_updater_error_log_entry,
)


class AlphaWordGetPrintFlowTests(unittest.TestCase):
    def test_classify_retrieve_opcode_distinguishes_interactive_and_save_archive_variants(self) -> None:
        self.assertEqual(classify_retrieve_opcode(0x12), "interactive_retrieve")
        self.assertEqual(classify_retrieve_opcode(0x1C), "save_archive_retrieve")

    def test_make_updater_error_log_entry_matches_retrieve_error_table_layout(self) -> None:
        entry = make_updater_error_log_entry(
            message="Error retrieving file.",
            operation="UpdaterRetrieveFile",
            source="C:\\AS Software\\OS3000\\Tool\\HostU...",
            source_line=0x12D8,
            error_code=0x12A,
            response_byte=0x4D,
            detail="Number of Device(s) compacted = %d.",
        )

        self.assertEqual(
            entry,
            UpdaterErrorLogEntry(
                message="Error retrieving file.",
                operation="UpdaterRetrieveFile",
                source="C:\\AS Software\\OS3000\\Tool\\HostU...",
                reserved=0,
                source_line=0x12D8,
                error_code=0x12A,
                response_byte=0x4D,
                detail="Number of Device(s) compacted = %d.",
            ),
        )

    def test_get_retrieved_text_replacements_returns_cr_crlf_and_space_constants(self) -> None:
        replacements = get_retrieved_text_replacements()

        self.assertEqual(replacements["cr"], "\r")
        self.assertEqual(replacements["crlf"], "\r\n")
        self.assertEqual(replacements["space"], " ")

    def test_updater_retrieve_applet_file_data_flow_happy_path_models_start_chunk_and_progress_close(self) -> None:
        result = build_updater_retrieve_applet_file_data_flow(
            command_selector=0x12,
            applet_id=0xA000,
            file_slot=2,
            requested_length=0x80000,
            reported_total_length=0x30,
            chunk_lengths=[0x10, 0x20],
            chunk_checksums=[0x0010, 0x0020],
            progress_current=0,
            progress_total=0,
        )

        self.assertEqual(result.return_code, 0)
        self.assertEqual(result.steps[0], GetPrintFlowStep(kind="controller", detail="UpdaterRetrieveAppletFileData"))
        self.assertEqual(result.steps[1], GetPrintFlowStep(kind="reset_last_updater_error"))
        self.assertEqual(result.steps[2], GetPrintFlowStep(kind="format_progress_caption", file_slot=2))
        self.assertEqual(result.steps[3], GetPrintFlowStep(kind="set_progress_percent", status=0))
        self.assertEqual(result.steps[4], GetPrintFlowStep(kind="send_start_command", file_slot=2, packet=bytes.fromhex("12 08 00 00 02 a0 00 bc")))
        self.assertEqual(result.steps[5], GetPrintFlowStep(kind="check_start_status_byte", status=0x53))
        self.assertEqual(result.steps[6], GetPrintFlowStep(kind="store_reported_total_length", status=0x30))
        self.assertEqual(result.steps[7], GetPrintFlowStep(kind="cap_total_length_to_requested", status=0x30))
        self.assertEqual(result.steps[8], GetPrintFlowStep(kind="set_progress_percent", status=0))
        self.assertEqual(result.steps[9], GetPrintFlowStep(kind="send_chunk_command", packet=bytes.fromhex("10 00 00 00 00 00 00 10")))
        self.assertEqual(result.steps[10], GetPrintFlowStep(kind="check_chunk_status_byte", status=0x4D))
        self.assertEqual(result.steps[11], GetPrintFlowStep(kind="read_chunk_payload", status=0x10))
        self.assertEqual(result.steps[12], GetPrintFlowStep(kind="write_chunk_to_sink", status=0x10))
        self.assertEqual(result.steps[13], GetPrintFlowStep(kind="accumulate_chunk_checksum", status=0x0010))
        self.assertEqual(result.steps[-3], GetPrintFlowStep(kind="verify_chunk_checksum", status=0x0020))
        self.assertEqual(result.steps[-2], GetPrintFlowStep(kind="set_progress_percent", status=100))
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="close_progress_scope"))

    def test_updater_retrieve_applet_file_data_flow_returns_invalid_response_when_start_byte_is_not_0x53(self) -> None:
        result = build_updater_retrieve_applet_file_data_flow(
            command_selector=0x12,
            applet_id=0xA000,
            file_slot=1,
            requested_length=0x80000,
            reported_total_length=0x10,
            chunk_lengths=[],
            chunk_checksums=[],
            start_status=0x52,
        )

        self.assertEqual(result.return_code, -1)
        self.assertEqual(result.steps[-2], GetPrintFlowStep(kind="set_last_updater_error_code", status=0x12A))
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="close_progress_scope"))

    def test_updater_retrieve_applet_file_data_flow_returns_bad_checksum_when_chunk_sum_mismatches(self) -> None:
        result = build_updater_retrieve_applet_file_data_flow(
            command_selector=0x12,
            applet_id=0xA000,
            file_slot=1,
            requested_length=0x80000,
            reported_total_length=0x10,
            chunk_lengths=[0x10],
            chunk_checksums=[0x0011],
            computed_chunk_checksums=[0x0010],
        )

        self.assertEqual(result.return_code, -1)
        self.assertEqual(result.steps[-2], GetPrintFlowStep(kind="set_last_updater_error_code", status=0x105))
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="close_progress_scope"))

    def test_updater_retrieve_applet_file_data_flow_uses_overall_progress_when_part_of_larger_transfer(self) -> None:
        result = build_updater_retrieve_applet_file_data_flow(
            command_selector=0x1C,
            applet_id=0xA000,
            file_slot=4,
            requested_length=0x80000,
            reported_total_length=0x20,
            chunk_lengths=[0x20],
            chunk_checksums=[0x0020],
            progress_current=0x40,
            progress_total=0x100,
        )

        self.assertEqual(result.return_code, 0)
        self.assertEqual(result.steps[2], GetPrintFlowStep(kind="set_progress_percent", status=25))
        self.assertEqual(result.steps[3], GetPrintFlowStep(kind="send_start_command", file_slot=4, packet=bytes.fromhex("1c 08 00 00 04 a0 00 c8")))
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="set_progress_percent", status=37))

    def test_retrieve_full_alpha_word_text_flow_for_direct_usb_uses_temp_sink_and_direct_transport(self) -> None:
        result = build_retrieve_full_alpha_word_text_flow(
            transport_mode=2,
            applet_id=0xA000,
            file_slot=2,
            retrieved_length=0x1234,
        )

        self.assertEqual(result.return_code, 0)
        self.assertEqual(result.steps[0], GetPrintFlowStep(kind="controller", detail="RetrieveFullAlphaWordText"))
        self.assertEqual(result.steps[1], GetPrintFlowStep(kind="dispatch_thread_pump"))
        self.assertEqual(result.steps[2], GetPrintFlowStep(kind="reset_retrieved_text_workspace"))
        self.assertEqual(result.steps[3], GetPrintFlowStep(kind="open_temporary_retrieved_text_file"))
        self.assertEqual(result.steps[4], GetPrintFlowStep(kind="build_retrieved_text_temp_path_base"))
        self.assertEqual(result.steps[5], GetPrintFlowStep(kind="open_retrieved_text_sink_for_write"))
        self.assertEqual(result.steps[6], GetPrintFlowStep(kind="seed_retrieved_text_sink"))
        self.assertEqual(result.steps[7], GetPrintFlowStep(kind="reopen_retrieved_text_sink_for_read"))
        self.assertEqual(result.steps[8], GetPrintFlowStep(kind="transport", detail="DirectUsbRetrieveAppletFileData"))
        self.assertEqual(result.steps[9], GetPrintFlowStep(kind="list_applets", file_slot=2, packet=bytes.fromhex("04 00 00 00 00 00 07 0b")))
        self.assertEqual(result.steps[11], GetPrintFlowStep(kind="retrieve_file", file_slot=2, packet=bytes.fromhex("12 08 00 00 02 a0 00 bc")))
        self.assertEqual(result.steps[13], GetPrintFlowStep(kind="close_file_descriptor"))
        self.assertEqual(result.steps[14], GetPrintFlowStep(kind="load_retrieved_text_file_as_cstring"))
        self.assertEqual(result.steps[15], GetPrintFlowStep(kind="read_retrieved_text_sink", status=0x1234))
        self.assertEqual(result.steps[16], GetPrintFlowStep(kind="normalize_retrieved_text_bytes"))
        self.assertEqual(result.steps[17], GetPrintFlowStep(kind="delete_temporary_retrieved_text_file"))
        self.assertEqual(result.steps[18], GetPrintFlowStep(kind="assign_output_cstring"))
        self.assertEqual(result.steps[19], GetPrintFlowStep(kind="finalize_retrieved_text_workspace", status=0))

    def test_retrieve_full_alpha_word_text_flow_for_alternate_transport_uses_port_selector(self) -> None:
        result = build_retrieve_full_alpha_word_text_flow(
            transport_mode=3,
            applet_id=0xA000,
            file_slot=4,
            retrieved_length=0x20,
            alternate_selector=7,
        )

        self.assertEqual(result.return_code, 0)
        self.assertIn(GetPrintFlowStep(kind="transport", detail="AlternateTransportRetrieveAppletFileData", status=7), result.steps)

    def test_retrieve_full_alpha_word_text_flow_for_unsupported_transport_skips_transport_call(self) -> None:
        result = build_retrieve_full_alpha_word_text_flow(
            transport_mode=1,
            applet_id=0xA000,
            file_slot=1,
            retrieved_length=0x10,
        )

        self.assertEqual(result.return_code, 0)
        transport_steps = [step for step in result.steps if step.kind == "transport"]
        self.assertEqual(transport_steps, [])
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="finalize_retrieved_text_workspace", status=0))

    def test_retrieve_full_alpha_word_text_flow_with_temp_sink_failure_returns_after_finalize(self) -> None:
        result = build_retrieve_full_alpha_word_text_flow(
            transport_mode=2,
            applet_id=0xA000,
            file_slot=1,
            retrieved_length=0x10,
            temp_open_success=False,
        )

        self.assertEqual(result.return_code, 0)
        self.assertNotIn(GetPrintFlowStep(kind="transport", detail="DirectUsbRetrieveAppletFileData"), result.steps)
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="finalize_retrieved_text_workspace", status=0))

    def test_retrieve_all_alpha_word_slots_flow_skips_slots_already_at_status_2(self) -> None:
        result = build_retrieve_all_alpha_word_slots_for_device_flow(
            applet_id=0xA000,
            file_slots=[1, 2, 3],
            initial_statuses={1: 2, 2: 0, 3: 2},
        )

        self.assertEqual(result.return_code, 0)
        self.assertEqual(result.steps[0], GetPrintFlowStep(kind="controller", detail="RetrieveAllAlphaWordSlotsForDevice"))
        self.assertEqual(result.steps[1], GetPrintFlowStep(kind="load_global_empty_string"))
        self.assertEqual(result.steps[2], GetPrintFlowStep(kind="prepare_retrieval_loop"))
        self.assertEqual(result.steps[3], GetPrintFlowStep(kind="format_progress_text", file_slot=1))
        self.assertEqual(result.steps[4], GetPrintFlowStep(kind="set_progress_dialog_text", file_slot=1))
        self.assertEqual(result.steps[5], GetPrintFlowStep(kind="read_slot_status", file_slot=1, status=2))
        self.assertEqual(result.steps[6], GetPrintFlowStep(kind="skip_slot_already_loaded", file_slot=1, status=2))
        retrieval_slots = [step.file_slot for step in result.steps if step.kind == "retrieve_file"]
        self.assertEqual(retrieval_slots, [2])
        self.assertEqual(result.steps[-2], GetPrintFlowStep(kind="read_slot_status", file_slot=3, status=2))
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="skip_slot_already_loaded", file_slot=3, status=2))

    def test_retrieve_all_alpha_word_slots_flow_restores_previous_status_on_retrieval_failure(self) -> None:
        result = build_retrieve_all_alpha_word_slots_for_device_flow(
            applet_id=0xA000,
            file_slots=[1, 2],
            initial_statuses={1: 1, 2: 0},
            fail_slot=1,
        )

        self.assertEqual(result.return_code, 0x2A)
        self.assertIn(GetPrintFlowStep(kind="remember_previous_status", file_slot=1, status=1), result.steps)
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="restore_previous_status_on_error", file_slot=1, status=1))

    def test_retrieve_all_alpha_word_slots_flow_returns_cancel_code_after_slot_commit(self) -> None:
        result = build_retrieve_all_alpha_word_slots_for_device_flow(
            applet_id=0xA000,
            file_slots=[1, 2],
            cancel_after_slot=1,
        )

        self.assertEqual(result.return_code, 0x29)
        self.assertEqual(result.steps[-2], GetPrintFlowStep(kind="read_progress_dialog_cancel_code", file_slot=1, status=0x33))
        self.assertEqual(result.steps[-1], GetPrintFlowStep(kind="return_cancelled", file_slot=1, status=0x29))

    def test_preview_refresh_flow_bootstraps_once_and_marks_preview_cache_updates(self) -> None:
        flow = build_direct_usb_preview_refresh_flow(applet_id=0xA000, file_slots=[1, 2])

        self.assertEqual(flow[0], GetPrintFlowStep(kind="controller", detail="RefreshAlphaWordPreviewCacheForTreeItem"))
        self.assertEqual(flow[1].kind, "reset_connection")
        self.assertEqual(flow[1].packet, bytes.fromhex("3f ff 00 72 65 73 65 74"))
        self.assertEqual(flow[2].kind, "switch_to_updater")
        self.assertEqual(flow[3].kind, "list_applets")
        self.assertEqual(flow[3].file_slot, 1)
        self.assertEqual(flow[4].kind, "raw_file_attributes")
        self.assertEqual(flow[5].kind, "retrieve_file")
        self.assertEqual(flow[5].packet, bytes.fromhex("12 00 00 b4 01 a0 00 67"))
        self.assertEqual(flow[7], GetPrintFlowStep(kind="cache_preview_text", file_slot=1, status=1))
        self.assertEqual(flow[8], GetPrintFlowStep(kind="cache_preview_size", file_slot=1, status=1))
        self.assertEqual(flow[-2], GetPrintFlowStep(kind="cache_preview_text", file_slot=2, status=1))
        self.assertEqual(flow[-1], GetPrintFlowStep(kind="cache_preview_size", file_slot=2, status=1))

    def test_full_get_print_flow_for_all_slots_adds_file_name_and_full_cache_updates(self) -> None:
        flow = build_direct_usb_full_get_print_flow(applet_id=0xA000, file_slots=[1, 2])

        self.assertEqual(flow[0], GetPrintFlowStep(kind="controller", detail="ExecuteGetPrintAlphaWordFlow"))
        self.assertEqual(flow[1], GetPrintFlowStep(kind="controller", detail="RetrieveAllAlphaWordSlotsForDevice"))
        self.assertEqual(flow[2].kind, "reset_connection")
        self.assertEqual(flow[3].kind, "switch_to_updater")
        self.assertEqual(flow[4].kind, "list_applets")
        self.assertEqual(flow[6].kind, "retrieve_file")
        self.assertEqual(flow[6].packet, bytes.fromhex("12 08 00 00 01 a0 00 bb"))
        self.assertEqual(flow[8], GetPrintFlowStep(kind="get_file_name", file_slot=1))
        self.assertEqual(flow[9], GetPrintFlowStep(kind="cache_full_text", file_slot=1, status=2))
        self.assertEqual(flow[10], GetPrintFlowStep(kind="cache_full_size", file_slot=1, status=2))
        self.assertEqual(flow[11], GetPrintFlowStep(kind="cache_file_name", file_slot=1, status=2))
        self.assertEqual(flow[-4], GetPrintFlowStep(kind="get_file_name", file_slot=2))
        self.assertEqual(flow[-3], GetPrintFlowStep(kind="cache_full_text", file_slot=2, status=2))
        self.assertEqual(flow[-2], GetPrintFlowStep(kind="cache_full_size", file_slot=2, status=2))
        self.assertEqual(flow[-1], GetPrintFlowStep(kind="cache_file_name", file_slot=2, status=2))

    def test_full_get_print_flow_for_selected_slots_uses_selected_controller(self) -> None:
        flow = build_direct_usb_full_get_print_flow(applet_id=0xA000, file_slots=[2, 4], selected_only=True)

        self.assertEqual(flow[1], GetPrintFlowStep(kind="controller", detail="RetrieveSelectedAlphaWordSlotsForDevice"))
        retrieval_slots = [step.file_slot for step in flow if step.kind == "retrieve_file"]
        self.assertEqual(retrieval_slots, [2, 4])

    def test_full_get_print_flow_for_single_slot_uses_single_slot_controller(self) -> None:
        flow = build_direct_usb_full_get_print_flow(applet_id=0xA000, file_slots=[3], single_slot=True)

        self.assertEqual(flow[1], GetPrintFlowStep(kind="controller", detail="RetrieveSingleAlphaWordSlotForDevice"))
        retrieval_slots = [step.file_slot for step in flow if step.kind == "retrieve_file"]
        self.assertEqual(retrieval_slots, [3])


if __name__ == "__main__":
    unittest.main()
