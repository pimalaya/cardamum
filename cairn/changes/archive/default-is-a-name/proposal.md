---
cairn: change
id: default-is-a-name
status: landed
created: 2026-11-15
---

# `default` is a name, and an account not taken is coded

## Why

`-a default` took the account marked default, so an account named `default` was unreachable whenever another one was marked, and a script could not tell the three ways of not taking an account apart under `--json`, all coded `null`. The owner ruled on 2026-11-15 (round 2, item 24) that `default` is a name like any other, which pimalaya-config d5a33cba implements in `TomlConfig::take_account`.

## What

- Pin pimalaya-config to d5a33cba under `[patch.crates-io]`: an absent or empty `-a` takes the account marked default, `-a default` the account named `default`, its `AccountError::Missing` adding ", omit `-a` to take the account marked default".
- Code the three `AccountError` variants in `ErrorCode`: `account-missing`, `account-default-missing`, `account-default-several`, read off the chain by a reader beside the backend ones, so the failure object and an `account check` row carry them.
- Spec: config ("Account resolution is explicit or default" and "Account resolution failures name what is missing" modified), commands (the code table).
