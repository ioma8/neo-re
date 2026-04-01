from dataclasses import dataclass

from neotools.os3kapp_format import Os3kAppHeaderFields, Os3kAppImage, build_os3kapp_image


@dataclass(frozen=True)
class Os3kAppCommand:
    raw: int
    namespace_byte: int
    selector_byte: int
    low_word: int
    uses_custom_dispatch: bool
    lifecycle_name: str | None


@dataclass(frozen=True)
class Os3kAppEntryAbi:
    entry_offset: int
    loader_stub_length: int
    init_opcode: int
    shutdown_opcode: int
    shutdown_status: int
    call_block_words: int
    input_length_index: int
    input_pointer_index: int
    output_capacity_index: int
    output_length_index: int
    output_buffer_pointer_index: int


@dataclass(frozen=True)
class Os3kAppTrapStub:
    file_offset: int
    opcode: int
    family_byte: int
    selector_byte: int
    inferred_name: str | None


@dataclass(frozen=True)
class Os3kAppTrapBlock:
    start_file_offset: int
    end_file_offset: int
    stubs: tuple[Os3kAppTrapStub, ...]


@dataclass(frozen=True)
class Os3kAppTrapPrototype:
    opcode: int
    name: str
    stack_argument_count: int
    return_kind: str
    notes: str


@dataclass(frozen=True)
class Os3kAppCommandPrototype:
    applet_name: str
    raw_command: int
    selector_byte: int
    handler_name: str
    status_code: int
    response_word_count: int
    notes: str


@dataclass(frozen=True)
class Os3kAppPayloadSubcommandPrototype:
    applet_name: str
    parent_command: int
    first_input_byte: int
    status_code: int | None
    response_length: int
    notes: str


KNOWN_TRAP_NAMES: dict[int, str] = {
    0xA000: "clear_text_screen",
    0xA004: "set_text_row_column_width",
    0xA008: "get_text_row_col",
    0xA010: "draw_predefined_glyph",
    0xA014: "draw_c_string_at_current_position",
    0xA020: "prepare_text_row_span",
    0xA094: "read_key_code",
    0xA098: "flush_text_frame",
    0xA09C: "is_key_ready",
    0xA0A4: "pump_ui_events",
    0xA0D4: "delay_ticks",
    0xA0F0: "render_wrapped_text_block",
    0xA0F4: "define_text_layout_slot",
    0xA0F8: "register_allowed_key",
    0xA190: "begin_output_builder",
    0xA198: "append_output_bytes",
    0xA1B4: "query_numeric_state",
    0xA1C8: "query_object_metric",
    0xA1CC: "commit_editable_buffer",
    0xA1F8: "begin_chooser_row_builder",
    0xA1FC: "advance_current_output_row",
    0xA200: "append_current_chooser_row",
    0xA204: "highlight_current_chooser_row",
    0xA208: "begin_chooser_input_session",
    0xA20C: "read_chooser_event_code",
    0xA210: "read_chooser_action_selector",
    0xA214: "read_chooser_selection_value",
    0xA25C: "yield_until_event",
    0xA364: "query_active_service_available",
    0xA368: "shared_runtime_a368",
    0xA36C: "query_active_service_status",
    0xA388: "query_active_service_disabled_state",
    0xA38C: "shared_runtime_a38c",
    0xA378: "shared_runtime_a378",
    0xA390: "shared_runtime_a390",
    0xA39C: "shared_runtime_a39c",
}


