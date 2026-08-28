# pimdir (local store): shared-command test report

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-pimdir and io-replica at their current git revisions)
- account: a throwaway `[accounts.pimdir]` in a temp config (`--config <scratch>/pimdir.toml`, never `~/.cardamumrc`) with `pimdir.root` under the scratch directory and `addressbook.default = contacts`
- date: 2026-08-09 (first run)
- method: **two fake local stores**. The first was created from scratch by pointing cardamum at an empty directory, which makes it initialise `pimdir.db` and `objects/`. The second was **seeded to look like one a sync engine populated**, by a ~130-line harness written for this run against io-pimdir and io-replica (through the same `[patch.crates-io]` revisions cardamum uses): it opens the store as a named remote source (`fastmail`) and writes placements carrying a sync base, one hydrated at `Full` level and one left at `Meta` level with a summary but no body, which is what a partial sync leaves behind. A companion inspector prints each placement's status and how its object compares to its base, which is what decides whether a sync would push a local change. Both stores were inspected throughout with the io-pimdir operator CLI and deleted with their temp configs at the end. pimdir has no protocol-specific surface by design, so this covers the shared API only.

## What the seeded store made reachable

cardamum's pimdir backend is an offline **cache** over a store a sync engine populates, and a store built by cardamum alone has one source, `local`, and no sync base, which puts its most interesting behaviour out of reach. The seeded store closes that gap: **partial hydration and the staged-write path are verified here for the first time**, and both work. Conflict handling and the three-way merge are out of scope here by design: they belong to the sync engine (Neverest), not to a client that stages a mutation and lets the engine reconcile it.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | empty directory, existing store, `-b pimdir`, `-b vdir` | ✅ `pimdir: OK`, initialising `pimdir.db` and `objects/` on first use; unconfigured `-b` bails |
| `addressbook list` | base, `--json` | ✅ collections as addressbooks |
| `addressbook create` | base | ✅ creates a `text/vcard` collection, confirmed with `pimdir collection list` |
| `addressbook update` | `-n`, `-d`, `-C`, no field flag | ⛔ every field bails with a message naming the reason (**PD1 fixed**); a flagless update bails first |
| `addressbook delete` | `-k` | ⛔ bails: "the pimdir backend cannot delete an addressbook; delete it on the server and sync, or remove the store directory" |
| `card create` | file, **unknown `-k`** | ✅ stores the body and writes the `v: 1` summary; an unknown collection now fails with "Addressbook `x` not found" (**PD3 fixed**) |
| `card read` | `<id>`, **unhydrated card**, absent id, non-numeric id, **missing blob** | ✅ returns the stored body; **an unhydrated card → "not downloaded yet (body not fetched); run a sync to hydrate it"**; an absent id → "Card `999` not found"; a non-numeric id → "Invalid card id `abc` (expected a number)"; a missing blob → "Body blob missing" |
| `card list` | base, **unhydrated card**, `-s`/`-p` paging, `--json`, **unknown `-k`**, **missing blob** | ✅ listing and paging, and **an unhydrated card previews from its `v: 1` summary** (name and email present, etag null); an unknown collection fails with "Addressbook `x` not found" (**PD2 fixed**); an item whose blob is missing previews from its summary and logs a warning (**PD4 fixed**) |
| `card update` | **synced card**, never-synced card, absent id | ✅ on a synced card the edit lands and is **staged for the next sync**; on a never-synced store the guard fires with a clear message; an absent id → "Card `999` not found" |
| `card delete` | **synced card**, never-synced card | ✅ on a synced card the placement becomes a **tombstone** the next sync pushes; on a never-synced store, same guard |

## Findings

### Bugs / issues

- **PD1 (the serious one): `addressbook update -n <name>` renamed the collection's id, not its display name: FIXED.** After `addressbook update -k contacts -n "Contacts"`, `addressbook list` reported `{"id": "Contacts", "name": "contacts"}`: the id took the new value and the display name kept the old one. The consequences followed the id: `card list -k contacts` returned an empty table with exit 0, the cards being reachable only under `-k Contacts`, and the account's `addressbook.default = "contacts"` silently stopped matching anything, so a plain `card list` reported zero contacts. Reproduced on both stores. The cause is a misuse: io-pimdir's `rename_collection(collection, new_id)` renames the identifier, as its parameter says, and the backend was passing it the display name. **Fix:** there is no local way to keep the flag, io-pimdir exposing no display-name setter (that column is the sync's to write), and a rename this backend cannot push is what it refuses to do elsewhere. `update_addressbook` now always fails, joining `delete_addressbook`, with a message naming the reason. The collection's kind was never affected, contrary to a first reading of the fixture: the seeded collections simply carry no kind, since only cardamum's own `addressbook create` sets one.
- **PD2: `card list -k <unknown>` printed an empty table and exited 0: FIXED.** An unknown collection was indistinguishable from an empty one, because io-pimdir's read seam answers an unknown collection with an empty page. A `known_collection` guard now resolves the addressbook first, and the command fails with "Addressbook `x` not found". Same defect as vdir's V7, still open there.
- **PD3: `card create -k <unknown>` created the collection: FIXED.** A typo in `-k` invented an addressbook and put the card in it, io-pimdir's write seam creating a collection on demand. The same guard now fails the write, and `pimdir collection list` confirms nothing is invented. Same defect as vdir's V6, still open there.
- **PD4: an item whose blob is missing listed as a blank row: FIXED.** `card_from_item` read the blob with `unwrap_or_default()`, so an item claiming an object whose file is gone yielded an empty vCard and rendered with blank FN, EMAIL and TEL, while `get_card` on the same item correctly reported "Body blob missing": the listing hid a broken store the read reports. This is **not** the "not fetched" case, which was already handled properly; it is the corrupt case, reproduced on both stores by removing the live object file of a `Full` item. **Fix:** the listing falls back to the summary preview, exactly as the unhydrated path does, and logs a warning naming the card, since a store missing a live blob is inconsistent and a blank row says nothing. `get_card` still refuses outright.

