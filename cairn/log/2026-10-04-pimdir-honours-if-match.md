---
cairn: log
change: pimdir-honours-if-match
landed: 2026-10-04
---

# pimdir honours `--if-match`, and a delete can be gated

The pimdir backend's `update_card` and `delete_card` refuse a stale `if_match` before queueing anything, with an error starting `Precondition failed:`; the version is the `etag` it already reported, the store's hash of the card's body with the pending queue folded in. `card delete` takes `--if-match`: CardDAV sends it, pimdir checks it, vdir, jmap, msgraph and gpeople refuse it.

Capabilities moved: **backends** (a pimdir card's version is its body hash; pimdir writes are staged queue actions, modified) and **commands** (a delete can be gated on a version).
