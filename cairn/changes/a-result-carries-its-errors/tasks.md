---
cairn: tasks
change: a-result-carries-its-errors
---

# Tasks

## Working rules

- Read first: this change's proposal.md and delta.md; himalaya/cairn/changes/a-result-carries-its-errors/{proposal,delta}.md and himalaya/cairn/log/2026-11-14-a-result-carries-its-errors.md; neverest/cairn/changes/a-refusal-carries-its-code/proposal.md; neverest/src/error.rs (`Failure`, `code`, `refusal_code`) and its readers (`refusal_code` in neverest/src/dav/client.rs, src/msgraph/client.rs, src/gpeople/client.rs; `google_code` in src/throttle.rs); himalaya/src/error.rs (`ItemError` alias, `missing_message`); pimalaya-cli's `error::ItemError` (cli/src/error.rs).
- Every cargo command runs as `nix develop --command cargo ...`. `cargo fmt` after each phase.
- An agent edits, builds and tests; it runs no git write command (no add, commit, checkout, stash, restore, reset). The session that owns the repository commits each phase with the message given here, no AI trailer.
- Never adjust production code to fit a test. A regression test must fail against the code before the phase.
- No em dashes; code comments wrap at 80 columns; markdown is never hard-wrapped.
- Phase checks, each one: `cargo test --all-features`, and the CI clippy matrix with `-- -D warnings` over `--all-targets`: default, `--all-features`, `--no-default-features --features rustls-ring`, `--no-default-features --features native-tls,jmap`, `--no-default-features --features rustls-ring,vdir,pimdir`, and `--no-default-features --features rustls-ring,<b>` for each of carddav, jmap, msgraph, gpeople, vdir, pimdir; then `cargo test --no-default-features --features rustls-ring,vdir`.

## Phase 0, outside this repository

- [x] io-pim-discovery's stream-0-4 is committed and pushed; note its commit hash for Phase 1. Stop here until it is. Committed locally as 1c3d195, not pushed: Phase 1 links it by a path patch (owner ruling).

## Phase 1, the libraries (commit `build: update pimalaya-stream to 0.4 and the io-* crates`)

- [x] Cargo.toml: `pimalaya-stream` 0.4, `io-http` 0.8, `io-jmap` 0.9, `io-msgraph` 0.9, dev `io-oauth` 0.7.
- [x] `[patch.crates-io]`, sorted, no comment: io-gpeople e4a9cac80d023688118dbae88f8535145e8d2c9b, io-pim-discovery (Phase 0 rev), io-webdav 185ab63be5859c5ef6115bef97e4df5cb1187973, pimalaya-cli 0be1b1f08d19ccb9d16ee8e1d4266c27c96e0cdd. A crate the owner released meanwhile is taken from crates.io instead, its pin dropped. io-pim-discovery is `{ path = "../io-pim-discovery" }` until 1c3d195 is pushed (owner ruling); none of the four was released meanwhile.
- [x] `cargo update`; the lock holds pimalaya-config 0.4.1.
- [x] src/config.rs:594-609 `TlsConfig::into_tls`: `server: self.cert.map(|cert| TlsServer { cert }), client: None` (import `TlsServer`).
- [x] `cargo tree -d --all-features -e normal`: no pimalaya-stream, io-http or io-oauth twice.
- [x] tests/google.rs:148 and tests/microsoft.rs:94 still build (`Tls::default()` against io-oauth 0.7).
- [x] CHANGELOG: the dependency entry follows; Fixed: a credential command prompting for a password (`systemd-ask-password`) no longer hangs (pimalaya-config 0.4.1).
- [x] Phase checks pass.

## Phase 2, the codes (commit `feat: code a server's refusal and a missing card or address book`)

