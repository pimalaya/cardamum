//! # Card update command
//!
//! Rewrites an existing card, from a vCard source, from field flags, from
//! the composer, or from any combination of the three, and reports the
//! properties the server would not let go.

use core::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::{printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::Capability;
use crate::shared::{
    arg::{AddressbookIdArg, CardComposerArgs},
    card::{composer::CardComposer, fields::CardFieldsArgs, vcard::read_source},
    client::AddressbookClient,
};

/// Rewrite an existing vCard.
///
/// The source, the field flags and `-i` stack: the source replaces the
/// card's bytes, the flags set the properties they name on it, and `-i`
/// opens the result in the composer. With no source the card is read from
/// the backend first, and the version it answered guards the write on a
/// backend taking a guard.
///
/// JSON output: `{"id", "keptProperties", "queued"}`, the second listing
/// what the update dropped and the server kept anyway, the third whether a
/// sync engine applies it later.
#[derive(Debug, Parser)]
pub struct CardUpdateCommand {
    /// Addressbook holding the card.
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// Gate the update on this ETag, as returned by a previous read.
    ///
    /// The update only lands if the server still holds that version (RFC
    /// 9110 `If-Match`), which is how a concurrent write is caught. With
    /// no source the command sends the ETag it read when this is omitted,
    /// on a backend taking one.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
    /// The composer the card is refined in before it is written.
    #[command(flatten)]
    pub composer: CardComposerArgs,
    /// The properties the command sets on the card.
    #[command(flatten)]
    pub fields: CardFieldsArgs,
    /// Card to update, as `card list` reports it.
    ///
    /// This is the backend's own id, not the vCard `UID`, which names no
    /// card on its own.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
    /// vCard replacing the current contents: a path to a file, raw vCard
    /// contents, or `-` for stdin. Omit to start from the stored card.
    #[arg(value_name = "VCARD")]
    pub vcard: Option<String>,
}

impl CardUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut AddressbookClient) -> Result<()> {
        let mut capabilities = vec![Capability::CardUpdate];
        if self.if_match.is_some() {
            capabilities.push(Capability::CardUpdateIfMatch);
        }
        client.require(&capabilities)?;

        if self.vcard.is_none() && self.fields.is_empty() && !self.composer.interactive {
            bail!("Nothing to update; give a vCard, a field flag, or -i to edit the card");
        }

        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;

        // NOTE: reading the card to start from also gives the version it
        // is being changed from, so an edit that took a minute cannot
        // silently overwrite a write that landed during it.
        let (base, etag) = match &self.vcard {
            Some(source) => (read_source(source)?, None),
            None => {
                let card = client.get_card(&addressbook_id, &self.card_id)?;
                (card.contents, card.etag)
            }
        };

        // NOTE: a backend taking no version guard would refuse, as one, a
        // version its read reports.
        let etag = etag.filter(|_| client.serves(Capability::CardUpdateIfMatch));
        let if_match = self.if_match.or(etag);
        let seeded = self.fields.apply(&base)?;

        if !self.composer.interactive {
            let outcome =
                client.update_card(&addressbook_id, &self.card_id, seeded, if_match.as_deref())?;

            return printer.out(CardUpdateOutput::Applied(CardUpdatedOutput {
                id: self.card_id,
                kept_properties: outcome.kept_properties,
                queued: client.queues_writes(),
            }));
        }

        let composer = CardComposer {
            command: client.account.card_composer(self.composer.composer)?,
        };

        // NOTE: reading the card is the one thing that has to happen
        // before the editor, so its connection is dropped here rather than
        // left idle for the minutes an edit takes: a server closes such a
        // connection, and the update below opens a fresh one.
        client.disconnect();

        let Some(draft) = composer.edit(printer, &seeded)? else {
            return printer.out(CardUpdateOutput::Abandoned {});
        };

        let updated = client.update_card(
            &addressbook_id,
            &self.card_id,
            draft.contents.clone(),
            if_match.as_deref(),
        );

        let outcome = draft.finish(updated)?;

        printer.out(CardUpdateOutput::Applied(CardUpdatedOutput {
            id: self.card_id,
            kept_properties: outcome.kept_properties,
            queued: client.queues_writes(),
        }))
    }
}

