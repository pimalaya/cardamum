# Google People specific API: test report

The `people` protocol-specific subcommands (nested by People resource), distinct from the shared `addressbook` / `card` report ([google-people.md](google-people.md)).

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-people 0.2)
- account: `people` (Bearer OAuth over the `contacts` scope, token from the config's `token.command`)
- date: 2026-08-09 (re-run; first run 2026-07-18)
- method: a throwaway contact group created for the run, and uniquely-marked throwaway contacts, since every contact lands in `myContacts` whatever group it joins. Everything was deleted afterwards and `myContacts` was diffed against its pre-run ids.

## Command surface

`contact-group {list, get, create, update, delete, members}` · `connection {list, get, create, update, delete, search}` · `other-contact {list, search, copy}` · `profile get` · `request`

## Results

| Command | Variants | Result |
| --- | --- | --- |
| `profile get` | base | ⛔ **403: the token lacks the `profile` scope (S1)**; the message names it |
| `contact-group list` | base, `--json` | ✅ user and system groups with their type and the server's member count (**S2 fixed**) |
| `contact-group get` | `<id>` | ✅ raw group with its member count (**S2 fixed** here too) |
| `contact-group create` | `<name>` | ✅ returns the raw created group |
| `contact-group update` | `<id> <name>` | ✅ renames, fetching the group's etag first to guard the write |
| `contact-group members` | `--add`, `--remove` | ✅ membership lands both ways, confirmed by a scoped `card list`; the message reports `(+1 / -0)` |
| `contact-group delete` | `<id>` | ✅ group removed, its contacts left in `myContacts` |
| `connection create` | raw People JSON | ✅ returns the created person |
| `connection get` | `<id>`, `--json` | ✅ raw person |
| `connection update` | raw People JSON patch | ✅ the mask is derived from the body's keys and only those fields change |
| `connection list` | base, `--json`, `--sync-token` | ✅ one page plus a `nextSyncToken`; the incremental round came back empty (S3) |
| `connection search` | `<query>` | ✅ matches by name |
| `connection delete` | `<id>` | ✅ removed |
| `other-contact list` / `search` | base | ⛔ **403: insufficient scopes (S1)**; `copy` untested for the same reason |
| `request` | `get`, `post`, `delete`, bad path | ✅ raw People in and out; a `delete` prints `{}`, which is what Google returns (S4); a bad path → 404 summarised to its title (**S5 fixed**) |
| aliases | `groups`, `contacts` / `people`, `other`, `add` / `new`, `del` / `rm` | ✅ |

## Findings

### Bugs / issues

- **S2: `contact-group list` and `get` reported a member count of 0 for every group: FIXED.** A group holding one contact listed as `MEMBERS 0` while the raw API reported `memberCount: 1`. Two causes stacked: the commands passed an empty `groupFields` mask, so People never returned the count, and the renderers then counted `memberResourceNames`, which the API fills only on a `get` and only up to the requested maximum. **Fix:** both commands ask for `name`, `groupType` and `memberCount`, and both renderers read the count the server sent, leaving the cell blank when it was not asked for rather than printing a zero that reads as an empty group. Re-verified on a group with one member: list and get both report 1.

### Behaviour (not bugs)

- **The specific API is raw People**, not vCard: `connection create` / `update` take a raw person JSON body, `update` deriving its `updatePersonFields` mask from the body's top-level keys, and `--json` prints the raw People payload. None of the vCard projection is involved, so the shared API's quirks (P3, P6, P8) do not appear here.
- **S1: the account's token carries the `contacts` scope only.** `profile get` needs `profile` and the `other-contact` surface needs `contacts.other.readonly`; both fail with a 403 that names the missing scope. Those three commands stay untested until the grant is widened, which is a token matter rather than a code one.
- **S3: an incremental `connection list --sync-token` came back empty** after a contact was updated between the two calls, on three successive attempts. A raw `people request get "people/me/connections?syncToken=…"` returns the same empty page, so the CLI is faithful and the lag is Google's.
- `contact-group members` is the People-native way to move a contact between books, the shared API having no such verb: memberships are m:n and a contact never leaves `myContacts`.

### Observations

- **S4: `request delete` prints `{}`.** Unlike Graph's 204, Google answers a delete with an empty JSON object, so the passthrough prints exactly what came back. Correct, if a little bare.
- **S5: a bad path surfaced Google's HTML error page: FIXED.** Its JSON errors always read well ("Requested entity was not found."), but the HTML fallback was the same noise the WebDAV layer shed earlier today. io-people now summarises a non-envelope body to the page's `title`, else the stripped markup: `People API returned HTTP 404: Error 404 (Not Found)!!1`.
- Person and group resource names round-trip verbatim (`people/<id>`, `contactGroups/<id>`), and the commands accept the bare id too.

## Verdict

The Google People specific API works for everything the account's scope allows: the whole contact-group surface including `members`, connection create / get / update / delete / search, and the raw `request` passthrough, all validated live and cleaned up. The one defect this run found, **S2** (a member count printed without ever being asked for), is fixed and re-verified, as is **S5**, the unsummarised HTML error body it shared with the shared-API report as P7; see the [people-honest-writes](../../log/2026-08-09-people-honest-writes.md) log entry. Three commands (`profile get`, `other-contact list` / `search`, and `copy` behind them) remain untested because the token's scope excludes them, which the errors state plainly.
