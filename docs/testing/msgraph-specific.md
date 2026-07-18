# Microsoft Graph specific API — test report

The `msgraph` protocol-specific subcommands (nested by Graph resource),
distinct from the shared `addressbook`/`card` report.

- cardamum: `v0.2.0 --all-features` (io-msgraph `0.2`; iteration 3)
- account: `msgraph` (Bearer OAuth, token via `ortie token show -a msgraph`)
- date: 2026-07-18
- method: a throwaway contact folder (Graph allows folder creation); all contact ops ran inside it, deleted afterward. The default Contacts folder and real contacts were never mutated.

## Command surface

`contact-folder {list, child-folders, get, create, rename, delete}` ·
`contact {list, get, create, update, delete, delta}` · `profile get`

## Results

| Command | Variants | Result |
| --- | --- | --- |
| `profile get` | base, `--json` | ✅ raw Graph `/me` (id + mail) |
| `contact-folder list` | base, `folders` alias | ✅ (0 user folders initially — the default Contacts folder is not listed by the folders endpoint) |
| `contact-folder create` | `<name>` | ✅ returns the raw created folder |
| `contact-folder get` | `<id>` | ✅ raw folder |
| `contact-folder rename` | `<id> <name>` | ✅ PATCH displayName, verified |
| `contact-folder child-folders` | `<id>` | ✅ empty page |
| `contact-folder delete` | `<id>` | ✅ folder + contacts removed |
| `contact create` | raw JSON inline, stdin `-`, `--folder` | ✅ returns the raw created contact |
| `contact get` | `<id>` (human + `--json` raw Graph), bad id | ✅; bad id → `400 ErrorInvalidIdMalformed` |
| `contact update` | raw JSON PATCH | ✅ only the sent fields change (phone + jobTitle), rest preserved |
| `contact list` | `--folder`, `--top` (pagination note), table + `--json` | ✅ single Graph page; `--top 1` prints "more available" |
| `contact delete` | `<id>` | ✅ removed |
| `contact delta` | `--folder` | ✅ **incremental sync**: changed rows + a resumable `@odata.deltaLink` |
| errors | bad JSON body | ✅ "Parse Graph contact JSON error" |
| aliases | `contacts`, `folder(s)`, `add`/`new`, `del`/`rm` | ✅ |

## Findings

### Bugs / issues

- **None on the wire.** One internal bug was found and fixed during
  implementation: the raw-JSON positional arg was field-named `json`, colliding
  with the global `--json` flag's clap id (panic: "Could not downcast to bool").
  Renamed the field to `body`. Lesson for later backends: never name a
  positional arg after a global flag (`json`, `account`, `backend`).

### Behaviour (not bugs)

- **The specific API is raw Graph**, not vCard: `contact create`/`update` take a
  raw Graph contact JSON body (file / inline / `-`), and `--json` prints the raw
  Graph contact/folder/user — the shared `card` API's vCard projection is bypassed.
- **`contact delta`** returns one Graph page plus `@odata.deltaLink` (the resume
  token) — the msgraph-native incremental sync the shared API hides. Rows carry a
  `changed` / `removed` status.
- List/delta are **single-page** (raw Graph pagination); `--top` sizes the page
  and a `@odata.nextLink` is surfaced as a "more available" note.
- Graph does **not** list the default Contacts folder via the folders endpoint
  (it is addressed implicitly), so `contact-folder list` shows only user folders.

## Verdict

The Microsoft Graph specific API works end-to-end: the full contact-folder and
contact surface, `profile get`, raw Graph JSON in/out, and the headline
**`contact delta`** incremental sync — all validated live against the `msgraph`
account with a throwaway folder, cleaned up afterward. No provider bugs; the one
arg-id collision was fixed. Next in the specific-API round: **people** (Google).
