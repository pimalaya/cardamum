---
cairn: log
change: pimdir-release-alignment
landed: 2026-10-30
---

# The pimdir backend says what a sync will do, and refuses what it never would

The Phase A items of the pimdir stable-release plan (2026-10-30) that the current io-pimdir API (4c3353e) allows. Capabilities **backends** and **commands** moved.

## What landed

- **A write into an address book no source syncs is refused** before anything reaches the blob tree or the queue (`collection_sources` empty). The owner would have applied it as whichever source drained first, binding a collection that source never lists, and the card would never have been pushed.
- **A card whose blob is missing has no version**: no `etag` in `card list`, and no `--if-match` matches it, as for a card never fetched.
- **The kind is compared on its media type**, the part before `;`, as io-pimdir compares it; a `text/calendar` collection stays out.
- **`account check` opens a producer** after listing the address books, so a store whose staging lock is out of reach fails the check.
- **One view, one account**: `pimdir.account` unset derives the store's one account and refuses a store grouping several, naming them, as himalaya does. Closes C4 of cli-alignment-leftovers.
- **A queued write says it is queued**: `card create`, `card update` and `card delete` on pimdir print that the next sync carries the write, and `card create` and `card update` carry `queued` in `--json`, `false` on the other backends. A create still reports the link id, never a placeholder (himalaya/cairn/log/2026-08-27-pimdir-queue-visibility.md). Closes C3 of cli-alignment-leftovers.
- README, src/pimdir/mod.rs and the spec now say a staged create is not listed until a sync applies it; citations name STORAGE sections.

## What did not land

`pimdir status` with parked actions: cardamum has no `pimdir` namespace. The body written through `PimdirProducer::stage_body`, JSON codes from `PimdirError::code()` and the duplicate-create refusal wait on io-pimdir: change pimdir-release-api-followups.

## Tests

Six regression tests, each failing on the code before this change: a write into an unsynced address book, a missing blob's version, a parameterised kind, an unset account on a two-account store, a store no producer opens failing `account check`, and the queued wording of the three write commands. 73 tests pass with every feature, 41 with pimdir alone.
