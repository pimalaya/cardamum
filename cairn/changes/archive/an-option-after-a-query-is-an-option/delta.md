---
cairn: delta
change: an-option-after-a-query-is-an-option
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: pimdir searches cards through the store's index
(commands) `cardamum pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (search-draft-11 §8) from the store's search index, scoped to the account's cards: the query is read as its conjuncts plus `kind:contact`, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--addressbook` names an address book. The scope is added beside the query, never around it, so a group that is itself a conjunct of the query keeps narrowing the answer and its coverage as its terms do bare (SEARCH §8.2). Without `-k` it SHALL search every address book of the account, `addressbook.default` not applied, and a `-k` naming no address book of the account SHALL fail naming those it holds rather than answer nothing. The words of the query are joined by spaces into one query. An option of `pimdir search` or `pimdir count` SHALL be read wherever it stands, before the query or after it, `--json` included; a query word starting with `-`, a negated term, SHALL pass after `--` alone, a word starting with `-` before it being refused as an unknown option or read as the option it spells, as calendula and Himalaya read theirs. Cardamum SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

Each hit SHALL carry the keys `card list` prints for a card (its `id` the public `seq`, which `card read` takes), its `addressbookId`, what a text term matched (`body`, `summary`, or `null` when only field terms did) and a `snippet`. `--sort name|relevance` orders the hits, `name` by default, and `--page` and `--page-size` page them. Under `--json` the page carries `hits`, `more` (whether hits follow it) and `coverage`.

#### Scenario: An option after the query
- GIVEN a pimdir account
- WHEN `cardamum pimdir search alice --page-size 3 --json` runs
- THEN it searches `alice`, three hits a page, and prints JSON
- AND `cardamum pimdir search -k book -- alice -bob` searches `alice -bob` in `book`, while `cardamum pimdir search alice -zoe` is refused, exit 2

#### Scenario: A word of a note
- GIVEN an indexed pimdir address book holding a card whose `NOTE` says `allergic to peanuts`
- WHEN `cardamum pimdir search peanuts` runs
- THEN the card is listed with `matched` `body` and a snippet holding `peanuts`

#### Scenario: A queued card is not listed
- GIVEN a card created in a pimdir address book and still in the queue
- WHEN `cardamum pimdir search` runs a query its body matches
- THEN it is not listed until the owner applies the create and refreshes the index

#### Scenario: An address book named in a group
- GIVEN an indexed pimdir store whose address book `book` holds a card listed and not fetched and whose address book `team` holds none
- WHEN `cardamum pimdir search "(collection:team) offsite"` runs
- THEN the hits and the coverage are those of `collection:team offsite`: `bodiless` 0 and `complete` true

## REMOVED Requirements

None.
