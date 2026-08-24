---
cairn: spec
capability: wizard
status: current
---

# Wizard

`cardamum configure` runs the interactive configuration wizard, and it is also offered when a bare `cardamum` or a command needing an account finds no configuration. The wizard discovers an account, tests it, and either writes it into the configuration file or prints it as a ready-to-save `[accounts.<name>]` block on stdout. Prompts render on stderr, so redirecting stdout into a config file works directly.

The flow mirrors Himalaya's wizard requirement for requirement, adapted to contacts: the two products share one onboarding model, so what is learned about one applies to the other.

### Requirement: A named command runs the wizard
A `configure` command (alias `wizard`) SHALL run the wizard by name, without the welcome, since whoever typed it knows what it does. It refuses to run when stdin is not a terminal, naming the sample configuration to write by hand instead.

### Requirement: The offer is a hook, not a gate
A missing configuration SHALL raise an offer to generate one, from a bare invocation and from any command needing an account. The offer never ends the process: a command carries on afterwards either way, so accepting gives it a chance to work and declining leaves it to fail on the configuration it still has not got. A bare invocation has nothing to carry on to, so a declined offer falls back to the help, which is also what someone already configured gets. Nothing is offered when stdin is not a terminal or `--json` is set.

### Requirement: The welcome names the missing path
The welcome SHALL name the configuration path that was looked for, which is the one `-c` or `CARDAMUM_CONFIG` gave or the default location, so a mistyped path shows up as itself rather than as a generic first run. It frames the product, points at the documented sample, and names the command that runs the wizard again later.

### Requirement: A generated account reads in a deliberate order
The serializer SHALL decide what a generated account holds, so a defaulted field is omitted and no field is enumerated twice, but the rendering SHALL order what it emits: the groups run most-defining first, an unrecognised group renders after them rather than being dropped, a group's endpoint key (`server`, `discover`, `home`, `home-dir`, `root`) reads before the credentials qualifying it, and a blank line separates groups.

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

### Requirement: Saved where the configuration lives, printed when redirected
When stdout is a terminal and `--json` is not set, the wizard SHALL offer to save the generated account to the configuration file, which is where `-c` or `CARDAMUM_CONFIG` pointed or the default location, creating the parent directory as needed. It SHALL NOT prompt for that path. A file that does not exist is written whole; one that does is appended to as plain text, never parsed and re-serialized, so comments, ordering and formatting survive. Declining the save SHALL fall back to printing so the generated account is never lost. In JSON mode or when stdout is redirected the document is emitted straight to stdout, so `cardamum configure > config.toml` and any script keep working.

### Requirement: The generated account claims the default only when it is free
The generated account SHALL claim `default = true` only when no other account in the configuration already does, and SHALL take a name the configuration does not already hold, suffixed until free. A second `[accounts.<name>]` table makes the whole document fail to parse, and two defaults resolve to whichever the account map yields first.

### Requirement: Guidance frames the run, not the document
The welcome banner SHALL render on stderr, and only from the offer: `configure`, asked for by name, goes straight to the prompts. The generated document itself SHALL carry no commentary, so what lands on stdout or in the config file is bare config. Once the account is written, the wizard SHALL report where it landed, under which name, and what to run next, naming `-a <NAME>` when another account holds the default.

### Requirement: Stop when nothing is discovered
When discovery yields no supported configuration for the given input, an empty result, the deadline elapsing with nothing completed, or a URL scheme filter leaving no entry, the wizard SHALL stop with a message stating it could not automatically discover a configuration for the input, and inviting the user to write the account by hand using the documented sample configuration (linked). It SHALL NOT prompt for any server field or emit a partial account. The wizard performs no hand-entry configuration of remote accounts.

### Requirement: Local backend auto-detected
A typed folder path or `file://` URL SHALL configure a local backend, auto-detecting the store kind from on-disk markers: a `pimdir.db` index means pimdir, an immediate subdirectory holding at least one `.vcf` card means vdir. The wizard SHALL prompt vdir-vs-pimdir only when both backends are compiled in and detection is inconclusive (an empty or ambiguous directory).

### Requirement: Discovery resolver is overridable
The DNS resolver backing discovery SHALL honour `CARDAMUM_DNS_RESOLVER` first, then the system resolver (`/etc/resolv.conf` on unix, the network adapters on windows), then Cloudflare's `1.1.1.1`. This avoids leaking the email domain to a third-party resolver and works around networks that block the default.
