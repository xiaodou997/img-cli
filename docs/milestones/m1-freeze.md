# M1 Freeze Receipt

> Status: **Freeze candidate** until PR #3 is merged.

M1 establishes the first stable YuTool runtime baseline. M2 may extend the runtime, but should not casually break the contracts recorded here.

## Repository

- Repository: `xiaodou997/yu-tool`
- CLI executable: `yu`
- Schema version: `1`

The final M1 baseline commit is recorded after PR #3 merges.

## Crate boundaries

```text
crates/
├── yu-cli
├── yu-core
├── yu-engine-api
├── yu-capability-image
└── yu-engine-image-rs
```

Responsibilities:

- `yu-cli` — argument parsing and human/JSON presentation;
- `yu-core` — protocol, registry, resolution, runtime health;
- `yu-engine-api` — engine descriptor/provider/state types;
- `yu-capability-image` — image capability requests/results/trait;
- `yu-engine-image-rs` — built-in Rust raster implementation.

## Frozen public commands

```bash
yu doctor
yu capabilities
yu engine list

yu image info <file>
yu image resize <file> --width <n> -o <output>
yu image resize <file> --height <n> -o <output>
```

The image commands support `--engine raster-rs` and automation-oriented commands support `--json`.

## Built-in engines

### `yu-runtime`

Provider: `built_in`

Capabilities:

- `runtime.doctor`
- `runtime.capabilities`
- `engine.list`

### `raster-rs`

Provider: `built_in`

Capabilities:

- `image.info`
- `image.resize`

Initial default formats:

- PNG
- JPEG
- WebP

## Resize semantics

- width only preserves aspect ratio;
- height only preserves aspect ratio;
- width + height uses exact dimensions;
- zero dimensions are invalid;
- existing output paths are rejected;
- source files are not overwritten;
- output encoding is inferred from the output extension;
- completed output is staged through a temporary file before rename.

## JSON protocol v1

Successful responses use a core-owned envelope:

```json
{
  "schema_version": "1",
  "operation": "image.info",
  "engine": {
    "id": "raster-rs",
    "provider": "built_in",
    "version": "0.1.0"
  },
  "result": {},
  "warnings": []
}
```

The `engine` field is omitted when an operation is runtime-level and no execution engine is relevant.

Errors use:

```json
{
  "schema_version": "1",
  "error": {
    "code": "INVALID_INPUT",
    "message": "..."
  }
}
```

## Error codes

The M1 protocol defines:

- `INVALID_ARGUMENT`
- `INVALID_INPUT`
- `UNSUPPORTED_CAPABILITY`
- `ENGINE_UNAVAILABLE`
- `ENGINE_INCOMPATIBLE`
- `EXECUTION_FAILED`
- `OUTPUT_CONFLICT`
- `VERIFICATION_FAILED`

Exit-code groups:

- `0` success;
- `1` execution/verification failure;
- `2` argument/input/output-conflict failure;
- `3` capability/engine availability or compatibility failure.

## Engine resolution

Resolution order:

1. explicit engine;
2. ready built-in engine;
3. ready managed engine;
4. ready system engine;
5. structured failure.

M1 proves this flow with `image.info` and `image.resize`.

## CI gate

Freeze requires all of the following on PR #3:

- Rustfmt;
- Ubuntu: check + Clippy `-D warnings` + tests;
- macOS: check + Clippy `-D warnings` + tests;
- Windows: check + Clippy `-D warnings` + tests.

## Intentionally deferred to M2+

M1 does not define:

- managed engine manifests;
- managed engine download/update/remove;
- system executable discovery;
- package signatures/checksums;
- ImageMagick/libvips integration;
- PSD engine selection;
- crop/rotate/convert;
- overwrite mode;
- GUI manager.
