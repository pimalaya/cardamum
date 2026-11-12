---
cairn: change
id: release-polish
status: landed
created: 2026-11-12
---

# Release polish

## Why

Last change of the-family-standards (Ortie's id, comodoro and calendula using it too): the documents a release reader meets say what the code now does.

## What

- MIGRATION.md, new: from 0.3 to the next release, the configuration and first run, the text and JSON outputs, the commands failing earlier, what is new.
- The src/main.rs header names the session, the REPL, the MCP server, the capability tables and the plain text.
- CONTRIBUTING.md: io-gpeople and the `gpeople` feature, every backend and `vendored` on by default, the capability union's rule for where a verb belongs, the clippy matrix and the live suites.
- README.md (in the earlier changes): the unified API's capabilities, REPL and MCP, the configuration section.
- The CHANGELOG `[Unreleased]` netted against 0.3.0, MIGRATION.md listed.

## Scope / non-goals

SECURITY.md keeps `0.x.x` supported, which holds whatever the next version. The version bump (0.4.0 suggested, the schemas' `$id` following) and the release are the owner's.
