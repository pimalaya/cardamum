---
cairn: change
id: pimdir-release-api-followups
status: active
created: 2026-10-30
---

# The pimdir backend moves onto the io-pimdir release API

## Why

The pimdir stable-release plan of 2026-10-30 (Apps section, Phase A) gives cardamum items that wait on API io-pimdir has not shipped yet, and release blockers that wait on the dependency chain being published. pimdir-release-alignment took everything the current API (io-pimdir 4c3353e) allows; this change holds the rest so none of it is lost.

- **The body is written through the reader's blob handle** (src/pimdir/backend.rs, `stage`), under the producer's staging lock but outside the producer. io-pimdir's the-producer-stages-its-own-body adds `PimdirProducer::stage_body`, hashing and writing under the lock, and makes the reader's blobs read-only.
- **Store refusals carry no JSON code** (src/pimdir/client.rs, `new` and `producer`, stringify every `PimdirError`), and io-pimdir's `Stale` Display advises to "delete the store and let it resync", which cardamum relays and its own spec repeats (cairn/spec/backends.md, "A store from an earlier draft is refused"). io-pimdir's errors-carry-a-stable-code adds `PimdirError::code()` and drops the advice; `Version` says upgrade. The §15.6 gate refusal (`Unsupported`) and `Busy` deserve their codes too.
- **A duplicate create prints success and is parked at drain** (STORAGE §15.3): a second `card create` of a `UID` live or already pending in the address book is queued, and with `-i` the composer file is deleted on that false success (src/shared/card/create.rs:89-93). io-pimdir's `PimdirProducer::enqueue_add` refuses it in the enqueue transaction, so every app refuses alike.
- **Release blockers**: the `[patch.crates-io]` git pins (Cargo.toml), the `io-pimdir` requirement following the version io-pimdir is released under, and CHANGELOG entries naming released versions instead of git revisions.

## What

- `stage` writes the body with `PimdirProducer::stage_body` and drops `PimdirClient.blobs` for `reader.blobs()` on reads.
- `PimdirClient::new` and `producer` map `PimdirError::code()` to cardamum `ErrorCode` values (`--json` `code`), with no "delete the store" wording in cardamum's own messages or spec; the spec requirement "A store from an earlier draft is refused" says what a newer, stale or uncreated store asks of the user.
- `create_card` stages its `add` through `enqueue_add`, which refuses a link id the address book already holds, live or pending, appending nothing, reported with the code `item-duplicate`; a regression test fails first on the current code. The body is staged first, since the §15.6 gate reads it from the tree, and a refused one is left unpinned for the collector.
- Once the dependency chain is published by the user: drop the git pins, set the `io-pimdir` requirement, rewrite the CHANGELOG dependency entries as a net diff naming released versions.