KNOWN_TRAP_PROTOTYPES: dict[int, Os3kAppTrapPrototype] = {
    0xA000: Os3kAppTrapPrototype(
        opcode=0xA000,
        name="clear_text_screen",
        stack_argument_count=0,
        return_kind="none",
        notes="screen clear / frame reset inferred from calculator menu redraw entry",
    ),
    0xA004: Os3kAppTrapPrototype(
        opcode=0xA004,
        name="set_text_row_column_width",
        stack_argument_count=3,
        return_kind="none",
        notes="row/column/width layout primitive inferred from calculator menu loop",
    ),
    0xA008: Os3kAppTrapPrototype(
        opcode=0xA008,
        name="get_text_row_col",
        stack_argument_count=2,
        return_kind="none",
        notes="writes two byte-sized row/column-like outputs that are then reused by later text-layout traps",
    ),
    0xA010: Os3kAppTrapPrototype(
        opcode=0xA010,
        name="draw_predefined_glyph",
        stack_argument_count=1,
        return_kind="none",
        notes="single small integer glyph id inferred from selection-marker call sites",
    ),
    0xA014: Os3kAppTrapPrototype(
        opcode=0xA014,
        name="draw_c_string_at_current_position",
        stack_argument_count=1,
        return_kind="none",
        notes="C-string pointer is passed on stack immediately after host string lookup",
    ),
    0xA020: Os3kAppTrapPrototype(
        opcode=0xA020,
        name="prepare_text_row_span",
        stack_argument_count=3,
        return_kind="none",
        notes="prepares a text region using row/column/width-style arguments before redraw work",
    ),
    0xA094: Os3kAppTrapPrototype(
        opcode=0xA094,
        name="read_key_code",
        stack_argument_count=0,
        return_kind="value",
        notes="returns current key code after readiness has been observed",
    ),
    0xA098: Os3kAppTrapPrototype(
        opcode=0xA098,
        name="flush_text_frame",
        stack_argument_count=0,
        return_kind="none",
        notes="shared no-argument UI/text helper observed after redraw batches; likely flushes or commits the current text frame",
    ),
    0xA09C: Os3kAppTrapPrototype(
        opcode=0xA09C,
        name="is_key_ready",
        stack_argument_count=0,
        return_kind="value",
        notes="polled in a wait loop before calling read_key_code",
    ),
    0xA0A4: Os3kAppTrapPrototype(
        opcode=0xA0A4,
        name="pump_ui_events",
        stack_argument_count=0,
        return_kind="none",
        notes="called while idling for input to keep UI state moving",
    ),
    0xA0D4: Os3kAppTrapPrototype(
        opcode=0xA0D4,
        name="delay_ticks",
        stack_argument_count=1,
        return_kind="none",
        notes="single timing-like argument used between visible UI transitions; may be pacing or timeout rather than a strict blocking sleep",
    ),
    0xA0F0: Os3kAppTrapPrototype(
        opcode=0xA0F0,
        name="render_wrapped_text_block",
        stack_argument_count=5,
        return_kind="none",
        notes="shared text-rendering helper used heavily by spellcheck and earlier alphaquiz/calculator UI paths; the call shape looks like text/source plus row/column/height/width-style layout arguments",
    ),
    0xA0F4: Os3kAppTrapPrototype(
        opcode=0xA0F4,
        name="define_text_layout_slot",
        stack_argument_count=6,
        return_kind="none",
        notes="shared companion to render_wrapped_text_block that prepares a text slot/rectangle before later drawing; exact field semantics are still provisional",
    ),
    0xA0F8: Os3kAppTrapPrototype(
        opcode=0xA0F8,
        name="register_allowed_key",
        stack_argument_count=1,
        return_kind="none",
        notes="single-key registration/filter helper used while constructing accepted input-key sets before event loops",
    ),
    0xA190: Os3kAppTrapPrototype(
        opcode=0xA190,
        name="begin_output_builder",
        stack_argument_count=3,
        return_kind="none",
        notes="initializes or clears an output destination before text bytes are appended",
    ),
    0xA198: Os3kAppTrapPrototype(
        opcode=0xA198,
        name="append_output_bytes",
        stack_argument_count=4,
        return_kind="none",
        notes="appends a byte span into the active output destination; commonly called with mode=1",
    ),
    0xA1B4: Os3kAppTrapPrototype(
        opcode=0xA1B4,
        name="query_numeric_state",
        stack_argument_count=1,
        return_kind="value",
        notes="returns a scalar runtime value keyed by one argument; used in comparisons and follow-on updates",
    ),
    0xA1C8: Os3kAppTrapPrototype(
        opcode=0xA1C8,
        name="query_object_metric",
        stack_argument_count=2,
        return_kind="value",
        notes="returns a measurable scalar property of a previously resolved runtime object or token",
    ),
    0xA1CC: Os3kAppTrapPrototype(
        opcode=0xA1CC,
        name="commit_editable_buffer",
        stack_argument_count=0,
        return_kind="none",
        notes="closes an editable-buffer transaction after begin/get-buffer helpers have exposed mutable storage to the applet",
    ),
    0xA1F8: Os3kAppTrapPrototype(
        opcode=0xA1F8,
        name="begin_chooser_row_builder",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus calls this once before assembling chooser/list rows; it appears to start a row-builder or chooser-row accumulation session",
    ),
    0xA1FC: Os3kAppTrapPrototype(
        opcode=0xA1FC,
        name="advance_current_output_row",
        stack_argument_count=2,
        return_kind="value",
        notes="shared row-builder helper: AlphaWordPlus calls it after composing each chooser line, and its return value is checked when newline-delimited text is expanded into rows; the exact two-word parameter meaning is still unresolved",
    ),
    0xA200: Os3kAppTrapPrototype(
        opcode=0xA200,
        name="append_current_chooser_row",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus uses this immediately after building each chooser row; it appears to append or commit the current row into the active chooser list",
    ),
    0xA204: Os3kAppTrapPrototype(
        opcode=0xA204,
        name="highlight_current_chooser_row",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus uses this when the just-appended row matches the current selection, before entering the chooser event loop",
    ),
    0xA208: Os3kAppTrapPrototype(
        opcode=0xA208,
        name="begin_chooser_input_session",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus calls this once after finishing chooser-row setup and before reading chooser event codes",
    ),
    0xA20C: Os3kAppTrapPrototype(
        opcode=0xA20C,
        name="read_chooser_event_code",
        stack_argument_count=0,
        return_kind="value",
        notes="returns the next chooser/menu event code such as 'H', '@', '\\x18', '\\x17', or application-specific command bytes",
    ),
    0xA210: Os3kAppTrapPrototype(
        opcode=0xA210,
        name="read_chooser_action_selector",
        stack_argument_count=0,
        return_kind="value",
        notes="returns an auxiliary chooser selector used after '@' or item-activation events to identify the selected row/action slot",
    ),
    0xA214: Os3kAppTrapPrototype(
        opcode=0xA214,
        name="read_chooser_selection_value",
        stack_argument_count=0,
        return_kind="value",
        notes="returns a 16-bit chooser selection value consumed by AlphaWordPlus after successful chooser completion",
    ),
    0xA25C: Os3kAppTrapPrototype(
        opcode=0xA25C,
        name="yield_until_event",
        stack_argument_count=0,
        return_kind="none",
        notes="paired with pump_ui_events in the calculator idle loop",
    ),
    0xA364: Os3kAppTrapPrototype(
        opcode=0xA364,
        name="query_active_service_available",
        stack_argument_count=0,
        return_kind="value",
        notes="AlphaWordPlus uses this as a feature-availability query before beamer, wireless transfer, and spell-check flows; it behaves like a current-service installed/available check rather than a feature-specific API",
    ),
    0xA378: Os3kAppTrapPrototype(
        opcode=0xA378,
        name="shared_runtime_a378",
        stack_argument_count=0,
        return_kind="unknown",
        notes="shared A3xx runtime helper used by both calculator and alphaquiz, but semantics are still unresolved",
    ),
    0xA390: Os3kAppTrapPrototype(
        opcode=0xA390,
        name="shared_runtime_a390",
        stack_argument_count=1,
        return_kind="value",
        notes="shared A3xx helper returning a scalar or pointer-like value from at least one explicit argument",
    ),
    0xA39C: Os3kAppTrapPrototype(
        opcode=0xA39C,
        name="shared_runtime_a39c",
        stack_argument_count=0,
        return_kind="unknown",
        notes="shared A3xx side-effect helper observed in both calculator and alphaquiz, likely copy or unpack related but not pinned further",
    ),
    0xA36C: Os3kAppTrapPrototype(
        opcode=0xA36C,
        name="query_active_service_status",
        stack_argument_count=0,
        return_kind="value",
        notes="current-service status/session query used after feature-specific setup in AlphaWordPlus; in different callers it gates spell-check, beamer, wireless-transfer, and file-selector readiness, so the shared name remains generic on purpose",
    ),
    0xA368: Os3kAppTrapPrototype(
        opcode=0xA368,
        name="shared_runtime_a368",
        stack_argument_count=0,
        return_kind="unknown",
        notes="shared A3xx runtime helper present in calculator imports, but not semantically pinned enough for a stronger cross-sample name",
    ),
    0xA388: Os3kAppTrapPrototype(
        opcode=0xA388,
        name="query_active_service_disabled_state",
        stack_argument_count=0,
        return_kind="value",
        notes="AlphaWordPlus uses this in the spell-check enable/disable flow, where zero means enabled and nonzero means turned off; it may be a generic disabled-state query for the currently selected service",
    ),
    0xA38C: Os3kAppTrapPrototype(
        opcode=0xA38C,
        name="shared_runtime_a38c",
        stack_argument_count=0,
        return_kind="unknown",
        notes="shared A3xx runtime helper present in calculator imports; the earlier input-buffer interpretation is not yet strong enough cross-sample",
    ),
}


