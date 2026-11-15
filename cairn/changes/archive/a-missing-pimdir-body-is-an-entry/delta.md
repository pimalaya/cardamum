---
cairn: delta
change: a-missing-pimdir-body-is-an-entry
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Each backend names the cards it could not read
(backends) A backend SHALL report a card it could not read as an `errors` entry and SHALL never drop it silently nor fail the listing for it. A card named in `errors` SHALL NOT also list among the cards ("A listing names the items it could not read"):

- every backend: a card whose body does not parse as a vCard, or is not UTF-8, SHALL be a `card-unreadable` entry naming the card, no row listed for it;
- vdir: a card file that cannot be read SHALL be a `card-unreadable` entry naming the card, its error in `sources`; a card removed while it lists SHALL be left out with no entry;
- JMAP: a card that does not project SHALL be a `card-unreadable` entry, its id when it has one;
- pimdir: a body that cannot be read, or that the item names but its blob tree no longer holds (``Body blob missing from the store``), SHALL be a `card-unreadable` entry, no row listed for it, though its summary reads, as calendula names such an item `item-unreadable`; a body not fetched yet SHALL be no entry, the card listed with its preview. `pimdir search`, which has no `errors` yet, previews a card whose blob is missing, with a warning;
- CardDAV: a row the server answered without the card's data (a row refused whole, a refused `address-data` property, or none sent) SHALL be an entry naming the card, coded `permission-denied` when the row or a refused property carries a 403, else `card-unreadable`, each refused property and its status in `sources`; the entries print on the first page only, as the whole address book is read before the page is cut;
- CardDAV, Graph, People: a page or an answer that fails SHALL fail the listing whole.

### Requirement: A cache renames nothing it cannot push
(backends) The pimdir backend SHALL refuse every `addressbook update` field. Its collection row (id, display name, description, colour) is written by the sync from the server, and the backend stages item actions only, so any local edit of it is a change no sync would carry. A body not fetched yet SHALL fall back to the summary preview in a listing, rather than render as a blank row; a body the item names but the store no longer holds is named in `errors` ("Each backend names the cards it could not read").

### Requirement: A JSON error carries a stable code
(commands) One row changes, the rest unchanged:

| Code | Raised when |
|---|---|
| `card-unreadable` | the backend answered for a card cardamum could not read: its listing, `report query` or `report multiget` row came without its data (CardDAV), its file could not be read (vdir), its body did not parse as a vCard or is not UTF-8, it did not project (JMAP), its body could not be read or is gone from the blob tree (pimdir); an `errors` entry |

## REMOVED Requirements
