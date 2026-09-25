# PSD / PSB Spike Fixture Corpus

This directory is the engine-neutral input corpus for the M3 PSD Engine Spike.

## Rules

- Every committed fixture must be listed in `corpus.json`.
- Paths are relative to this directory and may not escape it.
- Every fixture records provenance and whether it is redistributable.
- Prefer synthetic or explicitly redistributable files.
- Do not commit customer files, private design files, or fixtures with unclear redistribution rights.
- Local-only fixtures may be used during investigation, but should live outside the committed corpus.
- Candidate engines must consume the same corpus; candidate-specific fixtures are not a substitute for conformance coverage.

## Coverage tags

The schema currently recognizes:

- `simple_pixel_layers`
- `nested_groups`
- `duplicate_layer_names`
- `text_layers`
- `masks`
- `blend_modes`
- `effects`
- `smart_object_metadata`
- `high_bit_depth`
- `malformed`

The initial PR #10 corpus intentionally contains only one synthetic malformed input. Representative valid PSD/PSB files will be added as separate, reviewable corpus changes so provenance and expectations can be audited.

## Validate

```bash
cargo run -p yu-psd-spike -- validate fixtures/psd/corpus.json
```