KNOWN_APPLET_COMMAND_PROTOTYPES: dict[tuple[str, int], Os3kAppCommandPrototype] = {
    ("alphawordplus", 0x10001): Os3kAppCommandPrototype(
        applet_name="alphawordplus",
        raw_command=0x10001,
        selector_byte=0x01,
        handler_name="HandleAlphaWordNamespace1Commands",
        status_code=0x11,
        response_word_count=0,
        notes="resets the namespace-1 command-stream state, refreshes current file pointers, and marks the namespace as active",
    ),
    ("alphawordplus", 0x10004): Os3kAppCommandPrototype(
        applet_name="alphawordplus",
        raw_command=0x10004,
        selector_byte=0x01,
        handler_name="HandleAlphaWordNamespace1Commands",
        status_code=0,
        response_word_count=0,
        notes="returns one response byte containing the currently selected AlphaWord slot number (1..8)",
    ),
    ("alphawordplus", 0x20001): Os3kAppCommandPrototype(
        applet_name="alphawordplus",
        raw_command=0x20001,
        selector_byte=0x02,
        handler_name="HandleAlphaWordNamespace2Commands",
        status_code=0x11,
        response_word_count=0,
        notes="resets the namespace-2 import/export stream state before the following transfer phase",
    ),
    ("alphawordplus", 0x20002): Os3kAppCommandPrototype(
        applet_name="alphawordplus",
        raw_command=0x20002,
        selector_byte=0x02,
        handler_name="HandleAlphaWordNamespace2Commands",
        status_code=0,
        response_word_count=0,
        notes="accepts incoming transferred bytes, decodes them through the transferred-byte tables, and updates the active namespace-2 stream until an in-band 0xbb terminator arrives",
    ),
    ("alphawordplus", 0x40001): Os3kAppCommandPrototype(
        applet_name="alphawordplus",
        raw_command=0x40001,
        selector_byte=0x04,
        handler_name="HandleAlphaWordNamespace4Commands",
        status_code=0x11,
        response_word_count=0,
        notes="resets the namespace-4 command stream and prepares the namespace-4 status/help UI block for the next payload command",
    ),
    ("alphawordplus", 0x40002): Os3kAppCommandPrototype(
        applet_name="alphawordplus",
        raw_command=0x40002,
        selector_byte=0x04,
        handler_name="HandleAlphaWordNamespace4Commands",
        status_code=0,
        response_word_count=0,
        notes="routes to the namespace-4 byte-payload helper, which either emits an immediate error reply byte or starts/continues a streamed response",
    ),
    ("alphawordplus", 0x70001): Os3kAppCommandPrototype(
        applet_name="alphawordplus",
        raw_command=0x70001,
        selector_byte=0x07,
        handler_name="HandleAlphaWordNamespace7Commands",
        status_code=0x11,
        response_word_count=0,
        notes="resets the namespace-7 command stream and redraws the namespace-7 status prompt block",
    ),
    ("alphawordplus", 0x70002): Os3kAppCommandPrototype(
        applet_name="alphawordplus",
        raw_command=0x70002,
        selector_byte=0x07,
        handler_name="HandleAlphaWordNamespace7Commands",
        status_code=0,
        response_word_count=0,
        notes="routes namespace-7 payload bytes through the same encoded transfer machinery used by the other AlphaWordPlus stream handlers",
    ),
    ("alphaquiz", 0x40001): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x40001,
        selector_byte=0x04,
        handler_name="HandleAlphaQuizNamespace4Commands",
        status_code=0x11,
        response_word_count=0,
        notes="refreshes namespace-4 quiz state and then redraws the three-line status block",
    ),
    ("alphaquiz", 0x40002): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x40002,
        selector_byte=0x04,
        handler_name="HandleAlphaQuizNamespace4Commands",
        status_code=0,
        response_word_count=0,
        notes="routes to the byte-oriented helper used for namespace-4 command payload decoding",
    ),
    ("alphaquiz", 0x4000C): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x4000C,
        selector_byte=0x04,
        handler_name="HandleAlphaQuizNamespace4Commands",
        status_code=0,
        response_word_count=0,
        notes="runs the namespace-4 side-effect-only cleanup/reset helper",
    ),
    ("alphaquiz", 0x50001): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x50001,
        selector_byte=0x05,
        handler_name="HandleAlphaQuizNamespace5Commands",
        status_code=0x11,
        response_word_count=0,
        notes="refreshes namespace-5 quiz state and redraws the two-line status block",
    ),
    ("alphaquiz", 0x50002): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x50002,
        selector_byte=0x05,
        handler_name="HandleAlphaQuizNamespace5Commands",
        status_code=0,
        response_word_count=0,
        notes="routes to the byte-oriented helper for namespace-5 payload handling",
    ),
    ("alphaquiz", 0x50005): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x50005,
        selector_byte=0x05,
        handler_name="HandleAlphaQuizNamespace5Commands",
        status_code=0,
        response_word_count=0,
        notes="shares the same byte-oriented helper as 0x50002",
    ),
    ("alphaquiz", 0x5000C): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x5000C,
        selector_byte=0x05,
        handler_name="HandleAlphaQuizNamespace5Commands",
        status_code=0,
        response_word_count=0,
        notes="runs the namespace-5 side-effect-only cleanup/reset helper",
    ),
    ("alphaquiz", 0x60001): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x60001,
        selector_byte=0x06,
        handler_name="HandleAlphaQuizNamespace6Commands",
        status_code=0x11,
        response_word_count=0,
        notes="copies up to 0x27 input bytes into the applet-global title buffer and NUL-terminates it",
    ),
    ("alphaquiz", 0x6000D): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x6000D,
        selector_byte=0x06,
        handler_name="HandleAlphaQuizNamespace6Commands",
        status_code=4,
        response_word_count=1,
        notes="when the two runtime selection values differ, writes one 32-bit value into the output buffer",
    ),
    ("alphaquiz", 0x60010): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x60010,
        selector_byte=0x06,
        handler_name="HandleAlphaQuizNamespace6Commands",
        status_code=0,
        response_word_count=0,
        notes="sets the redraw mode flag to 2 and redraws the common three-line namespace-6 status block",
    ),
    ("alphaquiz", 0x60011): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x60011,
        selector_byte=0x06,
        handler_name="HandleAlphaQuizNamespace6Commands",
        status_code=0,
        response_word_count=0,
        notes="sets the redraw mode flag to 1 and redraws the common three-line namespace-6 status block",
    ),
    ("alphaquiz", 0x60020): Os3kAppCommandPrototype(
        applet_name="alphaquiz",
        raw_command=0x60020,
        selector_byte=0x06,
        handler_name="HandleAlphaQuizNamespace6Commands",
        status_code=0,
        response_word_count=0,
        notes="only when the first input byte is ASCII 'H' does it draw the help prompt, wait, and then set status 8",
    ),
}


