---
cairn: tasks
change: an-unknown-addressbook-is-named
---

# Tasks

- [x] src/error.rs `addressbook_not_found`; src/carddav/backend.rs `addressbook_error`; src/msgraph/backend.rs `folder_error`; src/jmap/backend.rs `holds_addressbook`.
- [x] Tests: `an_unknown_address_book_is_not_found` (CardDAV, JMAP) and `an_unknown_folder_is_not_found` (Graph), failing on the code before.
- [x] CHANGELOG; fold the delta; log.
