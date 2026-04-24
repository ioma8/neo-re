import contextlib
import io
from pathlib import Path
import tempfile
import unittest

from layout_patcher import main


ROOT = Path(__file__).resolve().parents[2]
STOCK_OS_PATH = ROOT / "analysis" / "cab" / "os3kneorom.os3kos"


class CLITests(unittest.TestCase):
    def test_help_mentions_replace_and_with(self) -> None:
        stdout = io.StringIO()

        with contextlib.redirect_stdout(stdout):
            exit_code = main(["--help"])

        self.assertEqual(exit_code, 0)
        self.assertIn("--replace", stdout.getvalue())
        self.assertIn("--with", stdout.getvalue())

    def test_cli_patches_image_and_prints_summary(self) -> None:
        stdout = io.StringIO()

        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = Path(tmpdir) / "patched.os3kos"
            with contextlib.redirect_stdout(stdout):
                exit_code = main(
                    [
                        "--input",
                        str(STOCK_OS_PATH),
                        "--output",
                        str(output_path),
                        "--replace",
                        "dvorak",
                        "--with",
                        "czech",
                    ]
                )

            self.assertEqual(exit_code, 0)
            self.assertTrue(output_path.exists())
            self.assertIn("replace=dvorak", stdout.getvalue())
            self.assertIn("with=czech", stdout.getvalue())
