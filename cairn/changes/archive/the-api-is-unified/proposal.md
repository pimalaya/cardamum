---
cairn: change
id: the-api-is-unified
status: landed
created: 2026-11-12
---

# The API is unified

## Why

The owner renamed Himalaya's "shared API" to "unified API" on 2026-11-12 (himalaya the-api-is-unified, b21c2d7f) and asked cardamum to follow the same day: with the capability union cardamum adopts in the-family-standards, the cross-protocol commands serve the union of what the backends can do, and "shared" suggested the least common denominator.

## What

- "shared API" reads "unified API" across the code, its `--help` text (`Manage vCards using the unified API.`, `Manage addressbooks using the unified API.`), README.md, CONTRIBUTING.md, the spec and its test reports and the active changes; so do the phrases naming the API's parts: its commands, subcommands, operations, writes, outputs, surface and arm, and the `unified-API client`.
- Renamed requirements, their text otherwise unchanged: "Unified commands stay least-common-denominator" (commands, replaced by capability-shared-api), "Unified operation set" and "Unified outputs carry no backend details" (backends).

## Scope / non-goals

No behaviour changes. "Shared" stays where it means shared by the code: the src/shared module, the shared addressbook and card types, the shared client, pimalaya-cli's shared REPL. The log, landed changes and released CHANGELOG sections keep the words of their time; the unreleased CHANGELOG does not name the API.
