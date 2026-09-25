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

## Candidate skeletons

PR #10 registers three intentionally unwired candidates:

| Candidate ID | Runtime family | PR #10 behavior |
| --- | --- | --- |
| `rust-native` | Rust | skipped / unavailable |
| `psd-tools` | Python | skipped / unavailable |
| `typescript-psd` | TypeScript/Node | skipped / unavailable |

An unwired adapter returns an explicit `unavailable` result. It is never treated as a passing implementation and none of the three candidates becomes the default by being listed first.

## Commands

Validate the committed corpus:

```bash
cargo run -p yu-psd-spike -- validate fixtures/psd/corpus.json
```

List candidate descriptors:

```bash
cargo run -p yu-psd-spike -- candidates
```

Produce a report for one candidate:

```bash
cargo run -p yu-psd-spike -- run psd-tools fixtures/psd/corpus.json
```

Until a candidate adapter is wired, its fixture results are `skipped` with an explicit diagnostic.

## CI contract

The normal workspace gate is sufficient for PR #10:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Tests cover corpus validation, expected-rejection semantics, path traversal rejection, and the fact that all PR #10 candidates remain explicit skeletons.

## Follow-up direction

Candidate integrations should land independently and use this same harness. A candidate PR should:

1. wire exactly one adapter;
2. declare runtime/distribution requirements;
3. add only fixture expectations needed for evidence;
4. avoid changing another candidate to make its comparison easier;
5. produce the same report schema.

The M3 engine recommendation comes after representative corpus coverage and candidate runs, not from PR #10.
