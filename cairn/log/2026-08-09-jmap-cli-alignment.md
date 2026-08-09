---
cairn: log
change: jmap-cli-alignment
landed: 2026-08-09
---

# The JMAP CLI follows Himalaya, and stops leaking internals

The 2026-08-09 Fastmail JMAP run found one command that could not run at all and a handful of places where an internal shape reached the user. All are fixed and re-verified live against `fastmail-jmap`; the capability [commands](../spec/commands.md) gained three requirements.

**`jmap contact-card copy` runs (S4).** It declared `--from-account` with `short = 'a'`, colliding with the global `-a/--account`, so clap asserted and the process died with exit 101 on any invocation, `--help` included; in a release build, where those assertions are compiled out, the letter was simply ambiguous. The command has therefore never run since it was written, which is why the previous report could only record it as "implemented, not live-tested". It now follows Himalaya's `jmap email copy`: ids positional and repeatable, `--from-account` and `--to-address-book` long-only, the latter repeatable since a JMAP copy can land in several books. Verified reaching the server, which answers `invalidArguments` when the source account is the destination account, as RFC 8620 §5.4 requires; a successful copy still needs a second JMAP account.

**`jmap address-book get` takes its ids positionally (S2)**, like `contact-card get` and like Himalaya, still optional so that omitting them keeps fetching every book.

**Writes print what came back (S1).** `contact-card create` and `update` rendered through the same three-field report, so a create printed two empty slots and an update printed a wholly empty block, both while succeeding. The report now prints only the fields the object carries, and the update falls back to a confirmation line when the response has nothing to show, which is the common case: Fastmail answers a `set` update with `updated`, a blob id and a size, and no card at all.

**A rejected write reads as prose (J7).** Every failure used to print its `SetError` through `Debug`, e.g. `Forbidden { description: Some("AddressBooks may not be created") }`. A new jmap/error.rs ports Himalaya's `JmapSetError` trait and `format_set_error`, so the same failure now reads `forbidden: AddressBooks may not be created`, and an `invalidProperties` names its properties.

**A `vCardProps` rejection names the culprit (J6).** Fastmail refuses the standard `vCardProps` member (RFC 9555 §2.15), so a vCard carrying any property JSContact cannot model fails to write with nothing but `invalidProperties` to go on. The shared write path now appends the vCard properties that landed there: `the card carries X-CUSTOM-FIELD, X-ABLABEL, which JMAP has no home for on this server`. Nothing is dropped silently; whether cardamum should stash or drop those properties stays an open product question.

Also: the shared JSON input argument no longer calls every body a JSContact, since `jmap request` takes a JMAP request object through the same argument.

The J5 blocker (the vcard-rs JSContact projection spelling its object types the pre-RFC way) was fixed in vcard-rs itself and is consumed here through a local-path patch until that release ships; with it, a vCard carrying URL, PHOTO, KEY, CALURI and SOURCE now round-trips through Fastmail JMAP intact.
