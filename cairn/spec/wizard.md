---
cairn: spec
capability: wizard
status: current
---

# Wizard

Bare `cardamum` (no subcommand) runs the interactive configuration wizard, and it is also proposed when a command finds no config at all. The wizard discovers an account, tests it, and either saves it to a config file or prints it as a ready-to-save TOML fragment on stdout. Prompts render on stderr, so redirecting stdout into a config file works directly.

The flow mirrors Himalaya's wizard requirement for requirement, adapted to contacts: the two products share one onboarding model, so what is learned about one applies to the other.

### Requirement: Input orients the flow
A single prompt SHALL accept an email address (or bare domain), a `scheme://` server URL, or a local folder path. An email, bare domain or server URL runs io-pim-discovery's parallel discovery; a folder is a local vdir or pimdir store. A server URL discovers from its host and its scheme narrows the discovered entries: `carddav`, `carddavs` and the HTTP-family schemes keep CardDAV, `jmap` and `jmaps` keep JMAP, and the proprietary entries (Google People, Microsoft Graph) are dropped when a scheme is given. An unknown scheme is rejected outright. The wizard SHALL NOT offer any hand-entry of server fields.

### Requirement: Discovery is time-bounded
The parallel discovery run SHALL be bounded by a short deadline (8 seconds) so a single unreachable endpoint (a firewalled port, a black-hole host) cannot stall the interactive wizard. Each mechanism runs independently; any that has not reported by the deadline is abandoned, and only what completed in time is offered. When nothing completes, the wizard stops (see "Stop when nothing is discovered").

### Requirement: One entry per service, then auth
The discovery list SHALL show one entry per reachable service (CardDAV, JMAP, Google People, Microsoft Graph), several records for the same service folding into one row that absorbs every advertised capability. After a service is picked, the authentication method SHALL be chosen in a second, service-specific prompt, skipped when only one method qualifies. CardDAV and JMAP offer the HTTP schemes their discovery advertised (Basic, Bearer), falling back to both when a mechanism named none. A detected Google or Microsoft account collapses to its dedicated contacts API, which is bearer-only and therefore prompts no scheme at all.

There is no CardDAV or JMAP analogue of Himalaya's pre-auth IMAP CAPABILITY probe: neither protocol advertises a mechanism list before authenticating, beyond the RFC 9110 `WWW-Authenticate` probe io-pim-discovery already runs inside `compose`. Nothing further is probed, deliberately.

### Requirement: OAuth folds into the API token
Cardamum runs no OAuth 2.0 grant itself, so OAuth SHALL NOT be a standalone list entry. It folds into the API-token credential prompt, which offers the OS keyrings (for a token the user generated) and the OAuth token brokers (Ortie, pizauth, oama) together, the brokers appearing only when the service advertises OAuth.

### Requirement: No addressbook alias pre-fill
The wizard SHALL NOT pre-fill any addressbook alias. Himalaya pre-fills `mailbox.alias.*` because mail has special-use roles (inbox, sent, drafts) a client must resolve without hand-editing ids. Contacts have no such roles, so there is nothing to discover and no alias model in the account config. This is a deliberate non-goal, not a gap.

### Requirement: Account name derived, not prompted
The wizard SHALL NOT prompt for an account name. It derives one from the input (the domain's first label, or the folder name) and uses it as the `[accounts.<name>]` table key; the user renames it by editing that key. The derivation never uses the email's local part.

### Requirement: Connection tested before saving
The account's connection SHALL be tested before the fragment is saved or printed, so a bad credential or endpoint stops the wizard instead of yielding a config that cannot connect. A flow that already validated its connection inline may skip the final test; no cardamum flow does today. The generated fragment is compact: only the `[accounts.<name>]` table stays a section header, other tables flatten into dotted keys, and empty tables and defaulted values are dropped.

### Requirement: Saved by default, printed when redirected
When stdout is a terminal and `--json` is not set, the wizard SHALL offer to save the generated config to a file, defaulting to `$XDG_CONFIG_HOME/cardamum/config.toml`, creating the parent directory as needed and never overwriting an existing file without confirmation. Declining the save, or declining an overwrite, SHALL fall back to printing so the generated config is never lost. In JSON mode or when stdout is redirected the document is emitted straight to stdout, so `cardamum > config.toml` and any script keep working.

### Requirement: The generated account is never the default
The generated account SHALL carry `default = false`, so a fragment merged into a config that already has a default does not hijack it. Being false, `default` is omitted from the printed TOML, and the user marks their choice with `default = true`.

### Requirement: Guidance frames the run, not the document
A welcome banner SHALL render on stderr before the first prompt (skipped in JSON mode), framing what Cardamum is, what the wizard does, and where the documented sample lives. The generated document itself SHALL carry no commentary, so what lands on stdout or in the config file is bare config.

### Requirement: Stop when nothing is discovered
When discovery yields no supported configuration for the given input, an empty result, the deadline elapsing with nothing completed, or a URL scheme filter leaving no entry, the wizard SHALL stop with a message stating it could not automatically discover a configuration for the input, and inviting the user to write the account by hand using the documented sample configuration (linked). It SHALL NOT prompt for any server field or emit a partial account. The wizard performs no hand-entry configuration of remote accounts.

### Requirement: Local backend auto-detected
A typed folder path or `file://` URL SHALL configure a local backend, auto-detecting the store kind from on-disk markers: a `pimdir.db` index means pimdir, an immediate subdirectory holding at least one `.vcf` card means vdir. The wizard SHALL prompt vdir-vs-pimdir only when both backends are compiled in and detection is inconclusive (an empty or ambiguous directory).

### Requirement: Discovery resolver is overridable
The DNS resolver backing discovery SHALL honour `CARDAMUM_DNS_RESOLVER` first, then the system resolver (`/etc/resolv.conf` on unix, the network adapters on windows), then Cloudflare's `1.1.1.1`. This avoids leaking the email domain to a third-party resolver and works around networks that block the default.
