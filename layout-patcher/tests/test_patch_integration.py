from pathlib import Path
import unittest

from layout_patcher.firmware import FirmwareImage
from layout_patcher.layouts import STOCK_PHYSICAL_TO_LOGICAL


ROOT = Path(__file__).resolve().parents[2]
STOCK_OS_PATH = ROOT / "analysis" / "cab" / "os3kneorom.os3kos"
LAYOUT_TABLE_OFFSET = 0x3C3FB


class PatchIntegrationTests(unittest.TestCase):
    def test_patches_only_requested_layout_column(self) -> None:
        image = FirmwareImage.load(STOCK_OS_PATH)
        patched = image.patch_layout(replace="dvorak", replacement="czech")

        y_logical = STOCK_PHYSICAL_TO_LOGICAL["y"]
        z_logical = STOCK_PHYSICAL_TO_LOGICAL["z"]
        a_logical = STOCK_PHYSICAL_TO_LOGICAL["a"]

        self.assertEqual(
            patched.data[LAYOUT_TABLE_OFFSET + y_logical * 3 + 0],
            z_logical,
        )
        self.assertEqual(
            patched.data[LAYOUT_TABLE_OFFSET + z_logical * 3 + 0],
            y_logical,
        )
        self.assertEqual(patched.data[LAYOUT_TABLE_OFFSET + a_logical * 3 + 0], a_logical)
        self.assertEqual(
            patched.data[LAYOUT_TABLE_OFFSET + y_logical * 3 + 1],
            image.data[LAYOUT_TABLE_OFFSET + y_logical * 3 + 1],
        )
        self.assertEqual(
            patched.data[LAYOUT_TABLE_OFFSET + y_logical * 3 + 2],
            image.data[LAYOUT_TABLE_OFFSET + y_logical * 3 + 2],
        )

    def test_patches_selected_visible_names_and_crops_when_needed(self) -> None:
        image = FirmwareImage.load(STOCK_OS_PATH)

        patched_dvorak = image.patch_layout(replace="dvorak", replacement="czech")
        self.assertEqual(
            patched_dvorak.read_c_string(0x35B67),
            "1: QWERTY (default)   2: Czech ",
        )
        self.assertEqual(
            patched_dvorak.read_c_string(0x35BF7),
            "Key layout changed to Czech.",
        )
        self.assertEqual(patched_dvorak.read_c_string(0x35C9F), "Czech")
        self.assertEqual(patched_dvorak.read_c_string(0x3989E), "Czech")

        patched_left = image.patch_layout(replace="left", replacement="polish")
        self.assertEqual(
            patched_left.read_c_string(0x35B87),
            "3: Right (one hand)   4: Polish         ",
        )
        self.assertEqual(
            patched_left.read_c_string(0x35C3D),
            "Key layout changed to Polish.",
        )
        self.assertEqual(patched_left.read_c_string(0x35CAC), "Poli")
        self.assertEqual(patched_left.read_c_string(0x39884), "Poli")

    def test_czech_keeps_first_stage_remap_to_y_z_swap_only(self) -> None:
        image = FirmwareImage.load(STOCK_OS_PATH)
        patched = image.patch_layout(replace="dvorak", replacement="czech")

        for key in ["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "[", "]", "\\", ";", "'", ",", ".", "/"]:
            logical = STOCK_PHYSICAL_TO_LOGICAL[key]
            self.assertEqual(patched.data[LAYOUT_TABLE_OFFSET + logical * 3 + 0], logical)

    def test_czech_installs_live_char_override_hook_and_tables(self) -> None:
        image = FirmwareImage.load(STOCK_OS_PATH)
        patched = image.patch_layout(replace="dvorak", replacement="czech")

        self.assertEqual(
            patched.data[0x13C7C:0x13C86].hex(),
            "4ef900452e8e4e714e71",
        )
        self.assertEqual(
            patched.data[0x42E8E:0x42ED6].hex(),
            (
                "48e70f18558f3e2f00200c070050620000320c3800005d36"
                "66000028700010073c070246040041fa00206700000641fa"
                "0118103008006700000a548f4cdf18f04e754ef900423c86"
            ),
        )
        base_table = patched.data[0x42ED6:0x42FD6]
        shift_table = patched.data[0x42FD6:0x430D6]
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["1"]], ord("+"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["1"]], ord("1"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["2"]], ord("e"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["2"]], ord("2"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["q"]], ord("q"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["q"]], ord("Q"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["a"]], ord("a"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["a"]], ord("A"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["y"]], ord("z"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["y"]], ord("Z"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["z"]], ord("y"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["z"]], ord("Y"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["["]], ord("u"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["["]], ord("/"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL[";"]], ord("u"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL[";"]], ord('"'))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["'"]], ord("#"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["'"]], ord("!"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["\\"]], ord("\\"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["\\"]], ord("|"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL[","]], ord(","))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL[","]], ord("?"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["."]], ord("."))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["."]], ord(":"))
        self.assertEqual(base_table[STOCK_PHYSICAL_TO_LOGICAL["/"]], ord("/"))
        self.assertEqual(shift_table[STOCK_PHYSICAL_TO_LOGICAL["/"]], ord("/"))
        self.assertEqual(
            patched.data[0x13BA2:0x13BAA].hex(),
            "13fc000000005d36",
        )

    def test_polish_keeps_layout_table_identity(self) -> None:
        image = FirmwareImage.load(STOCK_OS_PATH)
        patched = image.patch_layout(replace="left", replacement="polish")

        for logical in range(0x51):
            self.assertEqual(
                patched.data[LAYOUT_TABLE_OFFSET + logical * 3 + 1],
                logical,
            )
        self.assertEqual(
            patched.data[0x13BA2:0x13BAA].hex(),
            "13fc000100005d36",
        )
