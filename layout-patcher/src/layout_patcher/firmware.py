from dataclasses import dataclass
from pathlib import Path

from .layouts import REPLACEMENT_LAYOUTS, STOCK_LAYOUT_SLOTS, STOCK_PHYSICAL_TO_LOGICAL


class FirmwareValidationError(ValueError):
    pass


STRING_ANCHORS = {
    0x35B3E: "To change key layout, type 1, 2, 3 or 4.",
    0x35C98: "QWERTY",
    0x35C9F: "Dvorak",
}

BYTE_ANCHORS = {
    0x3C3FB: bytes.fromhex("252525010101472f14"),
}
LAYOUT_TABLE_OFFSET = 0x3C3FB
LOGICAL_KEY_COUNT = 0x51
CHAR_HOOK_ENTRY_OFFSET = 0x13C7C
CHAR_HOOK_ENTRY_BYTES = bytes.fromhex("48e70f18558f3e2f0020")
CHAR_HOOK_CODE_OFFSET = 0x42E8E
CHAR_HOOK_RUNTIME_ADDRESS = 0x00452E8E
CHAR_HOOK_CONTINUE_RUNTIME_ADDRESS = 0x00423C86
CHAR_HOOK_CODE_TEMPLATE = bytearray.fromhex(
    "48e70f18558f3e2f00200c070050620000320c3800005d366600002870001007"
    "3c070246040041fa00206700000641fa0118103008006700000a548f4cdf18f0"
    "4e754ef900423c86"
)
CHAR_HOOK_CODE_LENGTH = len(CHAR_HOOK_CODE_TEMPLATE)
CHAR_BASE_TABLE_OFFSET = CHAR_HOOK_CODE_OFFSET + CHAR_HOOK_CODE_LENGTH
CHAR_SHIFT_TABLE_OFFSET = CHAR_BASE_TABLE_OFFSET + 0x100
CHAR_HOOK_TOTAL_LENGTH = CHAR_HOOK_CODE_LENGTH + 0x200
STARTUP_LAYOUT_FALLBACK_OFFSET = 0x13BA2
STARTUP_LAYOUT_FALLBACK_BYTES = bytes.fromhex("13fc000300005d36")

MENU_SPANS = {
    "dvorak": (0x35B80, 6),
    "right": (0x35B8A, 16),
    "left": (0x35BA0, 15),
}

STATUS_STRING_SLOTS = {
    "dvorak": (0x35BF7, 29),
    "right": (0x35C15, 39),
    "left": (0x35C3D, 38),
}

COMPACT_STRING_SLOTS = {
    "dvorak": (0x35C9F, 6, None),
    "right": (0x35CA6, 5, None),
    "left": (0x35CAC, 4, None),
    "dvorak_resource": (0x3989E, 6, 0x3989D),
    "right_resource": (0x39878, 5, 0x39877),
    "left_resource": (0x39884, 4, 0x39883),
}


