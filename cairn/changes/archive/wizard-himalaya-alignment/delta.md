---
cairn: change
change: wizard-himalaya-alignment
---

# Delta

## ADDED Requirements

### Requirement: Discovery is time-bounded
The parallel discovery run SHALL be bounded by a short deadline (8 seconds) so a single unreachable endpoint cannot stall the interactive wizard.

### Requirement: One entry per service, then auth
The discovery list SHALL show one entry per reachable service, several records for the same service folding into one row that absorbs every advertised capability. After a service is picked, the authentication method SHALL be chosen in a second, service-specific prompt, skipped when only one method qualifies.

### Requirement: No addressbook alias pre-fill
The wizard SHALL NOT pre-fill any addressbook alias. Contacts have no special-use roles, so there is nothing to discover. A deliberate non-goal, not a gap.

### Requirement: Saved by default, printed when redirected
When stdout is a terminal and `--json` is not set, the wizard SHALL offer to save the generated config to a file, defaulting to `$XDG_CONFIG_HOME/cardamum/config.toml`, never overwriting without confirmation, and falling back to printing when declined.

### Requirement: The generated account is never the default
The generated account SHALL carry `default = false`, so a fragment merged into a config that already has a default does not hijack it.

### Requirement: Guidance frames the run, not the document
A welcome banner SHALL render on stderr before the first prompt (skipped in JSON mode). The generated document SHALL carry no commentary.

### Requirement: Stop when nothing is discovered
When discovery yields no supported configuration, the wizard SHALL stop with a message inviting the user to write the account by hand from the documented sample. It SHALL NOT prompt for any server field or emit a partial account.

## MODIFIED Requirements

### Requirement: Input orients the flow
A `scheme://` server URL now discovers from its host and its scheme narrows the discovered entries, instead of opening a hand-entry CardDAV flow. The wizard SHALL NOT offer any hand-entry of server fields.

### Requirement: Account name derived, not prompted
The wizard SHALL NOT prompt for an account name. It derives one from the domain's first label or the folder name, never from the email's local part.

### Requirement: OAuth folds into the API token
The API-token credential prompt now offers the OS keyrings and the OAuth token brokers through pimalaya-cli's shared picker, the brokers appearing only when the service advertises OAuth, replacing the hardcoded default command.

## REMOVED Requirements

### Requirement: Hand-entered CardDAV server
The wizard no longer configures a typed `scheme://` CardDAV server by prompting its authentication strategy and credentials. Such a server is now either discovered from its host or written by hand from config.sample.toml.