KNOWN_APPLET_PAYLOAD_SUBCOMMAND_PROTOTYPES: dict[tuple[str, int, int], Os3kAppPayloadSubcommandPrototype] = {
    ("alphaquiz", 0x40002, 0x0A): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x40002,
        first_input_byte=0x0A,
        status_code=0x0F,
        response_length=0,
        notes="immediate special case with status 0x0f and no response payload",
    ),
    ("alphaquiz", 0x40002, 0x1A): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x40002,
        first_input_byte=0x1A,
        status_code=None,
        response_length=-1,
        notes="copies input, then calls the namespace-4 helper with mode 8; status 4 only when the helper returns a nonzero output length",
    ),
    ("alphaquiz", 0x40002, 0x1B): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x40002,
        first_input_byte=0x1B,
        status_code=None,
        response_length=-1,
        notes="copies input, then calls the alternate namespace-4 helper; status 4 only when the helper returns a nonzero output length",
    ),
    ("alphaquiz", 0x40002, 0x1D): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x40002,
        first_input_byte=0x1D,
        status_code=4,
        response_length=2,
        notes="clears the UI and writes the fixed two-byte reply 0x5d 0x02",
    ),
    ("alphaquiz", 0x40002, 0x1E): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x40002,
        first_input_byte=0x1E,
        status_code=4,
        response_length=2,
        notes="writes the fixed two-byte reply 0x5e 0x02 and returns the helper flag to the caller",
    ),
    ("alphaquiz", 0x40002, 0x3F): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x40002,
        first_input_byte=0x3F,
        status_code=8,
        response_length=0,
        notes="immediate special case with status 8 and no response payload",
    ),
    ("alphaquiz", 0x50002, 0x1A): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x50002,
        first_input_byte=0x1A,
        status_code=None,
        response_length=-1,
        notes="copies input, then calls the namespace-5 helper with mode 0x3f; status 4 only when the helper returns a nonzero output length",
    ),
    ("alphaquiz", 0x50002, 0x1D): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x50002,
        first_input_byte=0x1D,
        status_code=4,
        response_length=2,
        notes="clears the UI and writes the fixed two-byte reply 0x5d 0x02",
    ),
    ("alphaquiz", 0x50005, 0x00): Os3kAppPayloadSubcommandPrototype(
        applet_name="alphaquiz",
        parent_command=0x50005,
        first_input_byte=0x00,
        status_code=4,
        response_length=1,
        notes="fallback path echoes the first input byte with | 0x80 into a one-byte response",
    ),
}


