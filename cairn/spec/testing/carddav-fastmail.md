# CardDAV on Fastmail: shared-command test report

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; `+carddav +jmap +msgraph +people +vdir +pimdir`)
- account: `fastmail` (`carddav.home = https://carddav.fastmail.com/dav/addressbooks/user/…/`, HTTP Basic, `addressbook.default = Default`)
- date: 2026-08-09 (re-run, plus a same-day re-test of the F4 / F5 fixes; first run 2026-07-17)
- method: every shared command and flag variant run by hand inside two throwaway addressbooks (`cardamum-test-<ts>` and `cardamum-test-<ts>-meta`), per [provider-test-plan.md](provider-test-plan.md). Both were deleted on cleanup and the account was verified back to its pre-run state. The real `Default` book was only counted, never printed.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `carddav: OK` |
| `addressbook list` | base, `--json`, `abook ls` / `addressbook ls` aliases, `-b carddav`, `-b jmap`, `-b vdir` | ✅ all pass; unconfigured `-b` bails cleanly with exit 1 |
| `addressbook create` | base, `-d`, `-C` | ✅ pass (id = the given name; description and color set) |
| `addressbook update` | `-n`, `-d`, `-C` (set), `-d ""` / `-C ""` (clear), set and clear at once, no flags | ✅ setting works; **clearing works (F4 fixed)**; a no-flag update reports success while sending an empty PROPPATCH (O2) |
| `addressbook delete` | `-k`, missing `-k`, unknown id | ✅ deletes; missing `-k` is a clap error (exit 2); unknown id → 404, exit 1 |
| `card list` | base, `-k`, `-s`/`-p` paging, page past the end, `--json` (etag), no-`-k` default, bad `-k`, empty book, `cards ls` alias, `-b carddav` | ✅ all pass; bad `-k` → error, exit 1 |
| `card create` | file path, stdin `-`, raw inline, `--json`, vCard with no UID | ✅ pass (id is `<uuid>.vcf`); the UID-less card surfaces the actionable hint (F3), exit 1 on both text and JSON |
| `card read` | `<id>`, `--json`, bogus id | ✅ pass; bogus id → 404, exit 1 |
| `card update` | file, stdin `-`, `--if-match` (fresh + stale), absent id with and without `--if-match` | ✅ real card updated, read-back confirms; stale `--if-match` → 412 and the card is untouched; **an absent id → 412, nothing created (F5 fixed)** |
| `card delete` | `<id>`, `card rm` alias, absent id | ✅ card gone from `card list` and `card read` → 404; absent id → 404, exit 1 |

## Findings

### Bugs / issues

- **F5: `card update <absent-id>` silently created the card instead of failing: FIXED.** `cardamum card update -k <book> phantom-xyz.vcf '<vcard>'` printed "successfully updated" with exit 0 and the card appeared in `card list`, although nothing was replaced, contradicting the command's own help ("Replace the bytes of an existing vCard"). Root cause: io-webdav's `CarddavCardUpdate` issues a plain `PUT` with `if_match: None`, and an unconditional WebDAV `PUT` is create-or-replace. **Fix:** cardamum's carddav `update_card` (src/carddav/backend.rs) now sends `If-Match: *` when the caller gave no ETag, which RFC 9110 §13.1.1 defines as "the resource must exist"; an explicit `--if-match <etag>` still wins, and the raw `carddav put` is untouched. Re-verified: an absent id (with or without `--if-match`) returns 412 and `card list` shows nothing created, while an existing card still updates with no ETag passed. The same asymmetry may affect the other write backends and should be checked when their reports are re-run.

- **F4: clearing addressbook metadata was a no-op: FIXED.** `addressbook update -k <book> -d ""` and `-C ""` printed "successfully updated" but left the description and color unchanged, contradicting the `--help` ("pass `\"\"` to clear"). cardamum's side was already correct (`AddressbookUpdateCommand` maps an empty string to `Some(None)`, i.e. "clear this field"); the intent died at the io-webdav boundary, whose update coroutine took a flat `CarddavAddressbook` that cannot tell "leave alone" from "remove" and whose PROPPATCH body only ever emitted `<D:set>`. **Fix:** io-webdav's `proppatch_body` and `WebdavProppatch` now take a removal list and emit `<D:remove>` (RFC 4918 §9.2), and a new `CarddavAddressbookPatch` carries the doubly-optional properties that `CarddavAddressbookUpdate` and the client's `update_addressbook` take. cardamum forwards its `AddressbookDiff` straight through, which also removed a read-then-merge round-trip. Re-verified: clearing the description leaves the color alone and vice versa, a set and a removal ride in one PROPPATCH, and a cleared value can be set again afterwards.

