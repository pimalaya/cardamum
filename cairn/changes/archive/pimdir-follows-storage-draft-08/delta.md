---
cairn: delta
change: pimdir-follows-storage-draft-08
---

# Delta

## ADDED Requirements

### Requirement: A pimdir store of another version is reinstalled
The pimdir backend SHALL open a store of the format version the io-pimdir it builds against reads (`io_pimdir::sql::FORMAT_VERSION`, `storage-draft-08` at io-pimdir bff5737) and no other, earlier or later, refusing it as `store-version` and reading nothing in it (pimdir STORAGE §4.2, §6). Nothing migrates a draft store. The refusal's message is cardamum's, not io-pimdir's: it names the version the store records, or that it records none, and the one this cardamum reads, and says that a store an earlier neverest wrote is reinstalled (that neverest's sync run until nothing is queued, neverest upgraded, the store moved aside, `neverest init` and a sync), while a store a later neverest wrote asks for a cardamum upgrade. It SHALL NOT advise deleting the store, nor syncing a new one before the queue is drained: the queued writes live in that store alone.

#### Scenario: A store an earlier neverest wrote
- GIVEN a pimdir store whose `store_meta.format_version` is `storage-draft-07`
- WHEN any cardamum command opens it
- THEN it fails with the code `store-version` and a message naming `storage-draft-07` and the neverest reinstall, its queue drained first

### Requirement: A pimdir store others can read is refused
The pimdir backend SHALL carry `store-exposed` when io-pimdir refuses a store whose directory or pimdir.db another user can read or write, or a link where a lock file belongs (pimdir STORAGE §18), before anything in it is opened. The message is io-pimdir's: restrict the store (`chmod 700` its directory, `600` its pimdir.db), never remove it.

## MODIFIED Requirements

### Requirement: A store refusal carries its code
The pimdir backend SHALL open a store as the io-pimdir it builds against defines it and SHALL NOT migrate one, migrating being the owner's. A failure to open, read the queue of, or stage into the pimdir store SHALL carry, under `--json`, the code io-pimdir's `PimdirError::code()` names for it, spelled as io-pimdir spells it, for the failures a caller acts on: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-exposed`, `store-busy`, `unsupported`, `item-duplicate` and `action-malformed`; an SQL, I/O or JSON failure carries none, and so does a code no cardamum path raises (`flag-invalid` among them: cardamum stages no flag, its `add` carrying the empty set and its `update` and `remove` none). `action-malformed` reaches cardamum on `pimdir status`, whose count of queued creates decodes every pending row of the address book, when another program queued a payload its schema refuses: the owner's drain parks it, and the status reads again. The message is io-pimdir's, but for `store-version` (see "A pimdir store of another version is reinstalled"), and SHALL NOT advise deleting the store: a resync restores neither the queue nor what the store keeps alone.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL leave a body's name to the producer staging it (`enqueue_with_body_checked`), which hashes it with BLAKE3, the one algorithm a store records in `store_meta.hash_algo` (pimdir STORAGE §5; a store recording another is refused as `store-hash-algo`), and SHALL NOT hash or write a body itself. A body named under another algorithm is a body no read ever finds, and one written outside the producer's staging lock is one a collector may remove before its row pins it.

### Requirement: A JSON error carries a stable code
In cairn/spec/commands.md, the code table loses `store-newer` and `store-stale` and gains:

| Code | Raised when |
|---|---|
| `store-version` | the pimdir store is of another format version than this cardamum reads; an earlier neverest's store is reinstalled through neverest, its queue drained first, a later one's asks for a cardamum upgrade |
| `store-exposed` | another user can read or write the pimdir store, or a link stands where a lock file belongs; restrict it |
| `action-malformed` | a queued pimdir action's payload is malformed (`pimdir status`); the sync engine's drain parks it |

`store-corrupt` also covers a store whose schema differs from the one its version declares (`PimdirError::Damaged`). The rest of the requirement is unchanged.

## REMOVED Requirements

None. The codes `store-newer` and `store-stale` leave the vocabulary through the two modified requirements above; neither was ever released (both came after 0.3.0).
