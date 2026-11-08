---
cairn: change
id: the-wizard-picks-and-asks-once
status: landed
created: 2026-11-08
---

# The wizard picks, and asks once

## Why

The wizard asks too much and stops too soon. After discovery it lists every service and asks to pick one, then asks an HTTP authentication scheme (Basic or Bearer), a login, a Microsoft Graph user id, and a secret strategy before any password is typed; the password itself is never typed into the wizard, the user has to store it in a keyring first under the entry the wizard names. OAuth is a broker picked from a list, its account set up by hand outside the wizard. And a domain publishing nothing ends the run with a pointer to the sample configuration, so a self-hosted CardDAV server cannot be set up through the wizard at all.

The Pimalaya Android app's onboarding (android/cairn/spec/onboarding.md, change one-onboarding-path) found the shape that fixes this: one input, the best option picked without a question, a review in plain words with the alternatives one "change" away, a password typed once, tested, then stored, OAuth handed to the broker, and a server URL prompt instead of a dead end. The owner approved transposing it to the CLIs on 2026-11-08. Himalaya gets the same change in parallel from the same flow document, and the two wizards mirror each other requirement for requirement.

## What

- **Discovery**: io-pim-discovery moves from 0.9 to 0.11, with the `autoconfig` feature added for its MX provider detection, so a custom domain hosted at a provider with a fixed rule gets that provider's contacts API. It also brings the RFC 6764 context-root resolution of advertised CardDAV origins, and a DNS resolver that does not answer is now an error, which the wizard reports and treats as nothing discovered. Discovery asks for the contacts services this build compiles: CardDAV, JMAP, the People API and Microsoft Graph contacts.
- **The plan**: what discovery found becomes options, each a protocol, its server and a sign-in (password, API token, OAuth 2.0), deduplicated, ranked, the first being Auto: the provider's own API first where a rule matched, then JMAP before CardDAV, and within one server OAuth, token, password when `ortie` is on the PATH (password, token, OAuth without it).
- **The review**: pimalaya-cli's `review::prompt` with one `Contacts` row in plain words (`CardDAV at dav.example.org, with your password`); Change lists Auto, every option, and `Custom server URL`.
- **Sign-in**: no HTTP scheme, Login or user id prompt. A password or API token is typed once through `keyring::prompt_password` (which names where it goes), tested at once with the existing account test, then stored through `keyring::prompt_store` under `cardamum-<account name>-carddav` or `-jmap`. OAuth runs `ortie::sign_in` for the Ortie account `cardamum-<account name>` and the option's service; without Ortie, one line names it and the broker list of `keyring::prompt_token` follows. A failed test offers Try again, Change the login (password only) and Back to the review.
- **Nothing discovered**: `Nothing could be discovered for <domain>` then `Server URL:`, whose scheme decides CardDAV or JMAP over https, `http://` refused; then Password or API token. An empty answer stops as the wizard stops today.
- **Account name first**: the name (derived, suffixed until free) is settled before discovery, since the keyring keys and the Ortie account carry it.

Everything else stays: the input orientation, local folder auto-detection, the deadline, the name derivation, the save rules, the resolver override, the offer hook, the welcome. Settings last: none for contacts.

## Decisions

- **All options listed, provider API first.** The old wizard collapsed a provider account to its contacts API alone. The flow ranks the provider API first and lists every other option found under Change, so a provider's CardDAV stays reachable behind it.
- **A Login prompt only when no address is known.** The login is the advertised username when it is an address, else the typed address. A bare domain or a server URL typed as input gives neither, and the server cannot be signed in without a login, so `Login:` is asked there.
- **No unauthenticated CardDAV.** The old scheme prompt offered "None" when a server advertised nothing. The flow has no such sign-in, and a server advertising nothing now offers a password and an API token; `carddav.auth = "none"` stays writable by hand.
- **An empty `Server URL:` from the review returns to it.** Only the prompt reached because nothing was discovered stops the wizard on an empty answer.
- **A discovery lookup failure is nothing discovered.** The resolver error is printed, then the server URL prompt follows, since the connection test does not go through the discovery resolver and may still work.
- **No RFC 9728 feature.** Its resource metadata fetches run after the discovery deadline, unbounded, and the wizard needs no grant detail: OAuth only selects Ortie, which discovers its own grant.

## Scope / non-goals

- No "Leave contacts out" in the refusal menu: cardamum has one domain, so leaving it out leaves nothing to save.
- No change to the configuration format, the save, append or print rules, or the default claim.
- The Ortie side (`ortie configure --name --service -- <input>`) is built in parallel; this change codes against pimalaya-cli's `ortie::sign_in` contract.
