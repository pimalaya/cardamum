---
cairn: tasks
change: a-check-probes-the-producer
---

# Tasks

- [x] Test: a held staging lock fails the check with `store-busy`, nothing in objects/ (hangs on the blob-writer check).
- [x] `PimdirBackend::check` probes the producer; the blob writer goes.
- [x] CHANGELOG `account check` bullet.
- [x] Fold the delta into [backends](../../spec/backends.md); log; land.
