---
cairn: log
change: a-missing-pimdir-body-is-pending
date: 2026-11-18
---

# A missing pimdir body is pending

Audit line TRI-3, the owner's ruling of 2026-11-18: a pimdir body the blob tree lacks is not hydrated and not an error (pimdir GUIDE §14), as Himalaya reads it. Capabilities **backends** and **commands** moved.

- **src/pimdir/backend.rs**: `listed_card` takes io-pimdir's `None` for a blob gone or failing its name as no body, the card listed with its summary preview and no version, where it was a `card-unreadable` entry (`Body blob missing from the store`) and exit 3. `get_card` answers such a card `body-pending`, ``Card `<id>` in `<addressbook>` is not downloaded yet (body not fetched); run a sync to hydrate it``, as an unfetched one, where it failed uncoded ("Body blob missing for ..."). `BodyMissing` is gone. A body failing with an I/O error stays `card-unreadable`.
- **src/pimdir/index.rs**: a `pimdir search` hit whose body is gone lists as usual.
- **Evidence**: `a_missing_body_lists_as_usual_and_reads_as_pending`, `a_card_whose_blob_is_missing_has_no_version` (src/pimdir/backend.rs) and `a_hit_whose_body_is_gone_lists_as_usual` (src/pimdir/index.rs) fail on the HEAD sources (an `errors` entry `card-unreadable`, `Body blob missing from the store`; Bob Durand missing from the hits) and pass after.

CHANGELOG (Changed, breaking; the Fixed line it reverses removed, the `pimdir search` line amended). MIGRATION (The pimdir store). Spec: backends ("Each backend names the cards it could not read", "pimdir is a cache, not a server", one scenario), commands (the `card-unreadable` and `body-pending` rows, "pimdir searches cards through the store's index" and its scenario). calendula takes the twin change.
