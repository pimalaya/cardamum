---
cairn: delta
change: the-wizard-leaves
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: The top-level help orients a newcomer
(commands) A bare `cardamum` SHALL print the top-level help, which SHALL end with the shared Pimalaya footer (`pimalaya_cli::footer`) pointing at the issue tracker and the sponsoring page, as every other Pimalaya CLI does.

### Requirement: Subcommands carry their own data and chain
(commands) The dispatch point resolves the account, builds the appropriate client, and hands it to the subcommand, proposing no wizard (the rest of the requirement unchanged).

### Requirement: Account resolution is explicit or default
(config) `cli::resolve_account` SHALL select the account named by `-a`, else the one marked `default = true`. A config that exists but lacks the requested account, or has no default, is a hard error naming the fix. When no configuration is found at all, a command needing one SHALL fail naming the path it looked at and pointing at the documented sample. Cardamum SHALL NOT prompt to create a configuration, nor write one.

### Requirement: A credential command is spawned once per account
(config) The backends of one account SHALL resolve their credentials through a single pimalaya-config `SecretResolver`, which spawns each distinct command once and hands its value to every field naming it. `account check` reaches every configured backend, so an account naming one `pass` entry from its `carddav` and `jmap` tables would otherwise pay two key unlocks for one entry.

### Requirement: A DAV server may ask for no credentials
(config) `CarddavAuthConfig` SHALL carry a credential-less variant alongside Basic and Bearer, spelled `auth = "none"`, which emits no `Authorization` header.

## REMOVED Requirements

The wizard capability leaves whole, spec/wizard.md with it: cardamum no longer generates a configuration.

### Requirement: A named command runs the wizard
### Requirement: The offer is a hook, not a gate
### Requirement: The welcome names the missing path
### Requirement: A generated account reads in a deliberate order
### Requirement: Input orients the flow
### Requirement: Discovery is time-bounded
### Requirement: Each domain gets its best option without a question
### Requirement: The review lists each pick in plain words
### Requirement: Nothing discovered asks for a server URL
### Requirement: The user sees where a password goes
### Requirement: A password is typed once, tested, then stored
### Requirement: A refusal offers a way on
### Requirement: OAuth hands off to Ortie
### Requirement: No addressbook alias pre-fill
### Requirement: Account name derived, not prompted
### Requirement: Connection tested before saving
### Requirement: Saved where the configuration lives, printed when redirected
### Requirement: The generated account claims the default only when it is free
### Requirement: Guidance frames the run, not the document
### Requirement: Local backend auto-detected
### Requirement: Discovery resolver is overridable
