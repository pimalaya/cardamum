# CardDAV on iCloud: shared and specific test report

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-webdav and vcard-rs at their current git revisions)
- account: `icloud` (`carddav.home = https://contacts.icloud.com/…/carddavhome/`, HTTP Basic, `addressbook.default = card`)
- date: 2026-08-09 (re-run covering both the shared and the specific surface; first run 2026-07-18)
- method: iCloud forbids creating addressbooks over CardDAV (I1), so per the [golden-rule fallback](provider-test-plan.md) everything ran inside the account's single existing book (`card`) on uniquely-marked throwaway contacts (`FN:Cardamum *`, `UID:cardamum-icloud-*`), each deleted afterwards. The book's card ids were captured before the run and diffed after: the account is byte-for-byte back to its pre-run state, and no pre-existing card was read, printed or written. Mutating the real addressbook's own metadata was deliberately not tested, since the account has no throwaway collection to do it in.

## Results: shared API

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `carddav: OK` |
| `addressbook list` | base, `--json`, `abook ls`, `-b carddav`, `-b jmap` | ✅ one book (`card`); unconfigured `-b` bails |
| `addressbook create` | base | ⛔ **403: iCloud forbids MKCOL (I1)** |
| `addressbook update` | unknown id, no field flag | ✅ after the I5 fix below: an unknown book fails, and a flagless update bails client-side. The real book was deliberately not mutated |
| `addressbook delete` | unknown id, missing `-k` | ⛔ 403 "Cannot delete carddav home"; missing `-k` → clap error, exit 2 |
| `card list` | base, `-k`, `-s`/`-p` paging, page past the end, `--json` (etag), no-`-k` default, bad `-k` | ✅ all pass; bad `-k` → 400 with the offending URI, exit 1 |
| `card create` | file, stdin `-`, vCard 3.0, vCard 4.0, no `UID`, no `N` | ✅ 3.0 and 4.0 both accepted and stored verbatim (I2); a missing `UID` → 400, a missing `N` → the actionable hint (I3) |
| `card read` | `<id>`, `--json`, bogus id | ✅ pass; bogus id → 404, exit 1 |
| `card update` | file, stdin `-`, `--if-match` (fresh + stale), absent id | ✅ after the guard fix below; stale `--if-match` → 412, absent id → "not found", nothing created |
| `card delete` | `<id>`, `card rm`, absent id | ✅ card gone; ⚠️ an absent id reports success, exit 0 (**I6**, server-side) |

## Results: specific API (`carddav …`)

| Command | Variants | Result |
| --- | --- | --- |
| `discover` | base | ✅ server and addressbook-home-set; `principal: (unresolved)`, the documented best-effort on a `home`-configured account |
| `propfind` | no arg (books + CTag/sync-token), `<addressbook>` (ids + ETags), `--json` | ✅ both; iCloud's CTag and sync-token are the same opaque base64 blob |
| `mkcol` | base | ⛔ **403, empty body (I1)** |
| `proppatch` | unknown collection, no field flag | ✅ after the I5 fix below; a flagless patch bails client-side |
| `get` | `<ab> <id>`, `--json`, after delete | ✅ raw vCard plus ETag; after delete → 404 |
| `put` | create at a chosen path, `--if-match` fresh, `--if-match` stale, `--if-none-match '*'` on an existing card, `--if-match '*'` | ✅ create and guarded update; stale → 412; `--if-none-match '*'` on a taken path → 403; **`--if-match '*'` → 412 even for a card that exists (I4)** |
| `report query` | `<ab>` | ✅ ids and ETags |
| `report multiget` | `<ab> <id…>` | ✅ batch fetch in one round-trip |
| `report sync` | initial (→ token), `--sync-token` after a write | ✅ **incremental sync works**: the changed href and its ETag, plus the next token |
| `report raw` | `addressbook-query`, `sync-collection` with `--depth 0`, malformed XML | ✅ parsed multistatus, sync-token surfaced; malformed → 400 "Didn't understand the report" |
| `delete` | `<ab> <id>`, `--if-match` stale, `--if-match` on a collection | ✅ card deleted; **a stale `--if-match` deletes anyway (I7)**; `--if-match` on a collection → friendly client-side bail |

## Findings

### Bugs / issues