- [ ] src/error.rs: `PermissionDenied`, `QuotaExceeded`, `RateLimited`, `CardNotFound`, `AddressbookNotFound`, `CardUnreadable` (and `AddressbookUnreadable` only if Phase 3 meets it), declared in every build; the enum's `allow(dead_code)` gated as neverest gates it; the header doc (:7-8) says a failure without a code prints `"code": null`.
- [ ] src/error.rs: `error::code(err)` and the chain walk over one reader per enabled backend (a slice of `fn` gated by `cfg`, as neverest/src/error.rs does); the `ItemError` alias.
- [ ] CardDAV reader in src/carddav/ (its client module): 507 or `quota-not-exceeded`, 429, 403 with `need-privileges` or no `DAV:error` body, 404/410 on a card request. src/carddav/backend.rs:215-232 keeps the typed io-webdav error as a cause instead of `anyhow!`.
- [ ] Graph reader in src/msgraph/: `ErrorQuotaExceeded` or 507, 429, 403, 404 or 400 `ErrorInvalidIdMalformed` on a contact request.
- [ ] People reader in src/gpeople/ with a copy of `google_code` and the `google_refusal` parts it reads.
- [ ] JMAP: `format_set_error` (src/jmap/backend.rs:113, :146, :162, :236-238, :278-280, :301-306) becomes a coded error builder; `get_card` not found (:211) `card-not-found`.
- [ ] vdir: a missing card file `card-not-found` (read io-vdir 0.3's `VdirClientError` for the variant), a missing address book `addressbook-not-found` (src/vdir/backend.rs:204).
- [ ] pimdir: src/pimdir/backend.rs:562 `card-not-found`, :514-517 `addressbook-not-found`.
- [ ] src/account/check.rs:289: rows coded through `error::code`.
- [ ] Tests, each through the real error path and failing before the phase: CardDAV and Graph over a scripted stream (`WebdavClientStd::new` and `MsgraphClientStd::new` take any `Read + Write`; reuse the `Script` of src/gpeople/backend.rs:405-590), People 403 and 429 through that `Script`, JMAP SetError types, a vdir and a pimdir `card read` of an unknown id under `--json` (tests/ over a temp vdir) printing `card-not-found` and exiting 1.
- [ ] Spec: "A JSON error carries a stable code" and "A server's refusal carries its code" folded (see delta.md).
- [ ] Phase checks pass.

## Phase 3, the listings (commit `feat!: list the cards and address books a listing could not read`)

- [ ] `addressbook list` (src/shared/addressbook/list.rs:96-102) and `card list` (src/shared/card/list.rs:83-106) outputs gain `errors: Vec<ItemError>`, required in their schema; the unified client (src/shared/client.rs:470-495) carries the entries from each backend beside the items.
- [ ] A body that does not parse or is not UTF-8 (src/shared/card/project.rs:47-50, src/shared/card/list.rs:237-240): listed as today, plus a `card-unreadable` entry.
- [ ] vdir (src/vdir/backend.rs:101-125): `list_readable_items`, one `card-unreadable` entry per failed file; a vanished card left out silently.
- [ ] JMAP (src/jmap/backend.rs:174-197, src/jmap/project.rs:30-43): one card that does not project is an entry, the others listed.
- [ ] pimdir (src/pimdir/backend.rs:458-494): a blob read error is an entry, the card listed with its preview; a missing blob unchanged.
- [ ] Paging: whole-read backends print their entries on the first page only; pimdir on the page holding the card.
- [ ] A non-empty `errors` returns `PartialOutcome::default()` after printing (exit 3).
- [ ] Text: `card list`'s paging fields move from the footer into the header after `Addressbook:`. Both lists print their `Error:` lines last, after the items and one blank line, nothing at all when `errors` is empty. The text tests (src/shared/card/list.rs:313-357) follow the moved paging and gain a case with an entry.
- [ ] json-schema: `cardamum-item-error` registered; src/json_schema.rs:354 knows it.
- [ ] MCP: the instructions (src/mcp.rs:44-56) name `errors`; tests/mcp.rs gains a `card_list` over a vdir holding an unreadable file, asserting `isError: true` with `structuredContent.errors`.
- [ ] REPL: a session keeps its client after a partial outcome.
- [ ] Tests: tests/ over a temp vdir with one file `chmod 000` (unix only): `card list --json` lists the others, prints the entry and exits 3; a card that does not parse lists with its entry; tests/exit.rs gains the exit 3 case.
- [ ] Spec: the listing requirements folded (see delta.md).
- [ ] Phase checks pass, the live suites' expectations read (tests/common.rs `assert_cards`) and adjusted for the `errors` key if they compare whole objects.

## Phase 4, CardDAV rows (after io-webdav's a-listing-returns-what-it-could-not-read is pushed and pinned)

- [ ] Pin io-webdav to that commit.
- [ ] src/carddav/backend.rs:105-120: each unread row an entry (`card-unreadable`, its status coding `permission-denied` for a 403); `truncated` an entry with a `null` id and no code.
- [ ] A scripted multistatus test with a 403 row and a 507 row.
- [ ] Spec: "A listing is whole or fails" says it. Commit `feat: list the CardDAV rows a listing could not read`.

## Landing

- [ ] CHANGELOG (Added, **Breaking:**, Changed, Fixed as in the proposal), MIGRATION.md ("JSON output"), README exit statuses, doc comments of the outputs.
- [ ] Delta folded into cairn/spec/commands.md, backends.md and mcp.md; log cairn/log/2026-11-14-a-result-carries-its-errors.md (or the landing date; frontmatter `cairn: log`, `change`, `date:` as cardamum's recent logs write it) naming what landed per phase, the evidence and the capabilities moved; status landed; the folder moved to cairn/changes/archive/.
- [ ] The live suites (google, microsoft, fastmail) run by the owner on the result.
