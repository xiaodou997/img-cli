# M2 Freeze Receipt

> Status: **Frozen**

M2 establishes the Engine Manager and engine-discovery baseline for YuTool. Later milestones may add new capabilities and engine integrations, but should not casually break the contracts recorded here.

## Repository

- Repository: `xiaodou997/yu-tool`
- CLI executable: `yu`
- JSON schema version: `1`
- M1 frozen baseline: `c1edcb3d7e1da6527d66246e58b6d2bcc3455216`
- M2 functional baseline: `d95ccae8cdaefd4ea63a35d39d6ec803748036e8`
- Freeze PR: #9

- M2 Freeze PR: #9
- M2 documentation freeze commit: `f4f6f922a8aee741a24c9e413ba03b1fa641f96a`
- CI: Rustfmt + Ubuntu/macOS/Windows check, Clippy and tests passed.

M2 functionality itself remains pinned to `d95ccae`.

## M2 scope

M2 proves that YuTool can:

1. describe optional engines with a versioned manifest;
2. install verified engine packages into YuTool-owned storage;
3. keep multiple managed versions;
4. explicitly activate/deactivate one managed version;
5. safely remove inactive managed versions;
6. serialize concurrent mutations per engine;
7. discover Built-in, Managed, and System providers through one inventory;
8. expose the lifecycle through the public `yu` CLI;
9. include engine health in `yu doctor`.

M2 does **not** yet make every discovered external engine executable through the capability resolver.

## Crate boundary

The M2 runtime includes:

```text
crates/
├── yu-cli
├── yu-core
├── yu-engine-api
├── yu-engine-manager
├── yu-capability-image
└── yu-engine-image-rs
```

Relevant responsibilities:

- `yu-cli` — CLI parsing, human output, schema-v1 rendering;
- `yu-core` — protocol, capability registry, runtime registry/resolver, Doctor summary;
- `yu-engine-api` — common engine provider/state/descriptor types;
- `yu-engine-manager` — manifests, managed storage, verified install, lifecycle, locking, unified discovery;
- `yu-capability-image` / `yu-engine-image-rs` — first built-in capability/engine used to prove runtime execution.

## Frozen public Engine CLI

M2 freezes the following command shapes:

```bash
yu doctor
yu capabilities

yu engine list
yu engine info <engine>

yu engine install --manifest <file>
yu engine versions <engine>
yu engine activate <engine> <version>
yu engine deactivate <engine>
yu engine remove <engine> <version>
```

All automation-oriented forms continue to support global `--json`.

### Install remains manifest-driven

M2 deliberately freezes installation as:

```bash
yu engine install --manifest ./engine.json
```

It does **not** claim the future Catalog syntax:

```bash
yu engine install imagemagick
```

A future Engine Catalog may add that form without redefining the meaning of `--manifest`.

### Install and activation are separate

A successful install does not silently change the active version.

The caller explicitly runs:

```bash
yu engine activate <engine> <version>
```

This is an M2 lifecycle invariant.

## Managed Engine Manifest v1

Manifest schema:

```json
{
  "schema_version": "1",
  "id": "example-engine",
  "display_name": "Example Engine",
  "version": "1.2.3",
  "capabilities": ["example.run"],
  "packages": [
    {
      "target": {
        "os": "macos",
        "arch": "aarch64"
      },
      "url": "https://example.invalid/engine.zip",
      "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "archive": "zip",
      "entrypoint": "bin/engine"
    }
  ]
}
```

M2 manifest invariants:

- schema version is `1`;
- package target is exact OS/architecture;
- download URL must use HTTPS;
- SHA-256 is mandatory;
- engine ID/version/entrypoint must be safe path components;
- duplicate packages for the same target are rejected.

Supported package kinds:

- `raw`
- `zip`
- `tar_gz`

## Managed storage

YuTool owns its private Engine Manager data root.

Conceptual layout:

```text
<yu-data>/
├── engines/
│   └── <engine-id>/
│       └── <version>/
│           ├── .yu-install.json
│           └── ...
├── state/
│   ├── engines/
│   │   └── <engine-id>.json
│   └── locks/
│       └── <engine-id>.lock
└── cache/
    ├── staging/
    └── trash/
```

Default roots:

- macOS: `~/Library/Application Support/YuTool`
- Linux: `$XDG_DATA_HOME/yu-tool` or `~/.local/share/yu-tool`
- Windows: `%LOCALAPPDATA%\YuTool`

`YU_DATA_HOME` overrides the root.

YuTool does not silently take ownership of packages installed by Homebrew, apt, winget, pip, npm, or another system manager.

## Verified installation pipeline

M2 freezes this ordering:

```text
validate manifest
       ↓
select exact target
       ↓
download into private staging
       ↓
verify SHA-256
       ↓
safe extraction
       ↓
validate entrypoint
       ↓
write .yu-install.json
       ↓
atomic directory activation
       ↓
engines/<id>/<version>
```

