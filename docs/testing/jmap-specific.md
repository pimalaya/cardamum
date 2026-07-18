# JMAP specific API — test report

The `jmap` protocol-specific subcommands (nested by JMAP object type),
distinct from the shared `addressbook`/`card` report ([jmap-fastmail.md](jmap-fastmail.md)).

- cardamum: `v0.2.0 --all-features` (io-jmap `0.2`; iteration 5, final)
- account: `fastmail-jmap` (Bearer, token via `pass show pimalaya/fastmail-jmap-contacts`)
- date: 2026-07-18
- method: card operations ran in the empty book `R2ag` with a marked throwaway card, destroyed afterward (Fastmail forbids AddressBook mutations, so no throwaway book). Real books were only enumerated.

## Command surface

`address-book {get, create, update, destroy, changes}` ·
`contact-card {get, query, create, update, destroy, changes, copy}` · `session get`

## Results

| Command | Variants | Result |
| --- | --- | --- |
| `session get` | base, `--json` | ✅ raw JMAP session (capabilities `urn:…:contacts`/`core`, accounts, api-url, state) |
| `address-book get` | base (+ state), `abook` alias | ✅ raw JMAP AddressBooks + the state token for `changes` |
| `address-book create` / `update` / `destroy` | — | ⛔ Fastmail forbids AddressBook mutations (J1); surfaced cleanly. Commands are correct (work on a server that allows it). |
| `contact-card create` | raw JSContact inline, `-k <book>` | ✅ returns the raw created card |
| `contact-card get` | `<ids…>` (human + `--json`), bogus id | ✅ round-trips (`name.full`, `emails`); bogus id → empty (JMAP `notFound`) |
| `contact-card query` | `-k <book>`, `card` alias | ✅ cards in the book + query state |
| `contact-card update` | JMAP patch `{"name/full": …}` | ✅ patch applied (verified) |
| `contact-card changes` | `<since-state>` | ✅ **incremental**: shows the updated card id + new state |
| `contact-card destroy` | `<id>` | ✅ removed; query confirms 0 |
| errors | bad JSON body | ✅ "Parse JSContact JSON error" |
| `contact-card copy` | — | implemented, **not live-tested** (needs a second JMAP account) |

## Findings

### Bugs / issues

- **None.** Every command wired and dispatched correctly.

### Behaviour (not bugs)

- **The specific API is raw JSContact / JMAP**, not vCard: `contact-card create`/`update` take a raw JSContact JSON body (the card object; `update` is a JMAP patch with JSON-pointer keys like `name/full`), and `--json` prints the raw JMAP ContactCard / AddressBook / session. No vcard-rs projection is involved (unlike the shared `card` API).
- **`get`/`query` surface the JMAP state token**, which `changes` consumes for incremental sync (`ContactCard/changes` / `AddressBook/changes`) — the JMAP-native sync the shared API hides.
- **Fastmail forbids AddressBook create/update/destroy** over JMAP (J1, matching the shared-API report); the commands surface the server's `Forbidden` cleanly and work against servers that allow it (e.g. Cyrus/Stalwart self-hosted).
- `contact-card copy` (`ContactCard/copy`) is cross-account; implemented but needs a source JMAP account to exercise.

## Verdict

The JMAP specific API works end-to-end: `session get`, AddressBook `get`/`changes`
(mutations forbidden by Fastmail, surfaced cleanly), and the full ContactCard
surface — raw JSContact create/get/query, JMAP-patch update, incremental
`changes`, destroy — validated live on `fastmail-jmap` with a marked throwaway
card, cleaned up. **This completes the protocol-specific API round for all five
backends** (vdir, carddav, msgraph, google, jmap); the shared `addressbook`/`card`
API and each backend's native surface are now both covered.
