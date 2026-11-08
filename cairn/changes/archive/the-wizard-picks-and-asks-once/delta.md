---
cairn: delta
change: the-wizard-picks-and-asks-once
---

# Delta

## ADDED Requirements

### Requirement: Auto picks the best-ranked option
Discovery SHALL turn what it found into options, each a protocol, its server and a sign-in (a password, an API token or OAuth 2.0), for the contacts services this build compiles: CardDAV, JMAP, the People API and Microsoft Graph contacts, the last two only where a provider rule matched, by domain or by MX, and signing in through OAuth alone. Two options reading the same (same protocol, server and sign-in) SHALL be one, and a server offering OAuth through several grants SHALL be one OAuth option. A server advertising no sign-in offers a password and an API token. The options SHALL be ranked, and the first SHALL be taken without a question as Auto: the provider's own API first, then JMAP before CardDAV, servers in the order discovery found them, and within one server OAuth, then an API token, then a password when `ortie` is on the `PATH`, else a password, then an API token, then OAuth. No HTTP authentication scheme and no user id SHALL be asked: a password signs in with HTTP Basic, a token with HTTP Bearer, and the provider APIs read the signed-in user (`me`).

#### Scenario: JMAP offering OAuth beside CardDAV
- GIVEN `alice@example.org`, whose discovery finds CardDAV with a password and JMAP with a password and OAuth, `ortie` on the `PATH`
- WHEN the plan is built
- THEN Auto is JMAP through OAuth, followed by JMAP with a password, then CardDAV with a password

#### Scenario: The same discovery without Ortie
- GIVEN the same discovery, `ortie` not on the `PATH`
- WHEN the plan is built
- THEN Auto is JMAP with a password

#### Scenario: A custom domain at a provider
- GIVEN an address whose MX records match a provider rule
- WHEN the plan is built
- THEN Auto is that provider's contacts API through OAuth, its two grants being one option

### Requirement: The review names the pick in plain words
Before any sign-in, the wizard SHALL show pimalaya-cli's review with one `Contacts` row, worded without jargon beyond the protocol name: `<Protocol> at <host>` then `, with your password`, `, with an API token`, `, through your browser` (OAuth through Ortie) or `, with a token from a broker` (OAuth without Ortie), a provider API reading `People API` or `Microsoft Graph` before the same endings. Continue, selected, SHALL go on to the sign-in. Change contacts SHALL list `Auto (<its row>)` when something was discovered, every option as `<Protocol> · <sign-in>` (`Password`, `API token`, `OAuth 2.0`) naming its server only where two entries would read the same, then `Custom server URL` when CardDAV or JMAP is compiled; picking one SHALL return to the review.

#### Scenario: A password option
- GIVEN Auto is CardDAV at `dav.example.org` with a password
- WHEN the review shows
- THEN stderr reads `Contacts  CardDAV at dav.example.org, with your password`, and the list offers `Continue` and `Change contacts`

#### Scenario: Two CardDAV servers
- GIVEN CardDAV with a password at `dav.example.org` and at `carddav.example.org`
- WHEN Change contacts lists them
- THEN they read `CardDAV · Password · dav.example.org` and `CardDAV · Password · carddav.example.org`, while a lone JMAP option reads `JMAP · Password`

### Requirement: The user sees where a password goes
Before a password or an API token is typed, the wizard SHALL name the server it goes to through pimalaya-cli's `keyring::prompt_password`, given the server URL and the domain of the input (the address's domain, the bare domain, or the host of a server URL typed as input), so a server off that domain reads `not on <domain>`.

#### Scenario: Contacts hosted elsewhere
- GIVEN `alice@example.org`, whose CardDAV server is `https://dav.example.net/`
- WHEN the password prompt shows
- THEN stderr reads `Your password goes to https://dav.example.net/, not on example.org`

### Requirement: A password is typed once, tested, then stored
A password or an API token SHALL be typed once, masked, with no confirmation, and tested at once; only once the test passes SHALL the wizard ask where to keep it, through pimalaya-cli's `keyring::prompt_store`, under the key `cardamum-<account name>-carddav` or `cardamum-<account name>-jmap`, where `<account name>` is the derived name the account is saved under. The login SHALL be the username discovery advertised when it is an address, else the typed address, with no prompt; `Login:` SHALL be asked only when neither is an address (a bare domain or a server URL typed as input).

#### Scenario: A password stored in pass
- GIVEN the account `example`, Auto CardDAV with a password, pass installed
- WHEN the password is typed and the server accepts it
- THEN the storage list offers pass first, and picking it stores the password at `cardamum-example-carddav`, which the account reads with `pass show cardamum-example-carddav`

#### Scenario: A bare domain
- GIVEN the input `example.org`, discovery advertising no username
- WHEN the password sign-in starts
- THEN `Login:` is asked before the password

### Requirement: A refusal offers a way on
A sign-in whose test fails SHALL print why, then offer `Try again` (the secret typed again, or the OAuth sign-in run again), `Change the login` (password sign-ins only: `Login:`, prefilled, then the password again) and `Back to the review`, where another option can be picked. Nothing SHALL be stored before a test passes. Cardamum has one domain, so no option leaves it out.

#### Scenario: A mistyped password
- GIVEN Auto CardDAV with a password
- WHEN the server refuses the typed password
- THEN the failure is printed and the list offers `Try again`, `Change the login`, `Back to the review`, nothing being stored

