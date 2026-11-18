---
cairn: log
change: the-v1-pass
date: 2026-11-18
---

# io-pimdir 0.17 and the unified card search

cardamum moved onto the released pimalaya-cli 0.6.1, pimalaya-config 0.6.0, io-jmap 0.10.1 and io-pimdir 0.17.0 (pimdir storage-draft-14), no `[patch]`.

- **C1**: a store of another format version is refused as `store-version` in io-pimdir's one wording, `version::PimdirFormatRefusal` naming cardamum, where cardamum wrote its own neverest-specific text. Capability **backends** ("A pimdir store of another version is refused in io-pimdir's words") and the code table in **commands** moved.
- **C2**: a pimdir card write stays on `enqueue_with_body`, which now removes every `X-PIMDIR-` property before queueing (pimdir STORAGE §18). No card instruction is the user's to ask, so cardamum calls no `enqueue_user_body`. **backends** moved.
- **C4**: `pimdir queue show` and `pimdir queue cancel` take a row handle (`PimdirActionHandle`, `<store-id>.<incarnation>.<id>`) beside a bare id; another store's handle reads `reset`, and `cancel` refuses it as `store-reset`. No receipt prints a handle yet: the unified writes carry no pimdir row, and a reader exposes no store identity for `queue list`. **commands** moved.
- **CAR-14**: the composer's file is created new and 0600 through pimalaya-cli `file::private_options`, refusing a file or a link planted at its path. **commands** moved.
- **card-search**, first landing: `card search <query>...` on pimdir (the index, through `pimdir search`'s page walk) and vdir (every card of the scope matched by io-pimdir's `match_cards`), with `query-unsupported` for a store term on vdir, `card.search` and `card.search.relevance` in the capability tables, and the MCP tool `card_search`. CardDAV, JMAP, Microsoft Graph and Google People declare `none` until a search over the largest test address book is measured; the change stays active for them. **commands** and **mcp** moved.
