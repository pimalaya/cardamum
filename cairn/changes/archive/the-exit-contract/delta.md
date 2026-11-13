---
cairn: delta
change: the-exit-contract
---

# Delta

## ADDED Requirements

### Requirement: Exit statuses
(commands) The binary SHALL exit 0 on success, 1 with the failure object printed, 2 on a command line the grammar refuses (an unknown flag or command, a missing argument, a bad value), and 3 on a partial outcome, a command that printed its result without fully succeeding (`pimalaya_cli::error::PartialOutcome`, `account check` with a failed backend). The binary SHALL parse its command line through `pimalaya_cli::clap::parse`: help and version print as clap prints them; a refused command line with a `--json` word before any `--` SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without the `error: ` prefix nor styles, and without `--json` clap's rendering on stderr.

#### Scenario: A refused command line under JSON
- GIVEN any configuration, or none
- WHEN `cardamum --json card list --nope` runs
- THEN it exits 2, prints nothing on stderr and prints `{"code":"usage-invalid","error":"unexpected argument '--nope' found…","sources":[],"backtrace":null}`

### Requirement: A call the grammar refuses is a usage failure
(mcp) A tool call whose arguments the tool's input schema admits but its command's grammar refuses (a `page-size` of `-1`) SHALL answer a result with `isError`, its failure object coded `usage-invalid` (`pimalaya_cli::error::usage_error`), the server going on.

## MODIFIED Requirements

### Requirement: account check fails when a backend fails
(commands) As before, but a failed backend SHALL exit 3, not 2, the family's partial outcome.

### Requirement: A JSON error carries a stable code
(commands) The code table gains `usage-invalid`: the command line, a REPL line or an MCP tool call's arguments do not match the grammar (pimalaya-cli's `CommonCode::UsageInvalid`); the command line exits 2.

### Requirement: The JSON contract is described and versioned
(commands) `cardamum-error` types `code` by the code vocabulary and pimalaya-cli's `usage-invalid`.

### Requirement: A session runs the CLI over one account
(repl) A parse error printed as the failure object SHALL be coded `usage-invalid`.

## REMOVED Requirements

None.
