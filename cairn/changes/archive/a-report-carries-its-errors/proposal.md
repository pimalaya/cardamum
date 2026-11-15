---
cairn: change
id: a-report-carries-its-errors
status: landed
created: 2026-11-15
---

# A report carries its errors

## Why

a-result-carries-its-errors (landed 2026-11-14) made `card list` name each CardDAV row the server answered without the card's data, but left `carddav report query` and `carddav report multiget` dropping those rows silently (its log, "Left open"). io-webdav 777d34e already hands both the rows: `CarddavCardListOk::unreadable` and `CarddavCardMultigetOk::unreadable`. The live run on 2026-08-09 met the multiget case as finding S4 (cairn/spec/testing/carddav-specific.md): a batch naming an absent id returned the other card and exited 0, the 404 row filtered out.

## What

- `report query` and `report multiget` print `errors` beside `cards` and `truncated`, `{id, code, error, sources}` per row, exactly as `card list` builds a CardDAV entry (`carddav::backend::unreadable_row`, made `pub(crate)`): a 403 row or refused `address-data` `permission-denied`, else `card-unreadable`, the refused properties in `sources`. A non-empty `errors` exits 3 once the result is printed (`error::partial_when`).
- `report multiget` codes a row answered 404 or 410 `card-not-found`: it names an id the caller gave, which the address book does not hold, as a card request answered 404 is coded. A listing never names a card it does not hold, so `card list` keeps its rule.
- The text prints the `Error:` lines last through pimalaya-cli's `ItemErrors`, after the table and the truncation line and one blank line; an output holding the errors alone starts with them.
- Not done, by design: a truncated result stays the `truncated` field and its text line, and adds no entry. `card list` adds one because its listing asks for every card; `report query --limit N` asks the server to cut (RFC 6352 §8.6.1 answers that cut with the 507 row), so an entry would turn the expected outcome into exit 3.
- Spec: commands ("A CardDAV report names the cards it could not read" added; "Exit statuses" and "A JSON error carries a stable code" modified), backends ("A server's refusal carries its code" modified).
