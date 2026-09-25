# Unified Engine Discovery

> Status: **M2 implemented**

YuTool exposes one inventory across three provider classes:

```text
Built-in + Managed + System
          ↓
   Engine Inventory
          ↓
yu engine list
yu engine info
yu doctor
```

## Built-in

Built-in entries come from the Rust runtime registry and remain immediately available.

## Managed

Managed discovery scans YuTool-owned storage under `engines/<id>/<version>` and reads `.yu-install.json`.

Inventory state:

- valid installed versions, no active version → `disabled`;
- valid active version → `ready`;
- stale active state, invalid metadata, missing active entrypoint, or incompatible installed target → `broken`.

Managed inventory reports installed versions, active version, active executable, persisted display name/capabilities, and discovery warnings.

## System

System discovery currently knows three read-only probes:

| Engine | Executable | Version probe |
| --- | --- | --- |
| ImageMagick | `magick` | `magick -version` |
| FFmpeg | `ffmpeg` | `ffmpeg -version` |
| ExifTool | `exiftool` | `exiftool -ver` |

Only executables actually found on `PATH` are included. Missing optional system tools do not appear in inventory and do not make `yu doctor` unhealthy.

Version probes have a bounded timeout and drain stdout/stderr so an abnormal external tool cannot block Doctor indefinitely.

System discovery is read-only. Discovery never gives YuTool ownership of the system package.

## ID collisions

A logical ID may appear more than once with different providers.

Example:

```text
imagemagick  managed  ready  7.1.2
imagemagick  system   ready  7.1.1
```

`yu engine info imagemagick` returns both records.

## JSON compatibility

`engine.list` keeps the original descriptor fields at the top level. Discovery adds optional fields rather than replacing the result with an incompatible nested shape.