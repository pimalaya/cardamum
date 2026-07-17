# CardDAV on Google — shared-command test report

- cardamum: `v0.2.0 --all-features` (io-webdav git `f2376eb` verbatim card-id fix; pimalaya-config git serializer; the card-write error hint)
- account: `google` (`carddav.home = https://www.googleapis.com/carddav/v1/principals/…/lists`, HTTP **Bearer** OAuth — token via `ortie token show -a google`)
- date: 2026-07-18
- method: connection + card CRUD by hand. Google forbids creating address books (`400`), so per the [golden rule](provider-test-plan.md) the card operations ran on a single uniquely-marked throwaway contact inside the `default` book. Real contacts were only enumerated (grep-filtered to the marker) to recover the test contact's canonical id for cleanup — never printed.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base (Bearer / OAuth via ortie) | ✅ `carddav: OK` |
| `addressbook list` | base | ✅ one book, `default` ("My Contacts") |
| `addressbook create` | base | ⛔ `400` — Google forbids new address books (G3) |
| `card create` | stdin `-`, vCard 3.0 | ✅ returns Google's canonical id (**G1 fixed** — from `Location`) |
| `card read` | `<create-id>` | ✅ pass (**F1 fixed**) |
| `card update` | `<create-id>`, `N` change | ✅ update lands — read-back confirms (**F2 fixed**) |
| `card delete` | `<create-id>` | ✅ delete by the create-returned id works (**G1 fixed**) |

## Findings

### Bugs / issues

- **G1 — Google reassigns the resource id — FIXED.** cardamum creates at `<uuid>.vcf` and Google accepts it, but Google stores the card under its own contact id (e.g. `2efb1df0937cd4af`, no `.vcf`) and reports it in the `201` `Location` header. Previously `create` returned the client-chosen name, which `card list`/`delete` did not recognise, so `card delete <create-id>` `404`'d while the card survived. **Fix (generic):** io-webdav's `CreateCard` now takes the created resource's id from the `Location` header (RFC 9110 §10.2.2), falling back to the caller's name when absent — no provider is special-cased. Re-verified: on Google `card create` now returns `2efb1df0937cd4af` and `card delete` by that id succeeds (read-after-delete is gone); on Fastmail and iCloud (no relocation, no distinct `Location`) `create` still returns `<uuid>.vcf` and the round-trip is unchanged.

### Provider-specific behaviour (not bugs)

- **G2 — Google rewrites the vCard server-side.** It reconstructs `FN` from `N`, replaces the client `UID` with its own, adds `REV`, and normalizes `EMAIL;TYPE`. So an `FN`-only edit reads back unchanged (FN is derived from N) — verify updates via `N` or another preserved field, as this report does.
- **G3 — Google forbids creating address books over CardDAV** (`400`); it exposes one fixed `default` book.

### Observations

- The io-webdav verbatim-id fix holds on Google (read/update/delete address the id verbatim), now confirmed on **three** servers (Fastmail, iCloud, Google). The **Bearer / OAuth** auth path works end-to-end (token fetched via ortie).

## Verdict

The full card surface (create / read / update / delete) now works on Google over Bearer OAuth. **G1 is fixed** — `CreateCard` honors the server's `Location`, so `card create` returns the id Google actually assigned and `card delete` by it succeeds, with no `card list` detour. The io-webdav verbatim-id fix (F1/F2) plus the `Location` fix (G1) are now confirmed across **three** servers (Fastmail, iCloud, Google), including the Bearer/OAuth path. **G2/G3** are Google behaviours to be aware of (server-side vCard rewrite; no new address books), not bugs. The card-write error hint (F3/P1) also makes a rejected vCard legible everywhere.