### Backend behaviour (not bugs)

- **The summary is written in the shape the spec requires.** After a `card create`, `pimdir item show` reports `meta: {"v":1,"uid":"cardamum-test-001","fn":"Alice Cardamum","emails":["alice@example.org"],"size":189}`, so the `v: 1` derivation cardamum writes matches what Neverest expects of the `text/vcard` kind, and the link id is `uid:<the vCard UID>`.
- **The never-synced write guard is exactly right.** `card update` and `card delete` on a store that was never synced as the configured source fail with "`contacts` was not synced as source `local`, so `1` cannot be edited here; set `pimdir.source` to the sync source and sync first". That is the guard turning a would-be silent no-op (an edit staged as a create that no sync would carry) into a clear instruction.
- **Partial hydration works exactly as specified.** In the seeded store, the item left at `Meta` level with no body lists with its name and email projected from the stored `v: 1` summary and a null etag, so a contact list reads correctly before a full sync, while `card read` on it refuses with "not downloaded yet (body not fetched); run a sync to hydrate it" rather than pretending the card is empty. This is the cache's headline behaviour and it holds.
- **Writes against a synced store are staged for the sync, through placement status rather than the action queue.** Verified by inspecting the placements after each write: an update leaves `status: Dirty` with the base still pointing at the pre-edit object, a delete leaves a `Tombstone`, and a create adds a placement with no base at all. All three are what a sync engine reads to know what to push. `pimdir queue list` stays empty throughout, which is correct: that queue is how a *producer* hands work to the store's owner, a different role from cardamum's here.
- **A cardamum-only store stages nothing, and says so.** With no sync base anywhere, the writes are purely local, and the never-synced guard above is what tells the user their edits will not reach a server.
- **Addressbook deletion and metadata are refused, not faked.** `addressbook delete` and `addressbook update -d/-C` both bail with a message naming the reason, rather than pretending.
- `pimdir check` reports the store consistent after the whole run: no orphan blob, no refcount drift, no dangling row.

### Observations

- **The released pimalaya-cli broken-pipe panic reproduced here**, in the io-pimdir operator CLI: `pimdir store info | head -6` died with `called Result::unwrap() on an Err value: Broken pipe`, the stack naming `pimalaya-cli-0.2.0/src/error.rs:25`. That is the unwrap fixed in the cli repo earlier today and not yet released, so every consumer still on the published crate carries it. It is racy, as it was in cardamum: it did not reproduce on demand in a loop straight afterwards.
- Card ids are the store-assigned public `seq` (`1`, `2`, …), scoped per collection, which is why a non-numeric id gets its own clear parse error.
- The seeding harness is the missing piece for testing this backend at all, and it is throwaway as it stands. Landing it as a test fixture, in cardamum or in io-pimdir, is what would make the partial-hydration and staged-write paths regression-testable rather than re-provable by hand.
- No real store was at risk: both instances lived in the scratch directory and were removed afterwards, and no pimdir store exists elsewhere on this machine.

## Verdict

The pimdir backend does the cache-specific things well, and the seeded store proves the two that matter most and had never been exercised: an unhydrated card **previews from its stored summary** in a listing and refuses to be read with a message telling the user to sync, and a write against a synced collection is **staged for the next sync** as a dirty placement, a tombstone or a baseless create, exactly as an engine would expect to find it. Around that, it initialises a store, writes a Neverest-compatible `v: 1` summary, refuses what it cannot do (addressbook deletion, description, color) with messages that say why, and guards edits on a never-synced store with the clearest error in the product.

Four defects, all fixed the same day and re-verified against both stores: **PD1**, where renaming an addressbook rewrote its id and quietly stranded every card under the old one, including the configured default, and **PD2 to PD4**, the same "write through without checking, then report nothing amiss" family found on vdir the same day, where it is still open. Conflict handling and the three-way merge are the sync engine's concern, not this client's.

---

# pimdir (local store): second run, 2026-08-28

