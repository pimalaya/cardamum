---
cairn: change
id: cairn-adoption
status: landed
created: 2026-08-09
---

# Replace docs/ with cairn/

## Why

Cardamum carried a docs/ folder mixing three kinds of writing: standing design contracts, a development narrative, and live provider test reports. Nothing said which statements were still true, and there was no AGENTS.md, so an agent working here had no activation stanza and no forcing rule. Every other active Pimalaya repository (himalaya, himalaya-tui, neverest, vcard-rs, ical-rs, io-webdav) had already moved to Cairn, which supersedes docs/ per the org guidelines.

## What

A cairn/ root with spec/, changes/ and log/, activated by AGENTS.md (and CLAUDE.md pointing at it).

The standing contracts become six capability specs written as normative requirements: backends, commands, config, wizard, projection and addressbooks. The twelve provider test reports move to cairn/spec/testing/ unchanged, indexed by a new README carrying the capability frontmatter, matching Himalaya's layout. The development narrative in architecture.md and specific-apis-design.md is not carried over as prose: what it asserted is now a requirement, and what it recorded is history that belongs to the log.

## Scope / non-goals

- This is a rewrite, not a move. docs/ was narrative; cairn/spec/ is normative. Only the test reports transfer verbatim.
- The module map in architecture.md is dropped rather than folded in: it duplicated the source tree and had already drifted (it listed neither `pimdir/` nor shared/table.rs), and src/main.rs is the architecture entry point.
- The three changes landed alongside this one are archived here with their own proposal, tasks and delta, so the spec's provenance is complete from day one.
