---
cairn: delta
change: an-unknown-key-is-refused
---

# Delta

Every requirement lives in config.md.

## ADDED Requirements

### Requirement: An unknown key is refused
Every table of the configuration, the root and each account included, SHALL refuse a key it does not know, failing the load with the key's name, so a typo is an error rather than a silently ignored option. The block of a backend the build leaves out (and `proxy` in a build with no network backend) SHALL load unread, so one file serves every build.

#### Scenario: A typo in an account
- GIVEN `[accounts.example]` holding `defualt = true`
- WHEN the configuration is loaded
- THEN it fails naming the unknown field `defualt`

## MODIFIED Requirements

### Requirement: One backend block per backend
Each account block SHALL carry at most one sub-block per backend (`vdir`, `pimdir`, `carddav`, `jmap`, `msgraph`, `gpeople`), each deserialized with `deny_unknown_fields` when its backend is compiled in. An account may declare several, and `--backend` picks between them (see [backends.md](backends.md)).

### Requirement: No colour is configured
The configuration SHALL hold no colour and no table shape. The tables v0.3 read for them SHALL be refused by name, the error pointing at MIGRATION.md: `table` at the root and in an account, the top-level `account`, and `addressbook.list` and `card.list.table` at the root and in an account.

#### Scenario: A v0.3 table preset
- GIVEN a configuration holding `table.preset`
- WHEN it is loaded
- THEN it fails naming the `table` table as removed and MIGRATION.md

## REMOVED Requirements
