---
cairn: delta
change: a-backend-left-out-is-named
---

# Delta

## ADDED Requirements

In backends.md:

### Requirement: A backend left out of the build is named
When the backend `auto` resolves to is one the build leaves out, the unified commands and `account capability` SHALL fail coded `backend-not-built`, the message naming the cargo feature ("This build leaves out the `gpeople` backend, rebuild with `--features gpeople`"), rather than take the next declared backend or report none configured. `--backend` naming a backend the build leaves out SHALL be refused as a command line (`usage-invalid`, exit 2), with the same message. Under `auto`, `account check` SHALL report each declared backend the build leaves out as a failed row coded `backend-not-built`, at its place in the resolution order. `account list` SHALL name every declared backend, built or not, in the resolution order, under `--json` in `backends`, and those the build leaves out in `notBuilt` too, the text marking each `(not built)`.

#### Scenario: An account over a backend left out
- GIVEN a build without `gpeople` and an account declaring `gpeople` alone
- WHEN `cardamum --json card list -k book` runs
- THEN it exits 1, its failure object coded `backend-not-built`, its error naming `--features gpeople`

#### Scenario: auto keeps its choice across builds
- GIVEN a build without `vdir` and an account declaring `vdir` and `carddav`
- WHEN `cardamum --json account capability` runs
- THEN it fails coded `backend-not-built`, where `--backend carddav` resolves to CardDAV

## MODIFIED Requirements

In backends.md:

### Requirement: Backend selection is local-first
The unified commands run over an `AddressbookClient` owning one `BackendClient` variant per compiled-in backend. The global `--backend` flag picks it: `auto` (the default) SHALL take the first configured-and-allowed backend in priority order, which puts the local stores (vdir, then pimdir) before the network ones so an offline store wins over a round-trip. A named value pins that backend and bails when the account has no matching config block. The order SHALL walk every block the account declares, the backends the build leaves out included, so the backend `auto` resolves to is the same in every build. The protocol-specific commands build their own client and ignore `--backend` entirely.

In config.md:

### Requirement: An unknown key is refused
Every table of the configuration, the root and each account included, SHALL refuse a key it does not know, failing the load with the key's name, so a typo is an error rather than a silently ignored option. The block of a backend the build leaves out (and `proxy` in a build with no network backend) SHALL load unread, so one file serves every build; a command reaching that backend fails coded `backend-not-built` (see [backends.md](backends.md), "A backend left out of the build is named"). A key inside such a block is not checked by that build.

#### Scenario: A typo in an account
- GIVEN `[accounts.example]` holding `defualt = true`
- WHEN the configuration is loaded
- THEN it fails naming the unknown field `defualt`

In commands.md:

### Requirement: account check fails when a backend fails
`account check` SHALL exercise every backend of the account `--backend` allows and print one verdict per backend, a backend the build leaves out failing coded `backend-not-built` under `auto`, under `--json` `ok`, the stable `code` and the error of a failed one (`code` `null` when the backend passed or its failure carries none), as the failure object of any other command names them. When any fails, it SHALL still print the report and exit 3, the family's partial outcome (`pimalaya_cli::error::PartialOutcome`), so a script reads the outcome from the exit code alone and which backends passed from the report. It mirrors Himalaya's `account check`.

### Requirement: An account lists what it serves
`account capability` SHALL print pimalaya-cli's `capability::AccountCapabilityOutput`, connecting nowhere: `{"account", "capabilities": [{"name", "backend", "level", "detail"}]}`, one row per capability, `backend` the backend the unified commands resolve to (`--backend` honoured), `level` one of `full`, `partial`, `server` and `none`, `detail` what the level leaves out or `null`. When the account configures no backend `--backend` allows, every row SHALL read `backend` `null` and level `none`, and the command SHALL succeed. When it resolves to a backend the build leaves out, the command SHALL fail coded `backend-not-built`, that backend serving nothing. The text SHALL be a list headed by `Account: <name>`, then one line per capability: the name, the backend, the level and the detail.

And "A JSON error carries a stable code", the table gains:

| Code | Raised when |
|---|---|
| `backend-not-built` | the backend the account resolves to is configured but left out of the build; the error names the cargo feature (see backends.md, "A backend left out of the build is named") |

## REMOVED Requirements
