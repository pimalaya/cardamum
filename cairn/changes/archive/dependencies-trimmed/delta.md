---
cairn: delta
change: dependencies-trimmed
---

# Delta

## ADDED Requirements

### Requirement: Dependencies name their features
(packaging) Every dependency, build and dev ones included, SHALL take `default-features = false` and name the features its items need, and the manifest SHALL list no dependency the code does not name.

### Requirement: Dependency checks pass clean
(packaging) `cargo deny check` SHALL pass its advisories, bans, licences and sources checks over the release targets deny.toml lists, the licences allowed being the ones the tree carries.

### Requirement: Hosts are ASCII
(packaging) A server or proxy URL with a special scheme (`http`, `https`) SHALL be refused when its host is not ASCII, its Punycode form working: IDNA goes through `idna_adapter` 1.0.0, without the ICU4X crates.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
