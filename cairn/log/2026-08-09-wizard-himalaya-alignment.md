---
cairn: log
change: wizard-himalaya-alignment
landed: 2026-08-09
---

# Align the wizard with Himalaya's

Rewrote the wizard against himalaya/cairn/spec/wizard.md, requirement for requirement, so the two products share one onboarding model again.

The visible changes: a stderr welcome banner replaces the twelve lines of guidance that used to head the generated TOML as comments, so the document is now bare config; the wizard offers to save to `$XDG_CONFIG_HOME/cardamum/config.toml` when stdout is a terminal, guarding an existing file behind a confirmation and falling back to printing, while a redirect or `--json` still prints straight to stdout; the account-name prompt is gone, the name deriving from the domain's first label (it used to prefer the email's local part); and the generated account carries `default = false` so a merged fragment cannot hijack an existing default.

Under that: discovery is now deadline-bounded at 8 seconds via `compose_all_within`, so one black-hole host cannot stall the prompt. `Discovered` carries `AuthCaps { basic, bearer, oauth }` folded across a service's advertised methods rather than one list row per (service x method), so the picker shows one row per service and a second, service-specific prompt chooses the scheme, skipped when only one qualifies. Several discovery records for the same service now fold into a single row absorbing every capability. The secret prompt delegates to pimalaya-cli's `wizard::keyring`, which offers the OS keyrings and the OAuth brokers (Ortie, pizauth, oama), replacing a two-way shell-command-or-plaintext prompt seeded with a hardcoded `ortie token show`.

The one deliberate loss: hand-entry is gone. A typed `scheme://` URL no longer opens a CardDAV configure-by-hand flow; it discovers from its host and its scheme narrows the results (`carddav(s)` and the HTTP family keep CardDAV, `jmap(s)` keeps JMAP, an unknown scheme is an error). Discovering nothing stops with a pointer to config.sample.toml rather than prompting for a server field. This means a non-discoverable self-hosted CardDAV server must now be written by hand. Himalaya made that trade deliberately and exact alignment was the ask, but it is the one thing a user could do before and cannot now.

Two Himalaya requirements have no contacts analogue and are recorded as non-goals rather than gaps: mailbox alias pre-fill (contacts have no special-use roles) and the pre-auth IMAP CAPABILITY probe (neither CardDAV nor JMAP advertises a mechanism list before authenticating, beyond the `WWW-Authenticate` probe discovery already runs).

Capabilities moved: wizard (rewritten), config (sample header).
