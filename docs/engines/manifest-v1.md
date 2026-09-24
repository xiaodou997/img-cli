# Managed Engine Manifest v1

> Status: **M2 foundation draft**

YuTool-managed engines are described by a manifest. The manifest answers:

- what engine/version is being installed;
- which YuTool capabilities it provides;
- which platform package applies;
- where the package comes from;
- how the artifact is verified;
- which relative entrypoint becomes executable after installation.

## Schema version

```json
{
  "schema_version": "1"
}
```

M2 starts with schema version `1`.

## Example

```json
{
  "schema_version": "1",
  "id": "imagemagick",
  "display_name": "ImageMagick",
  "version": "7.x.y",
  "capabilities": [
    "image.advanced"
  ],
  "packages": [
    {
      "target": {
        "os": "macos",
        "arch": "aarch64"
      },
      "url": "https://example.invalid/imagemagick-macos-aarch64.zip",
      "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "archive": "zip",
      "entrypoint": "bin/magick"
    }
  ]
}
```

The URL above is illustrative only. No ImageMagick package source is selected by this document.

## Engine ID

`id` is a stable machine identifier.

Allowed characters:

```text
a-z
0-9
.
_
-
```

Examples:

```text
imagemagick
psd-tools
libvips
ffmpeg
```

## Targets

A package target consists of:

```json
{
  "os": "macos",
  "arch": "aarch64"
}
```

YuTool initially compares exact Rust platform strings from `std::env::consts`.

Common examples:

| OS | Architecture |
| --- | --- |
| `macos` | `aarch64` |
| `macos` | `x86_64` |
| `linux` | `x86_64` |
| `linux` | `aarch64` |
| `windows` | `x86_64` |

A manifest must not contain duplicate packages for the same target.

## Verification

Every managed package requires a SHA-256 digest.

M2 foundation validates the digest format. Actual download hashing/verification is implemented in a later Engine Manager PR before installation is enabled.

## Archive kinds

Manifest v1 reserves:

- `raw`
- `zip`
- `tar_gz`

Support in the schema does not imply that extraction is already implemented.

## Entrypoint safety

`entrypoint` must be a relative path inside the installed engine version directory.

Rejected examples:

```text
/opt/tool/bin/tool
../escape
../../tool
```

This prevents a manifest from defining an entrypoint outside YuTool-managed storage.

## Managed storage

YuTool owns only its private managed-engine storage.

Conceptually:

```text
<yu-data>/
├── engines/
│   └── <engine-id>/
│       └── <version>/
│           └── ...
├── cache/
└── state/
```

Default roots:

- macOS: `~/Library/Application Support/YuTool`
- Linux: `$XDG_DATA_HOME/yu-tool`, otherwise `~/.local/share/yu-tool`
- Windows: `%LOCALAPPDATA%\YuTool`

`YU_DATA_HOME` overrides the default root on every platform.

## Managed engine state

Given a valid manifest and current target:

- no matching target package → `incompatible`
- version directory absent → `not_installed`
- version directory exists but entrypoint is missing → `broken`
- expected entrypoint exists → `ready`

These states reuse YuTool's existing M1 `EngineState` model.

## System engines

System engines are separate from managed engines.

YuTool may probe `PATH` for known executable names, but discovering a binary does not transfer ownership to YuTool.

System packages must never be deleted by `yu engine remove`.
