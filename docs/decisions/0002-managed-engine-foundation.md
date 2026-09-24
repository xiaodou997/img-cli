# ADR 0002: Managed Engine Foundation

- **Status:** Accepted
- **Date:** 2026-09-24
- **Milestone:** M2

## Context

M1 proved the capability/engine model using built-in Rust engines.

M2 must support optional engines such as native applications or language-specific runtimes without making them default dependencies or silently modifying the host package manager.

Before download/install logic is added, YuTool needs a stable model for:

- engine package metadata;
- platform selection;
- private storage;
- integrity metadata;
- managed installation state;
- system executable discovery.

## Decision

### Managed engines use manifests

Managed engine releases are described by a versioned manifest.

Manifest v1 includes:

- engine ID;
- display name;
- engine version;
- declared YuTool capabilities;
- platform packages;
- package URL;
- SHA-256 digest;
- archive kind;
- relative entrypoint.

### YuTool owns only private managed storage

Managed engines are installed under a YuTool-owned data root.

YuTool does not treat Homebrew, apt, winget, or manually installed software as managed assets.

### System discovery remains read-only

A system engine can be discovered from `PATH`, but discovery never implies ownership.

### Installation state is derived

For a compatible managed package:

- missing version directory → `not_installed`;
- version directory with missing entrypoint → `broken`;
- expected entrypoint present → `ready`.

No matching platform package produces `incompatible`.

### M2 foundation performs no download

The first M2 PR intentionally does not fetch remote artifacts or extract archives.

Network download, SHA-256 verification, staging, extraction, and atomic activation are implemented as a separate security-sensitive step.

## Consequences

Positive:

- Engine Manager data model can be tested without network access;
- platform incompatibility is explicit;
- private ownership boundaries are clear;
- system and managed tools cannot be confused;
- later installer work has a defined manifest contract.

Costs:

- a catalog/distribution source still needs to be designed;
- package signing policy remains open;
- package URLs and artifact hosting remain unresolved;
- version probing for system engines is deferred.

## Follow-up

Next M2 work should add, in order:

1. staged download API;
2. SHA-256 verification;
3. safe archive extraction;
4. atomic version activation;
5. install/remove CLI;
6. system-engine version probing;
7. catalog/update policy.
