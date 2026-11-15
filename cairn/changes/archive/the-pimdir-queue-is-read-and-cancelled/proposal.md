---
cairn: change
id: the-pimdir-queue-is-read-and-cancelled
status: landed
created: 2026-11-15
---

# The pimdir queue is read and cancelled

## Why

Himalaya reads and retracts its queue with `pimdir queue list`, `show` and `cancel`; cardamum had `pimdir status` alone, which counts the queued creations without listing them and names no way to retract a parked write but io-pimdir's operator binary. Finding F24, task C5 of cli-alignment-2026-11-15, twin of calendula's K7.

## What

- `pimdir queue list [-k ADDRESSBOOK]`: the card creations staged and the parked rows, every address book of the account or the one named, as `pimdir count` and `search` take `-k`; Himalaya's keys (`row`, `queuedAt`, `producer`, `kind`, `attempts`, `unconfirmed`, `error`), plus `collection` on each row (several address books in one listing) and `kind`, `linkId`, `fullName` on a queued card where Himalaya carries an envelope, the keys calendula's twin listing carries (`summary` for `fullName`).
- `pimdir queue show <ROW>`: Himalaya's output, `seq` the card an applied add created.
- `pimdir queue cancel <ROW> [-y]`: Himalaya's, the owner role taken for that one write.
- `pimdir status` stays its own command, not a hidden alias of `queue list`: it reports the store (the accounts it groups, each address book's cards and how many are downloaded), which no queue listing carries. Its parked rows key their kind `kind`, as the queue rows do, where they read `action`; `pimdir status` is new since v0.3, so nothing breaks.
- The spec's "never the owner" rule gives way to the one cancellation.
