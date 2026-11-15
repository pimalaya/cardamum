---
cairn: log
change: default-is-what-the-server-states
date: 2026-11-15
---

# default is what the server states

`addressbook list` prints `default: true` on JMAP's `isDefault` book, the Graph default Contacts folder and People's myContacts, where the three backends hard-coded `false`. Task C33 of cli-alignment-2026-11-15, live finding J1. Capability **commands** moved.

- **src/jmap/backend.rs** `into_addressbook`, **src/msgraph/backend.rs** and **src/gpeople/backend.rs** `list_addressbooks`; src/shared/addressbook/types.rs documents the sources.
- **Evidence**: `the_default_address_book_is_the_one_the_server_states`, `the_contacts_folder_is_the_default`, `my_contacts_is_the_default`, each failing on the code before.

CHANGELOG (the unreleased `default` entry). Spec: commands ("An address book says whether it is the default" added).
