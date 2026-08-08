# AGENTS.md: Cairn activation

This repository follows **Cairn**, a language- and tool-agnostic convention for keeping a living spec, reviewable change proposals, and an honest history next to the code. The normative format lives in [pimalaya/cairn](https://github.com/pimalaya/cairn) (`CAIRN.md` for the rules, `GUIDE.md` for the by-hand procedure). No tooling is required: you create and check the structure by reading and following the rules.

The Cairn root of this repository is [`cairn/`](./cairn). The [src/main.rs](./src/main.rs) header remains the architecture entry point for the code itself; `cairn/spec/` is the behavioural truth behind it.

If you are an agent working in this repository, do the following **by default, without being asked**.

## 1. Before non-trivial work, propose

For anything beyond a trivial fix, create `cairn/changes/<change-id>/` with:

- `proposal.md`: *why* and *what* (frontmatter: `cairn: change`, `id`, `status: active`, `created`).
- `tasks.md`: the checklist (frontmatter: `cairn: tasks`, `change`).
- `delta.md`: what this changes in the spec, under the three literal headings `## ADDED Requirements`, `## MODIFIED Requirements`, `## REMOVED Requirements`.

Let the human review intent **before** you write code. Trivial fixes may skip this and go straight to landing.

## 2. After work lands, fold and log (never skip)

- Fold the change's delta into `cairn/spec/<capability>.md` so the spec always reflects current truth (append ADDED, replace MODIFIED, delete REMOVED).
- Append a dated entry `cairn/log/YYYY-MM-DD-<change-id>.md` describing what landed and which capabilities moved.
- Set the change `status: landed` and move its folder to `cairn/changes/archive/`.

> **The forcing rule:** a change that affects behaviour is not *done* until the spec is updated and the log entry is written.

## 3. Stay conformant

Check the structure yourself against the strict rules (CAIRN.md §8): a discoverable root, `spec/ changes/ log/` present, every Cairn file carrying a valid `cairn:` type, each change having `proposal.md` and `tasks.md`, kebab-case ids, literal delta headings, and a log entry for every landed change. Everything else (prose, naming, ordering, extra files) is free.

## 4. Repository conventions

- Run every cargo command through the nix devshell: `nix develop --command cargo <...>`.
- Run `cargo fmt` on the crate after finishing code changes.
- Build the feature matrix, not just the default: every backend (`carddav`, `jmap`, `msgraph`, `people`, `vdir`, `pimdir`) must compile on its own, since the shared client is fully cfg-gated.
- Never adjust production code to fit a test: adjust the test to match correct behaviour.
- Never print a secret in output, a log line or a test fixture.
- No em dashes in prose, per the Pimalaya guidelines.
- The wizard mirrors Himalaya's, requirement for requirement. When a wizard question comes up, read himalaya/cairn/spec/wizard.md first and keep the two aligned.
- The pimdir derivations (link id, `v: 1` summary, sort key, content hash) must stay byte-compatible with Neverest's `text/vcard` kind, because both write into the same store.
- A user-facing change also lands in CHANGELOG.md, which is a release-scoped roll-up over the Cairn log, not a replacement for it.
