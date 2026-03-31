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


KNOWN_TRAP_NAMES: dict[int, str] = {
    0xA000: "calculator_menu_begin",
    0xA004: "calculator_menu_next_row",
    0xA010: "calculator_menu_draw_selection_marker",
    0xA014: "calculator_menu_flush_current_string",
    0xA094: "read_key_code",
    0xA09C: "poll_key_ready",
    0xA0A4: "yield_or_pump_events",
    0xA25C: "yield_or_sleep",
    0xA368: "calculator_runtime_init_slot_a",
    0xA36C: "calculator_runtime_init_slot_b",
    0xA38C: "calculator_runtime_prepare_input_buffer",
    0xA390: "calculator_runtime_store_result_string",
    0xA39C: "calculator_runtime_copy_input_string",
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
