---
cairn: change
id: pimdir-checks-and-enqueues-in-one-step
status: landed
created: 2026-10-30
---

# The pimdir backend checks and enqueues in one step

## Why

io-pimdir landed what-the-app-ports-found, a-query-only-index and store-audit-2026-10-30 on 2026-10-30 (local checkout, path-pinned).

- **The gate runs twice on a body write.** `stage` (src/pimdir/backend.rs) runs `PimdirProducer::check` before `enqueue_with_body` only to get the partial supports to log, since the enqueue answered none. io-pimdir's `enqueue_with_body_checked` answers them as `PimdirEnqueued::partial`, read with the body in hand. For an address book nothing is lost by the double run: io-pimdir derives no capability from a card's body (`capability::required_by_content` answers nothing outside calendars), so the two gates read the same.
- **`account check` no longer proves the staging lock.** io-pimdir's producer now takes `objects.lock` at its first staged body instead of at its open (store audit N8), so opening and dropping a producer reaches nothing the reader did not: `a_store_no_producer_opens_fails_the_check` fails (the check passed on a store whose `objects.lock` cannot be opened).
- **The spec's reason for refusing writes into an address book no source syncs is stale.** The drain no longer binds whichever source drains first for every action: an `update` of a card no source binds, in a collection no source syncs, is applied to the store alone and a `remove` retains it. An `add` still binds the draining source. The refusal stands (the plan of 2026-10-30), its reason is the `add` and the owner's §14.3 role.
- Nothing else of wave 3c reaches cardamum: `PimdirBlobs::reader` is only tested for `Some` (`version`), no drain report, no search, and `store-foreign` already maps (a reader now raises it for a `user_version` 0 database declaring tables, which cardamum reported as `store-uncreated`).

## What

- `stage` goes through `enqueue_with_body_checked` and logs `partial`; `delete_card`, which carries no body, keeps `check` then `enqueue`.
- `PimdirBackend::check` opens a blob writer on the producer and drops it uncommitted, which takes the staging lock and opens a temporary file in objects/, removed on drop.
- `a_store_refusal_carries_its_code` also opens a store whose `user_version` is set to 0 and expects `store-foreign`, no word of deleting it.
- backends.md: the write and duplicate requirements name `enqueue_with_body_checked`, the refusal requirement gives its current reason, the `account check` requirement says how the lock is reached.
