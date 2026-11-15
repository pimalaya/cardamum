---
cairn: delta
change: the-pimdir-queue-is-read-and-cancelled
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: The pimdir queue is read and retracted
(commands) `pimdir queue list`, `pimdir queue show <ROW>` and `pimdir queue cancel <ROW>`, gated on the `pimdir` feature, SHALL read and retract the account's queue as Himalaya's `pimdir queue` does. `list` SHALL print the card creations staged in the account's address books, or in the one `-k` names (`addressbook-not-found` for one the account does not hold), each by its queue row, when it was queued, the process that staged it, its address book, its kind (`add`), the link id `card add` reported and the full name of the body it pins (`null` when that body is not local), then every row of any kind the owner parked there, with its kind, its attempts, whether it is unconfirmed and the error that parked it: `{"cards": [{"row", "queuedAt", "producer", "collection", "kind", "linkId", "fullName"}], "parked": [{"row", "queuedAt", "producer", "collection", "kind", "attempts", "unconfirmed", "error"}]}`; the text prints them under `Account:` and `Addressbook:` with the cancelling command. `show` SHALL print where one row stands, `{"row", "state", "collection", "kind", "attempts", "error", "appliedAt", "seq"}`, `state` one of `pending`, `parked`, `unconfirmed`, `applied`, `unknown` and `reset`, `seq` the card an applied add created, the id `card read` takes; a row anchored on a collection that is not one of the account's address books SHALL read `unknown`. `cancel` SHALL ask for confirmation unless `-y`, refuse a row applied already or unknown to the account before the store is opened for writing, then take the store owner's role for that one write (io-pimdir's `PimdirStore::open_current` and `drop_action`), refused at once coded `store-owned` while a sync holds it, and print `{"row", "kind", "unconfirmed"}`. It is the one write cardamum makes as the store's owner, and the only way back for a queued creation, which has no id a `card delete` could take.

#### Scenario: A queued card is cancelled
- GIVEN a pimdir address book `book` a source syncs, and `cardamum card add -k book` queued a card as row 3
- WHEN `cardamum --json pimdir queue list` runs, then `cardamum --json pimdir queue cancel -y 3`
- THEN the listing holds `{"row":3,…,"collection":"book","kind":"add","linkId":"<UID>","fullName":"<FN>"}` under `cards`, the cancel prints `{"row":3,"kind":"add","unconfirmed":false}`, and `pimdir queue show 3` then reads `unknown`

### Requirement: pimdir takes the reader and producer roles, the owner's for one cancellation
(backends) The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL open a `PimdirStore` for `pimdir queue cancel` alone, through `PimdirStore::open_current`, which creates and migrates nothing, dropping the one row and releasing the handle inside the call, as Himalaya cancels a queued action. The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it any longer would both lock a sync out for the length of a command and put every destructive verb behind a frontend that never calls them; a sync holding it refuses the cancellation at once, coded `store-owned`. The reader SHALL be built with the pending overlay, so an update or removal this process staged reads back before the store's owner applies it.

## MODIFIED Requirements

### Requirement: Specific commands mirror their protocol
(commands) The second paragraph reads: pimdir is a store rather than a protocol: its `pimdir` family covers `status`, what a reader can tell of the store behind the account, `queue list`, `queue show` and `queue cancel`, the writes cardamum queued, and `search` and `count`, the store's search index queried in the pimdir SEARCH language. The other operator commands (retrying a parked action, the retained items) ship as the separate `pimdir` binary in io-pimdir.

### Requirement: pimdir status reports the store and its parked actions
(commands) Its JSON output is `{"account", "accounts", "addressbooks": [{"id", "name", "total", "hydrated", "queued"}], "parked": [{"row", "addressbookId", "kind", "error"}]}`, `parked` always present, `kind` the key a queue row names its action kind by in `pimdir queue list` and Himalaya. It mirrors calendula's `pimdir status`, `calendars` named `addressbooks`. It is no alias of `pimdir queue list`: it reports the store (its accounts, each address book's cards and how many are downloaded), which no queue command carries, where the queue commands list and retract the rows themselves. The rest of the requirement is unchanged.

## REMOVED Requirements

### Requirement: pimdir takes the reader and producer roles, never the owner
(backends) Replaced by "pimdir takes the reader and producer roles, the owner's for one cancellation" above.
