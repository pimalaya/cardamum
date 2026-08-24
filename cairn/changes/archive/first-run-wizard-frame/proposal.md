---
cairn: change
id: first-run-wizard-frame
status: landed
created: 2026-08-14
---

# Adopt Comodoro's first-run wizard frame

## Why

Comodoro's `first-run-wizard` landed the shape a Pimalaya CLI meets a newcomer with, and Himalaya and Ortie have taken it. Cardamum is where Himalaya was before: the wizard produces a good account, and everything around reaching it is the old shape.

`resolve_account` prompts, runs the wizard and calls `exit(0)`. The command someone actually typed never runs, whatever they answered, so configuring successfully and declining end the same way.

There is no way to run the wizard by name. Bare `cardamum` runs it, so a second account means running the binary bare, and someone already set up who types `cardamum` to see the commands gets a wizard instead of the help.

Nothing guards interactivity on the way in. A cron job or a `--json` caller hitting a missing configuration gets a prompt it cannot answer.

The welcome names no path, so a mistyped `-c` reads as a first run rather than as the typo it is.

An existing configuration can only be overwritten, never appended to, so generating a second account means merging it by hand.

Two of the three resolution failures say too little: a missing named account does not list the accounts that do exist, and a missing default does not name both ways of picking one.

## What

Discovery is untouched. The input prompt, the per-service flows, the auth prompts, the connection test and the local backend detection all stay exactly as they are.

A `configure` command (alias `wizard`) runs the wizard by name, with no welcome, since whoever typed it knows what it does. The welcome belongs to the offer, and gains the configuration path that was looked for.

The offer becomes a hook raised from a bare `cardamum` and from any command needing an account. It never exits: a command carries on afterwards either way. A bare invocation, having nothing to carry on to, falls back to the help, which is also what someone already configured gets.

Nothing prompts when stdin is not a terminal or `--json` is set.

The target path stops being prompted and comes from `Config::target_path`. A configuration already there is appended to as plain text rather than overwritten, under the two rules Comodoro established: a free account name, suffixed until it is, and a single default.

The three resolution failures each name what is missing and what to do.

The rendering gains Himalaya's ordering pass: the serializer still decides what is written, but the groups run most-defining first, a group's `server` reads before the credentials qualifying it, and a blank line separates them.

## Scope / non-goals

`CARDAMUM_CONFIG` already exists and already splits on `:`, so nothing to do there. Cardamum is the product the other two were catching up with on that point.

**One spec requirement is superseded rather than kept.** "The generated account is never the default" protects a merge into a configuration that already has a default, which is right, but it also means the very first account a new user generates is not the default: `cardamum addressbook list` straight after the wizard fails with `No default account found`, which is precisely the dead end this change exists to remove. Comodoro's rule gives the same protection and does not have that cost: claim the default only when no other account does. The generated account is still never able to hijack an existing default, because it only claims one when there is none to hijack.

"Saved by default, printed when redirected" survives in substance, but the path prompt and the overwrite confirmation inside it go: the path is resolved, and an existing file is appended to rather than overwritten, which is what makes an overwrite confirmation unnecessary.
