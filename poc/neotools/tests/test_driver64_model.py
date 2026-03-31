import unittest

from neotools.driver64_model import (
    Driver64CreateRoute,
    Driver64DeviceControlRoute,
    Driver64DispatchMap,
    Driver64Internal220003Request,
    Driver64InternalIoctlPlan,
    Driver64ProbeSequencePlan,
    Driver64PnpRoute,
    Driver64ReadWriteRoute,
    build_driver64_cancel_active_transfer_request,
    build_driver64_config_descriptor_full_request,
    build_driver64_config_descriptor_header_request,
    build_driver64_data_transfer_request,
    build_driver64_device_descriptor_request,
    build_driver64_endpoint_trigger_request,
    build_driver64_dispatch_map,
    build_driver64_probe_sequence_plan,
    describe_driver64_internal_ioctl,
    describe_driver64_urb_function,
    classify_driver64_create,
    classify_driver64_device_control,
    classify_driver64_pnp_minor,
    classify_driver64_read_write,
)


class Driver64DispatchMapTests(unittest.TestCase):
    def test_build_driver64_dispatch_map_matches_driver_entry_assignments(self) -> None:
        self.assertEqual(
            build_driver64_dispatch_map(),
            Driver64DispatchMap(
                create=0x11400,
                close=0x11528,
                device_control=0x115A4,
                pnp=0x119E4,
                power=0x12E98,
                system_control=0x13DD0,
                unload=0x12AF8,
            ),
        )


class Driver64DeviceControlTests(unittest.TestCase):
    def test_classify_device_control_maps_direct_output_copy_routes(self) -> None:
        self.assertEqual(
            classify_driver64_device_control(0x220000),
            Driver64DeviceControlRoute(kind="copy_cached_block", ioctl=0x220000),
        )
        self.assertEqual(
            classify_driver64_device_control(0x80002000),
            Driver64DeviceControlRoute(kind="copy_usb_device_descriptor", ioctl=0x80002000),
        )

    def test_classify_device_control_maps_private_helper_routes(self) -> None:
        self.assertEqual(
            classify_driver64_device_control(0x220004),
            Driver64DeviceControlRoute(
                kind="internal_probe_sequence",
                ioctl=0x220004,
                internal_plan=Driver64InternalIoctlPlan(first=0x220013, second=0x220007),
            ),
        )
        self.assertEqual(
            classify_driver64_device_control(0x220008),
            Driver64DeviceControlRoute(kind="file_handle_triggered_transfer", ioctl=0x220008),
        )

    def test_classify_device_control_rejects_unknown_ioctl(self) -> None:
        self.assertEqual(
            classify_driver64_device_control(0x12345678),
            Driver64DeviceControlRoute(kind="invalid_device_request", ioctl=0x12345678),
        )


class Driver64CreateTests(unittest.TestCase):
    def test_classify_create_for_control_handle_open(self) -> None:
        self.assertEqual(
            classify_driver64_create(state=2, has_configuration=True, file_name_suffix=None),
            Driver64CreateRoute(
                kind="control_handle",
                ntstatus=0,
                increments_open_count=True,
                cancels_timer_if_active=True,
                endpoint_index=None,
            ),
        )

    def test_classify_create_for_named_endpoint_handle(self) -> None:
        self.assertEqual(
            classify_driver64_create(state=2, has_configuration=True, file_name_suffix=5),
            Driver64CreateRoute(
                kind="endpoint_handle",
                ntstatus=0,
                increments_open_count=True,
                cancels_timer_if_active=True,
                endpoint_index=5,
            ),
        )

    def test_classify_create_rejects_unstarted_or_invalid_endpoint_requests(self) -> None:
        self.assertEqual(
            classify_driver64_create(state=1, has_configuration=True, file_name_suffix=None),
            Driver64CreateRoute(
                kind="device_not_ready",
                ntstatus=0xC0000184,
                increments_open_count=False,
                cancels_timer_if_active=False,
                endpoint_index=None,
            ),
        )
        self.assertEqual(
            classify_driver64_create(state=2, has_configuration=True, file_name_suffix=6),
            Driver64CreateRoute(
                kind="invalid_name_suffix",
                ntstatus=0xC000000D,
                increments_open_count=False,
                cancels_timer_if_active=False,
                endpoint_index=None,
            ),
        )


