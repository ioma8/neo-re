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
    0xA14C: "read_text_input_char",
    0xA190: "begin_output_builder",
    0xA198: "append_output_bytes",
    0xA1B4: "query_numeric_state",
    0xA1C8: "query_object_metric",
    0xA1CC: "commit_editable_buffer",
    0xA1D4: "assign_current_file_name_from_pending_text",
    0xA1E0: "query_advanced_file_iterator_ordinal",
    0xA1EC: "sync_current_slot_map_entry",
    0xA1F8: "begin_chooser_row_builder",
    0xA1FC: "advance_current_output_row",
    0xA200: "append_current_chooser_row",
    0xA204: "highlight_current_chooser_row",
    0xA208: "begin_chooser_input_session",
    0xA20C: "read_chooser_event_code",
    0xA210: "read_chooser_action_selector",
    0xA214: "read_chooser_selection_value",
    0xA25C: "yield_until_event",
    0xA2BC: "commit_current_file_edit_session",
    0xA2C0: "finalize_current_file_context",
    0xA2CC: "begin_current_replacement",
    0xA2D0: "query_current_replacement_status",
    0xA2D4: "reset_current_search_state",
    0xA2D8: "read_next_char_stream_unit",
    0xA2DC: "switch_to_current_file_context",
    0xA2EC: "query_current_workspace_file_status",
    0xA2FC: "initialize_empty_workspace_file",
    0xA364: "query_active_service_available",
    0xA368: "shared_runtime_a368",
    0xA36C: "query_active_service_status",
    0xA378: "render_formatted_pending_text",
    0xA380: "format_pending_text",
    0xA388: "query_active_service_disabled_state",
    0xA398: "query_pending_text_length",
    0xA38C: "shared_runtime_a38c",
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
    0xA14C: Os3kAppTrapPrototype(
        opcode=0xA14C,
        name="read_text_input_char",
        stack_argument_count=0,
        return_kind="value",
        notes="AlphaWordPlus uses this from editable-field handlers and live dialogs to fetch the current typed input character when text-entry mode is active",
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
    0xA1D4: Os3kAppTrapPrototype(
        opcode=0xA1D4,
        name="assign_current_file_name_from_pending_text",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus uses this immediately after loading a canned name string into the pending-text slot; it behaves like an assign/apply-current-file-name helper",
    ),
    0xA1E0: Os3kAppTrapPrototype(
        opcode=0xA1E0,
        name="query_advanced_file_iterator_ordinal",
        stack_argument_count=0,
        return_kind="value",
        notes="called immediately after advancing the AlphaWord file iterator; returns the resulting file ordinal or a nonpositive failure value when no file slot is available",
    ),
    0xA1EC: Os3kAppTrapPrototype(
        opcode=0xA1EC,
        name="sync_current_slot_map_entry",
        stack_argument_count=0,
        return_kind="none",
        notes="paired around reads and writes of the eight-entry AlphaWord slot-to-file map; AlphaWordPlus uses it as the slot-map synchronization hook before and after mutating the current slot entry",
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
    0xA2BC: Os3kAppTrapPrototype(
        opcode=0xA2BC,
        name="commit_current_file_edit_session",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus calls this after bulk overwrite/edit sequences have been finalized; it behaves like the current-file edit-session commit hook",
    ),
    0xA2C0: Os3kAppTrapPrototype(
        opcode=0xA2C0,
        name="finalize_current_file_context",
        stack_argument_count=0,
        return_kind="none",
        notes="paired with current-file workflows such as create/load/prompt flows; AlphaWordPlus uses it as the matching finalize/leave-current-file-context hook",
    ),
    0xA2CC: Os3kAppTrapPrototype(
        opcode=0xA2CC,
        name="begin_current_replacement",
        stack_argument_count=0,
        return_kind="none",
        notes="called at the start of spell-check and clear-file replacement paths, immediately after locating the current cursor/selection position",
    ),
    0xA2D0: Os3kAppTrapPrototype(
        opcode=0xA2D0,
        name="query_current_replacement_status",
        stack_argument_count=0,
        return_kind="value",
        notes="queried immediately after begin_current_replacement; the return value controls whether the replacement path produced an empty/no-op result",
    ),
    0xA2D4: Os3kAppTrapPrototype(
        opcode=0xA2D4,
        name="reset_current_search_state",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus uses this around search/find/replace prompts and after one-char lookahead checks; it behaves like a reset or clear hook for the current search/match state",
    ),
    0xA2D8: Os3kAppTrapPrototype(
        opcode=0xA2D8,
        name="read_next_char_stream_unit",
        stack_argument_count=0,
        return_kind="value",
        notes="low-level char-stream iterator primitive used underneath AlphaWordPlus spell-check, preview, selection, and namespace readback helpers",
    ),
    0xA2DC: Os3kAppTrapPrototype(
        opcode=0xA2DC,
        name="switch_to_current_file_context",
        stack_argument_count=0,
        return_kind="none",
        notes="called before destructive or content-sensitive file operations; AlphaWordPlus uses it to switch/bind the active current-file workspace before further edits or streaming",
    ),
    0xA2EC: Os3kAppTrapPrototype(
        opcode=0xA2EC,
        name="query_current_workspace_file_status",
        stack_argument_count=0,
        return_kind="value",
        notes="queried after selecting or naming the current file; zero triggers the empty-file initialization path, so the result behaves like a current-workspace file-state/status value",
    ),
    0xA2FC: Os3kAppTrapPrototype(
        opcode=0xA2FC,
        name="initialize_empty_workspace_file",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus calls this when query_current_workspace_file_status returns zero for the active file, making it the current empty-file/default-content initializer",
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
        name="render_formatted_pending_text",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus uses this immediately after loading a template string and setting up stacked arguments; it behaves like a direct formatted-text render helper for status/details dialogs",
    ),
    0xA380: Os3kAppTrapPrototype(
        opcode=0xA380,
        name="format_pending_text",
        stack_argument_count=0,
        return_kind="none",
        notes="AlphaWordPlus uses this after selecting a format/template string and before querying pending-text length or drawing; it behaves like a pending-text formatter/builder rather than the final draw call",
    ),
    0xA398: Os3kAppTrapPrototype(
        opcode=0xA398,
        name="query_pending_text_length",
        stack_argument_count=0,
        return_kind="value",
        notes="returns the current pending/formatted text length; AlphaWordPlus uses it to size wrapped dialogs and chooser-row preview buffers",
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
    direct_mode_callback: int | None = None,
    direct_mode_command_handler: bool = False,
    stay_open_on_init: bool = False,
    calculator_style_menu: bool = False,
    draw_on_any_command: bool = False,
    draw_on_menu_command: bool = False,
    arm_direct_on_menu: bool = False,
    host_usb_message_handler: bool = False,
    alphaword_write_metadata: bool = False,
    alphaword_state_machine_probe: bool = False,
    alphaword_init_command_probe: bool = False,
    alphaword_init_fault_probe: bool = False,
    alphaword_silent_init_probe: bool = False,
    alphaword_switch_on_init_probe: bool = False,
    alphaword_hid_complete_switch_probe: bool = False,
    alpha_usb_production: bool = False,
    command_fault_probe: bool = False,
    command_fault_after_shutdown_probe: bool = False,
) -> bytes:
    # Benign entry code:
    #   movea.l 0x0c(a7),a0
    #   clr.l   (a0)
    #   move.l  0x04(a7),d0
    #   cmpi.l  #0x19,d0
    #   bne.s   return
    #   moveq   #7,d1
    #   move.l  d1,(a0)
    # return:
    #   rts
    def branch_short(opcode: int, from_offset: int, target_offset: int) -> bytes:
        displacement = target_offset - (from_offset + 2)
        if not -128 <= displacement <= 127:
            raise ValueError("short branch target is out of range")
        return bytes([opcode, displacement & 0xFF])

    def patch_branch_word(code: bytearray, from_offset: int, target_offset: int) -> None:
        displacement = target_offset - (from_offset + 2)
        if not -32768 <= displacement <= 32767:
            raise ValueError("word branch target is out of range")
        code[from_offset + 2 : from_offset + 4] = displacement.to_bytes(2, "big", signed=True)

    def build_info_record(group: int, key: int, payload: bytes) -> bytes:
        record = (
            group.to_bytes(2, "big")
            + key.to_bytes(2, "big")
            + len(payload).to_bytes(2, "big")
            + payload
        )
        if len(payload) & 1:
            record += b"\x00"
        return record

    def build_alphaword_write_info_table() -> bytes:
        write_payload = b"write\x00"
        records = [build_info_record(0x0105, 0x100B, write_payload)]
        records.extend(build_info_record(0xC001, key, write_payload) for key in range(0x8011, 0x8019))
        return b"".join(records) + bytes.fromhex("00 00 00 00 00 00 00 00 ca fe fe ed")

    if command_fault_after_shutdown_probe:
        entry_code = bytes.fromhex(
            "20 6f 00 0c"  # movea.l 0x0c(a7),a0 ; status_out
            "42 90"  # clr.l (a0)
            "20 2f 00 04"  # move.l 0x04(a7),d0 ; command
            "0c 80 00 00 00 19"  # cmpi.l #0x19,d0
            "66 06"  # bne.s fault
            "72 07"  # moveq #7,d1
            "20 81"  # move.l d1,(a0)
            "4e 75"  # rts
            "22 00"  # move.l d0,d1
            "02 81 00 00 ff ff"  # andi.l #0x0000ffff,d1
            "00 81 00 58 00 00"  # ori.l #0x00580000,d1
            "20 41"  # movea.l d1,a0
            "22 10"  # move.l (a0),d1 ; intentional fault
            "4e 75"  # rts
        )
    elif command_fault_probe:
        entry_code = bytes.fromhex(
            "20 2f 00 04"  # move.l 0x04(a7),d0 ; command
            "22 00"  # move.l d0,d1
            "02 81 00 00 ff ff"  # andi.l #0x0000ffff,d1
            "00 81 00 58 00 00"  # ori.l #0x00580000,d1
            "20 41"  # movea.l d1,a0
            "22 10"  # move.l (a0),d1 ; intentional fault, address encodes command low word
            "4e 75"  # rts
        )
    elif draw_on_menu_command or draw_on_any_command:
        stub_calls: list[tuple[int, int]] = []

        def append_stub_call(code: bytearray, opcode: int) -> None:
            call_offset = len(code)
            code.extend(bytes.fromhex("61 00 00 00"))  # bsr.w trap_stub, patched after stubs are emitted
            stub_calls.append((call_offset, opcode))

        def append_branch_word(code: bytearray, opcode: int) -> int:
            branch_offset = len(code)
            code.extend(bytes([opcode, 0x00, 0x00, 0x00]))
            return branch_offset

        def append_text_screen(code: bytearray, text: bytes) -> None:
            append_stub_call(code, 0xA000)  # clear_text_screen()
            for row_offset, line in enumerate(text.split(b"\n")):
                row = 2 + row_offset
                code.extend(bytes.fromhex("2f 3c 00 00 00 1c"))  # push width 28
                code.extend(bytes.fromhex("2f 3c 00 00 00 01"))  # push column 1
                code.extend(bytes.fromhex("2f 3c 00 00 00") + bytes([row]))  # push row
                append_stub_call(code, 0xA004)  # set_text_row_column_width(row, column, width)
                code.extend(bytes.fromhex("4f ef 00 0c"))  # lea 12(a7),a7
                for character in line:
                    code.extend(bytes([0x70, character]))  # moveq #character,d0
                    code.extend(bytes.fromhex("2f 00"))  # move.l d0,-(a7)
                    append_stub_call(code, 0xA010)  # draw character
                    code.extend(bytes.fromhex("58 8f"))  # addq.l #4,a7
            append_stub_call(code, 0xA098)  # flush_text_frame()

        def append_idle_loop(code: bytearray) -> None:
            append_stub_call(code, 0xA25C)  # yield_until_event()
            code.extend(bytes.fromhex("60 fa"))  # bra.s yield loop

        def append_alphaword_attach_state(code: bytearray, *, mac_side: bool) -> None:
            code.extend(bytes.fromhex("28 3c 00 00 01 38"))  # move.l #0x138,d4
            code.extend(bytes.fromhex("1b bc 00 01 48 00"))  # move.b #1,(a5,d4.l)
            code.extend(bytes.fromhex("28 3c 00 00 01 42"))  # move.l #0x142,d4
            code.extend(bytes.fromhex("42 35 48 00"))  # clr.b (a5,d4.l)
            code.extend(bytes.fromhex("28 3c 00 00 01 43"))  # move.l #0x143,d4
            if mac_side:
                code.extend(bytes.fromhex("1b bc 00 01 48 00"))  # move.b #1,(a5,d4.l)
            else:
                code.extend(bytes.fromhex("42 35 48 00"))  # clr.b (a5,d4.l)
            code.extend(bytes.fromhex("28 3c 00 00 00 bc"))  # move.l #0xbc,d4
            code.extend(bytes.fromhex("42 35 48 00"))  # clr.b (a5,d4.l)

        def append_intentional_fault(code: bytearray, address: int) -> None:
            code.extend(bytes.fromhex("20 7c"))  # movea.l #address,a0
            code.extend(address.to_bytes(4, "big"))
            code.extend(bytes.fromhex("22 10"))  # move.l (a0),d1
            code.extend(bytes.fromhex("4e 75"))  # rts, normally unreachable

        def append_status_return(code: bytearray, status: int) -> None:
            code.extend(bytes.fromhex("72"))
            code.extend(bytes([status & 0xFF]))  # moveq #status,d1
            code.extend(bytes.fromhex("20 81"))  # move.l d1,(a0)
            code.extend(bytes.fromhex("4e 75"))  # rts

        code = bytearray(
            bytes.fromhex(
                "20 6f 00 0c"  # movea.l 0x0c(a7),a0 ; status_out
                " 42 90"  # clr.l (a0)
                " 20 2f 00 04"  # move.l 0x04(a7),d0 ; command
                " 0c 80 00 00 00 18"  # cmpi.l #0x18,d0
            )
        )
        beq_init_return_offset = append_branch_word(code, 0x67)  # beq.w return
        if draw_on_menu_command:
            code.extend(
                bytes.fromhex(
                    " 0c 80 00 00 00 19"  # cmpi.l #0x19,d0
                )
            )
            beq_menu_offset = append_branch_word(code, 0x67)  # beq.w draw_menu
            beq_usb_offsets: list[int] = []
            if direct_mode_callback is not None:
                if not 0 <= direct_mode_callback <= 0xFFFFFFFF:
                    raise ValueError("direct mode callback address must fit in 32 bits")
                for event_command in (0x20, 0x21):
                    code.extend(bytes.fromhex("0c 80") + event_command.to_bytes(4, "big"))  # cmpi.l #event,d0
                    beq_usb_offsets.append(append_branch_word(code, 0x67))  # beq.w draw_usb
            beq_host_usb_offsets: list[int] = []
            beq_host_mac_init_offsets: list[int] = []
            beq_host_alt_mac_init_offsets: list[int] = []
            beq_host_pc_init_offsets: list[int] = []
            beq_identity_offsets: list[int] = []
            if host_usb_message_handler:
                for event_command in (0x20, 0x21):
                    code.extend(bytes.fromhex("0c 80") + event_command.to_bytes(4, "big"))
                    beq_host_usb_offsets.append(append_branch_word(code, 0x67))  # beq.w draw_host_usb
                code.extend(bytes.fromhex("0c 80 00 00 00 26"))  # cmpi.l #0x26,d0
                beq_identity_offsets.append(append_branch_word(code, 0x67))  # beq.w identity
                code.extend(bytes.fromhex("0c 80 00 01 00 01"))  # cmpi.l #0x10001,d0
                beq_host_mac_init_offsets.append(append_branch_word(code, 0x67))  # beq.w draw_host_mac_init
                code.extend(bytes.fromhex("0c 80 00 03 00 01"))  # cmpi.l #0x30001,d0
                beq_host_alt_mac_init_offsets.append(append_branch_word(code, 0x67))  # beq.w draw_host_alt_mac_init
                code.extend(bytes.fromhex("0c 80 00 02 00 01"))  # cmpi.l #0x20001,d0
                beq_host_pc_init_offsets.append(append_branch_word(code, 0x67))  # beq.w draw_host_pc_init
                for event_command in (0x00010003, 0x00010006, 0x00020002, 0x00020006, 0x0002011F):
                    code.extend(bytes.fromhex("0c 80") + event_command.to_bytes(4, "big"))
                    beq_host_usb_offsets.append(append_branch_word(code, 0x67))  # beq.w draw_host_usb
            bra_return_offset = append_branch_word(code, 0x60)  # bra.w return
        else:
            compare_shutdown_offset = len(code)
            code.extend(
                bytes.fromhex(
                    " 0c 80 00 00 00 19"  # cmpi.l #0x19,d0
                    " 66 00"  # bne.s draw
                    " 72 07"  # moveq #7,d1
                    " 20 81"  # move.l d1,(a0)
                    " 4e 75"  # rts
                )
            )
            shutdown_return_offset = len(code) - 2
            bne_draw_offset = compare_shutdown_offset + 6
        return_offset = len(code)
        code.extend(bytes.fromhex("4e 75"))  # rts
        if draw_on_menu_command:
            draw_menu_offset = len(code)
            if arm_direct_on_menu:
                code.extend(bytes.fromhex("42 a7"))  # clr.l -(a7)
                code.extend(bytes.fromhex("4e b9 00 42 6b b0"))  # jsr 0x00426bb0
                code.extend(bytes.fromhex("13 fc 00 00 00 00 04 44"))  # clr.b-like absolute state reset
                code.extend(bytes.fromhex("4e b9 00 41 2c 82"))  # jsr 0x00412c82
                code.extend(bytes.fromhex("4e b9 00 41 09 ca"))  # jsr 0x004109ca
                code.extend(bytes.fromhex("48 79 00 01 11 11"))  # pea.l 0x00011111
                code.extend(bytes.fromhex("48 78 25 80"))  # pea.l 0x2580.w
                code.extend(bytes.fromhex("4e b9 00 42 4f b0"))  # jsr 0x00424fb0
                code.extend(bytes.fromhex("48 79 00 41 0b 26"))  # pea.l 0x00410b26
                code.extend(bytes.fromhex("4e b9 00 42 4f 66"))  # jsr 0x00424f66
                code.extend(bytes.fromhex("4f ef 00 10"))  # lea 16(a7),a7
            if alpha_usb_production:
                append_text_screen(code, b"Now connect the NEO\nto your computer or\nsmartphone via USB.")
            else:
                append_text_screen(code, b"ARM" if arm_direct_on_menu else b"USB")
            append_idle_loop(code)
            draw_usb_offset = len(code)
            if direct_mode_callback is not None:
                append_text_screen(code, b"DIR")
                code.extend(bytes.fromhex("4e b9"))  # jsr direct_mode_callback
                code.extend(direct_mode_callback.to_bytes(4, "big"))
                append_idle_loop(code)
            draw_host_usb_offset = len(code)
            if host_usb_message_handler:
                if alpha_usb_production:
                    append_status_return(code, 0x04)
                elif alphaword_init_fault_probe:
                    append_intentional_fault(code, 0x0058F00D)
                else:
                    append_text_screen(code, b"HOST")
                    append_status_return(code, 0x04)
            draw_host_mac_init_offset = len(code)
            if host_usb_message_handler:
                if alpha_usb_production:
                    append_status_return(code, 0x11)
                elif alphaword_init_fault_probe:
                    append_intentional_fault(code, 0x00581001)
                else:
                    if alphaword_state_machine_probe:
                        append_alphaword_attach_state(code, mac_side=True)
                    append_text_screen(code, b"N1" if alphaword_init_command_probe else b"LINK")
                    append_status_return(code, 0x11)
            draw_host_alt_mac_init_offset = len(code)
            if host_usb_message_handler:
                if alphaword_hid_complete_switch_probe or alpha_usb_production:
                    code.extend(bytes.fromhex("42 a7"))  # clr.l -(a7)
                    code.extend(bytes.fromhex("42 a7"))  # clr.l -(a7)
                    code.extend(bytes.fromhex("48 78 00 01"))  # pea.l 1
                    code.extend(bytes.fromhex("4e b9 00 41 f9 a0"))  # jsr HID/control write helper
                    code.extend(bytes.fromhex("48 78 00 64"))  # pea.l 100
                    code.extend(bytes.fromhex("4e b9 00 42 47 80"))  # jsr delay helper
                    code.extend(bytes.fromhex("13 fc 00 01 00 01 3c f9"))  # move.b #1,$13cf9
                    code.extend(bytes.fromhex("4e b9 00 44 04 4e"))  # jsr HID completion phase 1
                    code.extend(bytes.fromhex("48 78 00 64"))  # pea.l 100
                    code.extend(bytes.fromhex("4e b9 00 42 47 80"))  # jsr delay helper
                    code.extend(bytes.fromhex("4f ef 00 14"))  # lea.l 0x14(a7),a7
                    code.extend(bytes.fromhex("4e b9 00 44 04 7c"))  # jsr HID completion phase 2
                    append_status_return(code, 0x11)
                elif alphaword_switch_on_init_probe:
                    code.extend(bytes.fromhex("4e b9 00 41 0b 26"))  # jsr direct-mode callback
                    append_status_return(code, 0x11)
                elif alphaword_silent_init_probe:
                    append_status_return(code, 0x11)
                elif alphaword_init_fault_probe:
                    append_intentional_fault(code, 0x00583001)
                else:
                    if alphaword_state_machine_probe:
                        append_alphaword_attach_state(code, mac_side=True)
                    append_text_screen(code, b"N3" if alphaword_init_command_probe else b"LINK")
                    append_status_return(code, 0x11)
            draw_host_pc_init_offset = len(code)
            if host_usb_message_handler:
                if alpha_usb_production:
                    append_status_return(code, 0x11)
                elif alphaword_init_fault_probe:
                    append_intentional_fault(code, 0x00582001)
                else:
                    if alphaword_state_machine_probe:
                        append_alphaword_attach_state(code, mac_side=False)
                    append_text_screen(code, b"PC")
                    append_status_return(code, 0x11)
            identity_offset = len(code)
            if host_usb_message_handler:
                code.extend(bytes.fromhex("22 3c"))  # move.l #applet_id,d1
                code.extend(applet_id.to_bytes(4, "big"))
                code.extend(bytes.fromhex("20 81"))  # move.l d1,(a0)
                code.extend(bytes.fromhex("4e 75"))  # rts
            patch_branch_word(code, beq_menu_offset, draw_menu_offset)
            for beq_usb_offset in beq_usb_offsets:
                patch_branch_word(code, beq_usb_offset, draw_usb_offset)
            for beq_host_usb_offset in beq_host_usb_offsets:
                patch_branch_word(code, beq_host_usb_offset, draw_host_usb_offset)
            for beq_host_mac_init_offset in beq_host_mac_init_offsets:
                patch_branch_word(code, beq_host_mac_init_offset, draw_host_mac_init_offset)
            for beq_host_alt_mac_init_offset in beq_host_alt_mac_init_offsets:
                patch_branch_word(code, beq_host_alt_mac_init_offset, draw_host_alt_mac_init_offset)
            for beq_host_pc_init_offset in beq_host_pc_init_offsets:
                patch_branch_word(code, beq_host_pc_init_offset, draw_host_pc_init_offset)
            for beq_identity_offset in beq_identity_offsets:
                patch_branch_word(code, beq_identity_offset, identity_offset)
            patch_branch_word(code, bra_return_offset, return_offset)
        else:
            draw_offset = len(code)
            code[bne_draw_offset : bne_draw_offset + 2] = branch_short(0x66, bne_draw_offset, draw_offset)
            append_text_screen(code, b"USB")
            append_idle_loop(code)
        patch_branch_word(code, beq_init_return_offset, return_offset)
        stub_offsets: dict[int, int] = {}
        for opcode in (0xA000, 0xA004, 0xA010, 0xA098, 0xA25C):
            stub_offsets[opcode] = len(code)
            code.extend(opcode.to_bytes(2, "big"))
        for call_offset, opcode in stub_calls:
            displacement = stub_offsets[opcode] - (call_offset + 2)
            code[call_offset + 2 : call_offset + 4] = displacement.to_bytes(2, "big", signed=True)
        entry_code = bytes(code)
    elif calculator_style_menu:
        message = b"USB Direct open\x00"
        code = bytearray(
            bytes.fromhex(
                "20 6f 00 0c"  # movea.l 0x0c(a7),a0 ; status_out
                " 42 90"  # clr.l (a0)
                " 22 6f 00 08"  # movea.l 0x08(a7),a1 ; call_block
                " 20 2f 00 04"  # move.l 0x04(a7),d0 ; command
                " 0c 80 00 00 00 19"  # cmpi.l #0x19,d0
                " 66 00"  # bne.s not_shutdown
                " 72 07"  # moveq #7,d1
                " 20 81"  # move.l d1,(a0)
                " 4e 75"  # rts
            )
        )
        code[20:22] = branch_short(0x66, 20, len(code))
        code.extend(
            bytes.fromhex(
                " 0c 80 00 00 00 18"  # cmpi.l #0x18,d0
                " 67 00"  # beq.s return_zero
                " 0c 80 00 00 00 01"  # cmpi.l #1,d0
                " 67 00"  # beq.s command_1
                " 0c 80 00 00 00 02"  # cmpi.l #2,d0
                " 67 00"  # beq.s command_2
                " 72 01"  # moveq #1,d1
                " 20 81"  # move.l d1,(a0)
                " 4e 75"  # rts
            )
        )
        return_zero_offset = len(code)
        code.extend(bytes.fromhex("4e 75"))
        command_1_offset = len(code)
        code.extend(
            bytes.fromhex(
                " 24 69 00 10"  # movea.l 0x10(a1),a2 ; output buffer
                " 42 12"  # clr.b (a2)
                " 24 69 00 0c"  # movea.l 0x0c(a1),a2 ; &output length
                " 72 01"  # moveq #1,d1
                " 24 81"  # move.l d1,(a2)
                " 20 81"  # move.l d1,(a0)
                " 4e 75"  # rts
            )
        )
        command_2_offset = len(code)
        lea_offset = len(code) + 4
        code.extend(
            bytes.fromhex(
                " 26 69 00 10"  # movea.l 0x10(a1),a3 ; output buffer
                " 45 fa 00 00"  # lea message(pc),a2 ; patched below
                " 26 8a"  # move.l a2,(a3)
                " 24 69 00 0c"  # movea.l 0x0c(a1),a2 ; &output length
                " 72 04"  # moveq #4,d1
                " 24 81"  # move.l d1,(a2)
                " 20 81"  # move.l d1,(a0)
                " 4e 75"  # rts
            )
        )
        code[34:36] = branch_short(0x67, 34, return_zero_offset)
        code[42:44] = branch_short(0x67, 42, command_1_offset)
        code[50:52] = branch_short(0x67, 50, command_2_offset)
        message_offset = len(code)
        displacement = message_offset - (lea_offset + 4)
        code[lea_offset + 2 : lea_offset + 4] = displacement.to_bytes(2, "big", signed=True)
        code.extend(message)
        entry_code = bytes(code)
    elif stay_open_on_init:
        message = b"USB Direct open\x00"
        code = bytearray(
            bytes.fromhex(
                "20 6f 00 0c"  # movea.l 0x0c(a7),a0 ; status_out
                " 42 90"  # clr.l (a0)
                " 20 2f 00 04"  # move.l 0x04(a7),d0 ; command
                " 0c 80 00 00 00 19"  # cmpi.l #0x19,d0
                " 66 00"  # bne.s not_shutdown
                " 72 07"  # moveq #7,d1
                " 20 81"  # move.l d1,(a0)
                " 4e 75"  # rts
            )
        )
        code[16:18] = branch_short(0x66, 16, len(code))
        code.extend(
            bytes.fromhex(
                " 0c 80 00 00 00 18"  # cmpi.l #0x18,d0
                " 66 00"  # bne.s return
                " a0 00"  # clear_text_screen()
                " 2f 3c 00 00 00 28"  # push width 40
                " 2f 3c 00 00 00 00"  # push column 0
                " 2f 3c 00 00 00 00"  # push row 0
                " a0 04"  # set_text_row_column_width(row, column, width)
                " 4f ef 00 0c"  # lea 12(a7),a7
                " 48 7a 00 00"  # pea message(pc), patched below
                " a0 14"  # draw_c_string_at_current_position(message)
                " 4f ef 00 04"  # lea 4(a7),a7
                " a0 98"  # flush_text_frame()
                " a2 5c"  # yield_until_event()
                " 60 fc"  # bra.s yield loop
                " 4e 75"  # rts for non-init commands
            )
        )
        return_offset = len(code) - 2
        bne_return_offset = 30
        code[bne_return_offset : bne_return_offset + 2] = branch_short(0x66, bne_return_offset, return_offset)
        pea_offset = 58
        message_offset = len(code)
        displacement = message_offset - (pea_offset + 4)
        code[pea_offset + 2 : pea_offset + 4] = displacement.to_bytes(2, "big", signed=True)
        code.extend(message)
        entry_code = bytes(code)
    elif direct_mode_callback is None:
        if direct_mode_command_handler:
            raise ValueError("direct mode command handler requires a callback address")
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
    elif direct_mode_command_handler:
        if not 0 <= direct_mode_callback <= 0xFFFFFFFF:
            raise ValueError("direct mode callback address must fit in 32 bits")
        message = b"Opening direct USB...\x00"

        code = bytearray(
            bytes.fromhex(
                "20 6f 00 0c"  # movea.l 0x0c(a7),a0 ; status_out
                " 42 90"  # clr.l (a0)
                " 20 2f 00 04"  # move.l 0x04(a7),d0 ; command
                " 0c 80 00 00 00 19"  # cmpi.l #0x19,d0
                " 66 00"  # bne.s not_shutdown
                " 72 07"  # moveq #7,d1
                " 20 81"  # move.l d1,(a0)
                " 4e 75"  # rts
            )
        )
        code[16:18] = branch_short(0x66, 16, len(code))
        # Handle applet-private selector namespace 4, including commands such as
        # 0x00040001 and wider launcher forms whose selector byte is still 0x04.
        code.extend(
            bytes.fromhex(
                " 22 00"  # move.l d0,d1
                " 02 81 00 ff 00 00"  # andi.l #0x00ff0000,d1
                " 0c 81 00 04 00 00"  # cmpi.l #0x00040000,d1
                " 66 00"  # bne.s return
                " a0 00"  # clear_text_screen()
                " 2f 3c 00 00 00 28"  # push width 40
                " 2f 3c 00 00 00 00"  # push column 0
                " 2f 3c 00 00 00 00"  # push row 0
                " a0 04"  # set_text_row_column_width(row, column, width)
                " 4f ef 00 0c"  # lea 12(a7),a7
                " 48 7a 00 00"  # pea message(pc), patched below
                " a0 14"  # draw_c_string_at_current_position(message)
                " 4f ef 00 04"  # lea 4(a7),a7
                " a0 98"  # flush_text_frame()
                " 4e b9"  # jsr direct_mode_callback
            )
        )
        code.extend(direct_mode_callback.to_bytes(4, "big"))
        code.extend(
            bytes.fromhex(
                " 70 11"  # moveq #0x11,d0
                " 20 80"  # move.l d0,(a0)
                " 4e 75"  # rts
            )
        )
        return_offset = len(code) - 2
        bne_return_offset = 38
        code[bne_return_offset : bne_return_offset + 2] = branch_short(0x66, bne_return_offset, return_offset)
        pea_offset = 66
        message_offset = len(code)
        displacement = message_offset - (pea_offset + 4)
        code[pea_offset + 2 : pea_offset + 4] = displacement.to_bytes(2, "big", signed=True)
        code.extend(message)
        entry_code = bytes(code)
    else:
        if not 0 <= direct_mode_callback <= 0xFFFFFFFF:
            raise ValueError("direct mode callback address must fit in 32 bits")
        # Experimental direct-mode entry code:
        #   movea.l 0x0c(a7),a0
        #   clr.l   (a0)
        #   move.l  0x04(a7),d0
        #   cmpi.l  #0x18,d0
        #   bne.s   shutdown_check
        #   jsr     direct_mode_callback
        #   bra.s   return
        # shutdown_check:
        #   cmpi.l  #0x19,d0
        #   bne.s   return
        #   moveq   #7,d1
        #   move.l  d1,(a0)
        # return:
        #   rts
        entry_code = (
            bytes.fromhex(
                "20 6f 00 0c"
                " 42 90"
                " 20 2f 00 04"
                " 0c 80 00 00 00 18"
                " 66 08"
                " 4e b9"
            )
            + direct_mode_callback.to_bytes(4, "big")
            + bytes.fromhex(
                " 60 0c"
                " 0c 80 00 00 00 19"
                " 66 04"
                " 72 07"
                " 20 81"
                " 4e 75"
            )
        )
    payload = (0x94).to_bytes(4, "big") + (0).to_bytes(4, "big") + (1).to_bytes(4, "big") + (2).to_bytes(4, "big") + entry_code
    info_table_bytes = b""
    if alphaword_write_metadata:
        info_table_bytes = build_alphaword_write_info_table()
    else:
        payload += bytes.fromhex("ca fe fe ed")
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
        info_table_bytes=info_table_bytes,
    )
