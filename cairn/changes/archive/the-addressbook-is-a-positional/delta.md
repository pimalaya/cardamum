---
cairn: delta
change: the-addressbook-is-a-positional
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A collection command takes its address book as a positional
(commands) `addressbook delete` SHALL take the address book it deletes as a required positional, `addressbook delete <ADDRESSBOOK>`, and `addressbook update` as an optional one, `addressbook update [ADDRESSBOOK]`, falling back to `addressbook.default` as the card commands do, as Himalaya's `mailbox delete <MAILBOX>` takes its mailbox; `addressbook create <NAME>` takes its name so already. `-k/--addressbook`, the v0.3 spelling, SHALL stay accepted on both as a hidden alias of the positional: a second argument conflicting with it, so a line giving both is refused `usage-invalid`, and clap waives the required positional when that option is present. The help, the manual and the MCP input schema SHALL show the positional alone, the tool property named `addressbook` as the option named it, required on `addressbook_delete`. `--color` keeps its short letter `-C` on `addressbook create` and `addressbook update`.

#### Scenario: The v0.3 option still deletes
- GIVEN a vdir account holding the address books `personal` and `work`
- WHEN `cardamum --json addressbook delete work` runs, or `cardamum --json addressbook delete -k work`
- THEN either deletes `work` alone and prints `{"id":"work","queued":false}`; `cardamum --json addressbook delete work -k personal` is refused `usage-invalid`, exit 2, deleting nothing

## MODIFIED Requirements

### Requirement: A command with no address book is coded
(commands) A card command given no `-k/--addressbook`, and `addressbook update` given no address book, over an account setting no `addressbook.default` SHALL fail coded `addressbook-default-missing`, exit 1, the error naming the flag and the key: ``No default addressbook found, name one with `-k <ADDRESSBOOK-ID>` or set `addressbook.default` ``, as calendula refuses a missing default calendar (`calendar-default-missing`) and the family a missing default account (`account-default-missing`).

### Requirement: vdir refuses an unknown addressbook
(backends) The scenario "An id reaching out of the home" runs `cardamum --json addressbook delete ..` and `addressbook delete ../keep`, the positional; the requirement text is unchanged.

## REMOVED Requirements

None.
