---
cairn: change
id: wizard-himalaya-alignment
status: landed
created: 2026-08-09
---

# Align the wizard with Himalaya's

## Why

Cardamum's wizard and Himalaya's had drifted apart. Himalaya's is specified in himalaya/cairn/spec/wizard.md and has since gained a welcome banner, a save-to-file offer, a derived account name, a time-bounded discovery, a two-step service-then-auth prompt, the shared OS keyring picker, and a hard rule against hand-entering server fields. Cardamum had none of those, and still prompted for an account name, still hand-configured a typed CardDAV URL, and still offered a bare shell-command-or-plaintext secret prompt.

Two products sharing one onboarding model means what is learned about one applies to the other. The drift was costing that.

## What

Fifteen behavioural changes, grouped:

**Output and framing.** A stderr welcome banner. A save-to-file offer defaulting to `$XDG_CONFIG_HOME/cardamum/config.toml`, with an overwrite guard, falling back to printing. The guidance that used to head the generated document as TOML comments moves into the banner, so the document is bare config.

**Naming.** The account-name prompt is gone; the name derives from the domain's first label (never the email local part). The generated account carries `default = false` so it cannot hijack an existing default.

**Discovery.** `compose_all` becomes `compose_all_within` with an 8-second deadline. A typed `scheme://` URL no longer opens a hand-entry flow: it discovers from its host and its scheme narrows the results. Discovering nothing stops with a pointer to config.sample.toml.

**Auth.** `Discovered` carries `AuthCaps { basic, bearer, oauth }` folded across a service's methods instead of one row per (service x method), so the list shows one row per service and a second service-specific prompt picks the scheme, skipped when only one qualifies. The secret prompt delegates to pimalaya-cli's `wizard::keyring`, which offers the OS keyrings and the OAuth brokers (Ortie, pizauth, oama) rather than a hardcoded `ortie token show`.

## Scope / non-goals

- **Removing hand-entry is the one user-visible loss**: a non-discoverable self-hosted CardDAV server can no longer be configured through the wizard. Himalaya made this trade deliberately, the escape hatch being a hand-written config from the documented sample, and exact alignment was the ask.
- **No addressbook alias pre-fill.** Himalaya pre-fills `mailbox.alias.*` because mail has special-use roles; contacts have none, so there is nothing to discover. Recorded as a deliberate non-goal in the spec.
- **No pre-auth capability probe.** CardDAV and JMAP advertise no mechanism list before authenticating beyond the `WWW-Authenticate` probe discovery already runs.
