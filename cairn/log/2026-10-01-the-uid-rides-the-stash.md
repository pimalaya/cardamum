---
cairn: log
change: the-uid-rides-the-stash
landed: 2026-10-01
---

# The UID rides the stash

Graph and People have no UID field. Cardamum minted the card's UID from the Graph id or the People resource name on read and dropped the incoming UID on write, which was harmless for a standalone client and broke any sync matching the same person on another server by UID: the person had two identities, and a two-way sync re-duplicated the address book every run.

The projections in io-msgraph and io-gpeople now leave the UID in the stash both already carry, and read it back from there, minting from the provider id only for a contact the stash holds none for. Nothing new is stored server-side, and no new `$expand` is needed. A contact created by the provider, or written by an earlier Cardamum, reads back exactly as before.

Until io-msgraph and io-gpeople release, Cargo.toml patches both to their local checkouts.

Capability moved: **projection** (the UID rides the stash).
