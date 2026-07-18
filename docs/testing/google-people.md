# Google People API — shared-command test report

- cardamum: `v0.2.0 --all-features` (io-people `0.2` released; pimalaya-config git serializer)
- account: `people` (`google.auth.token` HTTP **Bearer** OAuth over the `https://www.googleapis.com/auth/contacts` scope — token via `ortie token show -a gmail`; no `addressbook.default`, so `-k` is required for card commands)
- date: 2026-07-18
- method: connection + contact-group & card CRUD by hand. Google People *does* allow creating contact groups, so per the [golden rule](provider-test-plan.md) group operations ran on a throwaway group (`cardamum-tmp-testgroup`). But **every contact also belongs to `myContacts`** (there is no create exclusive to a custom group), so the card operations ran on uniquely-marked throwaway contacts (`FN:Cardamum *`, `UID:cardamum-people-test-*`) deleted by id afterward. Card listings were scoped to the throwaway group (isolating them to my test cards); the real `myContacts` was only ever counted/marker-filtered, never printed. A final sweep confirmed 0 marked contacts left behind.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base, `-b google`, `-b carddav`, `-b msgraph` | ✅ `google: OK`; foreign `-b` bails cleanly |
| `addressbook list` | base, `--json`, `abook ls` / `addressbooks list` aliases | ✅ groups as books; `myContacts` first as "Contacts" (P1) |
| `addressbook create` | base, `--description`, `--color` | ✅ base returns the group id; `-d`/`-C` bail (P4) |
| `addressbook update` | `--name` (rename), `--description`, `--color`, `myContacts` | ✅ rename lands (etag-guarded); `-d`/`-C` and `myContacts` bail |
| `addressbook delete` | throwaway group, `myContacts`, missing `-k` | ✅ deletes; `myContacts` bails; missing `-k` → clean clap error (exit 2) |
| `card create` | inline, stdin `-` (`card new`) | ✅ server-assigned id; lands in `myContacts` + group membership added (P2) |
| `card read` | `<create-id>`, bogus id | ✅ pass; bogus id → `400 Invalid resource name` |
| `card list` | scoped to group (isolated), `-s`/`-p` paging, `--json` (etag), `cards ls` | ✅ all pass; paging splits client-side |
| `card update` | base (N/ORG), `--if-match` (valid), `--if-match` (stale) | ✅ update lands; valid etag works; stale etag → `400 FAILED_PRECONDITION`, card unchanged (P5) |
| `card delete` | `<create-id>`, `card rm` | ✅ delete works; read-after-delete → `404` |

## Findings

### Bugs / issues

- **None.** The full shared surface works end-to-end over Bearer OAuth. Two "issues" seen mid-session were **test-harness artifacts**, not defects: (a) a bare update whose vCard already matched server state sends no request (empty delta) and returns success — I initially mistook that for a stale-etag pass; (b) once the content genuinely differed, the stale etag was correctly rejected. Both explained under Provider-specific behaviour.

### Provider-specific behaviour (not bugs)

- **P1 — Contact groups are the addressbooks.** `myContacts` (the system group every contact belongs to) is surfaced first as "Contacts", then user groups. Memberships are m:n, so a card can appear under several books; `card list -k <group>` fetches connections and narrows to that group's members client-side.
- **P2 — Every contact lives in `myContacts`.** `card create -k <user-group>` creates the person in `myContacts`, then adds the group membership; there is no create exclusive to a custom group. So throwaway contacts show up in the real Contacts container — mark them uniquely and delete by id (this report does).
- **P3 — Google replaces the client `UID` with its own person id** (same as the CardDAV-Google G2). Read-back `UID` = the resource id; Google also reconstructs `FN` from `N` and normalizes some fields. Verify updates via `N`/`ORG`, not `UID`.
- **P4 — Groups carry neither description nor color** (guarded client-side); `myContacts` can be neither renamed nor deleted (guarded).
- **P5 — People requires the person's etag on updates.** A bare `card update` fetches the current person first and reuses its etag (so it is a read-then-write, last-writer-wins); `--if-match` passes an explicit etag through. A stale etag → `HTTP 400 FAILED_PRECONDITION` with no change. Updates are delta-masked (only changed fields), and a `clientData`/stash write merges the server's foreign entries under the same etag guard.
- **P6 — Param normalization.** `TEL;TYPE=CELL` reads back as `TEL;TYPE=cell` (lowercased), same as Microsoft Graph's M4.
- **Empty-delta no-op.** If the vCard matches server state, `changed_fields` is empty and cardamum sends **no** request (bare update returns success without touching the server). Expected; don't mistake it for a silent failure.

### Observations

- The **Bearer / OAuth** path works end-to-end; this is the People `contacts` scope, distinct from the `carddav` scope the `google` CardDAV account uses — so it needs its own token grant (see the OAuth setup notes; a verified public client for `contacts` does not exist, so this used an own/testing client or the OAuth Playground).
- The account has no `addressbook.default`, so card commands require an explicit `-k` (`myContacts` for the default container).
- Person ids and group ids are short opaque hex strings that round-trip verbatim.

## Verdict

The Google People backend is **fully working** over Bearer OAuth: `account check`, contact-group list/create/update(rename)/delete, and card create/read/update/delete/list all pass, including aliases, `--json`, paging, etag preconditions and the flag guards. No bugs found — both mid-session surprises were harness artifacts (empty delta, identical-content stale-etag), each resolved once the content genuinely differed. **P1–P6** are Google behaviours (groups-as-books, `myContacts` ubiquity, server-assigned UID, no group metadata, etag-guarded delta updates, param normalization) that mirror and extend the CardDAV-Google findings. The shared API is now validated across **carddav** (Fastmail, iCloud, Google), **msgraph** (Microsoft) and **google** (People API).