- cardamum: v0.2.0 `--features pimdir` (working tree; io-pimdir, io-replica and io-webdav at their current git revisions)
- account: a throwaway `[accounts.posteo-local]` in a scratch config (`--config <scratch>/cardamum.toml`, never a config in `~`), `pimdir.root` pointing at the real Neverest store `~/.local/state/neverest/posteo`
- date: 2026-08-28
- method: **read-only against a live store**, the first run of this backend against one a sync actually populated. The store holds `carddav/default` with 155 vCards from Posteo, all hydrated at `Full`, alongside twenty-odd `message/rfc822` collections. Nothing here writes: every card verb exercised is a read, the three `addressbook` verbs refuse before touching anything, and the store's queue was confirmed empty before and after. The seeded-store harness of the first run is what covers the write paths, and it has not been re-run against the queue model yet.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `pimdir: OK` |
| `addressbook list` | base, `--json` | ✅ one addressbook, `carddav/default`; the mail and calendar collections are filtered out by kind |
| `addressbook create` | base | ⛔ bails: the collection row is the sync's to write |
| `addressbook update` | `-n` | ⛔ bails, unchanged from the first run |
| `addressbook delete` | `-k` | ⛔ bails, unchanged from the first run |
| `card list` | base, `--page-size`, `--page` (first, middle, last), `--json` | ✅ 155 cards in display-name order, paging total across the boundaries, the last page short |
| `card read` | `<id>`, absent id, non-numeric id | ✅ the stored vCard byte for byte; an absent id → "Card `99999` not found in `carddav/default`"; a non-numeric one → "Invalid card id `abc` (expected a number)" |
| `card list` / `card read` | unknown `-k` | ⛔ "Addressbook `nope` not found", the PD2 guard holding |
| any command | `pimdir.root` naming no store | ⛔ "Open pimdir store `…`: unable to open database file", rather than the empty listing an auto-created store used to give |
| `card list` | **run while another process holds `owner.lock`** | ✅ lists normally, which is the point of the role change |

## Findings

### Bugs / issues

- **PD5: `card list` never showed a `TEL` whose line sat above the card's `EMAIL`: FIXED.** The shared listing's `vcard_preview` chained its three reads as `if FN … else if email.is_none() … else if tel.is_none() …`, so any line that failed the EMAIL read consumed the iteration and the TEL read was only ever reached once an EMAIL had already been found. Posteo writes `TEL` before `EMAIL`, so the column was empty for all 155 cards, and for the great majority, which carry a phone and no mail, empty on every row. This is shared code, so **every backend was affected**, not only pimdir; it went unnoticed because the earlier fixtures happen to order `EMAIL` first. The three reads are now independent.
- **PD6: the link id disagreed with the sync engine: FIXED.** Cardamum derived `uid:<UID>` while Neverest and io-pimdir's own conventions derive the bare `UID`. The live store confirms the bare form (`018662f1-3c96-…`). A card staged by Cardamum would have linked as a second identity, stored a second body and synced as a duplicate contact. The derivations now delegate to `io_pimdir::conventions::card`, so there is one implementation rather than two agreeing by hand.
- **PD7: the body hash was a digest of Cardamum's own choosing: FIXED.** The spec called for "the shared 128-bit FNV-1a digest rendered as 32 hex chars"; the live store names its bodies in base32 under the algorithm recorded in `store_meta.hash_algo`. A body written under the wrong name is a body no read finds. The hash now comes from the store's handle.
- **PD8: Cardamum took the owner role: FIXED.** io-pimdir 0.3 makes the owner exclusive, so `cardamum card list` and a sync could not run at the same time, either one failing with `PimdirError::Owned`. Cardamum is a reader plus a producer, and now holds those two handles. Verified by holding `owner.lock` from another process and listing anyway.

### Backend behaviour (not bugs)

- **The pending overlay is on.** The reader is built `with_pending`, so a staged `update` or `remove` reads back before the sync applies it. Not exercised here: exercising it means writing.
- **A queued create has no public id.** `create_card` reports the card's link id instead of a `seq`, the `seq` not existing until the owner applies the action. Deliberate, and recorded in the spec.
- **A queued action carries no sort key.** io-pimdir leaves the key to the sync that pushes the create, so a card staged here sorts at the head of the list until the next sync resolves it.
- **The queue stayed empty throughout**, checked with `pimdir queue list` before and after, which is the proof that this run wrote nothing.

## Verdict

The read side is correct against a real 155-card store: listing, ordering, paging, byte-faithful reads, and every error path naming what is wrong. Four defects found, all fixed, and two of them (**PD6**, **PD7**) were silent data-duplication bugs that only a live store could reveal, since both are about agreeing with what a sync engine already wrote. **PD8** is the structural one: the backend held the one handle the format says a frontend must not hold.

The write paths are unexercised on this run by design, the store being a production account. Re-running the seeded-store harness of 2026-08-09 against the queue model is what would close that, and it is the outstanding work on this backend.