@dataclass(frozen=True)
class FirmwareImage:
    data: bytes
    source: str

    @property
    def size(self) -> int:
        return len(self.data)

    @classmethod
    def load(cls, path: Path) -> "FirmwareImage":
        image = cls(data=path.read_bytes(), source=str(path))
        image.validate_stock()
        return image

    @classmethod
    def from_bytes(cls, data: bytes, *, source: str) -> "FirmwareImage":
        return cls(data=data, source=source)

    def validate_stock(self) -> None:
        for offset, expected in STRING_ANCHORS.items():
            actual = self.read_c_string(offset)
            if actual != expected:
                raise FirmwareValidationError(
                    f"string anchor mismatch at 0x{offset:x}: expected {expected!r}, got {actual!r}"
                )
        for offset, expected in BYTE_ANCHORS.items():
            actual = self.data[offset : offset + len(expected)]
            if actual != expected:
                raise FirmwareValidationError(
                    f"byte anchor mismatch at 0x{offset:x}: expected {expected.hex()}, got {actual.hex()}"
                )

    def read_c_string(self, offset: int) -> str:
        end = self.data.index(0, offset)
        return self.data[offset:end].decode("ascii")

    def patch_layout(self, *, replace: str, replacement: str) -> "FirmwareImage":
        slot = STOCK_LAYOUT_SLOTS[replace]
        replacement_layout = REPLACEMENT_LAYOUTS[replacement]
        patched = bytearray(self.data)
        for logical in range(LOGICAL_KEY_COUNT):
            patched[LAYOUT_TABLE_OFFSET + logical * 3 + slot] = logical
        for source_key, target_key in replacement_layout.key_map.items():
            source_logical = STOCK_PHYSICAL_TO_LOGICAL[source_key]
            target_logical = STOCK_PHYSICAL_TO_LOGICAL[target_key]
            patched[LAYOUT_TABLE_OFFSET + source_logical * 3 + slot] = target_logical
        self._patch_startup_layout_fallback(patched, slot=slot)
        self._patch_char_override_hook(patched, slot=slot, replacement=replacement_layout)
        self._patch_strings(patched, replace=replace, display_name=replacement_layout.display_name)
        return FirmwareImage.from_bytes(bytes(patched), source=self.source)

    def _patch_startup_layout_fallback(self, patched: bytearray, *, slot: int) -> None:
        if self.data[
            STARTUP_LAYOUT_FALLBACK_OFFSET : STARTUP_LAYOUT_FALLBACK_OFFSET + len(STARTUP_LAYOUT_FALLBACK_BYTES)
        ] != STARTUP_LAYOUT_FALLBACK_BYTES:
            raise FirmwareValidationError("unexpected startup layout fallback bytes")
        patched[
            STARTUP_LAYOUT_FALLBACK_OFFSET : STARTUP_LAYOUT_FALLBACK_OFFSET + len(STARTUP_LAYOUT_FALLBACK_BYTES)
        ] = bytes.fromhex(f"13fc000{slot}00005d36")

    def _patch_char_override_hook(
        self,
        patched: bytearray,
        *,
        slot: int,
        replacement,
    ) -> None:
        if not replacement.base_char_overrides and not replacement.shift_char_overrides:
            return
        if self.data[CHAR_HOOK_ENTRY_OFFSET : CHAR_HOOK_ENTRY_OFFSET + len(CHAR_HOOK_ENTRY_BYTES)] != CHAR_HOOK_ENTRY_BYTES:
            raise FirmwareValidationError("unexpected live char hook entry bytes")
        hook_region = self.data[CHAR_HOOK_CODE_OFFSET : CHAR_HOOK_CODE_OFFSET + CHAR_HOOK_TOTAL_LENGTH]
        if any(byte != 0xFF for byte in hook_region):
            raise FirmwareValidationError("live char hook region is not empty in stock firmware")

        patched[CHAR_HOOK_ENTRY_OFFSET : CHAR_HOOK_ENTRY_OFFSET + 10] = bytes.fromhex(
            f"4ef9{CHAR_HOOK_RUNTIME_ADDRESS:08x}4e714e71"
        )

        hook_code = bytearray(CHAR_HOOK_CODE_TEMPLATE)
        hook_code[0x15] = slot
        patched[CHAR_HOOK_CODE_OFFSET : CHAR_HOOK_CODE_OFFSET + CHAR_HOOK_CODE_LENGTH] = hook_code

        base_table = bytearray(0x100)
        shift_table = bytearray(0x100)
        for source_key, value in replacement.base_char_overrides.items():
            base_table[STOCK_PHYSICAL_TO_LOGICAL[source_key]] = ord(value)
        for source_key, value in replacement.shift_char_overrides.items():
            shift_table[STOCK_PHYSICAL_TO_LOGICAL[source_key]] = ord(value)
        patched[CHAR_BASE_TABLE_OFFSET : CHAR_BASE_TABLE_OFFSET + 0x100] = base_table
        patched[CHAR_SHIFT_TABLE_OFFSET : CHAR_SHIFT_TABLE_OFFSET + 0x100] = shift_table

    def _patch_strings(self, patched: bytearray, *, replace: str, display_name: str) -> None:
        menu_offset, menu_width = MENU_SPANS[replace]
        self._write_fixed_span(patched, menu_offset, menu_width, display_name, pad_byte=b" ")

        status_offset, status_width = STATUS_STRING_SLOTS[replace]
        status_text = f"Key layout changed to {display_name}."
        self._write_fixed_span(patched, status_offset, status_width, status_text, pad_byte=b"\x00")

        compact_offset, compact_width, compact_prefix = COMPACT_STRING_SLOTS[replace]
        self._write_fixed_span(patched, compact_offset, compact_width, display_name, pad_byte=b"\x00")
        if compact_prefix is not None:
            patched[compact_prefix] = min(len(display_name), compact_width) + 1

        resource_key = f"{replace}_resource"
        resource_offset, resource_width, resource_prefix = COMPACT_STRING_SLOTS[resource_key]
        self._write_fixed_span(patched, resource_offset, resource_width, display_name, pad_byte=b"\x00")
        patched[resource_prefix] = min(len(display_name), resource_width) + 1

    def _write_fixed_span(
        self,
        patched: bytearray,
        offset: int,
        width: int,
        text: str,
        *,
        pad_byte: bytes,
    ) -> None:
        encoded = text.encode("ascii")[:width]
        fill = pad_byte * (width - len(encoded))
        patched[offset : offset + width] = encoded + fill
