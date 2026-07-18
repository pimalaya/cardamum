# Microsoft Graph — shared-command test report

- cardamum: `v0.2.0 --all-features` (io-msgraph `0.2` released; pimalaya-config git serializer)
- account: `msgraph` (`msgraph.auth.token` HTTP **Bearer** OAuth — token via `ortie token show -a msgraph`; `user_id = me`, `addressbook.default = contacts`)
- date: 2026-07-18
- method: connection + addressbook & card CRUD by hand. Graph *does* allow creating contact folders, so per the [golden rule](provider-test-plan.md) every card operation ran inside throwaway folders (`cardamum-tmp-*`), each deleted at the end (which cascades to its cards). The default `contacts` folder was only ever enumerated, never mutated.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base, `-b msgraph`, `-b carddav`, `-b google` | ✅ `msgraph: OK`; foreign `-b` bails cleanly |
| `addressbook list` | base, `--json`, `abook ls` / `addressbooks list` aliases | ✅ all pass (default `contacts` first, then folders) |
| `addressbook create` | base, `--description`, `--color` | ✅ base returns the Graph folder id; `-d`/`-C` bail (M2) |
| `addressbook update` | `--name` (real id), `--description`, `--color`, default folder, empty `-k` | ✅ rename lands; `-d`/`-C` and the default folder bail; empty `-k` → server 405 (harness artifact, see Observations) |
| `addressbook delete` | throwaway id, default `contacts`, missing `-k` | ✅ deletes; default bails; missing `-k` → clean clap error (exit 2) |
| `card create` | inline, stdin `-`, file path | ✅ all pass; id is Graph-assigned (from the create response) |
| `card read` | `<create-id>`, bogus id | ✅ pass; bogus id → `400 ErrorInvalidIdMalformed` |
| `card list` | base, `-s`/`-p` paging, `--json` (etag), `cards ls` alias | ✅ all pass; paging splits client-side; etag = Graph `changeKey` |
| `card update` | `<create-id>` (FN/N/ORG), `--if-match` | ✅ update lands — read-back confirms; `--if-match` bails (M3) |
| `card delete` | `<create-id>`, `card rm` alias | ✅ delete works; read-after-delete → `404 ErrorItemNotFound` |

## Findings

### Bugs / issues

- **None.** The full shared surface (addressbook + card CRUD) works end-to-end over Bearer OAuth. The `405 The OData request is not supported` seen mid-session was a **test-harness artifact**, not a defect — see Observations.

### Provider-specific behaviour (not bugs)

- **M1 — Graph replaces the client `UID` with its own contact id.** A created card reads back with `UID:` set to the Graph resource id (the same id `create` returns and `read`/`update`/`delete` address), not the client-supplied `UID`. Mirrors Google's G2: verify updates via a preserved field (`N`, `ORG`), not `UID`.
- **M2 — Graph contact folders carry neither description nor color.** `addressbook create`/`update` with `-d`/`-C` bail client-side (`MsgraphBackend`) before any request, rather than silently dropping the field.
- **M3 — Graph has no If-Match/ETag precondition for contact updates** (last-write-wins). The `changeKey` is surfaced as the card `etag` for display, but `card update --if-match` bails client-side instead of pretending to honor a guard the server does not enforce.
- **M4 — Graph normalizes vCard params server-side.** `TEL;TYPE=CELL` reads back as `TEL;TYPE=cell` (lowercased). Cosmetic.
- **Delta updates.** `card update` fetches the current contact as a base and PATCHes only the changed fields (plus nulls for removed ones), so an edit touches nothing else server-side.

### Observations

- **Empty `-k ""` is not guarded client-side.** It reaches Graph as `PATCH /me/contactFolders/` (the collection), which answers `405 The OData request is not supported`. No one passes an empty id on purpose (this only happened when a broken shell pipeline left the variable empty), and a real folder id renames fine — confirmed both by a direct `PATCH` (HTTP 200) and by cardamum with a real id (HTTP 200). A client-side non-empty check on `-k` would turn the confusing 405 into a clear message; low priority.
- The **Bearer / OAuth** auth path (token fetched via ortie) works end-to-end for every verb.
- Graph contact ids and folder ids are long opaque base64-ish strings (with `-`, `=`); they round-trip verbatim through the URL path (`Url::join`) with no escaping issues.

## Verdict

The Microsoft Graph backend is **fully working** over Bearer OAuth: `account check`, addressbook list/create/update(rename)/delete, and card create/read/update/delete/list all pass, including aliases, `--json`, paging and the flag guards. No bugs found — the transient 405 was a harness artifact (empty `-k`), isolated and dismissed. **M1–M4** are Graph behaviours to be aware of (server-assigned UID, no folder metadata, no If-Match, param normalization), not defects. The only nice-to-have is a client-side non-empty `-k` guard to pre-empt the collection-PATCH 405.
