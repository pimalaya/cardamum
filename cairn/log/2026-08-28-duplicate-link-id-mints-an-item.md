---
cairn: log
change: duplicate-link-id-mints-an-item
landed: 2026-08-28
---

# pimdir: an address book may hold one UID twice, and nothing here may pretend otherwise

RFC 6352 §5.1 requires a card's `UID` to be unique in its collection and §6.3.2 forbids the `PUT` that would break it. Servers break it anyway, most often through a repeated import that re-adds a card the collection already holds, and until now the store could not represent it: one item per `(collection, link_id)`, so the second copy was frozen and mirrored nowhere. pimdir SPEC §9 changes that. `link_id` becomes the store's key rather than a restatement of the card's `UID`: the bare `UID` when it is free in the collection, a minted `dup:<hint>#<handle>` when the same source already binds it under another handle. Both cards are stored and both list.

Cardamum reads that store, so the invariant it never had to state, one card per `UID`, stops holding here. This is the tail of a cross-repo change carried by the same id through pimdir, io-replica, io-pimdir, io-webdav and Neverest.

**Nothing had to be fixed.** The audit went looking for a read resolving an identity to a row and found none: `get_card`, `update_card` and `delete_card` each parse a public `seq` and resolve it through `get_item`, `scan_items` pages on `(sort_key, seq)`, and nothing in the repo ever read `ambiguous_handles` or `ReplicaStatus::Ambiguous`, so io-pimdir dropping both is not breaking here. `create_card` is unchanged too, deliberately: it stages `card::derive`'s bare `UID` and a staged add colliding with a stored identity parks rather than mints (pimdir SPEC §15.3), which is the strict half of the same posture. The store is liberal in what it takes from a source and strict in what a producer may create.

**Where the duplication surfaces is the listing.** The preview a card projects from its stored summary emits `UID:`, so a `card list` may now legitimately show two rows carrying one of them. Nothing downstream folds them: `CardRow::from` builds one row per card positionally, the preview it renders reads `FN`, `EMAIL` and `TEL` and never a `UID`, and pagination slices the vector. The NOTE saying so sits on the `UID:` line inside `preview_vcard`, which is where the value enters a listing and the one place a reader would reach for a dedup.

**The argument docs were wrong, and now in principle.** `card read`, `card update` and `card delete` each called their positional a "Card UID". The shared id has never been one, being the href segment on CardDAV, the file stem on vdir and the store-assigned `seq` on pimdir, and a `UID` no longer identifies a card even in principle. The three now name the backend's own identifier, the one `card list` reports, in wording that stays protocol-agnostic for a shared-API doc.

**The dependencies point at the working trees.** None of io-pimdir, io-replica or io-webdav is released, and the `[patch.crates-io]` entries resolved to git revisions predating the change, so all three are patched by path to the sibling checkouts and the whole local suite is testable end to end. That is a state Cardamum cannot be published from, and repointing them at git or crates.io on release is the one task this change leaves open.

## Verification

- 56 tests green with `--features pimdir`, `cargo clippy --all-targets` clean on the default set and with `pimdir`, `cargo fmt`. The pimdir-only build is clean for the library; its `--all-targets` failure is the pre-existing one in the wizard's search tests, which reach for `AuthCaps` methods the CardDAV and JMAP features gate.
- `two_items_sharing_a_uid_project_two_distinct_cards` is the regression, a unit test over hand-built items in the idiom the rest of the backend uses, this repo having no store fixture. Two vCards carrying one `UID` derive the one bare link id, which is the reason the store mints at all; the two items it hands back, one under the bare key and one under a minted `dup:` one, project two distinct previews, both stating that shared `UID`, with no minted key leaking into the card a reader sees. The parking behaviour belongs to the store's owner and is tested in io-pimdir.

The [backends](../spec/backends.md) capability moved: one requirement added (a `UID` is not an address, with the scenario that a duplicated card lists twice) and one modified (`pimdir derivations match the sync engine`, which now says a derivation is what a write carries rather than a lookup).