def decompose_os3kapp_command(raw: int) -> Os3kAppCommand:
    lifecycle_name = None
    if raw == 0x18:
        lifecycle_name = "initialize"
    elif raw == 0x19:
        lifecycle_name = "shutdown"

    return Os3kAppCommand(
        raw=raw,
        namespace_byte=(raw >> 24) & 0xFF,
        selector_byte=(raw >> 16) & 0xFF,
        low_word=raw & 0xFFFF,
        uses_custom_dispatch=((raw >> 24) & 0xFF) != 0,
        lifecycle_name=lifecycle_name,
    )


def build_os3kapp_entry_abi(image: Os3kAppImage) -> Os3kAppEntryAbi:
    return Os3kAppEntryAbi(
        entry_offset=image.entry_offset,
        loader_stub_length=len(image.loader_stub),
        init_opcode=0x18,
        shutdown_opcode=0x19,
        shutdown_status=7,
        call_block_words=5,
        input_length_index=0,
        input_pointer_index=1,
        output_capacity_index=2,
        output_length_index=3,
        output_buffer_pointer_index=4,
    )


def scan_os3kapp_trap_blocks(image: Os3kAppImage, *, minimum_stub_count: int = 4) -> tuple[Os3kAppTrapBlock, ...]:
    blocks: list[Os3kAppTrapBlock] = []
    current: list[Os3kAppTrapStub] = []
    current_start = 0

    for payload_offset in range(0, len(image.payload) - 1, 2):
        opcode = int.from_bytes(image.payload[payload_offset : payload_offset + 2], "big")
        if 0xA000 <= opcode <= 0xAFFF:
            stub = Os3kAppTrapStub(
                file_offset=image.header_size + payload_offset,
                opcode=opcode,
                family_byte=(opcode >> 8) & 0xFF,
                selector_byte=opcode & 0xFF,
                inferred_name=KNOWN_TRAP_NAMES.get(opcode),
            )
            if current and stub.opcode != current[-1].opcode + 4:
                if len(current) >= minimum_stub_count:
                    blocks.append(
                        Os3kAppTrapBlock(
                            start_file_offset=current_start,
                            end_file_offset=current[-1].file_offset + 2,
                            stubs=tuple(current),
                        )
                    )
                current = []
            if not current:
                current_start = stub.file_offset
            current.append(stub)
            continue

        if current:
            if len(current) >= minimum_stub_count:
                blocks.append(
                    Os3kAppTrapBlock(
                        start_file_offset=current_start,
                        end_file_offset=current[-1].file_offset + 2,
                        stubs=tuple(current),
                    )
                )
            current = []

    if current:
        if len(current) >= minimum_stub_count:
            blocks.append(
                Os3kAppTrapBlock(
                    start_file_offset=current_start,
                    end_file_offset=current[-1].file_offset + 2,
                    stubs=tuple(current),
                )
            )

    return tuple(blocks)


