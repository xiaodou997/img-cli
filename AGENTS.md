# AGENTS.md

This repository builds **YuTool**, a lightweight, extensible local tool runtime. The public CLI command is **\`yu\`**.

This file defines development rules for coding agents and contributors.

## Product rules

1. Keep the core small.
2. Treat the public CLI as a stable API.
3. Prefer capability-oriented commands over exposing backend-specific syntax.
4. Do not make optional engines hard dependencies without an explicit architecture decision.
5. Prefer non-destructive file operations by default.
6. Machine-readable output is a first-class interface, not an afterthought.
7. Do not claim an engine supports a capability until it is verified by tests or a documented upstream guarantee.

## Architecture rules

YuTool separates **capabilities** from **engines**.

- A capability describes what the user wants to do.
- An engine describes how that capability is implemented.
- The resolver selects an engine based on availability, compatibility, user preference, and operation requirements.

Engine installation classes:

- **Built-in** — compiled into YuTool.
- **Managed** — installed and versioned by YuTool.
- **System** — discovered from the host environment.

Do not couple command parsing directly to ImageMagick, psd-tools, FFmpeg, or any other backend.

## Preferred implementation direction

The main application and orchestration layer should be Rust-first.

Planned workspace areas:

\`\`\`text
crates/
├── yu-core
├── yu-cli
├── yu-engine-api
├── yu-engine-manager
├── yu-capability-image
└── yu-engine-image-rs
\`\`\`

The exact crate layout may evolve, but boundaries between CLI, capabilities, and engines should remain explicit.

External engines may be written in Rust, Python, TypeScript, C/C++, or another language if they satisfy the engine contract.

## CLI contract

The command name is:

\`\`\`bash
yu
\`\`\`

Core global commands currently planned:

\`\`\`bash
yu doctor
yu capabilities
yu engine list
yu engine install <engine>
yu engine remove <engine>
\`\`\`

Capability commands start with a capability namespace:

\`\`\`bash
yu image ...
yu psd ...
yu pdf ...
\`\`\`

Do not add a new public command without updating \`docs/cli-spec.md\`.

## Structured output

Commands used by automation should support \`--json\`.

JSON output must:

- include a schema version where the structure is intended to be stable;
- avoid human-formatted text embedded in machine fields;
- use stable error codes;
- avoid breaking field changes without a documented schema/version change.

Human-readable output belongs on stdout. Diagnostics and warnings belong on stderr.

## Exit codes

Until a more complete error model is adopted:

- \`0\` — success
- \`1\` — execution/runtime failure
- \`2\` — invalid arguments or invalid input
- \`3\` — capability or engine unavailable

If this mapping changes, update \`docs/cli-spec.md\` and \`docs/agent-guide.md\` together.

## Engine execution safety

When invoking external processes:

- pass arguments as an argv array;
- do not construct shell command strings from user input;
- validate input/output paths;
- surface the exact selected engine in debug/structured output;
- capture version information for diagnostics;
- apply timeouts/cancellation where appropriate;
- never silently install or elevate privileges.

Managed-engine installation must verify downloads before activation.

## File safety

Mutating operations should write to a new output path by default.

Overwriting a source file must require an explicit option and should use safe replacement semantics where practical.

Future mutation commands should support \`--dry-run\` when an operation can be meaningfully previewed.

## Documentation synchronization

Update documentation with behavior changes:

| Change | Required docs |
| --- | --- |
| CLI command/flag | \`docs/cli-spec.md\` |
| Capability added/removed | \`docs/capabilities.md\` |
| Engine model change | \`docs/architecture.md\` |
| Milestone/scope change | \`docs/roadmap.md\` |
| Major architectural decision | new ADR under \`docs/decisions/\` |

## Tests

New functionality should include tests at the appropriate level:

- unit tests for parsing, resolution, and validation;
- fixture-based tests for file formats;
- integration tests for CLI behavior;
- engine conformance tests for interchangeable implementations;
- golden/structured-output tests for stable JSON contracts.

Do not rely only on tests against one locally installed system tool.

## Scope discipline

The first milestone is focused on proving the runtime and engine model through image/PSD workflows.

Do not expand into PDF, video, audio, OCR, archives, or unrelated capabilities in the first implementation milestone unless the roadmap is explicitly updated.
