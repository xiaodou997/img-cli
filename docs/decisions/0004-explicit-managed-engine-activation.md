# ADR 0004: Explicit Managed Engine Activation

- **Status:** Accepted
- **Date:** 2026-09-25
- **Milestone:** M2

## Context

After verified installation, YuTool needs a policy for multiple installed engine versions.

Automatically switching to the newest installed version would make an install operation change runtime behavior implicitly. Automatically falling back when removing an active version creates similar ambiguity.

## Decision

### Install and activate are separate

A successfully installed version is available but does not become active automatically.

### Active state is explicit and persistent

Each managed engine can have zero or one active version.

The state is stored in YuTool-owned lifecycle state and replaced atomically.

### No implicit fallback

Removing an active version is rejected.

The caller must activate another version or explicitly deactivate first.

### Managed ownership is strict

Lifecycle mutation accepts only descriptors with provider \`managed\`.

Built-in and System engines are never activated, deactivated, or removed through the managed lifecycle.

### Installed metadata is persisted per version

Each version records verified installation metadata in \`.yu-install.json\`.

Activation uses that metadata to verify the target and entrypoint before switching current state.

### Removal quarantines first

An inactive version is renamed into YuTool's trash area before recursive cleanup.

This removes it from the active engine namespace before potentially fallible physical deletion.

## Consequences

Positive:

- installing a version has no hidden behavior change;
- rollback is simply activation of an older installed version;
- removal never invents a fallback;
- System engines cannot be accidentally deleted;
- lifecycle state remains machine-readable and recoverable.

Costs:

- callers must perform an explicit activation step;
- stale/corrupt state becomes a diagnosable error rather than silently repairing itself;
- public CLI needs separate activate/deactivate commands or an explicit composed install policy.

## Follow-up

PR #7 should expose lifecycle operations through the CLI and decide the human-facing convenience behavior of \`yu engine install\`, while keeping the underlying lifecycle explicit.
