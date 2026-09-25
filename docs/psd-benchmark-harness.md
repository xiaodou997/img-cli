# M3 PSD Performance / Layer Export Benchmark Harness

PR #16 adds a candidate-neutral harness for two evidence gaps left by the M3 Candidate Comparison Report:

1. layer export conformance;
2. repeatable performance sampling.

It does **not** select an engine and does **not** publish a performance ranking.

## Layer export contract

The harness exports one logical layer into an engine-neutral observation:

```json
{
  "layer_name": "Слой",
  "width": 0,
  "height": 0,
  "pixel_format": "rgba8",
  "rgba_sha256": "...",
  "rgba_bytes": 0
}
```

The comparison uses decoded RGBA bytes rather than PNG files. This intentionally removes PNG encoder choices, metadata chunks, compression level, and container byte layout from the comparison.

The v1 workload selects:

```text
fixture: simple-pixel-layers-psd
layer:   Слой
```

The selector must match exactly one logical layer.

## Candidate implementations

### rawpsd 0.2.2

The Rust candidate hashes `LayerInfo::image_data_rgba` directly after validating that its byte length is `width * height * 4`.

### psd-tools 1.20.0

The Python adapter selects the layer by name, calls `layer.topil()`, converts the result to RGBA, and hashes `Image.tobytes()`.

### ag-psd 31.0.2

The Node adapter loads the document with `useRawData: true`, decodes only the selected layer through `getLayerImageData()`, requires 8-bit pixel storage, and hashes the raw RGBA byte view.

## Benchmark workload v1

Committed config:

```text
fixtures/psd/benchmark-v1.json
```

Current parameters:

```text
fixture             = simple-pixel-layers-psd
layer               = Слой
warmup_iterations   = 2
sample_iterations   = 5
```

The harness records two sample arrays:

- `inspect_samples_us` — parse plus normalized inspect/tree extraction;
- `layer_export_samples_us` — parse plus selected-layer RGBA decode/hash.

For each array the Rust harness reports min / median / max.

## Measurement policy

Benchmark schema v1 requires:

```text
measurement     = warm_runtime_operation
input_read      = once_before_timing
runtime_startup = excluded
```

That means:

- the input file is read into memory once before timed samples;
- Python and Node process startup are outside timed samples;
- candidate runtime setup and package installation are outside timed samples;
- each timed sample still performs that candidate's real parse/normalization or parse/export path;
- warmups are executed before samples.

This makes the benchmark useful for comparing the candidate operation path without turning runtime startup into parser time.

It does **not** make results from different CI machines directly comparable.

## Why CI has no speed threshold

GitHub-hosted runner timing is noisy and hardware varies. PR #16 therefore checks only that:

- the benchmark config is valid;
- the expected number of samples is returned;
- the measurement policy is preserved;
- layer export is valid RGBA8;
- standalone export and benchmark export produce the same fingerprint for each candidate;
- the benchmark completes without harness errors.

CI does not assert that one engine must be faster than another and does not freeze microsecond values.

A later controlled benchmark report should run on identified hardware with repeated trials and record environment details.

## Commands

Layer export:

```bash
cargo run -p yu-psd-spike -- \
  export-layer rust-native \
  fixtures/psd/corpus.json \
  simple-pixel-layers-psd \
  Слой
```

Benchmark:

```bash
cargo run -p yu-psd-spike -- \
  benchmark rust-native \
  fixtures/psd/corpus.json \
  fixtures/psd/benchmark-v1.json
```

Use `psd-tools` or `typescript-psd` as the candidate ID after installing their pinned spike runtime.

## Still outside PR #16

This harness does not yet establish:

- a frozen cross-engine RGBA reference fingerprint;
- large PSD / PSB benchmark inputs;
- peak RSS or allocation measurements;
- render/composite fidelity;
- effects fidelity;
- Smart Object export;
- round-trip/write safety;
- a production runtime distribution strategy.

Those remain evidence inputs for the final PSD Engine Strategy.
