---
cairn: change
change: the-contract-audit-rulings
---

# Delta

## ADDED Requirements

### Requirement: TLS is set per account and per backend
Every network backend block SHALL take a `tls` table spelled as pimalaya-config's toml.md writes it (`provider`, `rustls.crypto`, `cert`, `client.cert`, `client.key`), and an account MAY carry one its backends inherit field by field, `client` as one (config.md).

## MODIFIED Requirements

### Requirement: Capability registry
A capability SHALL be named after the unified command it stands for, `<noun>.<verb>`, or `<noun>.<verb>.<argument>` for a flag of that command spelled as its long flag, never in the pimdir store's vocabulary (commands.md).

### Requirement: The pimdir queue is read and retracted
`pimdir queue list` SHALL print `{actions, parked, errors}`, every pending row of any kind, an undecodable queue an `action-malformed` item error and exit 3; `pimdir queue show` SHALL print the list row plus `state`, `appliedAt` and `refusals`, an unknown row failing `row-not-found` and a handle of another store `store-reset` (commands.md).

### Requirement: account check fails when a backend fails
`account check` SHALL print pimalaya-cli's list of accounts, each backend row carrying `tls` (commands.md, "The account outputs are the family's").

### Requirement: A protocol refusal is a result
Every protocol command SHALL end its `--help` with the meaning of exit status 3.

### Requirement: A bare invocation shows the help
A bare `cardamum --json` SHALL fail coded `usage-invalid`, exit 2 (config.md).

### Requirement: A JSON error carries a stable code
`row-not-found` covers `pimdir queue show`, `page-token-invalid` `card list` and `jmap contact-card query`, `stdin-taken` the composer in a session, `card-invalid` a source that is no vCard, and `addressbook-duplicate` a pimdir collection the store holds, before `capability-missing`.

### Requirement: pimdir searches cards through the store's index
The text SHALL print `Next page: --page-token <token>`, the token `nextPage` holds.

## REMOVED Requirements
