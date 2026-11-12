---
cairn: log
change: pimdir-release-api-followups
date: 2026-11-12
---

# The pimdir backend on the io-pimdir release API

The code landed and was folded on 2026-10-30, the log owed since: the body is staged through `PimdirProducer::stage_body` and read through `reader.blobs()`; every `PimdirError` reaches `--json` as its io-pimdir code through `ErrorCode::of_pimdir` (src/error.rs), in `PimdirClient::new`, `producer` and the enqueue path; no message of cardamum's advises deleting the store; a create naming a `UID` live or pending in the address book is refused through `enqueue_add` with `item-duplicate`, `-i` keeping the composer file. The io-pimdir entry of `[patch.crates-io]` moved from the local path to a git rev, now 5c18a02.

The release blockers, dropping `[patch.crates-io]` for the published io-pimdir 0.10.0, io-http 0.6, io-msgraph 0.7.1 and pimalaya-cli 0.3.2 and netting the CHANGELOG dependency entries, move to the-family-standards, phase published-dependencies.

Capabilities moved on 2026-10-30: backends ("A store refusal carries its code" replacing "A store from an earlier draft is refused", "A duplicate pimdir create is refused"), commands (the pimdir codes in the error code table).
