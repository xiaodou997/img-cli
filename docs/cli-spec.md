# CLI Specification

> Status: **Draft contract for bootstrap / v0.1**

The public executable is:

\`\`\`bash
yu
\`\`\`

This document defines the intended public command model. Commands marked **planned** are not implemented yet.

## Design rules

1. Commands describe capabilities, not backend syntax.
2. Human-readable output is the default.
3. Automation-oriented commands should support \`--json\`.
4. Optional engine selection is explicit through \`--engine\` when needed.
5. Mutating operations should not overwrite source files by default.
6. Public command names and structured fields should remain stable once released.

## Global commands

### \`yu doctor\`

Diagnose YuTool and engine health.

\`\`\`bash
yu doctor
yu doctor --json
\`\`\`

Expected information includes:

- YuTool version;
- platform/architecture;
- built-in engine health;
- managed-engine health;
- discovered system engines;
- warnings and missing runtimes.

### \`yu capabilities\`

List currently usable capabilities.

\`\`\`bash
yu capabilities
yu capabilities --json
\`\`\`

This command reports **effective capabilities on the current machine**, not merely features known to the source code.

### \`yu engine list\`

List engines and their state.

\`\`\`bash
yu engine list
yu engine list --json
\`\`\`

Planned states include:

- \`ready\`
- \`not_installed\`
- \`broken\`
- \`incompatible\`
- \`disabled\`

Planned provider classes:

- \`built_in\`
- \`managed\`
- \`system\`

### \`yu engine install\`

Install a managed engine.

\`\`\`bash
yu engine install <engine-id>
\`\`\`

Installation must be explicit. YuTool must not silently install an engine while executing an unrelated operation.

### \`yu engine remove\`

Remove a YuTool-managed engine.

\`\`\`bash
yu engine remove <engine-id>
\`\`\`

This command must not uninstall a system package that YuTool does not own.

## Image commands

### \`yu image info\`

Inspect a raster image.

\`\`\`bash
yu image info photo.jpg
yu image info photo.jpg --json
\`\`\`

Potential structured fields:

- format;
- width;
- height;
- color model;
- bit depth where available;
- frame count where relevant;
- selected engine.

### \`yu image resize\`

Resize an image.

\`\`\`bash
yu image resize input.jpg --width 1024 -o output.jpg
yu image resize input.jpg --height 720 -o output.jpg
yu image resize input.jpg --width 1024 --engine raster-rs -o output.jpg
\`\`\`

Source overwrite is not the default.

### \`yu image crop\`

\`\`\`bash
yu image crop input.png \
  --x 100 \
  --y 100 \
  --width 500 \
  --height 500 \
  -o output.png
\`\`\`

### \`yu image rotate\`

\`\`\`bash
yu image rotate input.png --degrees 90 -o output.png
\`\`\`

### \`yu image convert\`

\`\`\`bash
yu image convert input.png -o output.webp
\`\`\`

The output format may be inferred from the output extension when unambiguous.

## PSD commands

PSD is currently a capability namespace rather than a promise about one specific backend.

### \`yu psd inspect\`

Inspect PSD/PSB document-level information.

\`\`\`bash
yu psd inspect design.psd
yu psd inspect design.psd --json
\`\`\`

### \`yu psd tree\`

Return the layer hierarchy.

\`\`\`bash
yu psd tree design.psd
yu psd tree design.psd --json
\`\`\`

A machine-readable result should expose stable selectors independent of duplicate layer names.

Example shape:

\`\`\`json
{
  "schema_version": "1",
  "operation": "psd.tree",
  "document": {
    "width": 1920,
    "height": 1080
  },
  "layers": [
    {
      "id": "L0001",
      "name": "Background",
      "kind": "pixel",
      "visible": true,
      "children": []
    }
  ]
}
\`\`\`

The exact ID implementation is not frozen yet. The requirement is that callers should not be forced to identify a layer only by its display name.

### \`yu psd layer list\`

\`\`\`bash
yu psd layer list design.psd
yu psd layer list design.psd --json
\`\`\`

### \`yu psd layer info\`

\`\`\`bash
yu psd layer info design.psd --id L0007
\`\`\`

Future selectors may include \`--path\` and \`--name\`, but ambiguous names must never silently select an arbitrary layer.

### \`yu psd layer export\`

\`\`\`bash
yu psd layer export design.psd --id L0007 -o layer.png
\`\`\`

### \`yu psd render\`

\`\`\`bash
yu psd render design.psd -o preview.png
\`\`\`

Rendering fidelity depends on the selected engine and document features. Structured output should report the actual engine used and relevant warnings.

## Common options

Planned common options:

\`\`\`text
--json
--engine <engine-id>
--verbose
--quiet
\`\`\`

Mutation-oriented commands may additionally support:

\`\`\`text
--dry-run
--overwrite
\`\`\`

\`--overwrite\` must be explicit when source or destination conflict would otherwise destroy data.

## Engine selection

Default engine resolution:

1. explicitly selected engine;
2. compatible built-in engine;
3. compatible managed engine;
4. compatible system engine;
5. structured unavailable/unsupported error.

An engine being installed does not guarantee compatibility with every input.

## JSON envelope

For stable automation-oriented results, prefer an envelope similar to:

\`\`\`json
{
  "schema_version": "1",
  "operation": "image.resize",
  "engine": {
    "id": "raster-rs",
    "provider": "built_in",
    "version": "0.1.0"
  },
  "result": {},
  "warnings": []
}
\`\`\`

Do not include terminal decoration or progress output in JSON mode.

## Structured errors

Suggested shape:

\`\`\`json
{
  "schema_version": "1",
  "error": {
    "code": "ENGINE_UNAVAILABLE",
    "message": "The requested operation requires an unavailable engine.",
    "details": {
      "engine": "imagemagick"
    }
  }
}
\`\`\`

Initial error codes should include at least:

- \`INVALID_ARGUMENT\`
- \`INVALID_INPUT\`
- \`UNSUPPORTED_CAPABILITY\`
- \`ENGINE_UNAVAILABLE\`
- \`ENGINE_INCOMPATIBLE\`
- \`EXECUTION_FAILED\`
- \`OUTPUT_CONFLICT\`
- \`VERIFICATION_FAILED\`

## Exit codes

Initial mapping:

| Code | Meaning |
| ---: | --- |
| 0 | success |
| 1 | execution/runtime failure |
| 2 | invalid arguments or invalid input |
| 3 | capability or engine unavailable |

More granular error meaning belongs in structured output rather than an excessively large exit-code table.

## stdout / stderr

- Successful human output: stdout
- Successful JSON output: stdout
- warnings/diagnostics: stderr
- progress UI: stderr or an interactive presentation layer, never mixed into JSON stdout

## Compatibility

After the first stable release:

- adding optional fields is preferred over changing field meaning;
- removing/renaming fields requires a schema-version decision;
- public commands should not silently change semantics;
- experimental commands must be explicitly labeled.
