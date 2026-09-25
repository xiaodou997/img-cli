# Engine Management CLI

PR #7 exposes the M2 managed-engine lifecycle through `yu`.

## Install

```bash
yu engine install --manifest ./engine.json
```

Until an Engine Catalog exists, installation is manifest-driven. The manifest path is an explicit flag so future catalog syntax can add `yu engine install <engine-id>` without overloading a path argument.

Install verifies and stores the version but does not activate it.

## Versions

```bash
yu engine versions imagemagick
```

## Activate / deactivate

```bash
yu engine activate imagemagick 7.1.2
yu engine deactivate imagemagick
```

## Remove

```bash
yu engine remove imagemagick 7.1.1
```

An active version must be switched or deactivated before removal.

## Concurrency

Install, activate, deactivate, and remove acquire the per-engine mutation lock. Concurrent mutation of the same engine returns `OUTPUT_CONFLICT`.