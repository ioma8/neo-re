import argparse
from collections.abc import Sequence
from pathlib import Path

from .firmware import FirmwareImage
from .layouts import REPLACEMENT_LAYOUTS, STOCK_LAYOUT_SLOTS


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="layout-patcher")
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--replace", choices=sorted(STOCK_LAYOUT_SLOTS), required=True)
    parser.add_argument("--with", dest="replacement", choices=sorted(REPLACEMENT_LAYOUTS), required=True)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    try:
        args = build_parser().parse_args(argv)
    except SystemExit as exc:
        return int(exc.code)
    image = FirmwareImage.load(args.input)
    patched = image.patch_layout(replace=args.replace, replacement=args.replacement)
    args.output.write_bytes(patched.data)
    print(
        f"input={args.input} output={args.output} replace={args.replace} with={args.replacement}",
    )
    return 0
