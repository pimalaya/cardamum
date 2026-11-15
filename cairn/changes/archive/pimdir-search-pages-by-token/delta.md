---
cairn: delta
change: pimdir-search-pages-by-token
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: pimdir searches cards through the store's index
(commands) `cardamum pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (search-draft-11 §8) from the store's search index, scoped to the account's cards: the query is read as its conjuncts plus `kind:contact`, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--addressbook` names an address book. The scope is added beside the query, never around it, so a group that is itself a conjunct of the query keeps narrowing the answer and its coverage as its terms do bare (SEARCH §8.2). Without `-k` it SHALL search every address book of the account, `addressbook.default` not applied, and a `-k` naming no address book of the account SHALL fail naming those it holds rather than answer nothing. The words of the query are joined by spaces into one query. An option of `pimdir search` or `pimdir count` SHALL be read wherever it stands, before the query or after it, `--json` included; a query word starting with `-`, a negated term, SHALL pass after `--` alone, a word starting with `-` before it being refused as an unknown option or read as the option it spells, as calendula and Himalaya read theirs. Cardamum SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

Each hit SHALL carry the keys `card list` prints for a card (its `id` the public `seq`, which `card read` takes), its `addressbookId`, what a text term matched (`body`, `summary`, or `null` when only field terms did) and a `snippet`. `--sort name|relevance` orders the hits, `name` by default. The search SHALL page as `card list` does, and as Himalaya's `pimdir search`: `--page` and `--page-size` page the hits, and `nextPage`, the next page number as a string, `null` on the last page, SHALL be taken back through `--page-token` (pimalaya-cli `PageTokenArg`), which wins over `--page`, a token that is no integer from 1 failing ``Invalid page token <token>``. A hit the store cannot read (its item, or its body that cannot be read, is gone from the blob tree or reads as no vCard) SHALL be an `errors` entry alone, as "A listing names the items it could not read" says, its id the hit's, coded as its failure is or `card-unreadable`, the other hits listed, and a non-empty `errors` SHALL make the command exit 3. Under `--json` the page carries `hits`, `nextPage`, `coverage` and `errors`. The text SHALL be pimalaya-cli's `Listing`: `Account:` and the backend, `Addressbook:` when `-k` names one, `Query:`, `Page:`, `Next page:` and `Incomplete:`, then one line per hit, its id, its address book, what matched when a hit of the page says, its email, phone and name, then the `Error:` lines.

#### Scenario: An option after the query
- GIVEN a pimdir account
- WHEN `cardamum pimdir search alice --page-size 3 --json` runs
- THEN it searches `alice`, three hits a page, and prints JSON
- AND `cardamum pimdir search -k book -- alice -bob` searches `alice -bob` in `book`, while `cardamum pimdir search alice -zoe` is refused, exit 2

#### Scenario: A page and a hit the store cannot read
- GIVEN an indexed pimdir address book of five cards, the body of one gone from the blob tree
- WHEN `cardamum --json pimdir search -s 2 kind:contact` runs
- THEN it prints `"nextPage":"2"`, and the page holding that card lists the others and names it in `errors`, coded `card-unreadable`, exit 3; `--page-token 2` prints the second page

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

### Requirement: A pimdir card search says what it did not cover
(commands) The output of `pimdir search` and `pimdir count` SHALL carry the coverage of the answer (SEARCH §8.2): `bodiless`, the cards in scope whose body is not local, matched on their summary alone, `missing`, those whose body the index could not read, and `unparseable`, those whose body no parser accepted, the three counted only when a term of the query reads card bodies (free text, `has:phone`, `has:address`, `has:birthday`, `has:photo`, `category:`), a query of summary terms alone counting none and being whole over an address book listed and not fetched; `unindexed`, those whose last change the index has not folded in; `behind`, how many changes the store drew above the index, store-wide; `filling`, whether a first round is still listing an address book in scope; and `complete`, io-pimdir's own verdict (`PimdirSearchCoverage::is_complete`, the lag included), true only when none of these holds. Whether a body out of sight counts is io-pimdir's to say (`PimdirSearchLocal::reads`, `PimdirSearchReads::counts_bodiless`), never a rule restated here, so each count at rest agrees with `complete`. The keys are flat and in camelCase, cardamum's own shape over the per-kind counts of SEARCH §8.2, every query being scoped to cards. The plain output SHALL name what did not hold, `Incomplete: <what>`, as a field above the hits of `pimdir search` and as a line under the count of `pimdir count`, and nothing when the answer is complete.

