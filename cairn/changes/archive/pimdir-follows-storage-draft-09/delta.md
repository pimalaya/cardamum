---
cairn: delta
change: pimdir-follows-storage-draft-09
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: A pimdir store of another version is reinstalled
In cairn/spec/backends.md, the version the pimdir backend reads is `storage-draft-09` at io-pimdir 5c18a02, and the scenario's earlier store records `storage-draft-08`. The rest of the requirement is unchanged.

#### Scenario: A store an earlier neverest wrote
- GIVEN a pimdir store whose `store_meta.format_version` is `storage-draft-08`
- WHEN any cardamum command opens it
- THEN it fails with the code `store-version` and a message naming `storage-draft-08` and the neverest reinstall, its queue drained first

### Requirement: pimdir searches cards through the store's index
In cairn/spec/commands.md, the query language is pimdir SEARCH search-draft-09 §8. The rest of the requirement is unchanged.

## REMOVED Requirements

None.