/// What `card update` prints, which is whether it wrote anything.
///
/// Untagged, so the write serializes exactly as it would on its own. The
/// second shape is reachable through `-i` alone, which `--json` refuses
/// to run.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum CardUpdateOutput {
    /// The card the backend updated.
    Applied(CardUpdatedOutput),
    /// Nothing, the edit having been abandoned: an empty object rather
    /// than null, so every payload of the command is a JSON object.
    Abandoned {},
}

impl fmt::Display for CardUpdateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Applied(out) => out.fmt(f),
            Self::Abandoned {} => writeln!(f, "Card not updated"),
        }
    }
}

/// The card the backend updated, and what it would not let go.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardUpdatedOutput {
    /// Backend-specific identifier of the updated card.
    pub id: String,
    /// Names of the vCard properties the update dropped and the server
    /// kept anyway, empty on a backend that can clear everything.
    pub kept_properties: Vec<String>,
    /// Whether a sync engine applies the write later rather than now.
    pub queued: bool,
}

impl fmt::Display for CardUpdatedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.queued {
            true => write!(
                f,
                "Card `{}` update queued; the next sync applies it",
                sanitize::text(&self.id)
            )?,
            false => write!(
                f,
                "Card `{}` successfully updated",
                sanitize::text(&self.id)
            )?,
        }

        // NOTE: the write landed, but not all of it: saying so beats a
        // clean success for a card that kept what the vCard dropped.
        if !self.kept_properties.is_empty() {
            write!(
                f,
                ", except {}, which the server will not let go",
                self.kept_properties.join(", ")
            )?;
        }

        writeln!(f)
    }
}

#[cfg(all(test, feature = "msgraph"))]
mod tests {
    use anyhow::Result;
    use clap::Parser;
    use io_msgraph::v1::client::MsgraphClientStd;
    use pimalaya_cli::{output::Bytes, printer::Printer};
    use serde::Serialize;
    use serde_json::{Value, json};

    use super::CardUpdateCommand;
    use crate::{
        backend::Backend,
        config::{AccountConfig, Config},
        msgraph::backend::MsgraphBackend,
        script::{self, Script},
        shared::client::AddressbookClient,
    };

    /// What a command printed, as JSON.
    #[derive(Default)]
    struct Captured(Value);

    impl Printer for Captured {
        fn out<T: core::fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
            self.0 = serde_json::to_value(&data)?;
            Ok(())
        }

        fn event<T: core::fmt::Display + Serialize>(&mut self, _event: T) -> Result<()> {
            Ok(())
        }

        fn out_bytes(&mut self, _bytes: Bytes) -> Result<()> {
            Ok(())
        }
    }

    /// An update with no `--if-match` on Graph sends one plain `PATCH`:
    /// the `changeKey` its read reports is a version Graph takes no guard
    /// on, so it stays off the write rather than refusing it.
    #[test]
    fn a_graph_update_without_if_match_sends_one_plain_patch() {
        let contact = r#"{"id":"AAMk","changeKey":"k1","displayName":"Ada"}"#;
        let answer = || script::response(200, "application/json", contact);
        let (script, requests) = Script::new([answer(), answer(), answer()]);

        let account = AccountConfig {
            msgraph: Some(toml::from_str(r#"auth.token.raw = "token""#).unwrap()),
            ..Default::default()
        };
        let mut client =
            AddressbookClient::new(Config::default(), "work".into(), account, Backend::Auto)
                .unwrap();
        client.open_msgraph(MsgraphBackend {
            inner: MsgraphClientStd::new(script, "token", Default::default()),
        });

        let mut out = Captured::default();
        let args = [
            "update",
            "-k",
            "contacts",
            "AAMk",
            "--full-name",
            "Ada Byron",
        ];
        CardUpdateCommand::parse_from(args)
            .execute(&mut out, &mut client)
            .unwrap();
        assert_eq!(
            out.0,
            json!({"id": "AAMk", "keptProperties": [], "queued": false})
        );

        let requests = requests.lock().unwrap();
        let requests: Vec<String> = requests
            .iter()
            .map(|request| String::from_utf8_lossy(request).into_owned())
            .collect();
        let patches: Vec<&String> = requests
            .iter()
            .filter(|request| request.starts_with("PATCH "))
            .collect();
        assert_eq!(patches.len(), 1, "{requests:#?}");
        assert!(patches[0].contains("Ada Byron"), "{}", patches[0]);
        let headers = patches[0].to_ascii_lowercase();
        assert!(!headers.contains("if-match"), "{}", patches[0]);
    }
}
