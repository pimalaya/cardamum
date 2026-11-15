---
cairn: tasks
change: a-missing-pimdir-body-is-an-entry
---

# Tasks

- [x] src/pimdir/backend.rs `listed_card`: a missing blob is a `BodyMissing` error; `card_from_item` (search) previews it with the warning.
- [x] Tests: `a_missing_body_is_an_entry_alone` (no entry before), `a_card_whose_blob_is_missing_has_no_version` asserts the entry.
- [x] Fold the delta, log, CHANGELOG.
