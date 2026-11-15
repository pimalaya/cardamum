---
cairn: delta
change: an-unreadable-card-is-an-entry-alone
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A listing names the items it could not read
(commands) `addressbook list` and `card list` SHALL print an `errors` list beside their items, under `--json` and in text: one pimalaya-cli item error `{id, code, error, sources}` per address book or card the command could not read, `[]` when there is none, in the order met, and one entry with a `null` id and no code when the server truncated the listing (CardDAV). `id` SHALL be the item's id as the command prints ids, `null` when the backend named none; `code` a code of the vocabulary or `null`; `error` the message; `sources` the backend's own reasons, outermost first. An item in `errors` SHALL NOT print among the items, under `--json` nor in text, whatever made it unreadable (a card whose body does not parse or is not UTF-8, a file or a body that cannot be read, a row without its data): it is an entry alone, as calendula's `event list` prints an item it cannot read. It SHALL keep its place in the paging, the page holding it naming it and the next page starting after it. A non-empty `errors` SHALL make the command exit 3 once it printed its result. A listing that reads the whole address book before cutting its page SHALL print its entries on the first page only.

#### Scenario: A vdir card that cannot be read
- GIVEN a vdir address book holding three cards, one of whose files cannot be read
- WHEN `cardamum --json card list` runs
- THEN it prints the two other cards and `"errors":[{"id":"<its id>","code":"card-unreadable",…}]`, and the exit status is 3

#### Scenario: A card that does not parse
- GIVEN a card whose body is no vCard
- WHEN `cardamum --json card list` runs
- THEN `cards` leaves it out, an entry coded `card-unreadable` names it, the text prints no row for it, and the exit status is 3

### Requirement: Each backend names the cards it could not read
(backends) A backend SHALL report a card it could not read as an `errors` entry and SHALL never drop it silently nor fail the listing for it. A card named in `errors` SHALL NOT also list among the cards ("A listing names the items it could not read"):

- every backend: a card whose body does not parse as a vCard, or is not UTF-8, SHALL be a `card-unreadable` entry naming the card, no row listed for it;
- vdir: a card file that cannot be read SHALL be a `card-unreadable` entry naming the card, its error in `sources`; a card removed while it lists SHALL be left out with no entry;
- JMAP: a card that does not project SHALL be a `card-unreadable` entry, its id when it has one;
- pimdir: a body that cannot be read SHALL be a `card-unreadable` entry, no row listed for it, though its summary reads; a body the store does not hold yet SHALL be no entry, the card listed with its preview;
- CardDAV: a row the server answered without the card's data (a row refused whole, a refused `address-data` property, or none sent) SHALL be an entry naming the card, coded `permission-denied` when the row or a refused property carries a 403, else `card-unreadable`, each refused property and its status in `sources`; the entries print on the first page only, as the whole address book is read before the page is cut;
- CardDAV, Graph, People: a page or an answer that fails SHALL fail the listing whole.

## REMOVED Requirements
