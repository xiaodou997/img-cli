from __future__ import annotations

import argparse
import hashlib
import io
import json
import sys
import time
from pathlib import Path


UNAVAILABLE_EXIT = 3


def unavailable(message: str) -> int:
    print(message, file=sys.stderr)
    return UNAVAILABLE_EXIT


def flatten_layers(psd):
    layers = []

    def visit(container, depth: int) -> int:
        maximum_tree_depth = 0
        for layer in container:
            layers.append((layer, depth))
            maximum_tree_depth = max(maximum_tree_depth, depth)
            if layer.is_group():
                maximum_tree_depth = max(maximum_tree_depth, visit(layer, depth + 1))
        return maximum_tree_depth

    return layers, visit(psd, 1)


def inspect_psd(psd) -> dict:
    layers, maximum_tree_depth = flatten_layers(psd)
    return {
        "parse_success": True,
        "width": psd.width,
        "height": psd.height,
        "layer_count": len(layers),
        "maximum_tree_depth": maximum_tree_depth,
        "layer_names": [layer.name for layer, _depth in layers],
        "text_layer_count": sum(1 for layer, _depth in layers if layer.kind == "type"),
        "pixel_mask_layer_count": sum(
            1 for layer, _depth in layers if layer.has_mask()
        ),
        "vector_mask_layer_count": sum(
            1 for layer, _depth in layers if layer.has_vector_mask()
        ),
    }


def find_layer(psd, layer_name: str):
    layers, _depth = flatten_layers(psd)
    matches = [layer for layer, _depth in layers if layer.name == layer_name]
    if len(matches) != 1:
        raise RuntimeError(
            f"layer selector {layer_name!r} matched {len(matches)} layers; expected exactly one"
        )
    return matches[0]


def export_layer(psd, layer_name: str) -> dict:
    layer = find_layer(psd, layer_name)
    image = layer.topil()
    if image is None:
        raise RuntimeError(f"layer {layer_name!r} has no exportable pixel data")
    rgba = image.convert("RGBA")
    raw = rgba.tobytes()
    return {
        "layer_name": layer_name,
        "width": rgba.width,
        "height": rgba.height,
        "pixel_format": "rgba8",
        "rgba_sha256": hashlib.sha256(raw).hexdigest(),
        "rgba_bytes": len(raw),
    }


def benchmark(PSDImage, input_bytes: bytes, layer_name: str, warmups: int, samples: int):
    def open_psd():
        return PSDImage.open(io.BytesIO(input_bytes))

    for _ in range(warmups):
        inspect_psd(open_psd())

    inspect_samples_us = []
    for _ in range(samples):
        started = time.perf_counter_ns()
        inspect_psd(open_psd())
        inspect_samples_us.append((time.perf_counter_ns() - started) // 1000)

    for _ in range(warmups):
        export_layer(open_psd(), layer_name)

    export_samples_us = []
    export_observation = None
    for _ in range(samples):
        started = time.perf_counter_ns()
        export_observation = export_layer(open_psd(), layer_name)
        export_samples_us.append((time.perf_counter_ns() - started) // 1000)

    return {
        "measurement": "warm_runtime_operation",
        "input_read": "once_before_timing",
        "runtime_startup": "excluded",
        "warmup_iterations": warmups,
        "sample_iterations": samples,
        "inspect_samples_us": inspect_samples_us,
        "layer_export_samples_us": export_samples_us,
        "layer_export": export_observation,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="YuTool psd-tools reference adapter")
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--expected-python", required=True)
    parser.add_argument(
        "--mode",
        choices=("inspect", "export-layer", "benchmark"),
        default="inspect",
    )
    parser.add_argument("--layer-name")
    parser.add_argument("--warmups", type=int, default=0)
    parser.add_argument("--samples", type=int, default=0)
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

    if args.mode == "benchmark":
        if not args.layer_name:
            parser.error("--layer-name is required for benchmark mode")
        if args.warmups < 0 or args.samples <= 0:
            parser.error("benchmark requires --warmups >= 0 and --samples > 0")
        try:
            input_bytes = args.input.read_bytes()
            result = benchmark(
                PSDImage,
                input_bytes,
                args.layer_name,
                args.warmups,
                args.samples,
            )
        except Exception as error:
            print(f"psd-tools benchmark failed: {error}", file=sys.stderr)
            return 4
        print(json.dumps(result, separators=(",", ":")))
        return 0

    try:
        psd = PSDImage.open(args.input)
    except Exception:
        if args.mode == "inspect":
            print(json.dumps({"parse_success": False}, separators=(",", ":")))
            return 0
        print("psd-tools layer export could not parse input", file=sys.stderr)
        return 4

    if args.mode == "export-layer":
        if not args.layer_name:
            parser.error("--layer-name is required for export-layer mode")
        try:
            result = export_layer(psd, args.layer_name)
        except Exception as error:
            print(f"psd-tools layer export failed: {error}", file=sys.stderr)
            return 4
        print(json.dumps(result, separators=(",", ":")))
        return 0

    print(json.dumps(inspect_psd(psd), separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
