# Managed Engine Installation Pipeline

> Status: **M2 internal installer**

This document describes the internal installation contract. The public `yu engine install` command is intentionally not enabled until this pipeline is proven.

## Goals

The installer must ensure that:

1. unverified bytes never become an active engine;
2. archive contents cannot escape YuTool's staging directory;
3. failed installs do not leave an apparently-ready engine version;
4. an existing engine version is never silently overwritten;
5. tests can validate the installer without internet access.

## Pipeline

```text
EngineManifest
      │
      ▼
validate manifest
      │
      ▼
select exact target package
      │
      ▼
create private staging directory
      │
      ▼
download artifact
      │
      ▼
SHA-256 verification
      │
      ▼
safe extraction
      │
      ▼
entrypoint exists?
      │
      ▼
prepare executable
      │
      ▼
atomic rename
      │
      ▼
engines/<id>/<version>
```

Every failure before the final rename removes the staging directory on a best-effort basis.

## Download abstraction

The installer depends on a `Downloader` trait.

Production uses `HttpDownloader`.

Tests use an in-memory/fixture downloader. CI therefore does not depend on remote servers.

## Default resource limits

Initial defaults:

- download: 512 MiB;
- extracted content: 2 GiB;
- extracted regular files: 20,000.

These are safety limits, not product promises. They can evolve before a stable release.

## Supported package kinds

### raw

The downloaded artifact itself becomes the configured relative entrypoint.

### zip

Entries are extracted individually after safe-path checks.

Symlink entries are rejected.

### tar_gz

Only regular files and directories are accepted.

Symlinks, hardlinks, devices, and other special TAR entry types are rejected.

## Activation

Activation uses a rename from:

```text
<yu-data>/cache/staging/.../payload
```

to:

```text
<yu-data>/engines/<engine-id>/<version>
```

Both locations are inside the same YuTool data root so the normal case stays on one filesystem and can use an atomic directory rename.

If the destination version already exists, installation stops with `AlreadyInstalled`; YuTool never replaces that version implicitly.

## Deferred public surface

This PR does not expose:

```bash
yu engine install
yu engine remove
yu engine update
```

Those commands should be added only after the installer and remover lifecycle are both covered by integration tests.
