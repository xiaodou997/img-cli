# ADR 0005: Unified Engine Inventory

- **Status:** Accepted
- **Date:** 2026-09-25
- **Milestone:** M2

## Context

After PRs #4–#7, YuTool could manage installed engines and probe system executables, but `yu engine list` and `yu doctor` still saw only the static Built-in registry.

That split makes Agent capability discovery unreliable and hides situations where a Managed and System implementation coexist.

## Decision

YuTool will build one runtime inventory from Built-in descriptors, YuTool-managed storage, and known System probes.

### Provider records remain distinct

Two providers with the same logical engine ID are retained as two records. YuTool does not silently prefer one in discovery output.

### Managed inactive is not broken

A correctly installed Managed engine with no active version is `disabled`, not `broken`.

### Missing optional System engines are absent

Known System probe definitions do not create `not_installed` rows when no executable exists on PATH. This keeps optional tools from degrading Doctor.

### System probing is bounded

Version commands use a timeout and bounded captured output.

### JSON evolves additively

Existing `engine.list` descriptor fields stay top-level; discovery metadata is added as optional fields.

## Consequences

- `engine list`, `engine info`, and `doctor` share the same source of truth;
- Managed/System coexistence becomes visible;
- Doctor can identify broken installed/probed engines;
- discovery does not yet make external engines executable through the capability resolver; integration remains a separate decision.