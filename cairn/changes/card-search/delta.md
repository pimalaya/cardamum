---
cairn: delta
change: card-search
---

# Delta

## ADDED Requirements

### Requirement: The unified card search speaks the pimdir query language
`cardamum card search <query>...` SHALL read its query as pimdir SEARCH §8, its words joined by spaces into one query, parsed by io-pimdir before any backend is reached. A query SEARCH §8.4 refuses SHALL fail with the code `query-refused` on every backend, naming the §8.4 code and span, nothing listed. The search SHALL cover every address book of the account, or the one `-k` names, a `-k` naming none of them failing with the ones it holds.

#### Scenario: One refusal everywhere
- GIVEN any account whose backend serves `card.search`
- WHEN `cardamum --json card search frm:alice` runs
- THEN the command exits 1 with the code `query-refused` and a message naming `unknown-field` at `0..4`, before connecting

### Requirement: Every backend answers the same hit set
On pimdir, `card search` SHALL answer from the store's index, the hits of `pimdir search` for the same query and scope. On every other backend serving `card.search`, it SHALL list the cards of the address books in scope and match each decoded vCard with io-pimdir's matcher, so a query answers the hit set an indexed pimdir store holding the same cards answers. A backend SHALL NOT answer a narrower or wider set by matching on the server.

#### Scenario: A diacritic-free query finds an accented name
- GIVEN a CardDAV address book holding a card with `FN:Élodie Martin`
- WHEN `cardamum card search elodie` runs
- THEN the card is listed, as it is on a pimdir store holding it

### Requirement: A store term is refused where there is no store
On a backend other than pimdir, a query holding `tag:`, `flag:`, `is:`, `changed:`, `thread:`, `id:`, `linked:` or `account:`, negated or not, SHALL be refused whole with the code `query-unsupported` before connecting, the message naming the term, its span and the backend. No term SHALL be dropped from a query. A term SEARCH answers for cards without a store (`status:`, `priority:`, the mail roles, `date:`) SHALL be answered as SEARCH says.

#### Scenario: A tag on CardDAV
- GIVEN a CardDAV account
- WHEN `cardamum --json card search tag:work alice` runs
- THEN the command exits 1 with the code `query-unsupported` naming `tag:work` at `0..8`

### Requirement: The card search output
Each hit SHALL carry the `card list` row with its `addressbookId`. Under `--json` the answer SHALL carry `nextPage` as `card list` does and `complete`: on pimdir the coverage verdict of SEARCH §8.2, elsewhere `true`. Hits SHALL be ordered by name (SEARCH §8.2) unless `--sort relevance`, which only a backend declaring `card.search.relevance` takes. `--page`, `--page-size` and `--page-token` SHALL page the hits as `card list` pages cards.

## MODIFIED Requirements

### Requirement: Unified capabilities
The capability tables gain `card.search`: `full` on pimdir, `partial` on carddav, jmap, msgraph and gpeople (store terms refused, matched over the whole listing), `none` on vdir unless the owner decides otherwise; and `card.search.relevance`: `full` on pimdir, `none` elsewhere.

### Requirement: The MCP tools
`cardamum mcp` lists `card_search` among the read-only tools, where the account's backend serves `card.search`.

### Requirement: The JSON contract is described and versioned
The code table gains `query-unsupported`, a query holding a term the backend cannot answer; `query-refused` is also raised by `card search`.

## REMOVED Requirements
