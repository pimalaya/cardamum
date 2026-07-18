# JMAP on Fastmail — shared-command test report

- cardamum: `v0.2.0 --all-features` (io-jmap `0.2` released; JMAP JSContact projection on **vcard-rs `0.1`** — calcard dropped; pimalaya-config git serializer)
- account: `fastmail-jmap` (`jmap.server = https://api.fastmail.com/jmap/session`, HTTP **Bearer** OAuth — token via `pass show pimalaya/fastmail-jmap-contacts`; `addressbook.default = RBk`)
- date: 2026-07-18
- method: connection + card CRUD by hand. Fastmail **forbids creating AddressBooks over JMAP** (J1), so per the [golden rule](provider-test-plan.md) card operations ran inside an existing **empty** book (`R2ag`, confirmed 0 cards first) with uniquely-marked throwaway contacts (`FN:Cardamum *`, `UID:cardamum-jmap-test-*`), all deleted by id afterward (book back to 0). Other books were only counted; real cards were never printed.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base, `-b jmap`, `-b carddav` | ✅ `jmap: OK`; foreign `-b` bails cleanly |
| `addressbook list` | base, `--json`, `abook ls` aliases | ✅ all pass (6 books) |
| `addressbook create` | base, `--description`, `--color` | ⛔ Fastmail forbids create (J1); `--color` bails client-side (J2) |
| `addressbook update` | `--color`, `--name` (bogus id) | ✅ `--color` bails client-side; ⛔ Fastmail forbids update (J1) |
| `addressbook delete` | bogus id, missing `-k` | ⛔ Fastmail forbids destroy (J1); missing `-k` → clap error |
| `card create` | inline, stdin `-` (`card new`) | ✅ **after the fix** (see Bugs); server-assigned id; UID preserved (J4) |
| `card read` | `<create-id>`, bogus id | ✅ pass; bogus id → not found |
| `card list` | scoped to book (isolated), `-s`/`-p` paging, `--json` (etag), `cards ls` | ✅ all pass; etag = JSON hash (J3) |
| `card update` | base (FN/N/ORG delta), `--if-match` | ✅ delta lands — read-back confirms; `--if-match` bails (J3) |
| `card delete` | `<create-id>`, `card rm` | ✅ delete works; read-after-delete → not found |

## Findings

### Bugs / issues

- **JMAP writes were blocked by calcard's non-standard `vCard` container — FIXED by porting the projection to vcard-rs.** The CLI's `to_jscontact` had been built on calcard's `VCard::into_jscontact`, which emits a top-level `vCard` object (`convertedProperties` plus the vCard properties calcard could not map) for lossless round-tripping. RFC 9553 defines no such property, so Fastmail's strict `ContactCard/set` rejected **every** create and update with `InvalidProperties { properties: ["vCard"] }`. (Root cause is inside calcard: its `ConversionOptions::include_vcard_parameters`, default true, is wired into the internal `include_vcard_converted` flag, so the container ships by default with no clean opt-out.) **Fix:** the JMAP projection now runs on **vcard-rs** (`Vcard::from_jscontact` / `VcardCst::decode().to_jscontact()`), exactly like cardamum-android, and the `calcard` dependency is dropped. vcard-rs emits spec-compliant JSContact and preserves unmapped vCard properties losslessly through the **standard `vCardProps`** member — so create/update work *and* the round-trip stays lossless (unlike a naive strip). Verified live on Fastmail: create/read/update/delete pass, core properties + UID round-trip (output is even cleaner — no spurious `JSCOMPS` param). Regression test `to_jscontact_emits_no_non_standard_vcard_container` added.

### Provider-specific behaviour (not bugs)

- **J1 — Fastmail forbids all JMAP AddressBook mutations.** create → "AddressBooks may not be created", update → "…may not be updated", destroy → "…may not be destroyed". Books are server-managed. Note the asymmetry: Fastmail **CardDAV** *does* allow addressbook creation (see [carddav-fastmail](carddav-fastmail.md)); JMAP does not (may be scope- or policy-driven). cardamum surfaces each as a clean error.
- **J2 — JMAP AddressBooks support name + description but no color** (`--color` bails client-side).
- **J3 — JMAP has no If-Match/ETag precondition** (last-write-wins). The etag cardamum surfaces is a hash of the card JSON for display only; `card update --if-match` bails rather than pretending to guard. Updates are delta-masked (only changed top-level JSContact props, nulls for removed ones; `uid` never nulled).
- **J4 — Fastmail (Cyrus) preserves the client `UID`** (unlike Google/Graph, which reassign it) and echoes JSContact-conversion artifacts on read-back (`CREATED`, `KIND:INDIVIDUAL`, `PRODID`, `REV`, `PROP-ID`, `JSCOMPS` params). Cosmetic, not data.

### Observations

- Golden-rule fallback: creation forbidden, so testing used an empty existing book with marked throwaway contacts, deleted afterward; real books/contacts never printed. Final state verified: `R2ag` back to 0 cards.
- The **Bearer** auth path (token via `pass`) works end-to-end; card ids are short opaque strings that round-trip verbatim.

## Verdict

The JMAP backend is **working over Bearer OAuth after porting the JSContact projection from calcard to vcard-rs**, which had blocked every create and update on Fastmail. The full card surface (create / read / update / delete / list, paging, aliases, `--json`) passes; AddressBook mutations are forbidden server-side by Fastmail (J1) and surface cleanly, and `--if-match` correctly bails (J3, no server guard). With this pass the shared API is validated across **all five backends** except vdir: carddav (Fastmail, iCloud, Google), msgraph (Microsoft), google (People API) and jmap (Fastmail). vcard-rs preserves unmapped vCard properties through the standard `vCardProps` member, so no JMAP-specific stash is needed.
