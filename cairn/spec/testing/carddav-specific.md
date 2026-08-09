# CardDAV specific API: test report

The `carddav` protocol-specific subcommands (the flat WebDAV-method list), distinct from the shared `addressbook` / `card` reports.

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree)
- accounts: `fastmail` (Basic, full surface, re-run 2026-08-09), `icloud` (Basic, partial, 2026-07-18), `google` (Bearer, token-blocked, 2026-07-18)
- date: 2026-08-09 (Fastmail re-run; first run 2026-07-18)
- method: two throwaway addressbooks created with `carddav mkcol` on Fastmail, exercised with every command and precondition variant, then both deleted with `carddav delete`; the account was verified back to its pre-run state. On iCloud (forbids MKCOL) a uniquely-marked throwaway card in the default book, deleted after; real books only enumerated (ids and flags, never contact bodies).

## Command surface

`discover · propfind · proppatch · mkcol · report {query,multiget,sync,raw} · get · put · delete`

## Results: Fastmail (full, 2026-08-09)

| Command | Variants | Result |
| --- | --- | --- |
| `discover` | base, `--json` | ✅ server / principal / addressbook-home-set all resolved (see S2) |
| `propfind` | no arg (books + CTag/sync-token), `<addressbook>` (card id + ETag), `--json`, unknown book | ✅ all pass; unknown book → 404, exit 1 |
| `mkcol` | base, `-n -d -C`, duplicate id, `--json` | ✅ collection created with exactly the passed props; duplicate → 403, exit 1 |
| `proppatch` | `-n`, then `-d -C`, no flags | ✅ only the passed props are set, the others survive (verified via `propfind` and the shared `addressbook list`); a no-flag patch reports success (S3). Re-checked after the carddav-write-guards patch rework: unchanged, still set-only (S6) |
| `get` | `<ab> <id>`, `--json`, after delete | ✅ raw vCard plus ETag; after delete → 404, exit 1 |
| `put` | `--if-none-match '*'` (create), same again, `--if-match <fresh>`, `--if-match <stale>`, stdin `-` unguarded, unguarded on a new id, both preconditions at once | ✅ guarded create and update; existing + `--if-none-match '*'` → 412; stale `--if-match` → 412; the two preconditions are mutually exclusive at the clap level (exit 2); an unguarded put on a new id still creates, which is the verb's own semantics and is deliberately not gated the way the shared `card update` now is |
| `report query` | `<ab>`, `--json` | ✅ id + ETag, bodies in the JSON payload |
| `report multiget` | `<ab> <id…>`, one unknown href | ✅ batch bodies in one round-trip; an unknown href is silently dropped (S4) |
| `report sync` | initial (→ token), `--sync-token` after a create and a delete | ✅ **incremental sync works**: the second call returns exactly the `changed` card and the `vanished` one, plus the next token |
| `report raw` | `addressbook-query`, `sync-collection` with `--depth 0`, malformed XML | ✅ parsed multistatus printed, sync-token surfaced; malformed body → 400, exit 1; the STATUS column is always empty (S5) |
| `delete` | `<ab> <id>`, `--if-match <fresh>`, `--if-match <stale>`, `<ab>` (collection), `--if-match` on a collection | ✅ card and collection delete; stale If-Match → 412; `--if-match` on a collection → friendly client-side bail |

## Results: iCloud (partial, 2026-07-18)

| Command | Result |
| --- | --- |
| `propfind` (no arg) | ✅ book listed, CTag and sync-token present |
| `mkcol` | ⛔ **403: iCloud forbids creating addressbooks over CardDAV** (like Google) |
| `put` / `get` / `report multiget` / `delete` | ✅ full card round-trip in the default book (vCard 3.0 plus `N`, per iCloud strictness); `get` after delete → 404 |
| `discover` | ✅ since the C1 fix: best-effort, `principal: (unresolved)` instead of an error |

## Results: Google (2026-07-18)

