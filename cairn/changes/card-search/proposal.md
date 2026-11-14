---
cairn: change
id: card-search
status: active
created: 2026-11-14
---

# Unified card search in the pimdir query language

## Why

[#25](https://github.com/pimalaya/cardamum/issues/25) asks for one search over every address book, whatever the backend, as khard and abq offer. Two things made it reachable:

- The unified API is the union of the backends since 2026-11-02, each declaring its level per capability, so a search no longer waits for every backend to support it.
- pimdir SEARCH (search-draft-11 §8) is a query language owned by a spec, not by a backend, and io-pimdir parses it (`PimdirQuery::parse`, with the §8.4 refusals). cardamum already answers it on pimdir (`pimdir search`, `pimdir count`), and Himalaya plans the same move for `envelope list` in v3 (himalaya cairn/changes/pimdir-search, "Later").

## What

A unified `card search <query>...`, capability `card.search`:

- **One grammar everywhere.** The query is SEARCH §8, parsed once by io-pimdir before any backend is reached, so a query is accepted or refused (`query-refused`, the §8.4 code and span) the same way on every backend.
- **Scope.** Every address book of the account, or the one `-k` names. The words of the query are joined into one query, as `pimdir search` does.
- **pimdir answers natively**, from the store's index, the same hits as `pimdir search` (1:1).
- **The other backends answer by matching.** Their server lists the cards of the address books in scope, and cardamum matches each decoded vCard against the query with io-pimdir's in-memory matcher (new, see Depends on), which follows SEARCH §5, §6 and §8.5. So a query answers the same hit set on CardDAV, JMAP, Graph and People as on an indexed pimdir store holding the same cards, a property the SEARCH §11 contact vectors check.
- **Terms of the store are refused, never dropped.** `tag:`, `flag:`, `is:`, `changed:`, `thread:`, `id:`, `linked:` and `account:` read the pimdir store, so a backend without one refuses the query with the new code `query-unsupported`, naming the term and its span, before connecting. Ignoring them would widen the answer silently, the outcome backends.md already forbids for `--if-match`. Terms that never match a card (`status:`, `priority:`, the mail roles) or exclude one (`date:` without `kind:contact`) answer as SEARCH says on every backend, with no refusal, so the answers stay equal.
- **Output.** The `card list` row of each hit with its address book, `nextPage`, and `complete`: the coverage verdict on pimdir, always `true` elsewhere since the whole listing was read. The coverage details, `matched` and the snippet stay on `pimdir search` (backends.md, "Unified outputs carry no backend details").
- **Order.** `--sort name` (SEARCH §8.2, the title folded) by default on every backend; `--sort relevance` is the index's own, so it is declared on pimdir alone (capability `card.search.relevance`).
- **Paging.** `--page`, `--page-size` and `--page-token` as `card list`.
- **MCP.** A read-only `card_search` tool, listed where the backend serves `card.search`.
- **Levels.** pimdir `full`; carddav, jmap, msgraph and gpeople `partial` (store terms refused, matched over the whole listing); vdir `none` for now (see Open questions).

## Non-goals

- **Pushing the query to the server.** CardDAV `addressbook-query`, JMAP `ContactCard/query`, Graph `$filter` and People `searchContacts` match differently from SEARCH: CardDAV's `i;unicode-casemap` keeps diacritics where §5 drops them, People matches prefixes of a few fields and caps its answer. Pushing a term down is only correct where the server's answer is provably a superset of SEARCH's, then matched again locally. That is a later optimisation, measured first, and never a different answer.
- Searching across accounts or backends in one invocation: a CLI acts on one account.
- Changing `card list`: it keeps listing one address book.

## Depends on

io-pimdir:

1. **An in-memory matcher.** Matches one card, from its body (`PimdirExtract::contact`, the `TEL`, `ADR`, `BDAY`, `PHOTO` marks and the `CATEGORIES` values the indexer records), against a parsed query: §5 folding, the `unicode61` token split, consecutive tokens with a trailing prefix (§8.5), the role fields of Annex A.6, `has:`, `category:`. It answers the §11 contact vectors as the index does.
2. **No SQLite outside pimdir.** io-pimdir already builds with `search` alone, without `client` or rusqlite (checked 2026-11-14 on master), but `search` also pulls ical-rs and mime-rs for the indexer. A carddav-only build can take that, or io-pimdir splits a lighter `query` feature (parser, folding, card matcher); decide when the matcher lands.

## Open questions

- **vdir.** It has no search protocol to mirror, but the unified API is not a protocol mirror: a local matcher over the address book's files answers the same hit set, as khard does, and vdir is the reporter's backend. Recommended: declare `card.search` `partial` on vdir in this change rather than `none`.
- **Cost on large remote books.** Every search lists the whole scope. Measure on the test accounts (a few thousand cards) before deciding whether pushdown is worth its own change.
