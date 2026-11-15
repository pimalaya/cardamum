---
cairn: delta
change: the-codes-follow-the-family
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A not-found failure is cardamum's sentence
(commands) A card a command names that the address book does not hold SHALL fail with cardamum's own sentence, ``Card `<id>` not found`` (``… not found in `<book>` `` on pimdir, which has no text of its own to add), the backend's answer below it in `sources` (io-vdir's, the CardDAV, Graph or People refusal, the JMAP `notFound`), as calendula words an item not found: the backend's wording never stands as the error. An item error SHALL NOT repeat its card's id in `error`, the id being its own key.

### Requirement: A command with no address book is coded
(commands) A card command and `addressbook update` or `delete` given no `-k/--addressbook` over an account setting no `addressbook.default` SHALL fail coded `addressbook-default-missing`, exit 1, the error naming the flag and the key: ``No default addressbook found, name one with `-k <ADDRESSBOOK-ID>` or set `addressbook.default` ``, as calendula refuses a missing default calendar (`calendar-default-missing`) and the family a missing default account (`account-default-missing`).

## MODIFIED Requirements

### Requirement: A store refusal carries its code
(backends) The pimdir backend SHALL open a store as the io-pimdir it builds against defines it and SHALL NOT migrate one, migrating being the owner's. A failure to open, read the queue of, or stage into the pimdir store SHALL carry, under `--json`, the code io-pimdir's `PimdirError::code()` names for it, spelled as io-pimdir spells it, for the failures a caller acts on: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-exposed`, `store-busy`, `lock-replaced`, `unsupported`, `item-duplicate` and `action-malformed`, and every other code Himalaya relays, so one store failure reads the same code in every program of the family: `store-not-current`, `sync-version`, `store-owned`, `store-staging`, `store-in-flight`, `store-reset`, `flag-invalid`, `conflict-unread`, `no-performer`, `ambiguous-performer`, `index-zone-unknown` and `tag-refused`, though no cardamum path raises some of them today (cardamum stages no flag, no intent and no tag). An SQL, I/O or JSON failure carries none, and so does a code no program of the family relays. `action-malformed` reaches cardamum on `pimdir status`, whose count of queued creates decodes every pending row of the address book, when another program queued a payload its schema refuses: the owner's drain parks it, and the status reads again. `lock-replaced` reaches cardamum on a write, the producer checking before its enqueue that the staging lock file it holds is still the one at its path (pimdir STORAGE §8): nothing is queued. The message is io-pimdir's, but for `store-version` (see "A pimdir store of another version is reinstalled"), and SHALL NOT advise deleting the store: a resync restores neither the queue nor what the store keeps alone.

### Requirement: A server's refusal carries its code
(backends) Two rows join the table, the rest unchanged:

| Code | When |
|---|---|
| `precondition-failed` | a CardDAV write answered 412, the `If-Match` it sent naming a version the card no longer has |
| `item-duplicate` | a CardDAV write refused for the address book's `no-uid-conflict` precondition (RFC 6352 §6.3.2.1), as calendula codes the CalDAV one |

### Requirement: A JSON error carries a stable code
(commands) The table rows below change or join, the rest unchanged; the scenario "An unknown card" now reads its `error` and `sources`:

| Code | Raised when |
|---|---|
| `card-not-found` | the card a command names is not in the address book: the failure of `card read`, `card update` or `card delete`, worded by cardamum ("A not-found failure is cardamum's sentence"); an `errors` entry of `carddav report multiget` |
| `addressbook-default-missing` | a command needing an address book got no `-k` and the account sets no `addressbook.default`; nothing was read or written |
| `store-not-current` | the pimdir store holds no schema yet and the verb creates none; its sync engine creates it |
| `sync-version` | the pimdir store's sync state was written by another version of its sync engine |
| `store-owned` | a sync owns the pimdir store; retry once it ends |
| `store-staging` | a producer is staging a body in the pimdir store; retry |
| `store-in-flight` | a pimdir store verb is between two of its chunks; retry |
| `store-reset` | the pimdir store is not the one that answered a queue row's handle, a copy restored since; nothing was done with the row |
| `item-duplicate` | a create names a `UID` the address book holds: on pimdir synced or queued, nothing queued; on CardDAV the server's `no-uid-conflict` (RFC 6352 §6.3.2.1) |
| `flag-invalid` | the pimdir store cannot hold a flag; nothing was staged |
| `conflict-unread` | a pimdir conflict holds a body nobody has read; fetch it first |
| `no-performer` | no source of the pimdir store can perform the intent |
| `ambiguous-performer` | several sources of the pimdir store can perform the intent and none is chosen |
| `index-zone-unknown` | the zone the pimdir search index reads floating times in is not an IANA zone |
| `tag-refused` | a pimdir tag names no flag, or adds and removes one flag; nothing was staged |
| `precondition-failed` | a write's `--if-match` names a version the card no longer has: pimdir's check, or a CardDAV server answering 412 |

## REMOVED Requirements
