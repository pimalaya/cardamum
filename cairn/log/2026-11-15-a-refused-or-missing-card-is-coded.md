---
cairn: log
change: a-refused-or-missing-card-is-coded
date: 2026-11-15
---

# A refused or missing card is coded

A vCard a CardDAV server refuses is coded `card-invalid`, a new code, where iCloud's 403 `VCARD parse error` read `permission-denied` beside a hint saying the opposite; and the protocol commands naming one card code a 404 `card-not-found`, as `card read` does. Tasks C29 and C30 of cli-alignment-2026-11-15, live findings C1 and C2 (reports/cardamum/carddav-icloud.md). Capabilities **backends** and **commands** moved.

- **The code**: `card-invalid`, by pimalaya-cli's noun rule (`<noun>-invalid`, as Himalaya's `attachment-invalid`); calendula and Himalaya have no code for a refused body, so none could be reused. calendula's CalDAV twin (`valid-calendar-data`) would be `item-invalid`.
- **src/carddav/client.rs**: `rejects_card` (a 400 or a 403 naming `valid-address-data`, `supported-address-data` or the vCard) read by `status_code` after `need-privileges`; `card_refusal`. **src/carddav/backend.rs**: `card_write_error` on the same predicate, so a 400 refusing a card without `UID` takes the hint too.
- **src/msgraph/client.rs** `contact_refusal`, **src/gpeople/client.rs** `person_refusal`: on `carddav get` and `delete`, `msgraph contacts get`, `update` and `delete`, `gpeople connections get`, `update` and `delete`, the server's message kept.
- **Evidence**: `a_refused_vcard_keeps_the_answer_as_its_cause` (403 precondition, 403 `VCARD parse error`, 400; create and update) read `None` and `permission-denied` before; `a_specific_command_on_a_missing_card_is_coded`, `..._contact_is_coded`, `..._person_is_coded` run the commands over scripted 404s and read `null` before.

CHANGELOG (Added, Changed), MIGRATION.md (the code list). Spec: backends ("A server's refusal carries its code"), commands ("A JSON error carries a stable code").