class Driver64ReadWriteTests(unittest.TestCase):
    def test_classify_read_write_for_explicit_endpoint_handle(self) -> None:
        self.assertEqual(
            classify_driver64_read_write(
                major_function=0x03,
                state=2,
                transfer_length=0x180,
                file_context_present=True,
                endpoint_type=2,
            ),
            Driver64ReadWriteRoute(
                kind="chunked_internal_transfer",
                ntstatus=0x103,
                direction="read",
                transfer_code=3,
                first_chunk_length=0x100,
                remaining_length=0x80,
                uses_ioctl=0x220003,
                falls_back_to_probe_sequence=True,
            ),
        )

    def test_classify_read_write_for_default_direction_match(self) -> None:
        self.assertEqual(
            classify_driver64_read_write(
                major_function=0x04,
                state=2,
                transfer_length=0x40,
                file_context_present=False,
                endpoint_type=3,
            ),
            Driver64ReadWriteRoute(
                kind="chunked_internal_transfer",
                ntstatus=0x103,
                direction="write",
                transfer_code=2,
                first_chunk_length=0x40,
                remaining_length=0,
                uses_ioctl=0x220003,
                falls_back_to_probe_sequence=True,
            ),
        )

    def test_classify_read_write_rejects_invalid_state_endpoint_or_length(self) -> None:
        self.assertEqual(
            classify_driver64_read_write(
                major_function=0x03,
                state=1,
                transfer_length=0x20,
                file_context_present=False,
                endpoint_type=2,
            ),
            Driver64ReadWriteRoute(
                kind="device_not_ready",
                ntstatus=0xC0000184,
                direction="read",
                transfer_code=3,
                first_chunk_length=0,
                remaining_length=0,
                uses_ioctl=0x220003,
                falls_back_to_probe_sequence=False,
            ),
        )
        self.assertEqual(
            classify_driver64_read_write(
                major_function=0x03,
                state=2,
                transfer_length=0x10002,
                file_context_present=False,
                endpoint_type=2,
            ),
            Driver64ReadWriteRoute(
                kind="invalid_transfer_length",
                ntstatus=0xC000000D,
                direction="read",
                transfer_code=3,
                first_chunk_length=0,
                remaining_length=0,
                uses_ioctl=0x220003,
                falls_back_to_probe_sequence=False,
            ),
        )
        self.assertEqual(
            classify_driver64_read_write(
                major_function=0x03,
                state=2,
                transfer_length=0x20,
                file_context_present=False,
                endpoint_type=1,
            ),
            Driver64ReadWriteRoute(
                kind="invalid_endpoint",
                ntstatus=0xC0000008,
                direction="read",
                transfer_code=3,
                first_chunk_length=0,
                remaining_length=0,
                uses_ioctl=0x220003,
                falls_back_to_probe_sequence=False,
            ),
        )


class Driver64PnpTests(unittest.TestCase):
    def test_classify_pnp_minor_maps_core_handlers(self) -> None:
        self.assertEqual(
            classify_driver64_pnp_minor(0x00),
            Driver64PnpRoute(kind="start_device", minor=0x00, handler="StartDeviceAndLoadUsbDescriptors"),
        )
        self.assertEqual(
            classify_driver64_pnp_minor(0x02),
            Driver64PnpRoute(kind="remove_device", minor=0x02, handler="HandleRemoveDevice"),
        )
        self.assertEqual(
            classify_driver64_pnp_minor(0x17),
            Driver64PnpRoute(kind="surprise_removal", minor=0x17, handler="HandleSurpriseRemoval"),
        )

    def test_classify_pnp_minor_maps_query_and_cancel_transitions(self) -> None:
        self.assertEqual(
            classify_driver64_pnp_minor(0x01),
            Driver64PnpRoute(kind="query_remove", minor=0x01, handler="forward_after_wait"),
        )
        self.assertEqual(
            classify_driver64_pnp_minor(0x05),
            Driver64PnpRoute(kind="query_stop", minor=0x05, handler="forward_after_wait"),
        )
        self.assertEqual(
            classify_driver64_pnp_minor(0x06),
            Driver64PnpRoute(kind="cancel_stop", minor=0x06, handler="HandleCancelStopDevice"),
        )

    def test_classify_pnp_minor_defaults_to_pass_through(self) -> None:
        self.assertEqual(
            classify_driver64_pnp_minor(0x0A),
            Driver64PnpRoute(kind="pass_through", minor=0x0A, handler="IofCallDriver"),
        )


