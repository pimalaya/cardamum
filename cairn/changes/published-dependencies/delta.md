---
cairn: delta
change: published-dependencies
---

# Delta

## ADDED Requirements

### Requirement: Released dependencies, patched only while unreleased
Dependencies SHALL name released versions. A `[patch.crates-io]` entry is allowed only while a needed change rides unreleased upstream, the cairn change adding it SHALL say what it waits on (Cargo.toml carrying no comment in its dependency or patch sections), and it SHALL be dropped as that crate publishes.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
