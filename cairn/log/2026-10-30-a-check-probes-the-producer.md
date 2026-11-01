---
cairn: log
change: a-check-probes-the-producer
landed: 2026-10-30
---

# A check probes the producer

`PimdirBackend::check` (src/pimdir/backend.rs) opens a producer and calls `PimdirProducer::probe` (io-pimdir a-producer-proves-it-can-write, 2026-10-30) in place of opening and dropping a blob writer. The probe takes and releases the shared staging lock without waiting and the database's write lock, writing nothing, so a collector holding the staging lock fails the check at once with `store-busy`, where the blob writer waited on it (`PimdirLock::stage`) and wrote a temporary file into objects/.

Test: `a_held_staging_lock_fails_the_check_as_busy` (src/account/check.rs) holds objects.lock exclusively, as a collector does, and checks the check fails with `store-busy` and leaves objects/ empty. On the blob-writer check it hung ("has been running for over 60 seconds", killed by a timeout). `a_store_no_producer_opens_fails_the_check` and `a_check_leaves_no_body_behind` pass unchanged.

Capabilities moved: **backends** (account check opens every role the backend takes, modified).