class Driver64InternalRequestTests(unittest.TestCase):
    def test_build_device_descriptor_request_matches_fetch_path(self) -> None:
        self.assertEqual(
            build_driver64_device_descriptor_request(),
            Driver64Internal220003Request(
                size=0x88,
                function=0x0B,
                transfer_buffer_length=0x12,
                request_type=1,
                endpoint_pointer_offset=0x18,
                response_buffer_pointer_offset=0x14,
            ),
        )

    def test_build_config_descriptor_requests_match_header_then_full_fetch(self) -> None:
        self.assertEqual(
            build_driver64_config_descriptor_header_request(),
            Driver64Internal220003Request(
                size=0x88,
                function=0x0B,
                transfer_buffer_length=9,
                request_type=2,
                endpoint_pointer_offset=0x18,
                response_buffer_pointer_offset=0x14,
            ),
        )
        self.assertEqual(
            build_driver64_config_descriptor_full_request(total_length=0x39),
            Driver64Internal220003Request(
                size=0x88,
                function=0x0B,
                transfer_buffer_length=0x39,
                request_type=2,
                endpoint_pointer_offset=0x18,
                response_buffer_pointer_offset=0x14,
            ),
        )

    def test_build_endpoint_trigger_request_matches_220008_helper(self) -> None:
        self.assertEqual(
            build_driver64_endpoint_trigger_request(),
            Driver64Internal220003Request(
                size=0x28,
                function=0x1E,
                transfer_buffer_length=0,
                request_type=None,
                endpoint_pointer_offset=0x18,
                response_buffer_pointer_offset=None,
            ),
        )

    def test_build_data_transfer_request_matches_dispatch_read_write_layout(self) -> None:
        self.assertEqual(
            build_driver64_data_transfer_request(chunk_length=0x100, direction="read"),
            Driver64Internal220003Request(
                size=0x80,
                function=9,
                transfer_buffer_length=0x100,
                request_type=3,
                endpoint_pointer_offset=0x18,
                response_buffer_pointer_offset=None,
            ),
        )
        self.assertEqual(
            build_driver64_data_transfer_request(chunk_length=0x40, direction="write"),
            Driver64Internal220003Request(
                size=0x80,
                function=9,
                transfer_buffer_length=0x40,
                request_type=2,
                endpoint_pointer_offset=0x18,
                response_buffer_pointer_offset=None,
            ),
        )

    def test_build_cancel_active_transfer_request_matches_cancel_helper(self) -> None:
        self.assertEqual(
            build_driver64_cancel_active_transfer_request(),
            Driver64Internal220003Request(
                size=0x28,
                function=2,
                transfer_buffer_length=0,
                request_type=None,
                endpoint_pointer_offset=0x18,
                response_buffer_pointer_offset=None,
            ),
        )

    def test_build_probe_sequence_plan_models_220013_then_optional_220007(self) -> None:
        self.assertEqual(
            build_driver64_probe_sequence_plan(flags=0x00),
            Driver64ProbeSequencePlan(first_ioctl=0x220013, second_ioctl=None, flags=0x00),
        )
        self.assertEqual(
            build_driver64_probe_sequence_plan(flags=0x02),
            Driver64ProbeSequencePlan(first_ioctl=0x220013, second_ioctl=0x220007, flags=0x02),
        )
        self.assertEqual(
            build_driver64_probe_sequence_plan(flags=0x01),
            Driver64ProbeSequencePlan(first_ioctl=0x220013, second_ioctl=None, flags=0x01),
        )

    def test_describe_internal_usb_ioctl_names(self) -> None:
        self.assertEqual(
            describe_driver64_internal_ioctl(0x220003),
            "IOCTL_INTERNAL_USB_SUBMIT_URB",
        )
        self.assertEqual(
            describe_driver64_internal_ioctl(0x220013),
            "IOCTL_INTERNAL_USB_GET_PORT_STATUS",
        )
        self.assertEqual(
            describe_driver64_internal_ioctl(0x220007),
            "IOCTL_INTERNAL_USB_RESET_PORT",
        )

    def test_describe_urb_function_names(self) -> None:
        self.assertEqual(
            describe_driver64_urb_function(0x0B),
            "URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE",
        )
        self.assertEqual(
            describe_driver64_urb_function(9),
            "URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER",
        )
        self.assertEqual(
            describe_driver64_urb_function(2),
            "URB_FUNCTION_ABORT_PIPE",
        )
        self.assertEqual(
            describe_driver64_urb_function(0x1E),
            "URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL (inference)",
        )


if __name__ == "__main__":
    unittest.main()
