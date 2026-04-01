from dataclasses import dataclass


@dataclass(frozen=True)
class AlphaWordPlusRuntimeValue:
    value: int
    name: str
    summary: str


_NAMESPACE2_COMMANDS = {
    0x20001: AlphaWordPlusRuntimeValue(
        value=0x20001,
        name="reset_command_stream",
        summary="Reset or initialize the active AlphaWord command stream.",
    ),
    0x20002: AlphaWordPlusRuntimeValue(
        value=0x20002,
        name="write_payload",
        summary="Main write-side payload path for typed document bytes and control markers.",
    ),
    0x20006: AlphaWordPlusRuntimeValue(
        value=0x20006,
        name="continue_chunked_readback",
        summary="Continue chunked readback while the namespace-2 readback state is active.",
    ),
    0x2011F: AlphaWordPlusRuntimeValue(
        value=0x2011F,
        name="status_one_reply",
        summary="Return status code 1 through the namespace-2 command channel.",
    ),
}

_NAMESPACE2_CONTROL_SELECTORS = {
    0x83: AlphaWordPlusRuntimeValue(
        value=0x83,
        name="rebuild_transfer_pointers",
        summary="Rebuild AlphaWord transfer pointers from the current file.",
    ),
    0x84: AlphaWordPlusRuntimeValue(
        value=0x84,
        name="begin_chunked_readback",
        summary="Begin chunked readback of the current file through the namespace-2 path.",
    ),
    0x87: AlphaWordPlusRuntimeValue(
        value=0x87,
        name="validate_span_reaches_file_end",
        summary="Validate that the current QueryCurrentAlphaWordEditorSpan end reaches the full current file size.",
    ),
    0x88: AlphaWordPlusRuntimeValue(
        value=0x88,
        name="query_current_slot",
        summary="Return the current slot number plus one.",
    ),
    0x90: AlphaWordPlusRuntimeValue(
        value=0x90,
        name="query_current_span_end",
        summary="Return a 16-bit value derived from the current QueryCurrentAlphaWordEditorSpan end position.",
    ),
    0x91: AlphaWordPlusRuntimeValue(
        value=0x91,
        name="query_current_file_size",
        summary="Return a 16-bit value from QueryCurrentAlphaWordFileSize.",
    ),
}

for slot in range(1, 9):
    _NAMESPACE2_CONTROL_SELECTORS[slot] = AlphaWordPlusRuntimeValue(
        value=slot,
        name=f"select_slot_{slot}",
        summary=f"Select AlphaWord slot {slot}.",
    )

_NAMESPACE2_STREAM_STATES = {
    0: AlphaWordPlusRuntimeValue(
        value=0,
        name="idle",
        summary="No namespace-2 stream or control sub-mode is active.",
    ),
    3: AlphaWordPlusRuntimeValue(
        value=3,
        name="chunked_readback",
        summary="Chunked readback of the current file is active.",
    ),
    5: AlphaWordPlusRuntimeValue(
        value=5,
        name="namespace1_special_handshake",
        summary="Special handshake/readback mode entered through the namespace-1 path.",
    ),
    6: AlphaWordPlusRuntimeValue(
        value=6,
        name="control_selector",
        summary="The next payload byte is interpreted as a namespace-2 control selector.",
    ),
}


def describe_namespace2_command(command: int) -> AlphaWordPlusRuntimeValue:
    return _NAMESPACE2_COMMANDS[command]


def describe_namespace2_control_selector(selector: int) -> AlphaWordPlusRuntimeValue:
    return _NAMESPACE2_CONTROL_SELECTORS[selector]


def describe_namespace2_stream_state(state: int) -> AlphaWordPlusRuntimeValue:
    return _NAMESPACE2_STREAM_STATES[state]
