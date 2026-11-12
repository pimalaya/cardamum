---
cairn: change
id: the-wizard-leaves
status: landed
created: 2026-11-12
---

# The wizard leaves

## Why

The owner decided on 2026-11-08 that the Pimalaya CLIs drop their setup wizards (himalaya and ortie did on 2026-11-11). Cardamum's hands an OAuth 2.0 sign-in to `ortie configure`, which Ortie 3.0 removed, and keeps a keyring dialogue, a review UI and a TOML writer in a binary whose job is contacts, for a flow the shared welcome of pimalaya-cli replaces family-wide. Second change of the-family-standards.

## What

- Delete src/wizard/ and the `configure` command (alias `wizard`), the `wizard` cargo feature and its place in `default`, the `offer_configuration` hook in src/cli.rs and the `long_about` pitch.
- Bare `cardamum` prints the help. A command needing an account and finding no configuration fails with `No configuration found at <path>, write one by hand: <sample URL>`, where it used to offer the wizard on a terminal. The shared welcome replaces that message in the-shared-kit.
- Drop the `cardamum-configure` JSON Schema, the wizard-only `test_account` of `account check`, and the `Serialize` derives, `skip_serializing_if` attributes, `is_default` and `AccountConfig::render` that only wrote a generated account.
- Unlike Himalaya's, io-pim-discovery stays, behind the `carddav` feature with `pacc`, `rfc6764` and `stream` only: `carddav.discover` resolves a bare domain through PACC and RFC 6764 on every run. `autoconfig` and `rfc8620` leave with the wizard, and so does shellexpand.
- `pimalaya_cli::prompt` stays: the composer's "Re-edit the card?" uses it.
- README, the sample header and AGENTS.md stop pointing at the wizard.

## Scope / non-goals

The shared first-run welcome, `-c` taking one path per flag and the pimalaya-cli and pimalaya-config 0.4 move are the-shared-kit's. `account check` keeps every connector it runs.