### Provider-specific behaviour (not bugs)

- **P1: the card id is the resource name, used verbatim.** `card create` mints `<uuid>.vcf` and every other verb addresses that exact string; the vCard's own `UID` is preserved in the body but is not the addressing key. A card created with `UID:cardamum-test-001` was addressed as `5025e306-….vcf` and still read back `UID:cardamum-test-001`. This is the 2026-07-17 F1/F2 fix holding.
- **P2: vCard 4.0 is stored and returned as 3.0.** A card written `VERSION:4.0` reads back `VERSION:3.0`, and parameter values are upcased (`TYPE=work` becomes `TYPE=WORK`). Fastmail down-converts.
- **P3: Fastmail requires a `UID`** on every vCard; a UID-less card is rejected with `403 valid-address-data: Missing mandatory UID property`. See F3 below for the client-side surface.
- **P4: unknown vCard properties survive a round-trip.** An `X-CUSTOM-FIELD` written on create reads back unchanged, so no custom-data stash is needed on CardDAV.
- **P5: the collection CTag is a `<unix-ts>-<counter>` pair** whose counter moves on every card write while the timestamp part stays at collection-creation time. Only the whole string is meaningful as a change marker.

### Observations

- **O1 (still open): error bodies are dumped raw.** A 404 (bad book or card) surfaces Fastmail's full HTML error page as the message, and a 403 surfaces the DAV XML. A 412 is worse in the other direction: the body is empty, so the message reads `WebDAV server returned HTTP 412:` with nothing after the colon. Functional (correct exit codes throughout) but a trimmed, status-aware message would read better.
- **O2: mutations with nothing to mutate report success.** `addressbook update -k <book>` with no field flag sends an empty PROPPATCH and prints "successfully updated". Harmless, but a "nothing to update" bail would be more honest.
- **O3: a stdout write failure panics instead of exiting.** Observed once when a `--json addressbook list` was piped into a command that failed to exec: `called Result::unwrap() on an Err value: Print JSON to stdout error / Caused by: Broken pipe (os error 32)`, from the `.unwrap()` in `ErrorReport::eval` (pimalaya-cli src/error.rs:25), which tries to print the error report to the same already-broken stdout. This is a latent unwrap in pimalaya-cli affecting every Pimalaya CLI, not cardamum specifically. It did not reproduce on demand (the outcome depends on whether the write lands in the pipe buffer before the reader is gone).
- `card list --json` and `card read --json` both expose the server ETag, usable for `card update --if-match`.
- Exit codes are correct across the run: 0 on success, 1 on a server or client error, 2 on a clap error. Errors go to stdout in both text and JSON mode, per the output-streams convention.
- The account still carries five leftover `io-webdav-test-*` addressbooks from prior io-webdav integration runs: unrelated clutter, left untouched.

## History

The 2026-07-17 run found and drove three fixes, all re-verified as holding in this run:

- **F1 / F2 (fixed):** `card read` 404'd and `card update` / `card delete` reported success while missing the real card, because io-webdav defined the card id with `.vcf` stripped but never re-appended it. io-webdav now carries the server's resource name verbatim end-to-end, and cardamum names new cards `<uuid>.vcf`.
- **F3 (resolved):** a vCard with no `UID` no longer dumps the raw DAV error. The `403 valid-address-data` case now prints an actionable hint ("providers disagree… most require a UID; some, e.g. iCloud, require vCard 3.0 with an N property") followed by the server's own `<responsedescription>`. Confirmed still in place, on both the text and the JSON path.

## Verdict

The shared surface on Fastmail CardDAV passes end to end: `addressbook` (list / create / update / delete, all flags including clearing, `--json`, aliases, backend selection) and `card` (list / create / read / update / delete, paging, `--json`, `--if-match`), with correct exit codes and a clean read-back after every mutation. The two defects this run found, **F5** (`card update` on an absent id fabricating a card) and **F4** (metadata clearing being a no-op), were both fixed the same day and re-verified live, along with a regression sweep over the surrounding commands; see the [carddav-write-guards](../../log/2026-08-09-carddav-write-guards.md) log entry. Everything the 2026-07-17 run fixed is still fixed. **Release-ready for the CardDAV surface** once the io-webdav removal work ships (currently consumed via a local-path patch). Only cosmetics remain open: the raw error bodies (O1) and the no-op-reports-success case (O2).
