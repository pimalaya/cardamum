---
cairn: tasks
change: a-refused-or-missing-card-is-coded
---

# Tasks

- [x] src/error.rs `CardInvalid`; src/carddav/client.rs `rejects_card` in `status_code`; src/carddav/backend.rs `card_write_error` on it.
- [x] src/carddav/client.rs `card_refusal`, src/msgraph/client.rs `contact_refusal`, src/gpeople/client.rs `person_refusal`, on the eight commands.
- [x] Tests: `a_refused_vcard_keeps_the_answer_as_its_cause` (a 403 `VCARD parse error` read `permission-denied` before), `a_specific_command_on_a_missing_card_is_coded`, `a_specific_command_on_a_missing_contact_is_coded`, `a_specific_command_on_a_missing_person_is_coded`, each failing on the code before.
- [x] CHANGELOG, MIGRATION's code list; fold the delta; log.
