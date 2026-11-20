---
cairn: log
change: the-contract-audit-rulings
date: 2026-11-20
---

# The contract audit rulings

The owner's rulings on the CLI contract audit of 2026-11-20 (A1, A4, section C), done in one pass with calendula. Capabilities **commands**, **config**, **mcp** moved.

- **pimalaya-cli working tree** (unreleased 0.8): `account check` prints `{accounts: [{account, backends: [{backend, ok, code, error, tls}]}]}`; a bare `--json` run fails `usage-invalid` exit 2 through `clap::meet_bare_invocation`, replacing cardamum's copy; `completion`, `manual` and `json-schema` without argument are clap refusals, exit 2; `--version` after a subcommand names the binary. Cargo.toml patches pimalaya-cli to ../cli until the release.
- **TLS (A1)**: `TlsConfig` gains `client` (`cert`, `key`) and `inherit`, the account `tls`, inherited field by field by each network backend's (`client` as one); the check row's `tls` reads what the table configures, `null` over a cleartext `http` server or `jmap://`.
- **Capabilities (CRD-X1)**: `card.delete`, `card.delete.if-match`, `addressbook.update`, `addressbook.update.description`, `addressbook.update.color`, `card.search.sort` replace the store-flavoured names.
- **Queue (A4, CRD-X3)**: `pimdir queue list` lists every pending row through io-pimdir's `pending_actions`; io-pimdir 0.18.1 fails a whole collection's read on one undecodable row and exposes no undecoded row, so the item error names the address book (`id` `null`) and leaves its pending rows out. `queue show` builds the list row (`listed_row`), an applied row the receipt alone.
- **Text cursor (CRD-X2)**: `pimdir search` sets the listing's next-page line from `nextPage`.
- **Codes (CRD-X4, TWN-X2)**: `page-token-invalid` on `card list` and `jmap contact-card query`; `stdin-taken` on the composer refused in a session; `card-invalid` on a source that is no vCard.
- **Duplicate (CRD-X6)**: `addressbook create` probes the pimdir store first, a store it cannot open leaving the refusal to the capability check.
- **Help (CRD-X5, X7)**: `protocol_exit` appends the exit 3 line under every `carddav`, `jmap`, `msgraph` and `gpeople` leaf; `accounts` is a hidden alias of `account`.
- **Evidence**: `a_row_names_its_tls_unless_cleartext`, `a_client_certificate_needs_its_key`, `an_account_tls_fills_each_field_a_backend_leaves_unset`, `a_bare_json_invocation_is_a_refused_command_line`, `every_pending_row_lists_whatever_its_kind`, `an_undecodable_pending_row_is_an_item_error`, `a_queued_row_prints_its_list_row`, `the_refusals_of_a_store_carry_their_codes` (`queue show 999`), `a_card_list_page_token_it_never_printed_is_coded`, `a_page_names_the_next_and_its_errors` (`--page-token 2`), `an_existing_addressbook_is_a_duplicate_before_a_missing_capability`, `every_protocol_command_documents_exit_status_3`, `a_source_that_is_neither_a_file_nor_a_card_is_refused`.

CHANGELOG (Added, Changed, Fixed). Spec: commands, config.
