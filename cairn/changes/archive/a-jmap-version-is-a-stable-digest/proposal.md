---
cairn: change
id: a-jmap-version-is-a-stable-digest
status: landed
created: 2026-11-15
---

# A JMAP version is a stable digest

## Why

JMAP has no per-card version, so cardamum reports a hash of the ContactCard's JSON as its `etag`, and since graph-and-jmap-guard-on-the-version-read compares `--if-match` with it. The hash came from Rust's `DefaultHasher`, whose algorithm the standard library may change between releases: a version printed by one build could fail `precondition-failed` in another built with a later Rust. Task C23 of cli-alignment-2026-11-15.

## What

- The BLAKE3 digest of the JSON, lowercase hex. blake3 is already in the default build through io-pimdir (same `version = "1"`, `default-features = false`), so it joins as an optional dependency of the `jmap` feature: the Cargo.lock diff is cardamum's own entry, and a build without pimdir adds blake3 alone. sha2 is not in the tree (only io-oauth's dev-dependency pulls it).
- The `etag` of a JMAP card changes value once, from 16 hex digits to 64: MIGRATION.md says a version stored from v0.3 is read again.