def describe_known_trap_prototype(opcode: int) -> Os3kAppTrapPrototype:
    prototype = KNOWN_TRAP_PROTOTYPES.get(opcode)
    if prototype is None:
        raise ValueError(f"unknown SmartApplet trap prototype: 0x{opcode:04x}")
    return prototype


def describe_known_applet_command_prototype(applet_name: str, raw_command: int) -> Os3kAppCommandPrototype:
    prototype = KNOWN_APPLET_COMMAND_PROTOTYPES.get((applet_name.lower(), raw_command))
    if prototype is None:
        raise ValueError(
            f"unknown SmartApplet applet command prototype: {applet_name} 0x{raw_command:05x}"
        )
    return prototype


def describe_known_applet_payload_subcommand_prototype(
    applet_name: str,
    parent_command: int,
    first_input_byte: int,
) -> Os3kAppPayloadSubcommandPrototype:
    prototype = KNOWN_APPLET_PAYLOAD_SUBCOMMAND_PROTOTYPES.get(
        (applet_name.lower(), parent_command, first_input_byte)
    )
    if prototype is None:
        raise ValueError(
            "unknown SmartApplet payload subcommand prototype: "
            f"{applet_name} 0x{parent_command:05x} 0x{first_input_byte:02x}"
        )
    return prototype


