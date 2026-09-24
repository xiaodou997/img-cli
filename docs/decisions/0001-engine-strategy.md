# ADR 0001: Capability-Oriented Runtime with Pluggable Engines

- **Status:** Accepted
- **Date:** 2026-09-24
- **Decision owners:** YuTool maintainers

## Context

The project began as an image/PSD CLI concept combining tools such as psd-tools and ImageMagick.

That approach exposed a broader problem: useful local capabilities are implemented across many languages and runtimes, and forcing all of them into the default installation would make the project heavy and difficult to distribute.

At the same time, binding the public CLI directly to one backend would make future replacement or compatibility fallback difficult.

The project therefore needs:

- a lightweight default installation;
- a stable CLI for developers and agents;
- support for implementations written in different languages;
- optional installation of heavyweight engines;
- system-tool discovery;
- room to expand beyond images without turning the CLI into a set of unrelated wrappers.

## Decision

YuTool will use a **capability-oriented architecture with pluggable engines**.

### 1. Rust-first core

The main runtime, CLI orchestration, capability registry, engine registry, engine resolver, and engine manager will be Rust-first.

Rust is the preferred core implementation language because the product is intended to ship as a small cross-platform local runtime and manage heterogeneous subprocesses/runtimes.

This does not require all engines to be written in Rust.

### 2. Separate capabilities from engines

Public commands represent capabilities:

\`\`\`text
image.resize
psd.tree
psd.layer.export
\`\`\`

Engines implement those capabilities.

The CLI must not expose backend-specific syntax as the primary product interface.

### 3. Three engine provider classes

Engines are classified as:

- **Built-in** — compiled/shipped as part of YuTool;
- **Managed** — downloaded and lifecycle-managed by YuTool;
- **System** — discovered from the host environment.

The provider class is part of engine metadata.

### 4. Optional engines are not default dependencies

Support for a backend does not mean every user installs it.

Examples of potential optional engines include Python packages/runtimes, ImageMagick, libvips, ExifTool, FFmpeg, or TypeScript/Node-based implementations.

The default distribution should remain useful without installing all of them.

### 5. Engine resolution is explicit and observable

The runtime selects an engine through an Engine Resolver.

Initial preference:

1. explicit engine selection;
2. suitable built-in engine;
3. suitable managed engine;
4. suitable system engine;
5. structured unavailable/unsupported result.

The actual selected engine should be observable in structured output/debugging.

### 6. Managed installation is owned by YuTool

Where YuTool offers managed installation, it must:

- use YuTool-controlled storage;
- verify downloaded artifacts;
- install/activate atomically where practical;
- expose installed version/source;
- remove only assets it owns.

YuTool should not silently alter system package-manager state.

### 7. GUI shares the same core

A future YuTool Manager may provide engine selection and installation through a GUI, preferably Tauri.

The GUI must call the same core engine-management logic as the CLI rather than reimplementing it.

## Consequences

### Positive

- small default install;
- one public CLI across heterogeneous tools;
- easier backend replacement;
- easier testing of multiple implementations;
- optional heavy runtimes;
- better Agent discoverability;
- clear path to future PDF/media/etc. capabilities.

### Costs

- engine abstraction adds design work;
- capability semantics must be defined carefully;
- managed runtime distribution creates signing/integrity/update responsibilities;
- multiple engines can produce different fidelity/performance;
- conformance tests become necessary.

## Alternatives considered

### Python-first monolith

A Python CLI with psd-tools/Pillow would provide a fast image/PSD prototype.

Rejected as the overall architecture because the product is now intended to be a broader cross-platform runtime with optional multi-language engines. Python remains valid for individual engines.

### Bundle every engine

Bundling ImageMagick, Python runtimes, Node runtimes, and other tools would simplify discovery.

Rejected because it conflicts with the lightweight product goal and creates large distribution/security/update responsibilities.

### Require users to install every backend manually

This is simple for YuTool maintainers but creates poor UX for developers and agents.

Rejected as the only strategy. System discovery remains supported, but important optional engines may also be offered as YuTool-managed installations.

### Expose backend CLIs directly

For example, make YuTool mostly forward ImageMagick/FFmpeg/etc. arguments.

Rejected because it does not provide a stable capability API and gives agents little abstraction benefit.

## Follow-up decisions

Separate ADRs should be written for:

- managed-engine package/manifest format;
- third-party engine protocol/ABI;
- PSD engine selection after the spike;
- update/signing policy;
- plugin security model;
- stable JSON schema/versioning policy.
