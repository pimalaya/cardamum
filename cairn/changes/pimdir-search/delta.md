---
cairn: delta
change: pimdir-search
---

## ADDED Requirements

### Requirement: pimdir searches cards through the store's index
`cardamum pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (search-draft-08 §8) from the store's search index, scoped to the account's cards: the query is read as its conjuncts plus `kind:contact`, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--addressbook` names an address book. Without `-k` it SHALL search every address book of the account, `addressbook.default` not applied. The words of the query are joined by spaces into one query. Cardamum SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

Each hit SHALL carry the keys `card list` prints for a card (its `id` the public `seq`, which `card read` takes), its `addressbookId`, what a text term matched (`body`, `summary`, or `null` when only field terms did) and a `snippet`. `--sort name|relevance` orders the hits, `name` by default, and `--page` and `--page-size` page them. Under `--json` the page carries `hits`, `more` (whether hits follow it) and `coverage`.

#### Scenario: A word of a note
- GIVEN an indexed pimdir address book holding a card whose `NOTE` says `allergic to peanuts`
- WHEN `cardamum pimdir search peanuts` runs
- THEN the card is listed with `matched` `body` and a snippet holding `peanuts`

#### Scenario: A queued card is not listed
- GIVEN a card created in a pimdir address book and still in the queue
- WHEN `cardamum pimdir search` runs a query its body matches
- THEN it is not listed until the owner applies the create and refreshes the index

### Requirement: pimdir counts cards from the index
`cardamum pimdir count <query>...` SHALL answer how many cards the scoped query matches, with its coverage, under the same scope as `pimdir search`.

### Requirement: A pimdir card search says what it did not cover
The output of `pimdir search` and `pimdir count` SHALL carry the coverage of the answer (SEARCH §8.2): `bodiless`, the cards in scope whose body is not local, matched on their summary alone; `unindexed`, those whose last change the index has not folded in; `behind`, how many changes the store drew above the index, store-wide; `filling`, whether a first round is still listing an address book in scope; and `complete`, true only when none of these holds. The plain output SHALL print one line under its table naming what did not hold, and nothing when the answer is complete.

#### Scenario: An index behind the store
- GIVEN a card updated in the store since the owner's last refresh
- WHEN `cardamum pimdir count kind:contact` runs
- THEN the coverage reports `behind` above 0, `complete` false, and the plain output says the sync engine refreshes the index

### Requirement: A pimdir index or query failure is coded
A store with no search index SHALL fail both commands with the code `index-missing`, an index another version built with `index-outdated`, one of another draft or shape with `index-version`, and a SQLite lacking FTS5 with `index-fts5-missing`, each saying the store's sync engine builds or rebuilds the index and none advising to delete anything. A query the language refuses SHALL fail with `query-refused`: in plain output the SEARCH §8.4 fault drawn over the query, under `--json` the §8.4 code, span and expectations in `error`. An accepted query the index cannot answer SHALL fail with `query-unanswerable`.

## MODIFIED Requirements

### Requirement: Specific commands mirror their protocol
A protocol-specific command set SHALL match the remote protocol's own structure and vocabulary rather than the shared API's, following Himalaya's direction. The shape follows the protocol: CardDAV (WebDAV) is a flat list of WebDAV method names, JMAP, Microsoft Graph and Google People are nested by resource with sub-verbs naming the protocol methods, and vdir is flat collection verbs plus an `item` subcommand. Verb names track the protocol, so JMAP uses `destroy` while Graph and People use `delete`.

pimdir is a store rather than a protocol: its `pimdir` family covers `status`, what a reader can tell of the store behind the account, and `search` and `count`, the store's search index queried in the pimdir SEARCH language. The operator commands (retrying or cancelling a parked action, the retained items) ship as the separate `pimdir` binary in io-pimdir.
