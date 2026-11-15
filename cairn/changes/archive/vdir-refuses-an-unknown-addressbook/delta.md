---
cairn: delta
change: vdir-refuses-an-unknown-addressbook
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: vdir refuses an unknown addressbook
(backends) A vdir addressbook id SHALL name an addressbook only when it is one directory name, one path segment that is neither `.` nor `..` and holds no separator (a trailing one included), naming a directory under the home. Every vdir command naming another SHALL fail before anything is read or written, coded `addressbook-not-found`: `addressbook update`, `addressbook delete` and every `card` read and write with ``Addressbook `<id>` not found``, `vdir delete`, `vdir rename` and every `vdir item` command with ``Collection `<id>` not found``. A new name that is not one directory name SHALL be refused before anything is written: `addressbook create` and `vdir create` uncoded, the new name of `vdir rename` by io-vdir.

A write creates whatever directory it is handed, a listing of a missing one answers nothing and a delete removes whatever the path reaches, so without it a typo in `-k` would read as an empty addressbook or invent one, and an id reaching out of the home (`..`, `../keep`, `friends/..`) would remove a directory there.

#### Scenario: An id reaching out of the home
- GIVEN the account `perso` whose vdir home `outer/home` holds the addressbook `friends` alone, beside the directory `outer/keep`
- WHEN `cardamum --json addressbook delete -k ..`, `addressbook delete -k ../keep` or `card list -k nope` runs
- THEN each exits 1 with the failure object coded `addressbook-not-found`, and the home, `outer/keep` and the directory holding them are left as they were

## MODIFIED Requirements

### Requirement: An empty addressbook id is rejected
(commands) The shared `-k/--addressbook` resolver SHALL reject an empty id rather than pass it to a backend, where it silently addresses the collection instead of a member. The refusal SHALL be coded `addressbook-not-found`, an empty id naming no address book.

## REMOVED Requirements
