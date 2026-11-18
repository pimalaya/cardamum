---
cairn: log
change: the-v1-pass
date: 2026-11-18
---

# A protocol refusal is a result, single reads print raw, the pimdir refusals coded

Owner rulings of 2026-11-18 (COH-5 with HIM-11 and HIM-21, D26's `request` half, the single-resource reads, the pimdir codes), in the shape himalaya landed in 002bf012, so the family prints one refusal.

- **COH-5**: src/shared/refusal.rs, a port of himalaya's: a server's refusal in the `carddav`, `jmap`, `msgraph` and `gpeople` namespaces, the `request` commands included, prints `{"refusal":{"status","reason","message","code","retryAfter"}}` and exits 3, where it failed with exit 1. One reader per protocol (`carddav::client::refusal`, `jmap::error::refusal`, `msgraph::client::refusal`, `gpeople::client::refusal`); `error::CodedCause` keeps the typed error a reader needs beside its code. `carddav` is included, as himalaya's `imap` and `raw` print their completion and calendula's `caldav` follows. `jmap contact-card copy` applied in part prints `{ids, errors}` (src/jmap/set.rs), exit 3. JMAP 403 and 429 code `permission-denied` and `rate-limited`, as himalaya's do. The unified `card` and `addressbook` commands keep the failure object.
- **Single reads raw**: `msgraph contacts get`, `contact-folders get`, `profile get` (`MsgraphClient::raw_get`), `gpeople connections get`, `contact-groups get`, `profile get` (`GpeopleClient::raw_get`), `jmap address-book get`, `contact-card get` (src/jmap/raw.rs) print the provider's payload as sent; their text decodes it to the same lines as before.
- **pimdir codes**: `store-account-ambiguous` and `store-account-unknown` (src/pimdir/client.rs `resolve_account`), `page-token-invalid` (src/pimdir/search.rs), `row-not-found` (src/pimdir/backend.rs, `pimdir queue cancel` refusing an uncancellable row before asking), `confirmation-required` (`pimdir queue cancel` with no `--yes` and no terminal).

Evidence: 16 tests failing on 0b2fc36 (src/{carddav,jmap,msgraph,gpeople}/cli.rs `a_refused_request_is_the_result` and siblings, `a_single_read_prints_the_payload_as_sent`, `a_get_prints_the_objects_as_sent`, tests/pimdir_refusals.rs), passing now; full suite and `clippy --all-features -D warnings` clean.

Known differences: a CardDAV 401 inside the `carddav` namespace prints as a refusal without the "no `auth`" hint (`account check` keeps it); session discovery still fails exit 1 before a namespace command runs, as in himalaya; `pimdir queue show` on an unknown row prints state `unknown`, as himalaya's; with `pimdir.account` unset cardamum counts every account of the store where himalaya counts the accounts holding its kind (open).

Spec: commands ("A protocol refusal is a result" added, "Specific commands are raw-faithful", exit statuses, receipts, the code table, the pimdir queue and search), backends ("A pimdir view is one account", "A server's refusal carries its code"). CHANGELOG (Added, Changed breaking).
