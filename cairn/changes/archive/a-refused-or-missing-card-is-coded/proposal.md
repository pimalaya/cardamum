---
cairn: change
id: a-refused-or-missing-card-is-coded
status: landed
created: 2026-11-15
---

# A refused or missing card is coded

## Why

Live testing of 2026-11-15 (reports/cardamum/carddav-icloud.md C1 and C2, contract.md, low):

- iCloud refuses a card without `N` with a 403 whose plain body is `VCARD parse error`. `card_write_error` read `vcard` in the body and printed the hint, while the code reader, exempting only a body naming a precondition element, coded it `permission-denied`: a script would look for an access problem. Task C29.
- `carddav get`, `msgraph contacts delete` and `gpeople connections get` on an absent card printed `code: null`, where `card read` prints `card-not-found`. Task C30.

## What

- A new code, `card-invalid`, by the family's noun rule (pimalaya-cli cairn/spec/error.md "A code names the domain's noun", `<noun>-invalid` as Himalaya's `attachment-invalid` and the family's `flag-invalid`): neither calendula nor Himalaya has a code for a refused body yet, so there was none to reuse. One predicate, `rejects_card`, read by the hint and by the CardDAV code reader: a 400 or a 403 whose body names `valid-address-data`, `supported-address-data` or the vCard. A 403 naming `need-privileges` stays `permission-denied`.
- The 400 a card without `UID` gets (iCloud's `null vcard or UID missing from vcard`) takes the hint and the code too.
- The protocol commands naming one card code a 404 (and 410, or Graph's malformed id) `card-not-found`, the server's message kept: `carddav get` and `delete`, `msgraph contacts get`, `update` and `delete`, `gpeople connections get`, `update` and `delete`, as calendula's `item_refusal` does.
