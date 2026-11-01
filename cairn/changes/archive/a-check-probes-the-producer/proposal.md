---
cairn: change
id: a-check-probes-the-producer
status: landed
created: 2026-10-30
---

# A check probes the producer

## Why

Since io-pimdir's N8 change (a-producer-proves-it-can-write, 2026-10-30, io-pimdir/cairn/log/2026-10-30-a-producer-proves-it-can-write.md), a producer takes the staging lock at its first staged body, so opening one proves nothing about the lock. `account check` works around it by opening a blob writer and dropping it uncommitted: that writes a temporary file into objects/ and waits on the lock (`PimdirLock::stage`) while a collector holds it, so the check hangs rather than reports. io-pimdir now offers `PimdirProducer::probe`, which takes and releases the staging lock without waiting and a write transaction, writing nothing.

## What

- `PimdirBackend::check` (src/pimdir/backend.rs) calls `producer.probe()` in place of the blob writer.
- A collector holding the staging lock fails the check at once with `store-busy`; nothing appears in objects/.
