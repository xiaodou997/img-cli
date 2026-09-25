# Managed Engine Lifecycle

> Status: **M2 internal lifecycle**

YuTool-managed engines may have multiple versions installed at the same time.

Installation, activation, and removal are deliberately separate operations.

## Storage model

Installed versions remain immutable by version:

\`\`\`text
<yu-data>/
├── engines/
│   └── imagemagick/
│       ├── 7.1.1/
│       └── 7.1.2/
│
├── state/
│   └── engines/
│       └── imagemagick.json
│
└── cache/
    └── trash/
\`\`\`

Each installed version contains YuTool installation metadata:

\`\`\`text
engines/<id>/<version>/.yu-install.json
\`\`\`

The metadata records:

- engine ID;
- version;
- target OS/architecture;
- relative entrypoint;
- verified artifact SHA-256.

## Install does not imply activate

Installing a version does not silently change the active version.

This avoids surprising an automation workflow merely because a newer engine was downloaded.

A future CLI may choose to compose:

\`\`\`text
install
→ explicit activate
\`\`\`

but the lifecycle layer keeps the operations separate.

## Active version

The active version is stored in:

\`\`\`text
state/engines/<engine-id>.json
\`\`\`

Example:

\`\`\`json
{
  "schema_version": "1",
  "engine_id": "imagemagick",
  "active_version": "7.1.2"
}
\`\`\`

Switching active versions uses a staged state file followed by an atomic replacement:

- Unix: filesystem rename replacement;
- Windows: \`MoveFileExW\` with replace + write-through flags.

Activation validates the installed metadata and entrypoint before changing current state.

## Multi-version behavior

Multiple versions can coexist:

\`\`\`text
7.1.1  installed
7.1.2  installed + active
7.1.3  installed
\`\`\`

Version discovery reports which version is active.

YuTool does not infer version precedence from folder names.

## Removal policy

Only **Managed** engines belong to this lifecycle.

Built-in and System engines are read-only from the managed lifecycle API.

### Active version

Removing the active version is rejected.

The caller must first:

1. activate another installed version, or
2. explicitly deactivate the engine.

This prevents a removal operation from silently choosing a fallback version.

### Inactive version

Removal first atomically renames the version directory into YuTool's private trash area.

After quarantine, physical cleanup is attempted.

The removal receipt reports whether cleanup completed. If deletion fails, the engine is already absent from the active engines directory and the quarantined path can be cleaned later.

## Symlink boundary

Lifecycle operations reject managed version directories, metadata files, and entrypoints that have been replaced with symbolic links.

This avoids turning YuTool's remove/activate operations into traversal primitives if local managed storage has been tampered with.

## Public CLI

This lifecycle is internal in PR #6.

The following remain deferred until PR #7:

\`\`\`bash
yu engine install
yu engine activate
yu engine deactivate
yu engine remove
\`\`\`

Before those commands are exposed, the CLI layer will map lifecycle receipts/errors into the frozen YuTool JSON protocol.