### Requirement: OAuth hands off to Ortie
An OAuth option SHALL sign in through pimalaya-cli's `ortie::sign_in`, given the typed address (else the domain), the Ortie account `cardamum-<account name>` and the option's service (`carddav`, `jmap`, `gpeople` or `msgraphContacts`), the account reading its token with the command it answers. Without `ortie` on the `PATH`, the wizard SHALL print one line saying a token broker issues the token and that Ortie can, then offer pimalaya-cli's `keyring::prompt_token` list, brokers included, seeded with the same account name. An Ortie run that fails SHALL show the refusal menu. Cardamum runs no OAuth grant itself.

#### Scenario: Ortie installed
- GIVEN `alice@example.org`, the account `example`, Auto JMAP through OAuth
- WHEN the user continues
- THEN `ortie configure --name cardamum-example --service jmap -- alice@example.org` then `ortie auth get -a cardamum-example` run, and the account reads its token with `ortie token show -a cardamum-example`

### Requirement: Nothing discovered asks for a server URL
When discovery yields no option for the input (an empty result, the deadline elapsing with nothing completed, a failed lookup, or a URL scheme filter leaving nothing), the wizard SHALL print `Nothing could be discovered for <domain>` and ask `Server URL:`, the prompt `Custom server URL` also opens. The scheme SHALL decide the protocol: `https`, `carddav` and `carddavs` read as CardDAV over https, `jmap` and `jmaps` as JMAP over https. Encrypted servers only: an `http://` URL SHALL be refused naming it, as SHALL a URL without a scheme or with another one, a protocol this build does not compile, or one naming no host, and the prompt SHALL be asked again. The sign-in follows as a pick list, `Password` (selected) or `API token`, then the review. An empty answer to the prompt reached because nothing was discovered SHALL stop the wizard with a message stating it could not discover a configuration for the input and pointing at the documented sample (linked), writing nothing; an empty answer from `Custom server URL` SHALL return to the review. A build compiling neither CardDAV nor JMAP SHALL stop without asking.

#### Scenario: A self-hosted server
- GIVEN `alice@example.org`, whose domain publishes nothing
- WHEN `Server URL:` is answered `carddavs://dav.example.org/dav/` and Password is picked
- THEN the review reads `Contacts  CardDAV at dav.example.org, with your password`, and the saved account holds `carddav.server = "https://dav.example.org/dav/"`

#### Scenario: A cleartext server
- GIVEN the `Server URL:` prompt
- WHEN it is answered `http://dav.example.org`
- THEN the wizard says it accepts encrypted servers only, naming `http://dav.example.org`, and asks again

#### Scenario: An empty answer
- GIVEN nothing discovered for `example.org`
- WHEN `Server URL:` is answered empty
- THEN the wizard stops, pointing at the documented sample, and writes nothing

## MODIFIED Requirements

### Requirement: Input orients the flow
A single prompt SHALL accept an email address (or bare domain), a `scheme://` server URL, or a local folder path, and nothing else SHALL be asked before discovery. An email, bare domain or server URL runs io-pim-discovery's parallel discovery; a folder is a local vdir or pimdir store. A server URL discovers from its host and its scheme narrows the discovered options: `carddav`, `carddavs` and the HTTP-family schemes keep CardDAV, `jmap` and `jmaps` keep JMAP, and the provider APIs (the People API, Microsoft Graph) are dropped when a scheme is given. An unknown scheme is rejected outright. A server is typed by hand only at the `Server URL:` prompt (see "Nothing discovered asks for a server URL"), never field by field.

#### Scenario: A typed JMAP URL
- GIVEN the input `jmaps://example.org`, whose discovery finds CardDAV and JMAP
- WHEN the plan is built
- THEN only the JMAP options are offered

### Requirement: Discovery is time-bounded
The parallel discovery run SHALL be bounded by a short deadline (8 seconds), under a spinner naming the domain looked up, so a single unreachable endpoint (a firewalled port, a black-hole host) cannot stall the interactive wizard. Each mechanism runs independently; any that has not reported by the deadline is abandoned, and only what completed in time is offered. A DNS resolver that does not answer fails the lookup, the failure printed on stderr. When nothing completes, or the lookup fails, the wizard asks for a server URL (see "Nothing discovered asks for a server URL").

### Requirement: Connection tested before saving
Each sign-in SHALL be tested as soon as it is typed, under a spinner, with the account connection test, so a bad credential or endpoint never yields a configuration that cannot connect; a failed test shows the refusal menu (see "A refusal offers a way on"). A local folder has no sign-in and is tested once before the fragment is saved or printed. The generated fragment is compact: only the `[accounts.<name>]` table stays a section header, other tables flatten into dotted keys, and empty tables and defaulted values are dropped.

#### Scenario: A refused API token
- GIVEN Auto JMAP with an API token
- WHEN the typed token is refused
- THEN the spinner reports the failure before anything is stored or saved

## REMOVED Requirements

### Requirement: One entry per service, then auth
Replaced by "Auto picks the best-ranked option" and "The review names the pick in plain words": the service list and the HTTP authentication scheme prompt are gone, the best-ranked option is taken without a question, and the alternatives sit behind Change.

### Requirement: OAuth folds into the API token
Replaced by "OAuth hands off to Ortie": OAuth is an option of its own, signed in through Ortie, the broker list remaining only for a machine without Ortie.

### Requirement: Stop when nothing is discovered
Replaced by "Nothing discovered asks for a server URL": an empty discovery asks for a server URL, and only an empty answer stops the wizard.
