---
cairn: tasks
change: a-missing-pimdir-body-is-pending
---

# Tasks

- [x] src/pimdir/backend.rs: `listed_card` reads a blob io-pimdir answers `None` for as no body, no error; `get_card` answers `body-pending` for it as for an unfetched card; `BodyMissing` removed.
- [x] src/pimdir/index.rs: the `card_hit` doc follows; a hit whose body is gone lists.
- [x] Tests `a_missing_body_lists_as_usual_and_reads_as_pending` (replacing `a_missing_body_is_an_entry_alone`), `a_card_whose_blob_is_missing_has_no_version` and `a_hit_whose_body_is_gone_lists_as_usual` (replacing `a_hit_the_store_cannot_read_is_an_entry`), each seen failing on the HEAD sources; src/pimdir/search.rs fixture text.
- [x] Spec (backends, commands), CHANGELOG, MIGRATION.
