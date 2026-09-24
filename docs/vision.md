# Vision

## Summary

**YuTool is a lightweight, extensible local tool runtime for developers, automation, and AI agents.**

Its public command is \`yu\`.

YuTool provides one discoverable interface over local capabilities while keeping individual implementations replaceable.

## The problem

Modern development and automation workflows rely on many excellent tools, but those tools expose different:

- installation methods;
- command syntaxes;
- output formats;
- error models;
- runtime dependencies;
- platform behavior;
- capability boundaries.

Humans can learn these differences. Coding agents often can too, but repeatedly rediscovering environment state and backend-specific syntax is inefficient and error-prone.

YuTool addresses the orchestration layer rather than attempting to replace every mature tool.

## Mission

Provide a small local runtime that can answer four questions reliably:

1. **What can this machine do?**
2. **Which engine can perform this operation?**
3. **How can the operation be invoked through one stable interface?**
4. **What happened, in a form both humans and software can understand?**

## Core principles

### Lightweight

The default installation should remain useful without bundling every possible engine.

Advanced engines are installed only when needed.

### Unified

A capability should feel consistent regardless of whether its implementation is Rust-native, Python-based, Node-based, or an external native executable.

### Extensible

YuTool should allow new capabilities and engines without requiring a redesign of the CLI core.

### Agent-friendly

Automation is a primary interface.

Important commands should provide:

- JSON output;
- capability discovery;
- engine discovery;
- stable error codes;
- predictable exit status;
- non-destructive defaults;
- dry-run support where useful.

### Local-first

YuTool is primarily a local runtime. Files should not leave the machine unless a future capability explicitly and transparently requires remote execution.

### Honest capability boundaries

YuTool must distinguish:

- supported;
- partially supported;
- experimental;
- unavailable.

An integration being technically possible does not make it a supported capability.

## What YuTool is not

YuTool is not:

- a Photoshop replacement;
- an ImageMagick replacement;
- an FFmpeg replacement;
- a general-purpose GUI editor;
- a promise to bundle every supported backend;
- an AI model or autonomous agent by itself.

It is the stable layer between callers and heterogeneous local tools.

## Initial domain

The first domain is image and layered-image automation.

This domain is intentionally chosen because it exercises:

- built-in Rust processing;
- external/native engines;
- Python engines;
- file inspection;
- rendering;
- structured document trees;
- capability fallback;
- cross-engine verification.

The initial milestone should prove the runtime before the project expands horizontally.

## Future domains

Potential future capability families include:

- PDF;
- video and audio;
- metadata;
- OCR;
- archives;
- document conversion;
- vector graphics;
- RAW media.

These are future directions, not v0.1 commitments.

## Brand

**Yu** comes from **羽** — feather.

The product metaphor is a lightweight core whose capabilities can be attached as needed.

> **Small core. Flexible engines. One interface.**
