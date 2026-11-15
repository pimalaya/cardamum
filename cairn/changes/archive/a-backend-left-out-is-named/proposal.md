---
cairn: change
id: a-backend-left-out-is-named
status: landed
created: 2026-11-15
---

# A backend left out of the build is named

## Why

The block of a backend the build leaves out loads unread, so one file serves every build, but nothing used it after that: a command over an account declaring only that backend failed with an uncoded "No backend matching `auto` is configured", `account capability` printed every capability at `none` as if the account configured nothing, `account list` dropped the backend, and an account declaring a left-out backend first silently ran on the next one, a different backend from the one every other build resolves to. The owner ruled on 2026-11-15 (round 2, item 16) that the table stays accepted and a command using that backend fails coded `backend-not-built`, naming the cargo feature.

## What

- `auto` walks every block the account declares in the resolution order, built or not; landing on a left-out backend fails coded `backend-not-built` ("This build leaves out the `gpeople` backend, rebuild with `--features gpeople`"), in the unified commands, the REPL and MCP alike, and in `account capability`.
- `--backend <name>` naming a left-out backend is refused as a command line (`usage-invalid`, exit 2) with the same message, where it read "Invalid backend".
- `account check` under `auto` reports each left-out backend as a failed row coded `backend-not-built`, at its place in the order.
- `account list` names every declared backend, built or not, in the resolution order, and those left out in a new `notBuilt` list, the text marking them `(not built)`.
- `proxy` in a build with no network backend still loads unread: it is no backend, and the backends it serves fail on their own.
- Spec: backends ("Backend selection is local-first" modified, "A backend left out of the build is named" added), config ("An unknown key is refused" modified), commands ("account check fails when a backend fails", "An account lists what it serves" and the code table modified).
