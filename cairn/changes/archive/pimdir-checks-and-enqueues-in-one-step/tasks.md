---
cairn: tasks
change: pimdir-checks-and-enqueues-in-one-step
---

# Tasks

- [x] `stage` through `PimdirProducer::enqueue_with_body_checked`, logging `partial` (src/pimdir/backend.rs); no body-dependent partial support exists for contacts, so no failing test
- [x] `PimdirBackend::check` takes the staging lock through an uncommitted blob writer (src/pimdir/backend.rs); `a_store_no_producer_opens_fails_the_check` fails on the io-pimdir of wave 3c and passes again
- [x] `a_store_refusal_carries_its_code` covers a database declaring tables at `user_version` 0 (`store-foreign`)
- [x] Fold the delta into [backends](../../spec/backends.md)
- [x] fmt, clippy, tests (every feature, default, pimdir alone); Cargo.lock unchanged
- [x] Log; land
