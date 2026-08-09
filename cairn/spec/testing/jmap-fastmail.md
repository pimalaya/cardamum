# JMAP on Fastmail: shared-command test report

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-jmap 0.2, JSContact projection on vcard-rs 0.2)
- account: `fastmail-jmap` (`jmap.server = https://api.fastmail.com/jmap/session`, HTTP Bearer OAuth, `addressbook.default = RBk`)
- date: 2026-08-09 (re-run, plus a same-day re-test of the J5 / J6 / J7 fixes; first run 2026-07-18)
- method: Fastmail forbids creating AddressBooks over JMAP (J1), so per the [golden-rule fallback](provider-test-plan.md) the card commands ran inside an existing **empty** book (`R2ag`, confirmed 0 cards first) with uniquely-marked throwaway contacts (`FN:Cardamum *`, `UID:cardamum-jmap-test-*`), every one destroyed by id afterwards. The other books were only counted, never printed; final state verified card-for-card against the pre-run counts.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `jmap: OK` |
| `addressbook list` | base, `--json` | ✅ 6 books, ids opaque |
| `addressbook create` | base, `-d`, `-C` | ⛔ Fastmail forbids create (J1); `-C` bails client-side (J2) |
| `addressbook update` | `-n`, `-d`, `-C`, unknown id | ⛔ Fastmail forbids update (J1); `-C` bails client-side (J2) |
| `addressbook delete` | unknown id, missing `-k` | ⛔ Fastmail forbids destroy (J1); missing `-k` → clap error, exit 2 |
| `card list` | base, `-k`, `-s`/`-p` paging, page past the end, `--json` (etag), no-`-k` default, bad `-k`, `cards ls`, `-b jmap`, `-b carddav` | ✅ all pass; bad `-k` → error, exit 1; unconfigured `-b` bails |
| `card create` | file, stdin `-` (`card new`), plain vCard, vCard carrying `URL` + `PHOTO` + `KEY` + `CALURI` + `SOURCE`, vCard carrying `X-` properties | ✅ including the resource properties (**J5 fixed**, they round-trip intact); still ⛔ for a property JSContact cannot model, now with a message naming it (J6) |
| `card read` | `<id>`, `--json`, bogus id | ✅ pass; bogus id → "not found", exit 1 |
| `card update` | file, stdin `-`, `--if-match`, absent id | ✅ delta lands, read-back confirms (a property dropped from the vCard is removed); `--if-match` bails (J3); absent id → "not found", nothing created |
| `card delete` | `<id>`, `card rm`, absent id | ✅ card gone, read-after-delete → not found; absent id → error, exit 1 |

## Findings

### Bugs / issues