- **The `If-Match: *` guard introduced this morning broke every update on iCloud: FIXED.** The [carddav-write-guards](../../log/2026-08-09-carddav-write-guards.md) change stopped `card update` creating a card under an unknown id by sending `If-Match: *` when the caller passed no ETag, which Fastmail honours. iCloud answers `412 Persist Error` to the wildcard **even for a card that is right there**, so `card update` failed outright here. Isolated against the server: an explicit correct ETag succeeds, a raw `put --if-match '*'` on an existing card fails, and the same put unguarded succeeds, so the wildcard is the only thing refused. **Fix:** the carddav adapter now reads the card's current version first and guards the write with the ETag it returns (see [carddav-update-guard-etag](../../log/2026-08-09-carddav-update-guard-etag.md)). Re-verified on both providers: the update lands, an unknown id fails on the read as "not found" (a better message than the bare 412 it used to give), and an explicit `--if-match` still guards. The guard is still needed here: a plain PUT to an unknown path does create a card on iCloud, verified with a fresh UID.

- **I5: a PROPPATCH the server never applied was reported as success: FIXED.** `addressbook update -k <unknown>` and `carddav proppatch <unknown>` printed "successfully updated" with exit 0 on iCloud, for a collection that does not exist. The response turned out to be worse than a refusal: iCloud answers 207 with a `200 OK` propstat carrying an **empty** `prop` element, naming nothing, where RFC 4918 §9.2.1 wants a propstat per requested property. Fastmail, captured side by side, lists each property it accepted, so a status-only check would have missed this. **Fix:** io-webdav's PROPPATCH coroutine now returns the parsed multistatus plus the properties the request carried, and the client fails when one comes back refused (`PropertiesRejected`) or unmentioned (`PropertiesIgnored`). Re-verified: the unknown collection now fails with "WebDAV server ignored the property update: displayname, addressbook-description", while a real update still succeeds on both providers. Nothing was damaged in the meantime: the DELETE counterpart is refused with 403 "Cannot delete carddav home", and the real book was verified intact.

### Provider-specific behaviour (not bugs)

- **I1: iCloud forbids creating and deleting addressbooks over CardDAV** (403 with an empty body on MKCOL, "Cannot delete carddav home" on DELETE). The account has exactly one server-managed book, so `addressbook create` / `delete` cannot be exercised at all, and every card test runs inside it.
- **I2: vCard 3.0 and 4.0 are both accepted and stored verbatim.** A 4.0 card reads back as 4.0, parameters and all. This **corrects the 2026-07-18 note** ("iCloud requires vCard 3.0 with an N property"): the version is not the constraint.
- **I3: the constraint is `UID` plus `N`.** A card without `UID` is rejected with `400 null vcard or UID missing from vcard`; a card without `N` is rejected with `403 VCARD parse error`, on **both** 3.0 and 4.0. Only the second goes through the `valid-address-data` hint, which now says "some (e.g. iCloud) also require an N property" instead of blaming the version.
- **I4: iCloud does not implement `If-Match: *`.** It answers 412 whether or not the resource exists, so the wildcard cannot be used as an existence precondition here. Real ETags work correctly in both directions.
- **I6: DELETE of an absent card returns 2xx**, so `card delete <absent-id>` reports success and exits 0, where Fastmail 404s. cardamum cannot tell the difference without a preflight read.
- **I7: `If-Match` is ignored on DELETE.** Verified end to end: a card deleted with a deliberately wrong ETag is gone, and the command reports success. Anything relying on a guarded delete gets no guard on iCloud. Fastmail rejects the same request with 412.
- **I8: a duplicate `UID` in the collection is rejected with `404 Persist Error`** on PUT, an odd status for a conflict. The same PUT with a fresh UID succeeds at the same path.
- **I9: CTag and sync-token are the same opaque base64 blob** on iCloud, unlike Fastmail's readable counter pair.

### Observations

- iCloud error bodies are short and useful ("record Id not found", "invalid collection ID URI = …", "Didn't understand the report"). Fastmail's HTML pages and iCloud's empty bodies both used to read badly; io-webdav now summarises a body to one line and drops the separator when there is none, so a Fastmail 404 reads "HTTP 404: 404 Not Found" and an iCloud MKCOL refusal reads "HTTP 403".
- `report sync` returned the change from a `put` immediately, with no observable propagation delay.
- Exit codes are correct throughout: 0 on success, 1 on a server error, 2 on a clap error.

## Verdict

The shared surface works on iCloud within what the provider allows: the whole card lifecycle (list / create / read / update / delete, paging, `--json`, `--if-match`) passes, and addressbook management is server-side forbidden (I1) rather than broken. The specific surface is validated in full except MKCOL, including the guarded `put`, the raw-XML escape hatch and incremental `report sync`. This run earned its keep twice over: it caught that this morning's `If-Match: *` guard made `card update` fail on every iCloud account, and it exposed **I5**, a PROPPATCH whose multistatus was never inspected, so an update the server never applied reported success. Both are fixed and re-verified on both providers, along with the error-body noise the run kept tripping over. What remains is provider behaviour cardamum cannot change: the ignored `If-Match` on DELETE (I7) and the 2xx on deleting an absent card (I6).
