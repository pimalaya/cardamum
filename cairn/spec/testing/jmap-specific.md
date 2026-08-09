# JMAP specific API: test report

The `jmap` protocol-specific subcommands (nested by JMAP object type), distinct from the shared `addressbook` / `card` report ([jmap-fastmail.md](jmap-fastmail.md)).

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-jmap 0.2)
- account: `fastmail-jmap` (Bearer, token from the config's `token.command`)
- date: 2026-08-09 (re-run, plus a same-day re-test of the S1 / S2 / S3 / S4 fixes; first run 2026-07-18)
- method: card operations ran in the empty book `R2ag` with uniquely-marked throwaway cards, all destroyed afterwards (Fastmail forbids AddressBook mutations, so no throwaway book). Other books were only counted, never printed; final state verified against the pre-run counts.

## Command surface

`address-book {get, create, update, destroy, changes}` · `contact-card {get, query, create, update, destroy, changes, copy}` · `session get` · `request`

## Results

| Command | Variants | Result |
| --- | --- | --- |
| `session get` | base | ✅ username, api-url, state, accounts, capabilities (`urn:…:contacts` / `core`) |
| `address-book get` | base (all), one positional id, several, unknown id, `--json` | ✅ raw AddressBooks plus the state token; an unknown id yields an empty table, exit 0 (JMAP `notFound`); ids are positional (**S2 fixed**) |
| `address-book create` / `update` / `destroy` | base, `-d`, `-n`, `--remove-contents` | ⛔ Fastmail forbids AddressBook mutations (J1); surfaced with a non-zero exit. The commands are correct and work on a server that allows them |
| `address-book changes` | `<since-state>` from `get`, `--json`, bogus state | ✅ empty change set plus `new-state`; a bogus state → `cannotCalculateChanges`, exit 1, without the placeholder type name (**S3 fixed**) |
| `contact-card get` | single id, several ids, `--json`, bogus id | ✅ round-trips (`uid`, `name`, `addressBookIds`); bogus id → empty table, exit 0 |
| `contact-card query` | `-k <book>`, `--text` (match and no match), `-l/--limit`, `--json` | ✅ scoped listing plus the query state; free-text search matches on the card name |
| `contact-card create` | file, stdin `-`, raw inline, `--json`, invalid JSON, missing `-k` | ✅ card created, `--json` returns the server-set properties (`id`, `created`, `prodId`); invalid JSON → parse error, exit 1; missing `-k` → clap error, exit 2. Human output prints just the id the server assigned (**S1 fixed**) |
| `contact-card update` | nested-object patch, dotted JSON-pointer patch (`name/full`), bogus id | ✅ both patch forms apply, read-back confirms, untouched properties survive; bogus id → `NotFound`, exit 1. Human output is a confirmation line, since Fastmail returns only its own bookkeeping (**S1 fixed**) |
| `contact-card destroy` | `<id>` | ✅ removed; `get` and `query` confirm |
| `contact-card changes` | `<since-state>` after a create, an update and a destroy | ✅ **incremental**: each id under the right status, plus the next state |
| `contact-card copy` | ids positional and repeatable, `--from-account`, `--to-address-book`, missing flags | ✅ **runs (S4 fixed)**: reaches the server, which answers `invalidArguments` when the source account is the destination account (RFC 8620 §5.4 wants them distinct); a successful copy still needs a second JMAP account |
| `request` | `AddressBook/get`, `ContactCard/query` with explicit `using`, unknown method, invalid JSON | ✅ raw JMAP response printed verbatim; an unknown method returns the server's `error` response with exit 0 (S5); invalid JSON → parse error, exit 1 |

## Findings

### Bugs / issues

- **S4: `jmap contact-card copy` could not run at all: FIXED.** Any invocation, `--help` included, aborted with a clap assertion saying that the short option `-a` was claimed by both `from_account` and the global `account_name`. The command declared its source account with `short = 'a'`, colliding with the global `-a/--account`. In a debug build that is a panic and exit 101; in a release build, where clap's debug assertions are compiled out, the duplicate short is simply ambiguous. Either way the command had never been usable, which is why the 2026-07-18 run recorded it as "implemented, not live-tested". **Fix:** it now follows himalaya's `jmap email copy`: ids positional and repeatable, `--from-account` and `--to-address-book` long-only, the latter repeatable since a JMAP copy can land in several books. Re-verified: it reaches the server, which answers `invalidArguments` when the source account is the destination account (RFC 8620 §5.4 wants them distinct), and `-a` unambiguously selects the account again. A successful copy still needs a second JMAP account.

- **S1: `contact-card create` and `update` printed empty fields: FIXED.** Both rendered through the same three-field `CardReport`, but a JMAP `set` response only echoes server-set properties: create returns id, created and prodId, so name and address-books printed blank, and Fastmail answers an update with `updated`, a blob id and a size and no card at all, so the whole block was blank while the operation succeeded. **Fix:** the report prints only the fields the object carries, and the update falls back to a confirmation line when the response has nothing to show. `--json` was already correct and is unchanged.

### Behaviour (not bugs)

- **The specific API is raw JSContact / JMAP**, not vCard: `contact-card create` / `update` take a raw JSContact body (`update` a patch, whose top-level keys may be dotted JSON pointers such as `name/full`), and `--json` prints the raw JMAP object. No vcard-rs projection is involved, which is why the projection defects that block the shared `card create` on Fastmail (J5, J6 in [jmap-fastmail.md](jmap-fastmail.md)) do not affect this surface: a raw JSContact body carrying the RFC 9553 `"@type": "Link"` is accepted here.
- **`get` and `query` surface the JMAP state token**, which `changes` consumes for incremental sync on both object types: the JMAP-native sync the shared API hides.
- **Fastmail forbids AddressBook create / update / destroy** over JMAP (J1, matching the shared-API report); the commands surface the server's `Forbidden` and work against servers that allow it (self-hosted Cyrus or Stalwart).

### Observations

- **S2 (fixed): `address-book get` took ids as a repeatable `--id` flag while `contact-card get` took them positionally.** Both help texts read "…by id", so the asymmetry only showed up when a positional id was rejected with "unexpected argument". Ids are now positional there too, still optional so that omitting them keeps fetching every book.
- **S3 (fixed in io-jmap): an error message leaked the RFC's placeholder type name.** A bogus state printed `JMAP AddressBook/changes failed: JMAP Foo/changes failed: …`, `Foo/changes` coming from io-jmap's generic changes error. Those generic errors now display only their own cause, leaving the concrete method name to the caller: `JMAP AddressBook/changes failed: JMAP cannotCalculateChanges: invalid sinceState`.
- **S5: `request` exits 0 on a method-level JMAP error.** An unknown method returns the server's `error` response and the command reports success, which is defensible for a raw passthrough (the HTTP exchange did succeed) but means a script cannot rely on the exit code alone. Left as is, deliberately: a raw escape hatch reports the exchange, not the server's opinion of it.
- The shared JSON argument no longer calls every body a JSContact: `request` takes a JMAP request object through the same argument, so its wording and its parse errors are now neutral.

## Verdict

The JMAP specific API is sound where it runs: `session get`, AddressBook `get` and `changes` (mutations forbidden by Fastmail, surfaced cleanly), the full ContactCard surface (raw JSContact create, get, query with free-text and limit, patch update in both spellings, incremental `changes`, destroy) and the raw `request` passthrough, all validated live with marked throwaway cards and cleaned up afterwards. The two defects this run found, **S4** (`contact-card copy` aborting on a clap short-option collision, so that verb had never run) and **S1** (create and update printing empty fields instead of what the server returned), were both fixed the same day and re-verified, along with the two smaller observations S2 and S3; see the [jmap-cli-alignment](../../log/2026-08-09-jmap-cli-alignment.md) log entry. Nothing here blocks writing a real contact, and unlike the shared API on this account, nothing here depends on the vcard-rs projection.
