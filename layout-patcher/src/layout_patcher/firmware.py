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
        for source_char, target_char in replacement_layout.char_map.items():
            source_logical = STOCK_PHYSICAL_TO_LOGICAL[source_char]
            target_logical = STOCK_PHYSICAL_TO_LOGICAL[target_char]
            patched[LAYOUT_TABLE_OFFSET + source_logical * 3 + slot] = target_logical
        self._patch_strings(patched, replace=replace, display_name=replacement_layout.display_name)
        return FirmwareImage.from_bytes(bytes(patched), source=self.source)

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
