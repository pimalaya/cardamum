---
cairn: log
change: the-v1-pass
date: 2026-11-19
---

# The v1.0.0 pass closes

- **CAR-12 (7)**: Graph `addressbook list` reaches every folder depth, each folder's `childFolders` listed in turn, every page read, a folder keyed by its id and named by its display name, as neverest lists Graph folders since 2026-11-17 (src/msgraph/backend.rs `list_addressbooks`). A folder below the top level was missing from `addressbook list`, though a command naming its id reached it. Test `every_folder_depth_is_listed`, failing on the one-level listing.
- **CAR-13 remainder**: the `-b` help says which commands read the flag (`addressbook`, `card`, `account check`, `account capability`, `repl`, `mcp`) and that `auto` takes the first configured backend whatever the command, a command it does not serve refused `capability-missing` (src/cli.rs), as the code does and calendula's help says.
- **CAR-14 remainder**: the composer's draft path is quoted by pimalaya-cli 0.6.1 `shell::quote`, double quotes on Windows where the shell line runs under `cmd /C`, single quotes elsewhere as before (src/shared/card/composer.rs). Not run on Windows.
- **CAR-5**: the Graph default-folder resolution is unchanged by the deeper listing: `GET /me/contactFolders` still lists the top level, whose parent is the default Contacts folder; the live check of 2026-11-18 stands.
- **CAR-1**: its delta, folded the first time as a row of the code table alone, is now the requirement "A vdir move or copy never replaces an item" in commands.

Spec: backends (Graph folders at every depth), commands (the composer quoting, CAR-1). Every task of the change ticked; the change is archived. C1 (`PimdirFormatRefusal`) had landed on 2026-11-18 (src/pimdir/client.rs `refusal`). Left outside the change: card-search's network declaration and live cells.
