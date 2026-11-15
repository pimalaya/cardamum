---
cairn: log
change: an-unreadable-card-is-an-entry-alone
date: 2026-11-15
---

# An unreadable card is an entry alone

`card list` names a card it could not read in `errors` alone, under `--json` and in text, where it also printed it as a row of empty fields (or, on pimdir, of its preview). Finding F13 of the cli-alignment-2026-11-15 audit; calendula's `event list` already behaved so. Capabilities **commands** and **backends** moved.

- **src/shared/client.rs**: `CardPage::of`, the seam every backend's listing goes through, cuts the page first (the card past it leaving with its entry, as before), then drops from `cards` each card `errors` names, the backend's entries (a pimdir body that could not be read) and its own (a body that does not parse or is not UTF-8) alike. The backend's window stays positional, so a dropped card keeps its place and the next page starts after it.
- **Backends checked**: vdir (an unreadable file was already an entry alone; a body that does not parse listed twice), pimdir (a body that could not be read listed its preview beside its entry; a body that does not parse listed twice), CardDAV (a row without data was already an entry alone; `address-data` that does not parse listed twice), JMAP (a card that does not project was already an entry alone; a projection that does not parse would have listed twice), Graph and People (only the parse case, through the same seam).
- **Docs**: `CardWindow`, the `card list` help, the pimdir `listed_card`.
- **Evidence**: on the old seam, tests/listing.rs `a_card_that_does_not_parse_is_an_entry_alone`, `a_card_that_is_not_utf8_is_an_entry_alone`, `a_card_that_does_not_parse_keeps_its_place_in_the_pages` and `the_text_listing_prints_its_errors_last`, `a_card_the_page_could_not_read_is_in_its_errors_alone` and the pimdir `an_unreadable_body_is_an_entry_alone` fail; tests/exit.rs and the `card list` text unit test asserted the old row and now assert none.

CHANGELOG (Fixed; the Added line on `errors` corrected), MIGRATION.md (the `errors` sentence corrected). Spec: commands ("A listing names the items it could not read" modified), backends ("Each backend names the cards it could not read" modified). Left as it was: a pimdir body the store does not hold still lists its preview with no entry (backends, "A cache renames nothing it cannot push"), where calendula names a missing blob `item-unreadable`; `pimdir search` still fails whole on a body it cannot read, having no `errors`.
