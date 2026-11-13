---
cairn: log
change: audited-dependencies
date: 2026-11-13
---

# Built on the audited releases of the io-* libraries

Cargo.toml requires io-http 0.7, io-oauth 0.6 (dev), io-jmap 0.8, io-msgraph 0.8, io-gpeople 0.7, io-pim-discovery 0.12, io-vdir 0.3 and io-webdav 0.8, the releases closing the family audit of 2026-11-13. pimalaya-cli keeps its git pin at `eb37962`. Cargo.lock holds one io-http (0.7.0), one io-oauth (0.6.0) and no io-sasl; io-pimdir 0.12, pimalaya-config, pimalaya-cli and pimalaya-stream pull none of them.

**What broke the build.** io-gpeople's `from_vcard` returning a `GpeoplePersonVcard`, io-jmap's `Unknown` set error becoming `Other { type_name, description }`, `existing_id` on the copy `AlreadyExists`, and `JmapAddressBookUpdate::description` becoming `Option<Option<String>>`. Nothing else: the PACC and RFC 6764 clients, `VdirClient`, the CardDAV verbs and the `request_*` calls of the live suites kept their signatures.

**Google People.** `update_card` computed its mask, then sent a person built from the vCard alone, so each masked field was replaced by its projection: editing `ORG` erased a second organization and the first one's symbol. It merged the `clientData` of other clients by hand and nothing else. It now calls `GpeoplePerson::merge_unprojected(&current, &fields)` between the mask and the update, its own merge gone as the crate's note asks. A scripted test over `GpeopleClientStd::new` holds it: without the merge it fails on `[{"name": "Globex", "title": "Engineer"}]` where Globex with its symbol and Initech were expected. A line over 8 KiB never reached the stash and nothing said so: `create_card` and `update_card` now log ``card `<id>` stored without PHOTO: too long for the server``, from `GpeoplePersonVcard::dropped`. It is a warning and not a key of the receipt because the backends spec keeps a field one backend alone fills out of a unified output; with the default log level off, the receipt still reads as a plain success, which is the limit of that rule and the owner's to revisit. io-msgraph reports no dropped line, so Graph stays silent on the same case.

**Microsoft Graph.** The unified listings followed `@odata.nextLink` with a `MsgraphSend::get` built on whatever URL the page named, bearer included. They go through `page_get`, which refuses a link off the Graph origin.

**JMAP.** `addressbook update --description ""` reached the adapter as `Some(None)` and was flattened to `None`, an update without the property: the clear was dropped, against "Clearing an optional field removes it". The diff is forwarded as it is and leaves as `null`. `jmap address-book update --description` keeps setting a value. A set error of an unmodelled type prints the type the server sent; one without a type still prints `unknown`.

**CardDAV.** `carddav put` echoed its argument as the id; it prints the id io-webdav 0.8 answers, the `Location` segment or the percent-encoded segment sent, sanitized in the text form. A listing that is not a whole multistatus now fails in the library, and no call site of cardamum swallowed a listing error.

**Not done.** io-pim-discovery's `trust` lives on `DiscoveryServiceConfig`, which cardamum never builds or prints, so nothing shows it. `addressbook create` on CardDAV still answers the name it was given; `encode_id` would make it the listed form and is left for a pass with the live servers. None of this was run against a provider: the live suites compile and were not run.

Verified with `cargo test --all-features` (103 unit tests, the suites of tests/, the live ones ignored), clippy `--all-targets -D warnings` over the eleven feature sets of CI, and `cargo test --no-default-features --features rustls-ring,vdir`.

Capabilities moved: projection ("Oversized lines stay local" modified, "A People update keeps what the vCard cannot show" added), backends ("Clearing an optional field removes it" and "Card ids are backend-native and verbatim" modified, "A listing is whole or fails" added).