#### Scenario: A card whose body is not local
- GIVEN an indexed pimdir address book holding a card listed and not fetched
- WHEN `cardamum pimdir count has:phone` runs
- THEN the coverage reports `bodiless` 1 and `complete` false, and the plain output names the card with no local body; `cardamum pimdir count email:@example.org` reports `bodiless` 0 and, nothing else missing, `complete` true, and prints no line

#### Scenario: A card the index could not parse
- GIVEN an indexed pimdir address book holding a card whose body no parser accepted
- WHEN `cardamum pimdir count has:phone` runs
- THEN the coverage reports `unparseable` 1 and `complete` false, and the plain output names the card that does not parse; `cardamum pimdir count email:@example.org` reports `unparseable` 0 and, nothing else missing, `complete` true

#### Scenario: An index behind the store
- GIVEN a card updated in the store since the owner's last refresh
- WHEN `cardamum pimdir count kind:contact` runs
- THEN the coverage reports `behind` above 0, `complete` false, and the plain output says the sync engine refreshes the index

### Requirement: Exit statuses
(commands) The binary SHALL exit 0 on success, 1 with the failure object printed, 2 on a command line the grammar refuses (an unknown flag or command, a missing argument, a bad value), and 3 on a partial outcome, a command that printed its result without fully succeeding (`pimalaya_cli::error::PartialOutcome`): `account check` with a failed backend, and a listing, `pimdir search` or a CardDAV `report query` or `report multiget` whose `errors` is not empty. The binary SHALL parse its command line through `pimalaya_cli::clap::parse`: help and version print as clap prints them; a refused command line with a `--json` word before any `--` SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without the `error: ` prefix nor styles, and without `--json` clap's rendering on stderr.

#### Scenario: A refused command line under JSON
- GIVEN any configuration, or none
- WHEN `cardamum --json card list --nope` runs
- THEN it exits 2, prints nothing on stderr and prints `{"code":"usage-invalid","error":"unexpected argument '--nope' found…","sources":[],"backtrace":null}`

### Requirement: Each backend names the cards it could not read
(backends) A backend SHALL report a card it could not read as an `errors` entry and SHALL never drop it silently nor fail the listing for it. A card named in `errors` SHALL NOT also list among the cards ("A listing names the items it could not read"):

- every backend: a card whose body does not parse as a vCard, or is not UTF-8, SHALL be a `card-unreadable` entry naming the card, no row listed for it;
- vdir: a card file that cannot be read SHALL be a `card-unreadable` entry naming the card, its error in `sources`; a card removed while it lists SHALL be left out with no entry;
- JMAP: a card that does not project SHALL be a `card-unreadable` entry, its id when it has one;
- pimdir: a body that cannot be read, or that the item names but its blob tree no longer holds (``Body blob missing from the store``), SHALL be a `card-unreadable` entry, no row listed for it, though its summary reads, as calendula names such an item `item-unreadable`; a body not fetched yet SHALL be no entry, the card listed with its preview. `pimdir search` SHALL name a hit the same way, an item the store cannot read and a body that reads as no vCard included, its id the hit's;
- CardDAV: a row the server answered without the card's data (a row refused whole, a refused `address-data` property, or none sent) SHALL be an entry naming the card, coded `permission-denied` when the row or a refused property carries a 403, else `card-unreadable`, each refused property and its status in `sources`; the entries print on the first page only, as the whole address book is read before the page is cut;
- CardDAV, Graph, People: a page or an answer that fails SHALL fail the listing whole.

## REMOVED Requirements

None.
