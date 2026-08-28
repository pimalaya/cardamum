# Microsoft Graph specific API: test report

The `msgraph` protocol-specific subcommands (nested by Graph resource), distinct from the shared `addressbook` / `card` report ([msgraph-microsoft.md](msgraph-microsoft.md)).

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-msgraph 0.2)
- account: `msgraph` (Bearer OAuth, token from the config's `token.command`)
- date: 2026-08-09 (re-run; first run 2026-07-18)
- method: a throwaway contact folder created for the run (Graph allows it), every contact operation inside it, deleted afterwards. The default Contacts folder was only counted; the account was verified back to its pre-run state.

## Command surface

`contact-folders {list, child-folders, get, create, rename, delete}` · `contacts {list, get, create, update, delete, delta}` · `profile get` · `request`

## Results

| Command | Variants | Result |
| --- | --- | --- |
| `profile get` | base, `--json` | ✅ id, display name, mail, UPN; `--json` is the raw Graph `/me` |
| `contact-folders list` | base, `folders` alias, `--json` | ✅ user folders with their parent id; the default Contacts folder is not among them (S1) |
| `contact-folders create` | `<name>` | ✅ returns the raw created folder |
| `contact-folders get` | `<id>` | ✅ raw folder |
| `contact-folders rename` | `<id> <name>` | ✅ PATCHes `displayName`, verified by a follow-up list |
| `contact-folders child-folders` | `<id>` | ✅ empty page |
| `contact-folders delete` | `<id>` | ✅ folder and its contacts removed |
| `contacts create` | raw inline, stdin `-`, `--folder`, bad JSON | ✅ returns the raw created contact; bad JSON → "Parse Graph contact JSON error", exit 1 |
| `contacts get` | `<id>`, `--json` (raw Graph) | ✅ round-trips |
| `contacts update` | raw JSON PATCH | ✅ only the sent fields change, the rest survive |
| `contacts list` | `--folder`, default folder, `--top 1`, `--json` | ✅ one Graph page; `--top` sizes it and a truncated page prints "more contacts available: raise --top" |
| `contacts delete` | `<id>` | ✅ removed |
| `contacts delta` | `--folder`, `--select`, `--delta-link` (after a create and after a delete), an empty round, conflicting flags, a bad link | ✅ **incremental sync, end to end (S2 fixed)**: a round opens, a created contact comes back `changed`, a deleted one `removed`, and a quiet round returns nothing; `--delta-link` with `--folder` → clap error, exit 2 |
| `request` | `get` (path and full URL), `post`, `patch`, `delete`, bad path | ✅ raw Graph in and out; bad path → 400 with the Graph message; a `delete` prints nothing in text mode and `null` with `--json` (**S3 fixed**) |
| aliases | `folders`, `contacts`, `add` / `new`, `del` / `rm` | ✅ |

## Findings

### Bugs / issues

- **S2: `contacts delta` could not resume, although its own help said to feed the link back: FIXED.** The command's `--help` read "Feed the returned `@odata.deltaLink` back to resume", but its only flags were `--folder` and `--select`, so every call started a fresh round and the incremental half of the incremental sync was unreachable from the command that advertises it. The capability was there underneath: `msgraph request get <deltaLink>` returned exactly the contact created since, which is how the run first verified it. **Fix:** io-msgraph gained `MsgraphContactsDelta::from_link` and `contacts_delta_from_link`, which the messages twin already had, and the command gained `--delta-link <URL>`, mutually exclusive with `--folder` and `--select` since the link carries both. Re-verified as a full cycle: open a round, create a contact and see it `changed`, delete it and see it `removed`, resume once more and see nothing.

### Behaviour (not bugs)

- **The specific API is raw Graph**, not vCard: `contacts create` / `update` take a raw Graph contact JSON body (file, inline or `-`), and `--json` prints the raw Graph contact, folder or user. None of the vCard projection is involved, which is why the shared API's provider quirks (M1, M4) do not appear here.
- **S1: Graph does not list the default Contacts folder** through the folders endpoint, it being addressed implicitly, so `contact-folders list` shows only user-created folders while the shared `addressbook list` shows both.
- List and delta are **single-page**, matching raw Graph: `--top` sizes the page, and a truncated delta page now prints its `next-link` alongside the note, so it goes back through `--delta-link` like any other round.
- `contacts update` is a genuine PATCH: the fields absent from the body are left alone, which is the raw-faithful counterpart of the shared API's delta.

### Observations

- **S3: `request delete` printed `null`: FIXED.** A Graph 204 has no body, and the raw passthrough printed the parsed JSON body regardless. Text output now prints nothing for an empty body, across every backend's `request` command; `--json` still prints `null`, which is the honest JSON for it.
- Graph's own error payloads are good enough to surface verbatim: `400 (ErrorInvalidIdMalformed): Id is malformed.`, `404 (ErrorItemNotFound): The specified object was not found in the store.`
- The `--json` shape of `contact-folders list` is `{"folders": [...]}` and of `contacts list` `{"contacts": [...]}`, while `contacts delta` returns the raw Graph keys (`@odata.deltaLink`, `contacts`). Consistent enough, but worth knowing when scripting.

## Verdict

The Microsoft Graph specific API works end to end: the whole contact-folders and contacts surface, `profile get`, the raw `request` escape hatch in all four methods, and the headline `contacts delta`, all validated live inside a throwaway folder and cleaned up. The one defect this run found, **S2** (`contacts delta` advertising a resume it had no flag for, making the incremental sync a one-shot from the CLI), is fixed and re-verified as a full cycle, along with the cosmetic **S3**; see the [msgraph-delta-resume](../../log/2026-08-09-msgraph-delta-resume.md) log entry. **S1** is Graph's own shape, not something to fix.
