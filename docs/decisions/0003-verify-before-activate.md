# ADR 0003: Verify Before Activate

- **Status:** Accepted
- **Date:** 2026-09-25
- **Milestone:** M2

## Context

YuTool-managed engines will eventually download and execute third-party binaries or runtimes. This creates a stronger trust boundary than built-in engines.

A partial or malicious package must not be able to:

- become visible as a ready engine before verification;
- write outside YuTool-managed storage;
- replace an existing version unexpectedly;
- depend on network access during CI tests.

## Decision

Managed installation uses a staging pipeline.

### Download into private staging

Artifacts are downloaded below YuTool's cache directory, never directly into the active engine directory.

### Require HTTPS

Manifest package URLs must use `https://`.

### Verify SHA-256 before extraction

The downloaded artifact is hashed and compared to the manifest digest before any archive is unpacked.

### Extract defensively

Archive extraction rejects traversal paths and special link/device entries.

Extraction is limited by total bytes and file count.

### Validate entrypoint before activation

The configured entrypoint must exist as a regular file inside the staged payload.

### Activate by directory rename

Only a complete staged payload is renamed to the final version directory.

An existing version directory is never overwritten.

### Keep download injectable

The installer depends on a download trait so CI can use deterministic fixture bytes rather than remote network calls.

## Consequences

Positive:

- unverified bytes never become active;
- failure leaves no ready engine version;
- archive traversal defenses are testable;
- installation behavior is deterministic in CI;
- later CLI work can reuse a proven lifecycle.

Costs:

- more code and dependencies in `yu-engine-manager`;
- archive formats must be handled deliberately;
- package publishers must provide SHA-256 values;
- repair/update policy still requires separate design.

## Follow-up

Next work:

1. managed remove lifecycle;
2. active-version/current-pointer policy if multiple versions are retained;
3. expose install/remove through the CLI;
4. catalog/source policy;
5. signature strategy beyond SHA-256 when distribution infrastructure is defined.
