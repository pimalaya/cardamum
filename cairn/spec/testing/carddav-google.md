# CardDAV on Google: shared and specific test report

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; every Pimalaya dep at its current git revision)
- account: `google` (`carddav.home = https://www.googleapis.com/carddav/v1/principals/…/lists`, HTTP Bearer OAuth, token via `ortie token show -a google`; `addressbook.default = default`)
- date: 2026-08-09 (first full run, covering the specific surface for the first time; the 2026-07-18 run reached the shared card verbs only, and the specific one was token-blocked)
- method: Google forbids creating addressbooks (G3), so per the [golden-rule fallback](provider-test-plan.md) everything ran on uniquely-marked throwaway contacts (`FN:Cardamum *`, `UID:cardamum-google-test-*`) inside the single `default` book, deleted afterwards. The book's card ids were captured before the run and diffed after: the account is back to its pre-run state, and no pre-existing contact was read, printed or written.

## Results: shared API

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `carddav: OK` over Bearer |
| `addressbook list` | base, `--json` | ✅ one book, `default` ("Address Book" / "My Contacts") |
| `addressbook create` | base | ⛔ **400: Google forbids new addressbooks (G3)** |
| `addressbook update` | unknown id, no field flag, empty `-k` | ⛔ unknown id → 404; a flagless update and an empty `-k` bail client-side. The real book was deliberately not mutated |
| `addressbook delete` | unknown id | ⛔ 404 |
| `card create` | file, stdin `-` | ✅ returns Google's own id, taken from the `Location` header (G1) |
| `card read` | `<id>`, `--json` (etag), bogus id | ✅ pass; bogus id → 404 with Google's JSON message |
| `card list` | base, `-k`, `-s`/`-p` paging, `--json`, no-`-k` default, bad `-k` | ✅ all pass; bad `-k` → 404 |
| `card update` | file, `--if-match` (fresh **and stale**), absent id | ✅ the write lands and the read-back confirms; an absent id fails on the preflight read, nothing created; **a stale `--if-match` also succeeds (G4)** |
| `card delete` | `<id>`, absent id | ✅ card gone; read-after-delete and delete-again → 404 |

## Results: specific API (`carddav …`)

| Command | Variants | Result |
| --- | --- | --- |
| `discover` | base | ✅ server, principal and addressbook-home-set all resolved |
| `propfind` | no arg, `<addressbook>`, `--json` | ✅ the book and its cards with ETags; **no CTag and no sync-token (G5)** |
| `mkcol` | base | ⛔ **400 (G3)** |
| `proppatch` | unknown collection | ⛔ 404 |
| `get` | `<ab> <id>`, `--json` | ✅ raw vCard plus ETag |
| `put` | replace, `--if-match` stale, `--if-none-match '*'` on an existing card | ✅ the write lands; **both preconditions are ignored (G4)**, and the create-only form silently overwrote the card |
| `report query` | `<ab>` | ✅ every card with its ETag |
| `report multiget` | `<ab> <id…>` | ✅ batch fetch in one round-trip |
| `report sync` | initial | ⛔ **400 INVALID_ARGUMENT: Google does not implement sync-collection (G5)** |
| `report raw` | `sync-collection`, `addressbook-query` | ✅ the query works; the sync body fails the same way, confirming the server and not the request |
| `delete` | `<ab> <id>`, `--if-match` stale, `<ab>` (collection) | ✅ card removed; **a stale `--if-match` deletes anyway (G4)**; the collection delete → 404 |

## Findings

### Bugs / issues

- **None in cardamum.** Both surfaces behave; what this run adds is a much sharper picture of what Google's CardDAV does not do.

### Provider-specific behaviour (not bugs)

- **G1: Google reassigns the resource id** and reports it in the `201` `Location` header. cardamum creates at `<uuid>.vcf`, Google stores the card under its own contact id (e.g. `3e02cec814738234`, no extension), and `card create` returns that id, which every other verb then addresses. Confirmed again this run; the io-webdav `Location` fix holds.
- **G2: Google rewrites the vCard server-side.** It reconstructs `FN` from `N` (so an `FN`-only edit reads back unchanged), replaces the client `UID` with its own id, adds `REV`, appends `PREF` to `TEL` and `EMAIL` types, and rewrites `URL` into an Apple-style group (`item1.URL;TYPE=PREF` plus `item1.X-ABLabel:Other`), escaping the value. Verify an update through `N` or `ORG`.
- **G3: Google forbids creating addressbooks** (`400` on both `addressbook create` and `carddav mkcol`); the account exposes one fixed `default` book.
- **G4 (new): Google honours no precondition at all.** A `card update` with a deliberately stale `--if-match` succeeds and the write lands; a raw `carddav put --if-match <garbage>` does too; `carddav put --if-none-match '*'` on an existing card, which means "create only", silently overwrote it; and `carddav delete --if-match <garbage>` removed the card. Verified at both the shared and raw levels. So optimistic concurrency does not exist here: anything relying on ETag guards (a sync engine above all) gets no protection on Google, and the create-only form is actively dangerous. iCloud at least honours `If-Match` on PUT and ignores it only on DELETE (I7); Fastmail honours all of them.
- **G5 (new): Google implements no `sync-collection`.** `report sync` fails with `400 INVALID_ARGUMENT`, and a raw `report raw` carrying a minimal RFC 6578 body fails identically, so it is the server. PROPFIND confirms it: the book exposes neither a CTag nor a sync-token. Incremental sync over Google CardDAV therefore has to be an ETag diff from `report query` or PROPFIND, with no server-side change feed. Fastmail and iCloud both provide one.
- **G6 (new): an unknown property is kept across a PUT.** A raw `carddav put` of a document without `X-CUSTOM-FIELD` reads back with it still there, while every property Google models (TEL, TITLE, NOTE, URL) is removed by that same request. The value can be changed, and a second unknown property can be added, but neither can be removed. This is the CardDAV twin of the People stash limit (P8 in [google-people.md](google-people.md)), and the same Google account underneath. Unlike the People backend, the CardDAV one cannot detect it: it forwards the vCard bytes without parsing them, by design, so the caveat cannot be reported per write.

### Observations

- Google's error bodies come in both shapes: a JSON envelope for the API-ish failures ("Requested entity was not found.") and an HTML page for the rest, the latter now summarised to its title by io-webdav (`HTTP 400: Error 400 (Bad Request)!!1`) rather than dumped.
- The ETag is a timestamp (`2026-08-09T09:58:26.261-07:00`), not an opaque token, and passes through `--if-match` unharmed, for all the good it does (G4).
- The Bearer path works end to end with a freshly minted `ortie` token; the July run's `InvalidScope` failure was a token-state problem, not a client one.
- Exit codes are correct throughout: 0 on success, 1 on a server error, 2 on a clap error.

## Verdict

Both surfaces work on Google within a notably narrow provider: the card lifecycle (create / read / update / delete, paging, `--json`), discovery, PROPFIND, `get`, `put`, `report query`, `report multiget` and the raw escape hatch all pass, and the id-from-`Location` fix (G1) still holds. No cardamum defect surfaced. What this run establishes, for the first time, is how little Google's CardDAV guarantees: **no preconditions of any kind** (G4), so a guarded write or a create-only write is silently unguarded, **no sync-collection** (G5), so there is no server-side change feed to sync from, and **unknown properties that cannot be removed** (G6). None of that is fixable client-side, and all of it matters for anything built on top: a sync engine against Google CardDAV must diff ETags itself and cannot rely on a single precondition.
