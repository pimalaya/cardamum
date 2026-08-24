---
cairn: log
change: first-run-wizard-frame
landed: 2026-08-24
---

# Adopt Comodoro's first-run wizard frame

Took the shape Comodoro landed and Himalaya, Ortie and Carillon adopted, so the four CLIs now meet a newcomer identically. Discovery itself is untouched: the input prompt, the per-service flows, the auth prompts, the connection test and the local backend detection are exactly what they were.

The wizard split in two. `wizard::discover::run` is now the discovery half alone, returning the proposed name with the account; `wizard::configure` owns everything around it, and ships as the `configure` command (alias `wizard`) so a second account no longer means running the binary bare. It bails when stdin is not a terminal, naming config.sample.toml.

`cli::offer_configuration` is the hook, raised by a bare `cardamum` and by `resolve_account`. It never exits: the `exit(0)` that used to swallow the command someone actually typed is gone, so the configuration is looked up again and the command either runs or fails the ordinary way. A bare invocation has nothing to carry on to, so a declined offer falls back to the help, which is also what an already configured user, a `--json` caller, a non-terminal stdin, or `cardamum --account <NAME>` with no subcommand gets. The welcome moved onto the offer and names the path that was looked for, so a mistyped `-c` reads as itself.

Saving stopped prompting for a path: it writes to `Config::target_path`, whole when the file is absent and as a plain text append when it is not, so comments, ordering and hand-written formatting survive. The account takes a name the file does not already hold, suffixed until free, and claims `default = true` only when no other account does. That last point supersedes "the generated account is never the default": the old rule protected an existing default at the cost of leaving the very first generated account unreachable without hand-editing, and claiming the default only when it is free protects just as well without that cost.

`AccountConfig::render` carries the ordering pass ported from Himalaya: `RENDER_ORDER` runs the groups most-defining first with an unknown group rendering after them rather than being dropped, and `ENDPOINT_KEYS` lifts a group's endpoint (`server`, `discover`, `home`, `home-dir`, `root`) above the credentials qualifying it, since serialized alphabetically `carddav.server` reads under `carddav.auth`. `AccountConfig::default` gained `skip_serializing_if`, so a non-default account writes no `default = false` line.

The three account-resolution failures now each name what is missing: the path read, the accounts the configuration does hold, and the two ways of picking a default.

The top-level help gained the `long_about` framing the first run and the shared `footer!()` (issue tracker, sponsoring), which every other Pimalaya CLI already had and this one was missing.

Housekeeping: the `dirs` dependency went with the path prompt that used it, and `toml` came in as a dev-dependency for the four new configure tests.

Capabilities moved: wizard (offer, welcome, save, default rule, rendering order), config (resolution failures), commands (the `configure` meta command, the help frame and footer).
