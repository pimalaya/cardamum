---
cairn: tasks
change: pimdir-release-api-followups
---

# Tasks

On the io-pimdir API of 2026-10-30 (the-producer-stages-its-own-body, errors-carry-a-stable-code), built against the local checkout through a temporary `[patch.crates-io]` path:

- [x] `stage` through `PimdirProducer::stage_body`; drop `PimdirClient.blobs` for `reader.blobs()` (src/pimdir/client.rs, src/pimdir/backend.rs)
- [x] Map `PimdirError::code()` to `ErrorCode` (`ErrorCode::of_pimdir`, src/error.rs) in `PimdirClient::new`, `producer` and the enqueue path (`refusal`, src/pimdir/client.rs); codes in cairn/spec/commands.md's table; tests: every mapped variant prints io-pimdir's code (src/error.rs), an uncreated store and a §15.6 refusal end to end (src/pimdir/backend.rs)
- [x] No "delete the store" in cardamum's messages; "A store from an earlier draft is refused" replaced by "A store refusal carries its code" in cairn/spec/backends.md
- [x] Refuse a duplicate create (link id live or pending in the address book) through `enqueue_add`, with `item-duplicate`; regression tests failing first (`a_create_naming_a_held_uid_is_refused`, `a_composed_duplicate_keeps_its_file`); `-i` keeps the composer file on that refusal. The body staged before the refusal, left unpinned for the collector, is gone since pimdir-follows-storage-draft-05 (2026-10-30): `enqueue_with_body` refuses before it writes the body
- [x] Fold the delta into [backends](../../spec/backends.md) and [commands](../../spec/commands.md) (2026-10-30); README and CHANGELOG entries

Waiting on the io-pimdir changes of 2026-10-30 being pushed:

- [ ] Return the io-pimdir `[patch.crates-io]` entry from the local path to a git rev (Cargo.toml, the io-pimdir entry of Cargo.lock)

Waiting on the user publishing the dependency chain:

- [ ] Drop `[patch.crates-io]` (io-http, io-msgraph, io-pimdir, io-webdav, pimalaya-cli) and set the `io-pimdir` requirement to the released version (Cargo.toml)
- [ ] CHANGELOG [Unreleased]: dependency entries name released versions, no git revisions

- [ ] Log; land
