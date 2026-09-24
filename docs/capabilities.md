# Capability Matrix

> This document distinguishes **product capabilities** from **candidate engine implementations**.

The matrix evolves as implementation spikes and conformance tests are completed.

## Status legend

| Status | Meaning |
| --- | --- |
| Planned | part of the intended milestone, not implemented yet |
| Experimental | implemented or spiked, but not stable |
| Supported | covered by the product contract and tests |
| Partial | usable with documented limitations |
| Unsupported | intentionally unavailable or blocked by known limitations |

## Bootstrap capabilities

| Capability | Status | Default implementation | Notes |
| --- | --- | --- | --- |
| `runtime.doctor` | Supported | Rust core | Diagnose runtime/engines |
| `runtime.capabilities` | Supported | Rust core | Effective capabilities |
| `engine.list` | Supported | Rust core | Built-in/managed/system |
| `engine.install` | Planned | Engine Manager | Managed engines only |
| `engine.remove` | Planned | Engine Manager | Must not remove system packages |

## Raster image

The current built-in engine is:

```text
raster-rs
provider: built_in
formats: PNG, JPEG, WebP
```

| Capability | Status | Default implementation | Optional engines |
| --- | --- | --- | --- |
| `image.info` | Supported | `raster-rs` / Rust `image` | ImageMagick/libvips later |
| `image.resize` | Supported | `raster-rs` / Rust `image` | fast_image_resize/ImageMagick/libvips later |
| `image.crop` | Planned | Rust built-in | ImageMagick/libvips |
| `image.rotate` | Planned | Rust built-in | ImageMagick |
| `image.convert` | Planned | Rust built-in where format support exists | ImageMagick/libvips |

Current resize semantics:

- width only: preserve aspect ratio;
- height only: preserve aspect ratio;
- width + height: exact target dimensions;
- output must be a new path;
- existing output files are rejected rather than overwritten;
- encoding is selected from the output extension.

The first implementation deliberately favors a small, predictable dependency footprint over maximum format breadth or peak resizing throughput.

## PSD / PSB

PSD support requires an implementation spike before a built-in engine is selected.

| Capability | v0.1 target | Engine status |
| --- | --- | --- |
| `psd.inspect` | Planned | engine selection pending |
| `psd.tree` | Planned | engine selection pending |
| `psd.layer.list` | Planned | engine selection pending |
| `psd.layer.info` | Planned | engine selection pending |
| `psd.layer.export` | Planned | engine selection pending |
| `psd.render` | Planned | engine selection pending |
| layer rename | Future | not frozen |
| show/hide layer | Future | not frozen |
| layer opacity | Future | not frozen |
| layer move/delete | Future | not frozen |
| text-layer editing | Future/Research | must be capability-tested |
| Smart Object editing | Future/Research | must be capability-tested |

Candidate engines to evaluate include:

- Rust PSD implementations;
- psd-tools;
- TypeScript PSD implementations where useful;
- other mature native implementations discovered during the spike.

No candidate is considered the permanent default until fixture testing is complete.

## Engine-management capabilities

Planned engine metadata:

| Field | Purpose |
| --- | --- |
| engine ID | stable identifier |
| provider class | built-in / managed / system |
| version | actual active version |
| state | ready / missing / broken / incompatible / disabled |
| capabilities | operations implemented |
| platform support | OS/architecture compatibility |
| install size | useful for managed-engine UI |
| license metadata | installation transparency |

## Future capability families

These are **not v0.1 scope**.

| Family | Examples |
| --- | --- |
| PDF | inspect, extract, merge, render |
| Media | probe, transcode, extract audio/frame |
| Metadata | read/write EXIF/XMP |
| SVG/vector | inspect/render/convert |
| OCR | local text recognition |
| Archive | inspect/extract/create |
| Document | conversion and structural extraction |
| RAW | inspect/decode/convert |

## Capability discovery

`yu capabilities --json` reports what can be executed **now**, given:

- current platform;
- built-in features;
- managed engines installed;
- system engines discovered;
- input-independent compatibility checks.

A later operation can still fail if a particular input uses unsupported features.

## Conformance requirement

Two engines claiming the same capability should be testable against the same behavioral contract.

For example, implementations of `image.resize` should agree on:

- argument validation;
- output-file safety;
- structured result shape;
- cancellation/error classification;
- basic dimension semantics.

Pixel-perfect equality is not necessarily required when algorithms differ, but behavioral differences must be explicit.
