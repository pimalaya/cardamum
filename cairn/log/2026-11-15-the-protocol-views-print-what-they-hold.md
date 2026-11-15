---
cairn: log
change: the-protocol-views-print-what-they-hold
date: 2026-11-15
---

# The protocol views print what they hold

`carddav propfind` and `vdir list` print the description and the colour they hold and no empty line, and `msgraph contacts` every email and phone of a contact. Tasks C31, C34 and C35 of cli-alignment-2026-11-15, live findings C3, M1 and V1. Capability **commands** moved.

- **src/carddav/propfind.rs**: Name, Description, Color, CTag, Sync token, each when sent. **src/vdir/list.rs**: Name, Description, Color when set, then Path. **src/msgraph/contacts/render.rs**: `contact_emails`, `contact_phones` (mobile, business, home), joined on one line each, a line only when there is a value.
- **Evidence**: `a_book_prints_the_properties_it_has`, `a_collection_prints_the_metadata_it_holds`, `a_contact_prints_the_emails_and_phones_it_has`, each failing on the code before.

CHANGELOG (Changed), MIGRATION.md ("Text output"). Spec: commands ("A command prints what the server returned").