No unverified artifact is activated.

Initial safety limits:

- download: 512 MiB;
- extracted bytes: 2 GiB;
- extracted regular files: 20,000.

Archive safety rejects:

- absolute/traversal paths;
- ZIP symlinks;
- TAR symlinks/hardlinks/devices/special entries;
- packages exceeding configured limits.

Existing installed versions are never silently overwritten.

## Managed lifecycle

Multiple versions may coexist.

Example:

```text
example-engine/
├── 1.0.0
├── 1.1.0  [active]
└── 2.0.0
```

Lifecycle invariants:

- zero or one active version;
- active state is persisted separately from installed versions;
- switching active versions is explicit;
- active version removal is rejected;
- no implicit fallback is selected after removal;
- Built-in/System providers are read-only to managed lifecycle mutation;
- version directories, install metadata, and entrypoints are rejected when replaced by symlinks.

Active-state replacement is atomic on supported platforms:

- Unix: rename replacement;
- Windows: `MoveFileExW` with replace/write-through semantics.

Removal quarantines an inactive version into YuTool-owned trash before recursive cleanup.

## Inter-process mutation lock

The following operations acquire a per-engine exclusive lock:

- install;
- activate;
- deactivate;
- remove.

Lock path:

```text
<yu-data>/state/locks/<engine-id>.lock
```

A conflicting mutation fails immediately and maps to the frozen `OUTPUT_CONFLICT` protocol error.

Different Engine IDs may mutate concurrently.

Read-only discovery does not take the mutation lock.

## Unified Engine Inventory

M2 freezes one discovery model:

```text
Built-in + Managed + System
          ↓
   Engine Inventory
          ↓
yu engine list
yu engine info
yu doctor
```

### Built-in

Built-in entries come from the runtime registry.

### Managed

Managed entries are reconstructed from YuTool-owned storage and persisted `.yu-install.json`.

State semantics:

- valid versions but no active version → `disabled`;
- valid active version → `ready`;
- stale/corrupt metadata/state/entrypoint → `broken`.

### System

M2 includes read-only System probes for:

| Engine | Executable | Version command |
| --- | --- | --- |
| ImageMagick | `magick` | `magick -version` |
| FFmpeg | `ffmpeg` | `ffmpeg -version` |
| ExifTool | `exiftool` | `exiftool -ver` |

Only actually discovered executables are included.

Missing optional System tools do not make Doctor unhealthy.

Version probes are bounded by timeout and captured-output limits.

### Provider collisions

The same logical ID may exist under multiple providers.

Example:

```text
imagemagick  managed  ready  7.1.2
imagemagick  system   ready  7.1.1
```

Discovery keeps both records.

## JSON compatibility

M2 does not introduce a new protocol version.

The frozen envelope remains schema `1`.

`engine.list --json` preserves these original top-level descriptor fields:

- `id`
- `display_name`
- `provider`
- `state`
- `version`
- `capabilities`

Discovery adds optional fields:

- `executable`
- `installed_versions`
- `active_version`
- `warnings`

This is an additive evolution of schema v1.

The M1 error-code set remains unchanged:

- `INVALID_ARGUMENT`
- `INVALID_INPUT`
- `UNSUPPORTED_CAPABILITY`
- `ENGINE_UNAVAILABLE`
- `ENGINE_INCOMPATIBLE`
- `EXECUTION_FAILED`
- `OUTPUT_CONFLICT`
- `VERIFICATION_FAILED`

## Doctor semantics

`yu doctor` uses the same unified inventory as `yu engine list`.

Doctor reports:

- total engines;
- ready engines;
- Built-in count;
- Managed count;
- System count;
- unhealthy count.

A valid inactive Managed engine (`disabled`) is not a health failure.

A missing optional System tool is not represented and therefore is not a health failure.

Broken/incompatible discovered providers degrade health and surface warnings.

## CI gate

M2 Freeze requires PR #9 to pass:

- Rustfmt;
- Ubuntu: check + Clippy `-D warnings` + tests;
- macOS: check + Clippy `-D warnings` + tests;
- Windows: check + Clippy `-D warnings` + tests.

The M2 functional baseline `d95ccae` already passed the same cross-platform gate in PR #8.

## Intentionally deferred to M3+

M2 does not freeze or promise:

- PSD/PSB engine choice;
- execution of discovered ImageMagick/FFmpeg/ExifTool through capability routing;
- Engine Catalog / registry hosting;
- `yu engine update` policy;
- signatures beyond manifest SHA-256;
- automatic package-manager installation;
- third-party plugin ABI/process protocol;
- image crop/rotate/convert;
- PSD mutation;
- YuTool Manager GUI.

The next product milestone is M3: PSD Engine Spike.
