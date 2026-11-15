---
cairn: tasks
change: the-codes-follow-the-family
---

# Tasks

- [x] src/carddav/client.rs `refusal_code`; test `a_carddav_refusal_carries_its_code` (403 and 409 `no-uid-conflict`, which asserted no code before) and `a_stale_version_is_a_precondition_failed`.
- [x] src/account/context.rs `addressbook_id`; tests/exit.rs `a_missing_default_addressbook_is_coded` (no code before); the `addressbook-default-missing` code joins the vocabulary.
- [x] src/vdir/backend.rs and src/jmap/backend.rs through `error::card_not_found`; tests/not_found.rs asserts the sentence and `sources`, the JMAP unit test the same.
- [x] src/error.rs `PIMDIR_CODES`, Himalaya's table; tests on the spelling and on the new failures (each `None` before).
- [x] Fold the delta, log, CHANGELOG.
