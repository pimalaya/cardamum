---
cairn: delta
change: a-refusal-never-advises-removing-the-store
---

## MODIFIED Requirements

### Requirement: pimdir addressbook limits are explicit
Every `addressbook` write SHALL fail on pimdir, each naming its reason. `create_addressbook` refuses because declaring a collection is an owner write and this backend is a producer, and because a collection no sync knows about is one no sync would carry. `update_addressbook` refuses because the collection row (id, display name, description, colour) is what a sync writes from the server, and io-pimdir's `rename_collection` renames the identifier rather than the display name. `delete_addressbook` refuses, because io-pimdir exposes no collection removal, with a message naming the alternative: delete it on the server and sync. It SHALL NOT advise removing the store, which loses the queued writes a resync never restores.

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`. The code SHALL be found anywhere in the error chain, so added context does not hide it. A failure with no code SHALL print no `code` field. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. A pimdir store's code is the one io-pimdir names (`PimdirError::code()`), never read off its message.

| Code | Raised when |
|---|---|
| `body-pending` | a listed card's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the card no longer has (pimdir) |
| `store-uncreated` | the pimdir root holds no store; its sync engine creates it |
| `store-newer` | the pimdir store is newer than this cardamum reads; upgrade cardamum |
| `store-foreign` | the pimdir root holds a database another program wrote |
| `store-stale` | the pimdir store lacks part of its schema; run or upgrade its sync engine |
| `store-corrupt` | the pimdir store's two version stamps disagree |
| `store-hash-algo` | the pimdir store names its bodies by a hash this cardamum does not know |
| `store-busy` | another writer held the pimdir store, past the wait on a write and at once on `account check`, which does not wait; retry |
| `unsupported` | a source syncing the address book does not support the write (pimdir STORAGE §15.6); nothing was queued |
| `item-duplicate` | a create names a `UID` the pimdir address book holds, synced or queued; nothing was queued |

#### Scenario: A stale version
- GIVEN a pimdir card whose body changed since it was read
- WHEN `cardamum --json card delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

