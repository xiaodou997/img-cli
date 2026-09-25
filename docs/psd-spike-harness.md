# M3 PSD Engine Spike Harness

PR #10 establishes the test harness used to compare PSD/PSB candidates. It does **not** select a PSD engine and it does not add a public `yu psd` command.

## Goals

The harness provides one neutral place to define:

1. the fixture corpus and its provenance;
2. expected behavior for each fixture;
3. candidate adapter boundaries;
4. per-fixture conformance results;
5. a machine-readable report that can later feed the M3 comparison report.

This keeps candidate evaluation separate from the M2 Engine Manager baseline.

## Workspace component

```text
crates/yu-psd-spike/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── main.rs

fixtures/psd/
├── README.md
├── corpus.json
└── malformed/
    └── truncated-header.psd
```

`yu-psd-spike` is a development/evaluation tool. Its command shape is not part of the frozen public `yu` CLI contract.

## Corpus schema v1

Each fixture records:

- stable fixture ID;
- relative file path;
- PSD or PSB format;
- feature tags;
- expected parse result;
- optional expected width, height, layer count, and minimum tree depth;
- provenance, optional license metadata, and redistributability.

The loader rejects:

- unsupported corpus schema versions;
- empty corpora;
- duplicate fixture IDs;
- fixtures without feature tags;
- absolute paths and parent-directory traversal;
- format/extension mismatches;
- missing fixture files.

PR #11 expands the corpus to seven fixtures: PSD and PSB pixel-layer baselines, a nested group, duplicate layer names, a type layer, mask coverage, and the original malformed input. Third-party fixtures are pinned to `psd-tools/psd-tools` commit `f1256273ffb9b39b9efa19d486c86f594c831f42` with the upstream MIT notice committed alongside the corpus.

## Candidate adapter contract

Every candidate implements the same internal adapter boundary:

```text
descriptor()
inspect(input, fixture) -> observation | adapter error
```

The observation currently carries the comparison primitives required for the first spike:

- parse success;
- width/height;
- logical layer count;
- maximum tree depth;
- layer-name multiset;
- text-layer count;
- pixel-mask layer count;
- vector-mask layer count.

The contract can be extended as the corpus starts testing masks, text, Smart Objects, export, and rendering. New fields should be driven by actual comparison needs rather than one candidate's native API.

## Candidate status

PR #10 registered three candidate slots. PR #12 wired the psd-tools reference, PR #13 wired rawpsd, and PR #14 wires ag-psd:

| Candidate ID | Runtime family | Current behavior |
| --- | --- | --- |
| `rust-native` | Rust | rawpsd 0.2.2 candidate |
| `psd-tools` | Python | wired reference adapter |
| `typescript-psd` | TypeScript/Node | ag-psd 31.0.2 candidate |

All three M3 candidate slots are now wired. A wired candidate is evidence for comparison, not a default-engine selection.

### rawpsd Rust-native candidate

PR #13 pins:

```text
rawpsd 0.2.2
```

The dependency is scoped to the development-only `yu-psd-spike` crate. It is not added to YuTool's production runtime.

Normalization currently uses rawpsd's PSD metadata and raw layer records to report:

- parse success/rejection;
- width and height;
- logical layer count;
- group-derived maximum tree depth;
- Unicode-aware layer names exposed by rawpsd;
- pixel-mask presence derived from mask channels.

Known API/capability gaps are intentionally represented as missing observations rather than guessed values:

- PSB is not supported by rawpsd 0.2.2;
- normalized text-layer classification is not exposed;
- normalized vector-mask classification is not exposed.

These gaps are expected to appear as fixture failures in the comparison report. They are not converted into harness errors.

The first corpus-v1 run establishes this rawpsd baseline:

| Fixture | rawpsd 0.2.2 |
| --- | --- |
| `simple-pixel-layers-psd` | pass |
| `simple-pixel-layers-psb` | fail — PSB rejected |
| `nested-group` | pass |
| `duplicate-layer-names` | pass |
| `text-layer` | fail — normalized text-layer count unavailable |
| `layer-masks` | fail — fixture rejected by parser |
| `malformed-truncated-header` | pass — correctly rejected |

Summary:

```text
passed:  4
failed:  3
skipped: 0
errors:  0
```

This baseline is intentionally asserted by the spike tests so future rawpsd upgrades cannot silently change the comparison result.

### ag-psd TypeScript/Node candidate

PR #14 pins:

```text
Node.js 22
ag-psd 31.0.2
```

The published ag-psd package is installed only inside `crates/yu-psd-spike/adapters/typescript/` by the dedicated PSD Spike workflow. Normal YuTool runtime and normal Rust CI do not install Node modules.

The adapter runs `adapters/typescript/ag_psd_candidate.cjs` through argv without shell interpolation and normalizes ag-psd's document tree into:

- parse success/rejection;
- width and height;
- logical layer count;
- maximum tree depth;
- layer names;
- text-layer count;
- bitmap-mask layer count;
- vector-mask layer count.

For bitmap masks, an ag-psd `mask` or `realMask` whose `fromVectorData` flag is not true counts as a bitmap mask. A `vectorMask` counts independently.