def build_minimal_smartapplet_image(
    *,
    applet_id: int,
    name: str,
    version_major_bcd: int,
    version_minor_bcd: int,
    flags_raw: int = 0xFF000000,
    base_memory_size: int = 0x100,
    extra_memory_size: int = 0,
    copyright: str = "Custom SmartApplet",
) -> bytes:
    # Entry code:
    #   movea.l 0x0c(a7),a0
    #   clr.l   (a0)
    #   move.l  0x04(a7),d0
    #   cmpi.l  #0x19,d0
    #   bne.s   return
    #   moveq   #7,d1
    #   move.l  d1,(a0)
    # return:
    #   rts
    entry_code = bytes.fromhex(
        "20 6f 00 0c"
        " 42 90"
        " 20 2f 00 04"
        " 0c 80 00 00 00 19"
        " 66 04"
        " 72 07"
        " 20 81"
        " 4e 75"
    )
    payload = (
        (0x94).to_bytes(4, "big")
        + (0).to_bytes(4, "big")
        + (1).to_bytes(4, "big")
        + (2).to_bytes(4, "big")
        + entry_code
    )
    return build_os3kapp_image(
        header_fields=Os3kAppHeaderFields(
            magic=0xC0FFEEAD,
            base_memory_size=base_memory_size,
            flags_raw=flags_raw,
            applet_id_and_version=((applet_id & 0xFFFF) << 16) | ((version_major_bcd & 0xFF) << 8) | (version_minor_bcd & 0xFF),
            name=name,
            version_major_bcd=version_major_bcd,
            version_minor_bcd=version_minor_bcd,
            version_build_byte=0x00,
            applet_class=0x01,
            copyright=copyright,
            extra_memory_size=extra_memory_size,
        ),
        payload=payload,
    )
