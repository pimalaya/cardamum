# Memberships as structural addressbook data

Group memberships are **not** a card property. A contact's Google group memberships (and the equivalent elsewhere) are its **addressbook memberships** — which addressbook(s) the card belongs to — so they are surfaced structurally, at the addressbook layer, rather than projected onto the vCard.

Consequently `X-GOOGLE-MEMBERSHIP` is no longer minted by the People projection (`google/project.rs`). It stays on the *consumed* list so that lines written by earlier projections are dropped on the way back rather than stashed into the [custom-data](custom-data.md) remainder, keeping old documents from re-injecting stale membership lines.

This matches cardamum-android, where the merged contact-first view is assembled over per-replica storage and a card can appear under several addressbooks at once.