Not testable that run: the `-a google` OAuth token failed to refresh with `InvalidScope` (an ortie token-state issue, unrelated to the CardDAV commands). `propfind` returned 0 books (no auth). To retest, re-mint the Google token.

## Findings

### Bugs / issues

- **S1: user-facing messages leaked the Rust type name: FIXED.** `carddav mkcol` printed "CarddavAddressbook `x` successfully created" and `carddav proppatch` printed "CarddavAddressbook `x` properties successfully patched", where every other command says "Addressbook" or "Card" (`carddav delete` already got it right); the `carddav propfind` clap doc had the same leak, its `[ADDRESSBOOK]` argument reading "CarddavAddressbook to enumerate". All three now say "Addressbook", re-verified live on Fastmail in both text and `--json` mode. A sweep over every message and clap doc in src/ found no other wire type escaping into the CLI surface.

### Provider behaviour (not bugs)

- iCloud and Google **forbid MKCOL** (403 / 400): addressbooks are server-managed; only Fastmail allowed creating a throwaway book. The card-level commands (`get` / `put` / `report` / `delete`) work everywhere.
- `propfind` (no arg) surfaces **CTag and sync-token** per book on every provider: the sync machinery the shared API hides.

### Observations

- **S2: `discover` reports the client base URL as `server`.** On a `carddav.home`-configured account the base URL *is* the home URL, so `server` and `addressbook-home-set` print the same thing and the configured `carddav.server` is never echoed. Correct as documented ("the resolved endpoints"), just not obviously so at a glance.
- **S3: `proppatch` with no flags reports success** after sending an empty PROPPATCH, mirroring the same behaviour on the shared `addressbook update`.
- **S4: `report multiget` silently drops unknown hrefs.** A batch of one good and one absent id returns just the good one, exit 0; the 404 in the multistatus is filtered out rather than reported.
- **S6: `proppatch` stays set-only although the protocol can remove.** io-webdav's PROPPATCH path gained `DAV:remove` for the shared `addressbook update --description ""` fix (see [carddav-write-guards](../../log/2026-08-09-carddav-write-guards.md)), so the raw command could now expose removal too, which the "specific commands mirror their protocol" requirement argues for. It deliberately was not added with that fix: it needs its own flag spelling. The `--help` stays accurate meanwhile ("clearing a property is not exposed").
- **S5: `report raw` always prints an empty STATUS column** on Fastmail, because the per-resource status is carried inside `<D:propstat>` rather than as a response-level `<D:status>`. `report sync`, which reads the same responses, does classify them (`changed` / `vanished`) correctly.
- The `report raw` escape hatch handles both an `addressbook-query` and a `sync-collection` body, honours `--depth`, and surfaces the returned sync-token.

### History

- **C1 (fixed, 2026-07-18):** `discover` used to fail outright on a `carddav.home` account (`WebDAV client missing principal URL`) because a configured home short-circuits the discovery walk. It now reports each endpoint best-effort. Confirmed holding.
- **`report raw` (landed, 2026-07-18):** the arbitrary-REPORT-body escape hatch. `propfind` and `proppatch` stay typed: `Propfind::new` / `Proppatch` take typed props, not a raw body, so a raw variant there would need an io-webdav raw-request method.

## Verdict

The flat WebDAV-method CardDAV API is validated in full on Fastmail: every command, every precondition (`If-Match` fresh and stale, `If-None-Match: *` on both a free and a taken resource, the clap-level exclusion between them), the raw-XML escape hatch, and the headline **incremental `report sync`** including `vanished`. It is confirmed on iCloud for the read plus card round-trip surface, and still untested on Google pending a re-minted token. The only defect found this run, **S1** (a type-name leak in two messages and one clap doc), was fixed the same day and re-verified. The specific API continues to expose what the shared API hides: sync-token, multiget, CTag, PROPPATCH, ETag-aware get/put, and card delete.
