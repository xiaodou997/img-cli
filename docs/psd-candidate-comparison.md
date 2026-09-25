# M3 PSD Candidate Comparison Report v1

PR #15 consolidates the evidence produced by PRs #10–#14. It is a **comparison report**, not the PSD Engine Strategy decision.

Machine-readable source: `docs/data/psd-candidate-comparison-v1.json`.

## Scope

The current corpus contains 7 engine-neutral fixtures covering:

- simple PSD pixel layers;
- PSB;
- nested groups;
- duplicate Unicode layer names;
- text-layer classification;
- pixel and vector masks;
- malformed-input rejection.

All candidate results below have been reproduced on Ubuntu, macOS, and Windows.

## Conformance summary

| Candidate | Runtime | PSD | PSB | Groups | Duplicate names | Text | Masks | Malformed | Corpus |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | ---: |
| psd-tools 1.20.0 | Python 3.12 | pass | pass | pass | pass | pass | pass | pass | **7/7** |
| rawpsd 0.2.2 | Rust native | pass | fail | pass | pass | fail | fail | pass | **4/7** |
| ag-psd 31.0.2 | Node.js 22 | pass | pass | pass | pass | pass | pass | pass | **7/7** |

rawpsd's three frozen corpus-v1 failures are:

- `simple-pixel-layers-psb` — PSB is rejected;
- `text-layer` — the normalized text-layer count is unavailable;
- `layer-masks` — the committed mask fixture is rejected.

A passing corpus result means only that the candidate satisfies the current normalized observation contract. It does **not** prove rendering, layer export, round-trip safety, or production suitability.

## Runtime and distribution facts

| Candidate | Integration shape | YuTool core dependency? | License | Direct runtime/package facts |
| --- | --- | --- | --- | --- |
| rawpsd 0.2.2 | in-process Rust | no — spike crate only | CC0-1.0 | minimal Rust library; no required runtime process |
| psd-tools 1.20.0 | Python subprocess | no | MIT | Python >=3.10 upstream; spike pins 3.12; base package depends on typing-extensions, attrs, Pillow, NumPy |
| ag-psd 31.0.2 | Node subprocess | no | MIT | spike pins Node 22; package depends on base64-js and pako |

For image/rendering workflows, runtime cost can grow:

- psd-tools exposes optional `composite` dependencies including aggdraw, SciPy, and scikit-image;
- ag-psd can inspect structure without canvas, but Node bitmap/image workflows may require node-canvas;
- rawpsd exposes low-level/raw image data but does not provide a high-level compositor.

The project should keep these optional runtimes outside the YuTool core until the strategy decision explicitly chooses a distribution model.

## Upstream capability evidence vs YuTool-verified evidence

The distinction matters:

### psd-tools

Upstream documents:

- low-level PSD/PSB read and write;
- raw layer image export through NumPy/PIL;
- limited compositing;
- limited mutation/editing.

YuTool currently verifies only the corpus-v1 inspection primitives.

### rawpsd

Upstream describes rawpsd as a minimally processed PSD reader with exact layer hierarchy support. It explicitly states:

- PSD only, no PSB;
- 8-bit RGB/CMYK/Grayscale focus;
- compatibility line around Photoshop CS6-era features;
- no high-level rendering/editing layer.

That matches the current 4/7 corpus evidence.

### ag-psd

Upstream exposes:

- PSD object-tree reading;
- PSD writing;
- layer/mask image helpers;
- text, masks, vector-mask metadata;
- raw-data modes intended for safer structural processing.

The current README still contains a stale line saying PSB is unsupported, but upstream's changelog records PSB support from v12.1.0 and later PSB fixes. More importantly for this spike, ag-psd 31.0.2 passes the committed PSB fixture on all three CI platforms.

## Maintenance observation

This section records facts, not a maintenance score.

| Candidate | Repository | Latest commit observed for PR #15 |
| --- | --- | --- |
| psd-tools | `psd-tools/psd-tools` | 2026-09-25 — `f1256273...` |
| ag-psd | `Agamnentzar/ag-psd` | 2026-07-02 — `38704967...` |
| rawpsd | `wareya/rawpsd-rs` | 2025-05-11 — `dfc8c072...` |

A commit date alone is not a quality or sustainability rating. It is retained so the later engine-strategy ADR can distinguish current repository activity from parser capability.

## Performance and memory

**Not measured in PR #15.**

The durations currently emitted by `yu-psd-spike` are unsuitable for cross-engine benchmarking because:

- psd-tools and ag-psd include subprocess startup;
- CI runners vary by platform and load;
- the current fixtures are deliberately tiny;
- the harness does not isolate parse time, export time, rendering time, peak RSS, or allocations.

Using those values as a speed ranking would create false precision.

A meaningful benchmark should separately measure:

1. cold start;
2. warm structural parse;
3. layer-tree extraction;
4. representative layer export;
5. representative render/composite;
6. peak memory;
7. large PSD and PSB behavior.

## Comparison implications

The current evidence establishes several facts:

- rawpsd has the lowest runtime integration complexity because it is Rust-native, but the current pinned version does not meet the corpus-v1 structural baseline for PSB, text classification, and masks;
- psd-tools and ag-psd both satisfy the entire corpus-v1 normalized inspection contract;
- psd-tools and ag-psd both introduce an external runtime if used as YuTool execution engines;
- ag-psd has a smaller direct package dependency surface for the structural-only spike, while image workflows can introduce canvas requirements;
- psd-tools has explicit upstream APIs for raw layer export and compositing, while advanced compositing can add substantial optional dependencies.

These are **inputs** to the strategy decision, not a default-engine selection.

## What is still missing before Engine Strategy

The comparison is not decision-complete until the project adds representative evidence for:

- effects;
- Smart Object metadata;
- higher bit depth;
- layer export;
- rendering fidelity;
- round-trip/write safety;
- controlled parse/export/render benchmarks;
- controlled memory measurements;
- production packaging/update strategy for Python or Node engines.

## Decision posture

PR #15 intentionally records:

```text
decision_state = evidence_only
default_engine = not selected
benchmark = not measured
```

The next strategy step should either close the highest-value evidence gaps or explicitly decide which gaps are acceptable for the initial `yu psd inspect/tree/layer list` scope.
