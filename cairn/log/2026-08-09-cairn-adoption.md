---
cairn: log
change: cairn-adoption
landed: 2026-08-09
---

# Replace docs/ with cairn/

Retired docs/ in favour of a Cairn root, activated by AGENTS.md (with CLAUDE.md pointing at it), bringing cardamum in line with every other active Pimalaya repository and with the org guidelines, which record Cairn as superseding docs/.

The twelve provider test reports moved to cairn/spec/testing/ unchanged, indexed by a README carrying the capability frontmatter and a backend-by-provider table, matching Himalaya's layout. Two stale docs/testing/ paths inside provider-test-plan.md were repointed.

The standing contracts were rewritten, not moved, into six capability specs: backends (the per-backend adapter contract, card id handling, the pimdir cache rules, CardDAV home-set resolution), commands (the three command families, the doc-comment-is-help rule, the raw-faithful specific APIs), config (paths, account resolution, secrets, the table preset), wizard (the Himalaya-aligned flow), projection (managed, minted and stashed vCard fields) and addressbooks (memberships as structural data). docs/ was narrative and mixed live truth with history; a spec file states only what is true now.

Two things were deliberately not carried over. The module map in architecture.md duplicated the source tree and had already drifted, and src/main.rs is the architecture entry point. The iteration plan in specific-apis-design.md was history: what it decided is now a requirement in commands.md, and the deciding belongs to the log.

The three changes that landed alongside this one are archived under cairn/changes/archive/ with their proposals, task lists and deltas, so the spec's provenance is complete from the first commit rather than starting blank.

Capabilities moved: all of them, this being the initial spec.
