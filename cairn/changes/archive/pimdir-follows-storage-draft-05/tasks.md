---
cairn: tasks
change: pimdir-follows-storage-draft-05
---

# Tasks

- [x] `update_card` states `if_match` on the queued `update`: the version it was gated on, else `None` (src/pimdir/backend.rs); test `an_edit_whose_body_a_sync_replaced_is_parked`, failing first
- [x] `stage` through `PimdirProducer::enqueue_with_body`, dropping `stage_body` and the `enqueue_add` dispatch (src/pimdir/backend.rs); `a_create_naming_a_held_uid_is_refused` asserts objects/ unchanged, failing first
- [x] `store-foreign` mapped, `store-older` dropped (src/error.rs); the code test covers `Foreign`
- [x] No exhaustive match on `PimdirError` and no "delete the store" wording for the new variants
- [x] `an_address_book_is_told_by_its_media_type` declares a bare kind, the store now refusing one with parameters; the parameter case, which a store an earlier io-pimdir wrote still carries, is checked on `is_card_kind`
- [x] Fold the delta into [backends](../../../spec/backends.md) and [commands](../../../spec/commands.md); README and CHANGELOG
- [x] Log; land
