---
cairn: delta
change: follows-storage-draft-17
---

# Delta

## ADDED Requirements

### Requirement: Only the pimdir backend links SQLite
(packaging) The `pimdir` feature SHALL take io-pimdir's `client`, `search` and `contact`; `carddav`, `jmap`, `msgraph`, `gpeople` and `vdir` SHALL take its `contact` alone (the card matcher, no SQLite, no mail or calendar parser). The tests take its `indexer` as a dev-dependency, so no release binary carries the index refresh.

#### Scenario: A CardDAV-only build
- GIVEN `--no-default-features --features rustls-ring,carddav`
- WHEN `cargo tree -e normal -i rusqlite` runs
- THEN no package matches

### Requirement: A reserved form of the query language is refused
(commands) A query holding a text or a value beginning with `~`, one of two characters or more beginning with `/`, or a value beginning with `{` (pimdir SEARCH §8.3, search-draft-17) SHALL be refused `query-refused` by `card search`, `pimdir search` and `pimdir count` on every backend, the §8.4 code `reserved-form` and its span named, nothing read; quoted, the characters are searched.

#### Scenario: A stemmed word
- GIVEN any account whose backend serves `card.search`
- WHEN `cardamum --json card search ~alice` runs
- THEN it exits 1 coded `query-refused`, its error naming `reserved-form` at `0..6`; `card search '"~alice"'` is accepted

## MODIFIED Requirements

### Requirement: A pimdir store of another version is refused in io-pimdir's words
(backends) As now, the format version read being `storage-draft-17` (`io_pimdir::sql::FORMAT_VERSION` at io-pimdir 7f16b27, its release 0.20 once published) and the scenario's store `storage-draft-15`, the version io-pimdir 0.19 wrote: the message names `storage-draft-15`, `storage-draft-17`, an earlier sync engine, the queue synced until nothing is queued and the directory moved aside; the new store is synced anew by a neverest of the same io-pimdir.

### Requirement: pimdir searches cards through the store's index
(commands) As now, reading search-draft-17 §8, `--sort title` being SEARCH §8.2's `title` order (`name` a hidden alias of it): the full name folded as STORAGE Annex A.2 folds `fn`, a card with none first, ties by `seq`.

### Requirement: A store term is refused where there is no store
(commands) The terms only a store answers are `PimdirQuery::store_terms`: `tag:`, `flag:`, `is:` but `is:encrypted`, `account:`, `collection:`, `folder:`, `changed:`, `thread:`, `id:`, `linked:`, `label:` and `property:`. On pimdir `label:` and `property:` are answered from the store's item properties (STORAGE §4.3), as io-pimdir answers them. The `query-unsupported` row of the code table lists the same terms.

### Requirement: The pimdir queue is read and retracted
(commands) As now, a `set-property` row (STORAGE §15.3, queued by another program) naming the card it addresses: `itemId` its id and `fullName` its full name, `linkId` `null`, listed, shown and cancelled as any row.