- **J5: every `*Resource` JSContact object type was spelled the draft way, so mainstream vCards were rejected: FIXED.** `card create` on a vCard as ordinary as `URL:https://pimalaya.org` failed with `InvalidProperties { properties: ["links/1/@type"] }`, and the same on `PHOTO` (`media/1/@type`) and `KEY` (`cryptoKeys/1/@type`). Root cause was in **vcard-rs**, whose JSContact projection tagged these objects `LinkResource`, `MediaResource`, `CryptoResource`, `CalendarResource` and `DirectoryResource`, the names from the pre-RFC JSContact drafts; RFC 9553 §2.6 registers them as `Link`, `Media`, `CryptoKey`, `Calendar` and `Directory`. Proven both ways against the server before fixing: a raw `jmap contact-card create` carrying the RFC name is accepted for all five collections, the draft name rejected. The affected vCard properties were `URL`, `PHOTO`, `LOGO`, `SOUND`, `KEY`, `CALURI`, `FBURL`, `CALADRURI` and `SOURCE`, so a contact carrying any one of them could not be written at all. **Fix:** vcard-rs now emits the RFC names (its own [log entry](https://github.com/pimalaya/vcard) covers it), which also lands for cardamum-android, sharing the projection. Re-verified: one card carrying `URL`, `PHOTO`, `KEY`, `CALURI` and `SOURCE` creates, and reads back with all five intact. Consumed here through a local-path patch until the vcard-rs release ships.

- **J6: Fastmail rejects the standard `vCardProps` member, so an unmapped property blocks the whole write.** A vCard carrying `X-CUSTOM-FIELD` fails with `invalidProperties (\`vCardProps\`)`, and a raw `jmap contact-card create` carrying nothing but a `vCardProps` array fails identically, so the rule is the member itself, not cardamum's use of it. This contradicts the 2026-07-18 verdict ("vcard-rs preserves unmapped vCard properties through the standard `vCardProps` member, so no JMAP-specific stash is needed"): that run evidently exercised only cards whose properties all mapped, so the member was never emitted. RFC 9555 §2.15 defines `vCardProps`, so Fastmail is at fault, but the consequence is real: a lossless round-trip of an arbitrary vCard through Fastmail JMAP is impossible today. **Partly addressed:** the write still fails, and nothing is dropped silently, but the message now names the properties that landed in `vCardProps` ("the card carries X-CUSTOM-FIELD, X-ABLABEL, which JMAP has no home for on this server") instead of leaving the user with a bare `invalidProperties`. Whether to stash those properties the way the Graph and People backends do stays an open product question.

- **J7: a JMAP `SetError` reached the user as Rust `Debug` output: FIXED.** Every rejected write printed its error struct verbatim: `Forbidden { description: Some("AddressBooks may not be created") }`, `NotFound { description: None }`. **Fix:** a new src/jmap/error.rs ports himalaya's `JmapSetError` trait and `format_set_error`, used at every rejection site in the backend and the specific commands. The same failure now reads `JMAP AddressBook create rejected: forbidden: AddressBooks may not be created`, and an `invalidProperties` lists its properties.

### Provider-specific behaviour (not bugs)

- **J1: Fastmail forbids all JMAP AddressBook mutations.** create → "AddressBooks may not be created", update → "…may not be updated", destroy → "…may not be destroyed". Books are server-managed. Note the asymmetry with the same account over CardDAV, where creating an addressbook works ([carddav-fastmail](carddav-fastmail.md)). cardamum surfaces each as a clean non-zero exit.
- **J2: JMAP AddressBooks have no color**, so `-C/--color` bails client-side rather than pretending.
- **J3: JMAP has no If-Match precondition** (last-write-wins). The etag cardamum shows is a hash of the card JSON, for display only, and `card update --if-match` bails with "JMAP does not support If-Match guarded updates" rather than silently ignoring the guard.
- **J4: Fastmail (Cyrus) preserves the client `UID`** (unlike Google and Graph, which reassign it) and echoes JSContact-conversion artifacts on read-back (`CREATED`, `KIND:individual`, `PRODID`, `PROP-ID` params). Cosmetic, not data. `VERSION:4.0` survives, unlike the CardDAV path, which down-converts to 3.0.
- **J8: an update is a delta, not a replacement.** Updating with a vCard that omits a property removes it (a dropped `TEL` disappeared on read-back), which matches the shared "replace the bytes" contract; the wire form is a JSContact patch of the changed top-level properties.

### Observations

- The **Bearer** auth path (token via `pass`) works end to end; card ids are short opaque strings that round-trip verbatim.
- `card update` on an absent id fails with "not found" and creates nothing, so the CardDAV F5 defect ([carddav-fastmail](carddav-fastmail.md)) has no JMAP counterpart: `ContactCard/set update` targets an existing object by id.
- Golden-rule fallback held: creation forbidden, so testing used an empty existing book with marked throwaway contacts, all destroyed afterwards. Final state verified: `R2ag` back to 0 cards, every other book at its pre-run count.

## Verdict

The JMAP backend works over Bearer OAuth for the operations Fastmail allows: the card surface (create / read / update / delete / list, paging, aliases, `--json`) passes, AddressBook mutations are forbidden server-side (J1) and surface cleanly, and `--if-match` correctly bails (J3). This run first found it **unusable for real contact data**, a sharp correction of the 2026-07-18 verdict, since a vCard carrying `URL` or `PHOTO` (most real contacts) could not be written at all; that blocker (J5) was traced to vcard-rs, fixed there, and re-verified with a card carrying all five affected property families round-tripping intact. What remains is J6, Fastmail refusing the standard `vCardProps` member: a contact carrying a property JSContact cannot model still fails, now with a message naming that property, and what cardamum should ultimately do with such properties is a product decision rather than a defect. Release-ready for JMAP contact data that JSContact can model, once the vcard-rs fix ships.
