---
cairn: log
change: pimdir-checks-and-enqueues-in-one-step
landed: 2026-10-30
---

# The pimdir backend checks and enqueues in one step

cardamum builds against io-pimdir's what-the-app-ports-found, a-query-only-index and store-audit-2026-10-30 (2026-10-30, the local checkout still path-pinned; Cargo.lock unchanged). Capability **backends** moved.

## What landed

- **One gate per body write** (src/pimdir/backend.rs, `stage`): `PimdirProducer::enqueue_with_body_checked` replaces `check` then `enqueue_with_body`, and its `partial` is logged as before. `delete_card`, carrying no body, keeps `check` then `enqueue`. No partial support is lost or gained for an address book: io-pimdir derives no capability from a card's body (`capability::required_by_content` answers nothing outside calendars), so no test can fail on the old pair and none was added.
- **`account check` takes the staging lock again** (src/pimdir/backend.rs, `check`): io-pimdir's producer now takes `objects.lock` at its first staged body (store audit N8), so opening a producer proved nothing the reader had not. The check opens a blob writer and drops it uncommitted, which takes the lock and removes its temporary file.
- **The refusal of writes into an address book no source syncs keeps its rule, with its current reason**: a queued `add` there still binds whichever source drains first; an `update` of a card no source binds now applies to the store alone and a `remove` retains it, neither reaching a server.
- Nothing else of wave 3c reaches cardamum: `PimdirBlobs::reader` is only tested for `Some`, no drain report is read, no search runs, and `store-foreign`, which a reader now raises for a `user_version` 0 database declaring tables, was already mapped.

## Tests

- `a_store_no_producer_opens_fails_the_check` failed on io-pimdir's N8 (`assertion failed: connect_pimdir(&config).is_err()`) and passes again.
- `a_check_leaves_no_body_behind`: the check passes on a healthy store and leaves objects/ empty.
- `a_store_refusal_carries_its_code` also sets a real store's `user_version` to 0 and expects `store-foreign` with no word of deleting it (io-pimdir answered `store-uncreated` there before).
- 80 tests pass with every feature and with the default ones, 48 with pimdir alone; clippy is clean with every feature, the default ones, and each backend alone with `rustls-ring`.
