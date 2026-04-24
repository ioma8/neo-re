from dataclasses import dataclass


STOCK_LAYOUT_SLOTS = {
    "dvorak": 0,
    "left": 1,
    "right": 2,
}

STOCK_PHYSICAL_TO_LOGICAL = {
    "a": 0x18,
    "b": 0x4E,
    "c": 0x41,
    "d": 0x16,
    "e": 0x20,
    "f": 0x19,
    "g": 0x10,
    "h": 0x11,
    "i": 0x1C,
    "j": 0x1B,
    "k": 0x12,
    "l": 0x13,
    "m": 0x46,
    "n": 0x4F,
    "o": 0x1D,
    "p": 0x1E,
    "q": 0x22,
    "r": 0x23,
    "s": 0x17,
    "t": 0x07,
    "u": 0x24,
    "v": 0x44,
    "w": 0x21,
    "x": 0x42,
    "y": 0x09,
    "z": 0x43,
}


@dataclass(frozen=True)
class ReplacementLayout:
    display_name: str
    char_map: dict[str, str]


REPLACEMENT_LAYOUTS = {
    # ASCII fallback of the Czech base layer: the stable difference versus
    # stock QWERTY is the common QWERTZ letter swap.
    "czech": ReplacementLayout(
        display_name="Czech",
        char_map={
            "y": "z",
            "z": "y",
        },
    ),
    # ASCII fallback of the common Polish programmer layout is the stock
    # QWERTY base layer because accented letters normally live behind AltGr.
    "polish": ReplacementLayout(
        display_name="Polish",
        char_map={},
    ),
}
