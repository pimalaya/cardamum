---
cairn: log
change: graph-projection-moves-to-io-msgraph
landed: 2026-10-01
---

# The Graph contact projection moves to io-msgraph

Neverest needs the Graph contact to vCard projection to sync Microsoft 365 contacts, and a sync engine depending on a CLI for it would put a leaf under another leaf. The projection moved down into io-msgraph behind its new `vcard` feature, as methods on `MsgraphContact` (`to_vcard`, `from_vcard`, `create_from_vcard`, `update_from_vcard`), with its 14 tests, which pass there unchanged: the output is byte for byte what `src/msgraph/project.rs` produced.

The stash keeps its id, `cardamum-vcard` name included, so a contact stashed by an earlier Cardamum still reads back; the `$expand` clause the backend built by hand is now `MSGRAPH_CONTACT_STASH_EXPAND`. The string helpers it used became vcard-rs API (`VcardProp::text`, `VcardCst::push_raw`, `VcardDateAndOrTime::full_date`); `src/project.rs` stays for the Google People projection until that one moves to io-gpeople.

It landed on io-msgraph 0.4.1 and vcard-rs 0.5.1.

Capability moved: **projection** (the Graph link now points at io-msgraph). No behaviour change.
