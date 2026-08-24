---
cairn: change
change: first-run-wizard-frame
---

# Delta

## ADDED Requirements

### Requirement: A named command runs the wizard
A `configure` command (alias `wizard`) SHALL run the wizard by name, without the welcome, since whoever typed it knows what it does. It refuses to run when stdin is not a terminal, naming the sample configuration to write by hand instead.

### Requirement: The offer is a hook, not a gate
A missing configuration SHALL raise an offer to generate one, from a bare invocation and from any command needing an account. The offer never ends the process: a command carries on afterwards either way, so accepting gives it a chance to work and declining leaves it to fail on the configuration it still has not got. A bare invocation has nothing to carry on to, so a declined offer falls back to the help, which is also what someone already configured gets. Nothing is offered when stdin is not a terminal or `--json` is set.

### Requirement: The welcome names the missing path
The welcome SHALL name the configuration path that was looked for, which is the one `-c` or `CARDAMUM_CONFIG` gave or the default location, so a mistyped path shows up as itself rather than as a generic first run. It frames the product, points at the documented sample, and names the command that runs the wizard again later.

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for, a missing named account lists the accounts the configuration does hold, and a missing default names both ways of picking one.

### Requirement: A generated account reads in a deliberate order
The serializer SHALL decide what a generated account holds, so a defaulted field is omitted and no field is enumerated twice, but the rendering SHALL order what it emits: the groups run most-defining first, an unrecognised group renders after them rather than being dropped, a group's `server` key reads before the credentials qualifying it, and a blank line separates groups.

## MODIFIED Requirements

### Requirement: Saved where the configuration lives, printed when redirected
When stdout is a terminal and `--json` is not set, the wizard SHALL offer to save the generated account to the configuration file, which is where `-c` or `CARDAMUM_CONFIG` pointed or the default location, creating the parent directory as needed. It SHALL NOT prompt for that path. A file that does not exist is written whole; one that does is appended to as plain text, never parsed and re-serialized, so comments, ordering and formatting survive. Declining the save SHALL fall back to printing so the generated account is never lost. In JSON mode or when stdout is redirected the document is emitted straight to stdout, so `cardamum configure > config.toml` and any script keep working.

### Requirement: The generated account claims the default only when it is free
The generated account SHALL claim `default = true` only when no other account in the configuration already does, and SHALL take a name the configuration does not already hold, suffixed until free. A second `[accounts.<name>]` table makes the whole document fail to parse, and two defaults resolve to whichever the account map yields first. This supersedes the account never being the default: that protected an existing default, which claiming one only when it is free protects just as well, without leaving the first account a new user generates unreachable without hand-editing.

## REMOVED Requirements
