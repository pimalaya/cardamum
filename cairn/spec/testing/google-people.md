# Google People API: shared-command test report

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-people 0.2, the other Pimalaya deps at their current git revisions)
- account: `people` (`people.auth.token` HTTP Bearer OAuth over the `contacts` scope, token via `ortie token show -a cardamum`; `addressbook.default = myContacts`)
- date: 2026-08-09 (re-run; first run 2026-07-18)
- method: Google People allows creating contact groups, so group operations ran on a throwaway group. But **every contact also belongs to `myContacts`** (there is no create exclusive to a custom group), so per the [golden-rule fallback](provider-test-plan.md) the cards were uniquely-marked throwaways (`FN:Cardamum *`, `UID:cardamum-people-test-*`), listed scoped to the throwaway group and deleted by id afterwards. The real `myContacts` was only ever counted: its contact ids were captured before the run and diffed after, and the account is back to its pre-run state.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `people: OK` |
| `addressbook list` | base, `--json`, `abook ls`, `-b carddav` | ✅ groups as books, `myContacts` first as "Contacts" (P1); unconfigured `-b` bails |
| `addressbook create` | base, `-d`, `-C` | ✅ returns the group id; `-d` / `-C` bail client-side (P4) |
| `addressbook update` | `-n` (rename), `-d`, `-C`, `myContacts`, no field flag, empty `-k`, unknown group | ✅ rename lands (etag-guarded); `-d` / `-C` and `myContacts` bail; a flagless update bails; an empty `-k` bails; an unknown group → 404, now summarised (**P7 fixed**) |
| `addressbook delete` | throwaway group, `myContacts`, missing `-k` | ✅ deletes; `myContacts` bails; missing `-k` → clap error, exit 2 |
| `card create` | file, raw inline | ✅ server-assigned id; lands in `myContacts` plus the group membership (P2) |
| `card read` | `<id>`, `--json` (etag), bogus id | ✅ pass; bogus id → 404 "Requested entity was not found" |
| `card list` | scoped to the group, `-s`/`-p` paging, `--json`, `cards ls`, default `-k` | ✅ all pass; paging splits client-side |
| `card update` | file, `--if-match` (fresh + stale), absent id, dropping a stashed property | ✅ the delta lands, `--if-match` guards, a stale etag → 400 `FAILED_PRECONDITION` with no change (P5), an absent id → 404 and nothing created; a dropped stashed property is **reported as kept** rather than silently left (**P8 addressed**) |
| `card delete` | `<id>`, `card rm`, absent id | ✅ card gone; read-after-delete → 404; delete-again → 404 |

## Findings

### Bugs / issues

- **None in cardamum.** The shared surface passes over Bearer OAuth, including the guards added since the first run. **P8** below is a Google limitation the CLI used to paper over, and now reports.

### Provider-specific behaviour (not bugs)

- **P1: contact groups are the addressbooks.** `myContacts`, the system group every contact belongs to, is surfaced first as "Contacts", then the user groups. Memberships are m:n, so a card can appear under several books; `card list -k <group>` fetches connections and narrows client-side.
- **P2: every contact lives in `myContacts`.** `card create -k <user-group>` creates the person in `myContacts` and then adds the group membership; there is no create exclusive to a custom group. Throwaway contacts therefore appear in the real container, which is why they are marked and deleted by id.
- **P3: Google replaces the client `UID` with its own person id.** Read-back `UID` is the resource id, as on Microsoft Graph (M1). Verify an update through `N` or `ORG`.
- **P4: groups carry neither description nor color** (guarded client-side), and `myContacts` can be neither renamed nor deleted (guarded).
- **P5: People requires the person's etag on an update.** A bare `card update` fetches the current person and reuses its etag, so it is a read-then-write; `--if-match` passes one through, and a stale etag is rejected with `400 FAILED_PRECONDITION`, the card unchanged.
- **P6: param normalization.** `TEL;TYPE=CELL` and `EMAIL;TYPE=HOME` read back lowercased (`cell`, `home`), Graph's M4 again. Unlike Graph, the type itself survives.
- **P8: a stashed property can be changed but never removed; the CLI now says so.** An `X-CUSTOM-FIELD` round-trips intact, and an update carrying a new value replaces it, but an update that simply drops it leaves the old value on the server, while every first-class property (TEL, TITLE, URL) is correctly removed by the same request. Isolated to Google, not to cardamum: the update mask does carry `clientData`, and a raw `people request patch` with `updatePersonFields=clientData` and an explicit `"clientData": []` also leaves the entry standing. Microsoft Graph, whose stash works the same way, clears it correctly (see [msgraph-microsoft.md](msgraph-microsoft.md)). **Addressed** as far as a client can: the write still lands, and `card update` now names what stayed, "Card `x` successfully updated, except X-CUSTOM-FIELD, which the server will not let go", instead of reporting a clean success. Re-verified both ways: the caveat appears when the property is dropped and not when it is kept.

### Observations

- **P7: an unknown group id surfaced a full Google HTML error page: FIXED.** Google's own JSON errors always came through well ("Requested entity was not found.", "Request had insufficient authentication scopes."); only the HTML fallback was at fault, the same noise the WebDAV layer shed earlier today. io-people now summarises such a body to the page's title.
- **Scope limits, not defects.** This token carries the `contacts` scope only, so `people profile get` fails with a clear 403 naming the missing `profile` scope, and the whole `other-contact` surface fails with `403 Request had insufficient authentication scopes`. Both need a broader grant to exercise; the messages say exactly that.
- **Sync tokens are eventually consistent.** A contact updated immediately after taking a sync token did not appear in the next incremental list, on three successive attempts. Verified against a raw `people request get …&syncToken=…`, which returns the same empty page, so the CLI is faithful and the delay is Google's.
- Person and group ids are opaque strings that round-trip verbatim; the etag is percent-encoded and long, and passes through `--if-match` unharmed.
- Exit codes are correct throughout: 0 on success, 1 on a server error, 2 on a clap error.

## Verdict

The Google People backend passes the shared surface over Bearer OAuth: contact-group list / create / rename / delete and card create / read / update / delete / list, with aliases, `--json`, paging, the etag precondition and every flag guard, plus the empty-`-k` and flagless-update guards added this round. The one thing this run turned up, **P8**, is a Google limitation rather than a defect: a property living in the provider-side stash cannot be removed, so an update that drops it leaves it in place. The server is immovable, verified down to a raw PATCH, so the fix was to stop claiming otherwise: the command now names what stayed. **P7**, the HTML error page, is fixed upstream in io-people. See the [people-honest-writes](../../log/2026-08-09-people-honest-writes.md) log entry.
