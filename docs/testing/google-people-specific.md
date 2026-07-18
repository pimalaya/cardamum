# Google People API specific API — test report

The `google` protocol-specific subcommands (nested by People resource),
distinct from the shared `addressbook`/`card` report ([google-people.md](google-people.md)).

- cardamum: `v0.2.0 --all-features` (io-people `0.2`; iteration 4)
- account: `people` (Bearer OAuth, `contacts` scope; token via `ortie token show -a cardamum`)
- date: 2026-07-18
- method: a throwaway contact group (People allows group creation) + a uniquely-marked throwaway contact (lands in `myContacts`, deleted afterward). Real contacts were only counted or reached by searching the marker; a final search-sweep confirmed 0 left behind.

## Command surface

`contact-group {list, get, create, update, delete, members}` ·
`connection {list, get, create, update, delete, search}` ·
`other-contact {list, search, copy}` · `profile get`

## Results

| Command | Variants | Result |
| --- | --- | --- |
| `contact-group list` | base, `groups` alias, `--page-size` | ✅ raw People groups (id/name/type/members) |
| `contact-group create` | `<name>` | ✅ returns the raw created group |
| `contact-group get` | `<id>` | ✅ raw group |
| `contact-group update` | `<id> <name>` | ✅ etag-guarded rename (etag fetched first), verified |
| `contact-group delete` | `<id>`, `--delete-contacts` | ✅ removed |
| `contact-group members` | `--add people/<id>`, `--remove people/<id>` | ✅ membership +1 / −1, verified via member count |
| `connection create` | raw People JSON inline, stdin | ✅ returns the raw created person |
| `connection get` | `<id>` (human + `--json`), bad id | ✅; bad id → `404` |
| `connection update` | raw JSON PATCH | ✅ **update mask derived from the JSON's top-level keys**; phone patched, rest preserved |
| `connection list` | `--page-size`, `--sync-token` | ✅ single page; sync-token requested (see notes) |
| `connection search` | `<query>` | ✅ isolates to the marked contact |
| `connection delete` | `<id>` | ✅ removed; get-after → `404` |
| errors | bad JSON body | ✅ "Parse People person JSON error" |
| aliases | `group(s)`, `people`/`contacts`, `other`, `add`/`new`, `del`/`rm` | ✅ route correctly |

## Findings

### Bugs / issues

- **None.** Every command wired and dispatched correctly; the failures below are the server rejecting on OAuth scope (well-formed requests), not code defects.

### Token-scope-gated (not testable with this token)

- **`profile get`** → `403 requires scope: profile`. `people/me` needs the `profile` scope; the test token carries only `contacts`.
- **`other-contact {list, search, copy}`** → `403 insufficient authentication scopes`. `otherContacts` needs `contacts.other.readonly`, absent from the test token. The commands are implemented and send valid requests; re-mint the token with those scopes to verify live.

### Behaviour (not bugs)

- **The specific API is raw People**, not vCard: `connection create`/`update` take a raw People person JSON body, and `--json` prints the raw People person/group. `connection update` derives the People `updatePersonFields` mask from the JSON's top-level keys and fetches the current etag to guard the write.
- **`connection create` lands in `myContacts`** (People has no create-into-a-group); use `contact-group members --add people/<id>` to file it under a group — validated.
- **Sync-token** is requested (`requestSyncToken=true`) but the People API only returns `nextSyncToken` on the **final** page; a truncated `--page-size` page omits it (expected).
- The connections/other-contacts ID column shows the bare id (`people/` stripped); other-contact ids keep the `otherContacts/` prefix, which `other-contact copy` takes verbatim.

## Verdict

The Google People specific API is **implemented and validated end-to-end for the
`contacts`-scoped surface**: full contact-group CRUD + `members`, and connection
create/get/update/delete/search with raw People JSON in/out and a JSON-key-derived
update mask — all live-tested with a throwaway group and marked contact, cleaned
up (sweep = 0). `profile get` and the `other-contact` family are implemented but
**scope-gated** by the test token (need `profile` / `contacts.other.readonly`);
re-mint the token to verify them. This completes the specific-API round for all
four backends that expose one (vdir, carddav, msgraph, google) — jmap remains.
