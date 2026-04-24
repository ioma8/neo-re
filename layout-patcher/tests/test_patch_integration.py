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
        self.assertEqual(
            patched.data[LAYOUT_TABLE_OFFSET + a_logical * 3 + 0],
            a_logical,
        )
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
