---
cairn: delta
change: default-is-a-name
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

In config.md:

### Requirement: Account resolution is explicit or default
`cli::resolve_account` SHALL load the configuration through pimalaya-config's `TomlConfig::load` and take the account through its `take_account`: the one named by `-a`, `default` being a name like any other, else, when `-a` is absent or empty, the one marked `default = true`. A config that exists but lacks the requested account, or has no default, is a hard error naming the fix. When no configuration is found at all, a command needing one SHALL fail with a `ConfigMissingError` naming the path it looked at (the first `-c` path, else the default location) and pointing at the documented sample, unless a person meets the welcome instead. Cardamum SHALL NOT prompt to create a configuration, nor write one.

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for, a missing named account lists the accounts the configuration does hold (and, for the name `default`, says to omit `-a` to take the account marked default), and a missing default names both ways of picking one. An account not taken SHALL carry its code under `--json`, read from pimalaya-config's `AccountError`: `account-missing` (`Missing`), `account-default-missing` (`NoDefault`) and `account-default-several` (`SeveralDefaults`).

#### Scenario: `-a default` names an account
- GIVEN `[accounts.a]` marked `default = true` and no account named `default`
- WHEN `cardamum --json -a default account capability` runs
- THEN it exits 1, its failure object coded `account-missing`, its error saying to omit `-a`

In commands.md, "A JSON error carries a stable code", the table gains:

| Code | Raised when |
|---|---|
| `account-missing` | no account has the name `-a` gives, `default` included (pimalaya-config's `AccountError::Missing`) |
| `account-default-missing` | no `-a` names an account and none is marked `default = true` (`AccountError::NoDefault`) |
| `account-default-several` | no `-a` names an account and several are marked `default = true` (`AccountError::SeveralDefaults`) |

## REMOVED Requirements
