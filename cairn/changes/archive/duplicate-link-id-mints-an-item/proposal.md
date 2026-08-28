---
cairn: change
id: duplicate-link-id-mints-an-item
status: landed
created: 2026-08-28
---

# An address book may hold one UID twice, and the store now says so

> Cross-repo change, same id in eight repositories. This crate is at the end of the chain, and its part is an audit plus a bump: **pimdir** → **io-replica** → **io-pimdir** → **io-webdav** → **neverest** → **himalaya**, **cardamum** (here), **calendula**.

## Why

RFC 6352 §5.1 requires a vCard `UID` to be unique inside its address book collection, and §6.3.2 forbids the `PUT` that would break it. Servers break it anyway, most often through a repeated import (Google, iCloud, a CSV re-run) that re-adds a card the collection already holds. Verified on the calendar side of the same account, 2026-08-28: four identities held under two hrefs each.

Until now the store could not represent that: one item per `(collection, link_id)`, so a second copy was frozen and mirrored nowhere. pimdir SPEC §9 changes it. `link_id` becomes the store's key rather than a restatement of the card's `UID`: the bare `UID` when it is free in the collection, a minted `dup:<hint>#<handle>` when the same source already binds it under another handle. Both cards are stored and both list.

Cardamum reads that store, and the invariant it never had to state (one card per `UID`) stops holding.

## What

- **Nothing may treat a body's `UID` as an address.** A read resolving a `UID` to the card, or re-deriving one to look a row up, is now wrong: the key stays unique, but it is no longer what the body says. The backend already shows and accepts the public `seq`, so the exposure is limited to whatever resolves an identity directly.
- **`create_card` is unchanged, deliberately.** It stages an `add` carrying `card::derive`'s bare `UID` and returns it, and a staged add colliding with a stored identity still parks (pimdir SPEC §15.3). Minting is what reading a server requires; a locally authored card that collides is a producer error and parks, which is the strict half of the same posture.
- **The derivation is unchanged.** `io_pimdir::conventions::card` keeps producing the bare `UID` with nothing prepended, and this crate keeps using it. What changes is that the store may key a card it received from a source differently from what the derivation produces, so a derivation is no longer a lookup.
- **Two cards under one `UID` display as two cards.** No badge, no dedup, no merge prompt. The merged-view work reads placements, and a consumer that wants to offer a merge does so from what the store reports, which is the format's existing position ("Multiplicity is reported, never resolved").

## Scope / non-goals

- **No repair and no merge offer** in this change. Cardamum already has a projection story for cards; offering to merge two copies of one `UID` is a feature, not a consequence of this, and would be proposed on its own.
- **No listing change.** Two rows list as two rows, in the store's own order.
- **No queue change.** Parking on a colliding local add is the existing rule and stays.
