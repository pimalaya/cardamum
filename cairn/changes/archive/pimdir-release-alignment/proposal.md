---
cairn: change
id: pimdir-release-alignment
status: landed
created: 2026-10-30
---

# The pimdir backend says what a sync will do, and refuses what it never would

## Why

The pre-freeze review of 2026-10-30 (pimdir stable-release plan, Apps section, Phase A common and cardamum items) found the pimdir backend reporting writes that no server will ever see, and a few reads that claim more than the store holds. Each item below was re-checked against the code on 2026-10-30, io-pimdir at 4c3353e.

- **A write into an address book no source syncs is accepted** (src/pimdir/backend.rs:286-299 filters by kind only). The owner's drain applies an `add` there as whichever source drains first and binds it to a collection that source never syncs, so the card is never pushed while cardamum said it was created. A device-kept collection (STORAGE §14.3) is the owner's `put_item` to write, never a producer's.
- **A card whose blob is missing still reports an etag** (src/pimdir/backend.rs:365-369), against the spec's "a card whose body is not local SHALL report none", and an `--if-match` built from it passes a guard for a body nobody can read.
- **README.md:239 promises a staged write lists straight away**, and so does src/pimdir/mod.rs:14-15: a staged update or removal does, a staged create does not, the overlay folding only writes that address an existing item.
- **The kind is compared exactly with `text/vcard`** (src/pimdir/backend.rs:297) while io-pimdir compares the media type before any `;` parameter, so a parameterised kind it summarizes as contacts is not an address book here.
- **`account check` opens only the reader** (src/account/check.rs:216-219): a store whose staging lock the producer cannot take checks OK, then fails on the first write.
- **`pimdir.account` unset reads every collection of a store several accounts share** (src/pimdir/client.rs:39-50), mixing their address books in one view; himalaya derives the account from a one-account store and refuses a multi-account one (himalaya/src/pimdir/client.rs:225). One CLI view is one account; shared views are for the Android and Linux apps. This closes C4 of cli-alignment-leftovers.
- **A queued write prints "successfully created/updated/deleted"** (src/shared/card/create.rs:133, update.rs:162, delete.rs:40) for a change that only reaches the server on the next sync. This closes C3 of cli-alignment-leftovers.
- **Queued creations follow the no-placeholder-id rule** (himalaya/cairn/log/2026-08-27-pimdir-queue-visibility.md): a create returns the link id it staged, never a `q<id>`, `0` or `""`. cardamum already returns the link id; the output now says it is queued and what the id is.
- **Citations drift**: cairn/spec/backends.md cites "pimdir SPEC" four times, now STORAGE.md; src/pimdir/backend.rs cites §9.2 (accounts) for kinds and §14 for blob durability.

`pimdir status` with parked actions does not apply: cardamum has no `pimdir` namespace, the `pimdir` binary of io-pimdir being the store's operator surface.

## What

- pimdir `create_card`, `update_card` and `delete_card` refuse, before anything is written, an address book no source syncs (`collection_sources` empty), naming why.
- A card whose blob is missing reports no `etag` and matches no `--if-match`.
- The kind filter compares the media type before `;`.
- `account check` opens and drops a producer after listing the address books.
- `pimdir.account` unset derives the store's one account and refuses a store holding several, naming them.
- `card create`, `card update` and `card delete` on pimdir say the write is queued for the next sync; `card create` and `card update` gain a `queued` boolean in `--json`, `false` on every backend whose writes reach the server as they are made.
- README, src/pimdir/mod.rs and the spec say a staged create is not listed until a sync applies it.
- Citations name STORAGE sections.

What waits on the next io-pimdir API (`stage_body`, `PimdirError::code()`, a duplicate-create refusal) is the change pimdir-release-api-followups.
