# Manually testing a provider

A followable checklist to deeply exercise every **shared** command (`addressbook`, `card`) against a real provider/account. Unit tests run each `io-*` crate and each projection in isolation; this pass catches behaviour that only shows up end-to-end against a live server (resource-path handling, vCard round-trips, metadata updates, error surfaces, flag rendering).

One report is produced per `(backend, provider)` pair, e.g. `docs/testing/carddav-fastmail.md`. Follow the steps below, record each variant, and fill in the report template at the end.

## Golden rules

- **Throwaway addressbooks only.** Always create a fake addressbook and operate inside it. NEVER reuse an existing addressbook: some accounts are shared test books, others hold real contacts.
- **Fallback for locked backends.** If addressbook management is not allowed (one fixed default addressbook and nothing more, e.g. Google People / Microsoft Graph exposing a single container), operate on random generated contacts you create and delete afterwards, never touching pre-existing ones.
- **Clean up.** Delete the throwaway addressbook at the end — that removes every card inside it in one shot, even cards a broken `card delete` could not remove. When you had to fall back to the default addressbook, delete every card you created (match them by a unique `FN`/`UID` marker).
- **Never print secrets.** Credentials come from the config via `password.command` / `token.command`; never echo them.

## Prerequisites

- The account is configured and `cardamum -a <account> account check` reports the backend `OK`.
- A built binary: `nix develop --command cargo build`, then `C="./target/debug/cardamum -a <account>"`.

## Fixtures

Create one throwaway addressbook, then populate it with a few vCards that differ across every axis a command might key on — `FN`, `EMAIL`, `TEL`, and one card carrying properties with no provider slot (to test the custom-data stash on the API backends):

```bash
TMP="cardamum-test-$(date +%s)"
$C addressbook create "$TMP" -d "manual test" -C "#00FF00"
```

Add ~3 cards via `card create` (this doubles as the `card create` test). Every card **must carry a `UID` property**: strict CardDAV servers (Fastmail) reject a UID-less vCard with `403 Missing mandatory UID`, and the API backends key their resource on it. Capture the id each `create` returns — it is what the other commands address.

## Command checklist

Run every variant; note the outcome (pass / fail / finding). `<AB>` is an addressbook id, `<ID>` a card id.

### addressbook list

| Variant | Command |
| --- | --- |
| base | `addressbook list` |
| json | `--json addressbook list` |
| aliases | `abook ls`, `addressbooks list` |
| explicit backend | `addressbook list -b <backend>` |
| wrong backend | `addressbook list -b <other>` (expect a clean bail) |

### addressbook create / update / delete

| Variant | Command |
| --- | --- |
| create base | `addressbook create "$TMP"` |
| create + meta | `addressbook create "$TMP" -d "desc" -C "#00FF00"` |
| update rename | `addressbook update -k "$TMP" -n "new name"` |
| update meta | `addressbook update -k "$TMP" -d "new" -C "#123456"` |
| update clear | `addressbook update -k "$TMP" -d "" -C ""` (documented to clear) |
| delete | `addressbook delete -k "$TMP"` |
| error: no `-k` | `addressbook delete` (mandatory flag) |

Confirm each mutation with a follow-up `addressbook list --json` (id, name, description, color).

### card list

| Variant | Command |
| --- | --- |
| base | `card list -k "$TMP"` |
| paging | `card list -k "$TMP" -s 2 -p 1` then `-p 2` |
| json | `--json card list -k "$TMP"` (carries `etag`) |
| default addressbook | `card list` (no `-k` — see default-addressbook note) |
| bad addressbook | `card list -k does-not-exist` (expect a clean error) |

### card create

| Variant | Command |
| --- | --- |
| raw inline | `card create -k "$TMP" 'BEGIN:VCARD…UID:…END:VCARD'` |
| file path | `card create -k "$TMP" contact.vcf` |
| stdin | `printf … \| card create -k "$TMP" -` |
| json | `--json card create -k "$TMP" -- …` |
| error: no UID | create a vCard without a `UID` (watch the server's reaction) |

### card read

| Variant | Command |
| --- | --- |
| pretty | `card read -k "$TMP" <ID>` (raw vCard) |
| json | `--json card read -k "$TMP" <ID>` (etag + contents) |
| error | `card read -k "$TMP" does-not-exist` |

### card update

| Variant | Command |
| --- | --- |
| base | `card update -k "$TMP" <ID> '<new vCard with UID>'` |
| if-match ok | `card update -k "$TMP" <ID> … --if-match <etag-from-read>` |
| if-match stale | `card update -k "$TMP" <ID> … --if-match deadbeef` (expect 412) |

After each update, **read the card back and confirm the change actually landed** — a "successfully updated" message is not proof the right resource was written.

### card delete

| Variant | Command |
| --- | --- |
| base | `card delete -k "$TMP" <ID>` |

After delete, **list (or read) to confirm the card is truly gone**, not just reported deleted.

## Backend / provider-specific notes to capture

- **Resource path / id shape**: how the card id relates to the on-server resource href (e.g. CardDAV stores at `<name>.vcf`; is the id the href basename, the vCard `UID`, or something else, and does every verb build the same path?). This is the most common place a `(create ok, read/update/delete broken)` asymmetry hides.
- **UID handling**: does the server require a `UID`? Does the backend mint one when absent, and is the vCard `UID` the same as the resource id?
- **vCard version**: which version the server stores/returns (a 4.0 write may come back as 3.0).
- **Metadata clearing**: whether `addressbook update -d "" / -C ""` actually removes the property or is a no-op.
- **Custom-data stash**: on the API backends (Graph, People), whether properties with no provider slot survive a round-trip (see [../custom-data.md](../custom-data.md)).
- **Default addressbook**: whether omitting `-k` falls back to `addressbook.default`, and whether that is safe to test against.

## Cleanup

```bash
# one delete removes the throwaway book and every card inside it
$C addressbook delete -k "$TMP"
# fallback path (locked backend): delete each card you created
$C card delete -k <default> <ID>
```

## Report template

Copy this into `docs/testing/<backend>-<provider>.md`:

```markdown
# <BACKEND> on <PROVIDER> — shared-command test report

- cardamum: <version + features + build rev>
- account: <name> (<backend block>)
- date: <yyyy-mm-dd>

## Results
<per-command pass/fail table>

## Findings
### Bugs / issues
### Provider-specific behaviour (not bugs)
### Observations

## Verdict
<release-readiness note for this backend+provider>
```
