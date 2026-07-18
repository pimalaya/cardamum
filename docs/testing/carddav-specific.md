# CardDAV specific API — test report

The `carddav` protocol-specific subcommands (the flat WebDAV-method list),
distinct from the shared `addressbook`/`card` reports.

- cardamum: `v0.2.0 --all-features` (io-webdav git; the flat WebDAV surface, iteration 2)
- accounts: `fastmail` (Basic, full surface), `icloud` (Basic, partial), `google` (Bearer, token-blocked)
- date: 2026-07-18
- method: throwaway addressbook on Fastmail (allows MKCOL); on iCloud (forbids MKCOL) a uniquely-marked throwaway card in the default book, deleted after; real books only enumerated (ids/flags, never contact bodies).

## Command surface

`discover · propfind · proppatch · mkcol · report {query,multiget,sync} · get · put · delete`

## Results — Fastmail (full)

| Command | Variants | Result |
| --- | --- | --- |
| `discover` | base | ✅ server / principal / addressbook-home-set |
| `propfind` | no arg (books + **CTag/sync-token**), `<addressbook>` (card id+ETag) | ✅ both; CTag & sync-token surfaced (hidden by shared `addressbook list`) |
| `mkcol` | base, `-n` | ✅ collection created |
| `proppatch` | `-n`, `-d`, `-C` | ✅ only the passed props set (verified via `propfind`) |
| `get` | `<ab> <id>` | ✅ raw vCard + ETag |
| `put` | `--if-none-match '*'` (create), `--if-match <etag>` (update), `--if-none-match '*'` on existing | ✅ create + guarded update; existing → **412** |
| `report query` | `<ab>` | ✅ id + ETag + bodies (`--json`) |
| `report multiget` | `<ab> <id…>` | ✅ batch bodies in one round-trip |
| `report sync` | initial (→ token), `--sync-token` (incremental) | ✅ **incremental sync works**: initial token → change → `--sync-token` returns the changed href + next token |
| `delete` | `<ab> <id>`, `--if-match <stale>`, `<ab>` (collection), `--if-match` on collection | ✅ card + collection delete; stale If-Match → **412**; `--if-match` on a collection → friendly bail |

## Results — iCloud (partial)

| Command | Result |
| --- | --- |
| `propfind` (no arg) | ✅ book listed, CTag + sync-token present |
| `mkcol` | ⛔ **403 — iCloud forbids creating addressbooks over CardDAV** (like Google) |
| `put` / `get` / `report multiget` / `delete` | ✅ full card round-trip in the default book (vCard 3.0 + `N`, per iCloud strictness); `get` after delete → 404 |
| `discover` | ❌ fails — see C1 |

## Results — Google

Not testable this run: the `-a google` OAuth token fails to refresh with
`InvalidScope` (an ortie token-state issue, unrelated to the CardDAV commands).
`propfind` returned 0 books (no auth). To retest, re-mint the Google token.

## Findings

### Bugs / issues

- **C1 — `discover` fails on a home-configured account** (iCloud): `WebDAV client missing principal URL; call current_user_principal first`. When the account is configured with `carddav.home`, `open_carddav_client` short-circuits the discovery walk, so the principal is never resolved; `discover` then cannot report it. **Pre-existing** (the command was unchanged this iteration), but it makes `carddav discover` unusable for any `home`-configured account. Fix candidate: have `discover` report the resolved endpoints best-effort (show `server` + `addressbook-home-set`, mark `principal` unresolved) instead of erroring. Deferred to a follow-up.

### Provider behaviour (not bugs)

- iCloud and Google **forbid MKCOL** (403 / 400): addressbooks are server-managed; only Fastmail allowed creating a throwaway book. The card-level commands (`get`/`put`/`report`/`delete`) work everywhere.
- `propfind` (no arg) surfaces **CTag and sync-token** per book on every provider — the sync machinery the shared API hides.

### Deferred

- **Raw-XML escape hatch** (`propfind --xml` / `report --xml` / `proppatch --xml`, decision C). Feasible — io-webdav exposes `pub stream` + `auth()` — but a faithful raw PROPFIND/REPORT over the connected stream is its own focused sub-step; the semantic commands cover the real surface. Deferred.

## Verdict

The flat WebDAV-method CardDAV API works end-to-end — validated in full on Fastmail
(every command, including guarded `put`/`delete` preconditions and the headline
**incremental `report sync`**) and confirmed on iCloud for the read + card
round-trip surface. The big shared-API gaps are now exposed: **sync-token**,
**multiget**, **CTag**, **PROPPATCH**, ETag-aware **get/put**, and card **delete**.
Two follow-ups: **C1** (`discover` on home-configured accounts) and the raw-XML
escape hatch. Next in the specific-API round: **msgraph**.
