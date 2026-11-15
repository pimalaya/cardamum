---
cairn: log
change: a-jmap-version-is-a-stable-digest
date: 2026-11-15
---

# A JMAP version is a stable digest

A JMAP card's `etag`, which `--if-match` is compared with, is the BLAKE3 digest of its JSON, where it was Rust's `DefaultHasher`, whose algorithm may change with the Rust release a build uses. Task C23 of cli-alignment-2026-11-15. Capabilities **backends** and **commands** moved.

src/jmap/project.rs `etag`. blake3 joins as an optional dependency of the `jmap` feature (`version = "1"`, `default-features = false`, io-pimdir's own requirement), already in the default build through io-pimdir: the Cargo.lock diff is cardamum's dependency line. sha2 was the other candidate, not in the normal tree. Evidence: `the_version_is_the_blake3_digest_of_the_json`, a fixed card whose JSON b3sum 1.8.5 digests to the value asserted; the old 16-digit hash fails it. CHANGELOG (Changed), MIGRATION.md. Spec: backends ("A JMAP card's version is a stable digest" added), commands ("An update guards on the version it read").
