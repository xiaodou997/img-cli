from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


UNAVAILABLE_EXIT = 3


def unavailable(message: str) -> int:
    print(message, file=sys.stderr)
    return UNAVAILABLE_EXIT


def main() -> int:
    parser = argparse.ArgumentParser(description="YuTool psd-tools reference adapter")
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--expected-python", required=True)
    parser.add_argument("input", type=Path)
    args = parser.parse_args()

    actual_python = f"{sys.version_info.major}.{sys.version_info.minor}"
    if actual_python != args.expected_python:
        return unavailable(
            f"Python version mismatch: expected {args.expected_python}, got {actual_python}"
        )

    try:
        from psd_tools import PSDImage
        from psd_tools.version import __version__ as psd_tools_version
    except Exception as error:
        return unavailable(f"psd-tools is unavailable: {error}")

    if psd_tools_version != args.expected_version:
        return unavailable(
            f"psd-tools version mismatch: expected {args.expected_version}, got {psd_tools_version}"
        )

    try:
        psd = PSDImage.open(args.input)
    except Exception:
        print(json.dumps({"parse_success": False}, separators=(",", ":")))
        return 0

    layers = []
    maximum_tree_depth = 0

    def visit(container, depth: int) -> None:
        nonlocal maximum_tree_depth
        for layer in container:
            layers.append(layer)
            maximum_tree_depth = max(maximum_tree_depth, depth)
            if layer.is_group():
                visit(layer, depth + 1)

    visit(psd, 1)

    observation = {
        "parse_success": True,
        "width": psd.width,
        "height": psd.height,
        "layer_count": len(layers),
        "maximum_tree_depth": maximum_tree_depth,
        "layer_names": [layer.name for layer in layers],
        "text_layer_count": sum(1 for layer in layers if layer.kind == "type"),
        "pixel_mask_layer_count": sum(1 for layer in layers if layer.has_mask()),
        "vector_mask_layer_count": sum(
            1 for layer in layers if layer.has_vector_mask()
        ),
    }
    print(json.dumps(observation, ensure_ascii=False, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
