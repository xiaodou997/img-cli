# PSD / PSB Spike Fixture Corpus

This directory is the engine-neutral input corpus for the M3 PSD Engine Spike.

## Corpus v1

| Fixture | Coverage |
| --- | --- |
| `simple-pixel-layers-psd` | two ordinary pixel layers / PSD |
| `simple-pixel-layers-psb` | same basic shape / PSB |
| `nested-group` | one group containing a child layer |
| `duplicate-layer-names` | two logical layers with the same name |
| `text-layer` | minimal Photoshop type layer |
| `layer-masks` | pixel masks and vector masks |
| `malformed-truncated-header` | deterministic parser rejection |

The corpus is intentionally small. Effects, Smart Objects, blend modes, higher bit depths, and broader malformed inputs remain follow-up coverage.

## Expectation semantics

- `layer_count` counts logical user-facing layers/groups and excludes PSD section-divider sentinel records.
- `minimum_tree_depth` is a lower bound. A flat document has depth 1; a group containing a child has depth at least 2.
- `layer_names` is compared as a multiset, not by parser-native ordering.
- `text_layer_count` counts logical text layers.
- `pixel_mask_layer_count` and `vector_mask_layer_count` are tracked independently.

## Provenance rules

- Every committed fixture must be listed in `corpus.json`.
- Paths are relative to this directory and may not escape it.
- Every fixture records provenance and whether it is redistributable.
- Prefer synthetic or explicitly redistributable files.
- Do not commit customer files, private design files, or fixtures with unclear redistribution rights.
- Local-only fixtures may be used during investigation, but should live outside the committed corpus.
- Candidate engines must consume the same corpus; candidate-specific fixtures are not a substitute for conformance coverage.

The upstream fixtures in `upstream/psd-tools/` are copied from `psd-tools/psd-tools` at commit `f1256273ffb9b39b9efa19d486c86f594c831f42` under its MIT license. See `THIRD_PARTY.md` and `licenses/psd-tools-MIT.txt`.

The duplicate-name fixture is a YuTool derivative of the upstream `2layers.psd`: only its Unicode `luni` layer names are normalized to `X`; legacy Pascal names and the original document structure are preserved.

## Coverage tags

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

## Validate

```bash
cargo run -p yu-psd-spike -- validate fixtures/psd/corpus.json
```
