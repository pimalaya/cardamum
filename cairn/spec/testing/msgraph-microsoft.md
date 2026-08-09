# Microsoft Graph: shared-command test report

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-msgraph 0.2, io-webdav / vcard-rs / pimalaya-cli at their current git revisions)
- account: `msgraph` (`msgraph.auth.token` HTTP Bearer OAuth, token via `ortie token show -a msgraph`; `addressbook.default = contacts`)
- date: 2026-08-09 (re-run; first run 2026-07-18)
- method: Graph allows creating contact folders, so per the [golden rule](provider-test-plan.md) every card operation ran inside a throwaway folder (`cardamum-test-<ts>`), deleted at the end, which cascades to its contacts. The default `contacts` folder was only ever counted; its card ids were captured before the run and diffed after, and the account is back to its pre-run state.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `msgraph: OK` |
| `addressbook list` | base, `--json`, `abook ls`, `-b msgraph`, `-b carddav` | ✅ the default `contacts` plus the user folders; unconfigured `-b` bails |
| `addressbook create` | base, `-d`, `-C` | ✅ returns the Graph folder id; `-d` / `-C` bail client-side (M2) |
| `addressbook update` | `-n` (real id), `-d`, `-C`, the default folder, no field flag, unknown id, empty `-k` | ✅ rename lands; `-d` / `-C` and the default folder bail client-side; a flagless update bails ("Nothing to update"); unknown id → 400 `ErrorInvalidIdMalformed`; empty `-k` bails client-side (**M5 fixed**) |
| `addressbook delete` | throwaway id, the default folder, missing `-k` | ✅ deletes and cascades; the default folder bails; missing `-k` → clap error, exit 2 |
| `card create` | file, stdin `-`, raw inline | ✅ all pass; the id is Graph-assigned |
| `card read` | `<id>`, `--json` (etag = `changeKey`), malformed id | ✅ pass; malformed id → 400 `ErrorInvalidIdMalformed` |
| `card list` | base, `-k`, `-s`/`-p` paging, `--json`, `cards ls`, bad `-k` | ✅ all pass; paging splits client-side |
| `card update` | file, `--if-match`, absent id (well-formed and malformed) | ✅ the delta lands and untouched fields survive; `--if-match` bails (M3); an absent id → 404, nothing created |
| `card delete` | `<id>`, `card rm`, absent id | ✅ card gone; read-after-delete and delete-again both → 404 `ErrorItemNotFound` |

## Findings

### Bugs / issues

- **None.** The shared surface passes end to end over Bearer OAuth, including the guards added since the first run.

### Provider-specific behaviour (not bugs)

- **M1: Graph replaces the client `UID` with its own contact id.** A created card reads back with `UID:` set to the Graph resource id, the same id `create` returns and every other verb addresses. Verify an update through a preserved field (`N`, `ORG`), never through `UID`. Google People behaves the same way; Fastmail and iCloud preserve the client UID.
- **M2: Graph contact folders carry neither description nor color**, so `addressbook create` / `update` with `-d` or `-C` bail client-side before any request rather than silently dropping the field.
- **M3: Graph has no If-Match precondition for contacts** (last-write-wins). The `changeKey` is surfaced as the card `etag` for display, and `card update --if-match` bails rather than pretending to guard.
- **M4: Graph normalizes what it stores.** `TEL;TYPE=CELL` reads back as `TYPE=cell`, and `EMAIL;TYPE=work` loses its type entirely, Graph having three untyped email slots. Cosmetic, and unavoidable in the projection.
- **A property Graph cannot model survives anyway.** An `X-CUSTOM-FIELD` round-tripped intact through the provider-side stash, as did `TITLE`, `URL` and `NOTE`. This is the contrast with JMAP on Fastmail, where the same card cannot be written at all (J6 in [jmap-fastmail.md](jmap-fastmail.md)).
- **Updates are deltas.** `card update` fetches the current contact as a base and PATCHes only what changed, plus nulls for what the new vCard dropped: a property removed from the vCard disappears server-side, and nothing else moves.

### Observations

- **M5: an empty `-k ""` reached the server: FIXED.** It arrived as a PATCH on the folder collection, which Graph answers `405 The OData request is not supported`, explaining nothing. Carried over from the 2026-07-18 run and cleared now: the shared `-k/--addressbook` resolver rejects an empty id for every backend ("Addressbook id cannot be empty"), since an empty path segment addresses a parent collection wherever it lands.
- Graph error messages are excellent: every failure names its Graph error code (`ErrorInvalidIdMalformed`, `ErrorItemNotFound`, `ErrorInvalidRequest`) and a human sentence, so the CLI needs no summarising of its own here.
- Graph contact and folder ids are long opaque base64-ish strings with `-` and `=`; they round-trip verbatim through the URL path with no escaping trouble.
- Exit codes are correct throughout: 0 on success, 1 on a server or client error, 2 on a clap error.

## Verdict

The Microsoft Graph backend passes the whole shared surface over Bearer OAuth: addressbook list / create / update / delete and card create / read / update / delete / list, with aliases, `--json`, paging and every flag guard. Nothing was found to fix. The cross-checks this round's other runs prompted all come out clean here: an update on an absent id fails without creating anything (unlike the CardDAV defect F5), a delete of an absent id fails properly (unlike iCloud, I6), and a property the provider cannot model survives through the stash (unlike Fastmail JMAP, J6). **M1 to M4** are Graph behaviours to know; **M5**, the unguarded empty `-k` carried over from the first run, is now fixed for every backend.