The adapter refuses a different Node major version or ag-psd package version so comparison runs cannot silently drift.

The first corpus-v1 run establishes this ag-psd baseline:

| Fixture | ag-psd 31.0.2 |
| --- | --- |
| `simple-pixel-layers-psd` | pass |
| `simple-pixel-layers-psb` | pass |
| `nested-group` | pass |
| `duplicate-layer-names` | pass |
| `text-layer` | pass |
| `layer-masks` | pass |
| `malformed-truncated-header` | pass |

Summary:

```text
passed:  7
failed:  0
skipped: 0
errors:  0
```

This 7/7 baseline is asserted by the spike test. The current ag-psd README still contains an outdated PSB limitation line, but the library code and changelog include Large Document support and the committed PSB fixture passes the pinned 31.0.2 candidate.

### psd-tools reference runtime

The reference environment is intentionally pinned:

```text
Python 3.12
psd-tools 1.20.0
```

The Rust harness invokes `crates/yu-psd-spike/adapters/psd_tools_reference.py` through argv without shell interpolation. The adapter refuses a different Python minor version or psd-tools version so comparison runs do not silently drift.

The interpreter defaults to `python`. Set `YU_PSD_TOOLS_PYTHON` to an explicit Python executable when needed.

YuTool does not install this Python environment during normal runtime or normal Rust CI. The dedicated `PSD Spike` workflow creates the pinned reference environment only for conformance testing and runs it on Ubuntu, macOS, and Windows.

Adapter normalization currently maps psd-tools into:

- parse success/rejection;
- document width and height;
- logical layer count;
- maximum tree depth;
- layer names;
- text-layer count;
- pixel-mask count;
- vector-mask count.

A PSD parse exception is represented as `parse_success=false`; missing Python, missing psd-tools, or version mismatch is `unavailable`; process/protocol failures are `error`.

## Commands

Validate the committed corpus:

```bash
cargo run -p yu-psd-spike -- validate fixtures/psd/corpus.json
```

List candidate descriptors:

```bash
cargo run -p yu-psd-spike -- candidates
```

Validate and print the frozen candidate comparison snapshot:

```bash
cargo run -p yu-psd-spike -- comparison docs/data/psd-candidate-comparison-v1.json
```

Produce a report for one candidate:

```bash
cargo run -p yu-psd-spike -- run psd-tools fixtures/psd/corpus.json
```

For `psd-tools`, the command runs the pinned reference adapter when its Python environment is available. If the pinned runtime is absent or mismatched, fixture results are `skipped` with an explicit `unavailable` diagnostic.

For `rust-native`, the command runs rawpsd directly in-process:

```bash
cargo run -p yu-psd-spike -- run rust-native fixtures/psd/corpus.json
```

For `typescript-psd`, install the pinned adapter package first and run:

```bash
npm install --prefix crates/yu-psd-spike/adapters/typescript --ignore-scripts --no-audit --no-fund
cargo run -p yu-psd-spike -- run typescript-psd fixtures/psd/corpus.json
```

Set `YU_TYPESCRIPT_PSD_NODE` when an explicit Node executable is required.

## CI contract

The normal workspace gate remains independent of optional PSD runtimes:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

PR #12 adds the three-platform psd-tools reference gate. PR #13 adds the rawpsd candidate gate. PR #14 adds a third Ubuntu/macOS/Windows job for Node.js 22 + ag-psd 31.0.2.

Candidate jobs require adapter execution without harness errors. A candidate is not required to reach 7/7 unless its observed capability actually matches the corpus.

Normal tests cover corpus validation, expected comparison semantics, path traversal rejection, all three wired descriptors, graceful `unavailable` behavior for missing external runtimes, and rawpsd candidate execution.

## Candidate comparison snapshot

PR #15 adds:

- `docs/data/psd-candidate-comparison-v1.json` — machine-readable frozen evidence;
- `docs/psd-candidate-comparison.md` — human-readable comparison report.

The snapshot is validated by Rust tests against:

- all three registered candidate IDs;
- pinned candidate versions;
- the 7-fixture corpus count;
- psd-tools 7/7 baseline;
- rawpsd 4/7 baseline and exact failed fixture set;
- ag-psd 7/7 baseline;
- `decision_state = evidence_only`;
- `benchmark.status = not_measured`.

A change to the machine-readable comparison baseline triggers the PSD Spike workflow so all candidate evidence can be reproduced before the snapshot moves.

## Follow-up direction

PR #15 closes the first conformance/distribution comparison, but it does not make the engine decision. The highest-value remaining evidence gaps are:

1. effects and Smart Object metadata fixtures;
2. higher-bit-depth coverage;
3. layer export conformance;
4. rendering fidelity;
5. round-trip/write safety;
6. controlled parse/export/render benchmarks;
7. controlled memory measurements;
8. production packaging/update strategy for optional Python or Node runtimes.

The M3 engine strategy should explicitly state which of these gaps must be closed for the initial `inspect/tree/layer list` scope and which can be deferred.
