from pathlib import Path
import unittest

from layout_patcher.firmware import FirmwareImage, FirmwareValidationError
from layout_patcher.layouts import REPLACEMENT_LAYOUTS, STOCK_LAYOUT_SLOTS


ROOT = Path(__file__).resolve().parents[2]
STOCK_OS_PATH = ROOT / "analysis" / "cab" / "os3kneorom.os3kos"


class FirmwareTests(unittest.TestCase):
    def test_supported_layout_metadata_is_explicit(self) -> None:
        self.assertEqual(STOCK_LAYOUT_SLOTS["dvorak"], 0)
        self.assertEqual(STOCK_LAYOUT_SLOTS["left"], 1)
        self.assertEqual(STOCK_LAYOUT_SLOTS["right"], 2)
        self.assertEqual(set(REPLACEMENT_LAYOUTS), {"czech", "polish"})

    def test_identifies_stock_os_image(self) -> None:
        image = FirmwareImage.load(STOCK_OS_PATH)

        self.assertEqual(image.size, STOCK_OS_PATH.stat().st_size)
        self.assertEqual(image.read_c_string(0x35c98), "QWERTY")

    def test_rejects_image_with_bad_anchor(self) -> None:
        data = bytearray(STOCK_OS_PATH.read_bytes())
        data[0x35c98] = ord("X")
        image = FirmwareImage.from_bytes(bytes(data), source="mutated")

        with self.assertRaisesRegex(FirmwareValidationError, "0x35c98"):
            image.validate_stock()
