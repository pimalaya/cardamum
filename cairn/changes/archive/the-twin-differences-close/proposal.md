---
cairn: change
id: the-twin-differences-close
status: landed
created: 2026-11-15
---

# The twin differences close

> Line X7 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), rows M5, L1, L3, L5, L8, L9, L19 and L23 of its X3 report (reports/x3-2026-11-15.md): where cardamum still differs from calendula, Himalaya or pimalaya-cli's spec.

## Why

- **M5.** `carddav get`, `report query` and `report multiget` print `contents` without `encoding`, where pimalaya-cli's output.md ("Bytes are never raw under JSON") asks `encoding` beside every vCard string and calendula's `caldav` reads print it; `carddav get` fails, uncoded, on a card that is not UTF-8.
- **L1.** The `addressbook list` text prints an `ID DEFAULT COLOR NAME` header and no `Account:` line, where calendula's `calendar list` and Himalaya's `mailbox list` print `Account:` then one line per collection.
- **L3.** `vdir create <NAME>` writes its directory name as the display name with no way to give another; `vdir rename <NAME> <TARGET>` and `vdir delete <NAME>`, where calendula has `vdir create <ID> -n --display-name`, `vdir rename <OLD-ID> <NEW-ID>` and `vdir delete <ID>`.
- **L5.** `addressbook delete` and `card delete` show `rm`, `card add` `new` and `msgraph contact-folders` `folders` as visible aliases; calendula's counterparts show none.
- **L8.** `addressbook update -n` takes a `<TEXT>`, calendula's `calendar update -n` a `<NAME>`.
- **L9.** The `-b` help is worded apart from Himalaya's, and `-b nope` is refused without the possible values, which Himalaya and carillon list.
- **L19.** A card not found reads ``Card `x` not found`` (``… not found in `y` `` on pimdir), where Himalaya says ``Message `x` not found in mailbox `y` ``.
- **L23.** The spec calls `card add` the hidden v0.3 name (it is `card create`), and MIGRATION.md's "The libraries" lists stale versions.

## What

- src/carddav/get.rs, src/carddav/report/entries.rs: the vCard through `CardContents` (`contents`, `encoding`), as `card read`; `carddav get` prints a card that is not UTF-8 as its bytes in text, as `card build`. Breaking for a script reading `carddav get` `contents` as the vCard: such a card failed and now reads as base64 (MIGRATION.md, JSON output). A REPORT row is the `address-data` text io-webdav reads, so its `encoding` is `utf-8`.
- src/shared/addressbook/list.rs: the text through pimalaya-cli's `Listing`, as `card list` and calendula's `calendar list`; the JSON unchanged.
- src/vdir/create.rs, rename.rs, delete.rs: `<ID>`, `-n/--display-name <NAME>` (the id when omitted, as today, so nothing changes for a line without it), `<OLD-ID> <NEW-ID>`, `<ID>`. src/shared/addressbook/update.rs: `-n <NAME>`. Value names only, nothing breaks.
- src/shared/addressbook/cli.rs, src/shared/card/cli.rs, src/msgraph/cli.rs: the four aliases hidden. Aliases are no contract.
- src/backend.rs, src/cli.rs: `-b` parsed by a `PossibleValuesParser` over `auto` and every backend an account may name, built or not, each with Himalaya's line, then mapped through `Backend::from_str`, so a backend left out still fails naming its feature; the help in Himalaya's words.
- src/error.rs `card_not_found` takes the address book; every backend and the CardDAV id check pass it, pimdir words the same. Codes unchanged.
- Spec: commands ("An address book listing names its account", "A CardDAV read carries its vCard beside its encoding", "The collection arguments are named as the twin's", added; "A former command name stays hidden", "A card is added", "A not-found failure is cardamum's sentence" and the "An unknown card" scenario of "A JSON error carries a stable code", modified), backends ("Backend selection is local-first" and "A CardDAV id addresses its own resource", modified). MIGRATION.md "The libraries" names the versions Cargo.toml requires.
