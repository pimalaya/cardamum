---
cairn: log
change: default-is-a-name
date: 2026-11-15
---

# `default` is a name, and an account not taken is coded

`-a default` takes the account named `default`, an absent or empty `-a` the account marked default (owner ruling 2026-11-15, round 2 item 24), and the three ways of not taking an account carry their own code. Capabilities **config** and **commands** moved.

- **Cargo.toml**: pimalaya-config pinned to d5a33cba under `[patch.crates-io]`, whose `TomlConfig::take_account` reads `default` as a name and whose `AccountError::Missing` for it adds ", omit `-a` to take the account marked default". The Cargo.lock diff is its source line alone.
- **src/error.rs**: `ErrorCode` gains `AccountMissing`, `AccountDefaultMissing` and `AccountDefaultSeveral` (`account-missing`, `account-default-missing`, `account-default-several`), read from `AccountError` by `account_code`, the first reader of the chain walk behind `code`, which the failure object and an `account check` row both go through. The walk was `refusal_code`, now `typed_code`.
- **Evidence**: `an_account_not_taken_is_coded` (unit) and tests/account.rs (`an_unknown_account_name_is_coded_account_missing`, `the_name_default_is_a_name_like_any_other`, `no_account_marked_default_is_coded_account_default_missing`, `several_accounts_marked_default_are_coded_account_default_several`) through the binary under `--json`; on the old code each printed `"code": null`, and `-a default` took the marked account and succeeded. No doc, sample or test of the app read `-a default` as the default account.

CHANGELOG (**Breaking:** `-a default`; the codes), MIGRATION.md. Spec: config ("Account resolution is explicit or default", "Account resolution failures name what is missing"), commands (the code table).
