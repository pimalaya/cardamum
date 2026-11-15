//! # Microsoft Graph client
//!
//! Wraps [`io_msgraph::v1::client::MsgraphClientStd`] for the Graph
//! commands.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_msgraph::v1::client::{
    MsgraphClientStd, MsgraphClientStdConnectOptions, MsgraphClientStdError,
};
use pimalaya_cli::error::with_code;
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    config::{AccountConfig, ProxyConfig},
    error::ErrorCode,
};

/// The connected Graph client.
pub struct MsgraphClient {
    inner: MsgraphClientStd,
}

impl Deref for MsgraphClient {
    type Target = MsgraphClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MsgraphClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Opens the Graph client of the account.
///
/// Bails when the account declares no `[msgraph]` block.
pub fn build_msgraph_client(
    name: String,
    mut account_config: AccountConfig,
) -> Result<MsgraphClient> {
    let msgraph_config = account_config
        .msgraph
        .take()
        .ok_or_else(|| anyhow!("Microsoft Graph config is missing for account `{name}`"))?;

    let token = msgraph_config.auth.token.get()?;
    let options = MsgraphClientStdConnectOptions {
        tls: msgraph_config.tls.into_tls(msgraph_config.alpn),
        proxy: ProxyConfig::resolve(msgraph_config.proxy, &mut SecretResolver::new())?,
        user_id: msgraph_config.user_id,
    };
    let inner = MsgraphClientStd::connect(token.expose_secret(), options)?;

    Ok(MsgraphClient { inner })
}

/// The code of a Graph refusal in an error chain, as neverest reads it:
/// Graph's `ErrorQuotaExceeded` or a 507 is `quota-exceeded`, a 429
/// `rate-limited`, a 403 `permission-denied`.
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    let MsgraphClientStdError::Send(send) = cause.downcast_ref::<MsgraphClientStdError>()? else {
        return None;
    };
    let err = send.api()?;

    match (err.status, err.code.as_str()) {
        (_, "ErrorQuotaExceeded") | (507, _) => Some(ErrorCode::QuotaExceeded),
        (429, _) => Some(ErrorCode::RateLimited),
        (403, _) => Some(ErrorCode::PermissionDenied),
        _ => None,
    }
}

/// The failure of a request naming one contact, coded `card-not-found`
/// when Graph holds no such contact, its message Graph's: the specific
/// commands code it as the unified ones do.
pub(crate) fn contact_refusal(err: MsgraphClientStdError) -> anyhow::Error {
    match is_missing(&err) {
        true => with_code(err.into(), |_| Some(ErrorCode::CardNotFound)),
        false => err.into(),
    }
}

/// The failure of a request naming one contact folder, coded
/// `addressbook-not-found` when Graph holds no such folder, its message
/// Graph's: the specific commands code it as the unified ones do.
pub(crate) fn folder_refusal(err: MsgraphClientStdError) -> anyhow::Error {
    match is_missing(&err) {
        true => with_code(err.into(), |_| Some(ErrorCode::AddressbookNotFound)),
        false => err.into(),
    }
}

/// Whether Graph refused to create a folder because its parent holds one
/// of that name already: `ErrorFolderExists`, answered on a 409.
pub(crate) fn is_folder_taken(err: &MsgraphClientStdError) -> bool {
    let MsgraphClientStdError::Send(send) = err else {
        return false;
    };

    send.api()
        .is_some_and(|err| err.code == "ErrorFolderExists")
}

/// Whether a request naming one contact or one folder found none: a 404,
/// or the 400 Graph answers an id that cannot name one
/// (`ErrorInvalidIdMalformed`).
pub(crate) fn is_missing(err: &MsgraphClientStdError) -> bool {
    let MsgraphClientStdError::Send(send) = err else {
        return false;
    };

    send.api().is_some_and(|err| {
        err.status == 404 || (err.status == 400 && err.code == "ErrorInvalidIdMalformed")
    })
}

#[cfg(test)]
mod tests {
    use io_msgraph::v1::client::MsgraphClientStd;
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::json;

    use crate::{
        error::{self, ErrorCode},
        msgraph::{
            client::MsgraphClient,
            contacts::{delete::MsgraphContactDeleteCommand, get::MsgraphContactGetCommand},
        },
        script::{self, Script},
    };

    /// The client over a Graph server answering `status` with an error
    /// envelope of `code`.
    fn refusing(status: u16, code: &str) -> MsgraphClient {
        let body = json!({"error": {"code": code, "message": "Refused"}}).to_string();
        let (script, _) = Script::new([script::response(status, "application/json", &body)]);
        let inner = MsgraphClientStd::new(script, "token", Default::default());
        MsgraphClient { inner }
    }

    /// A specific command naming a contact Graph does not hold carries
    /// `card-not-found`, as `card read` does, Graph's message kept; another
    /// refusal keeps its own code.
    #[test]
    fn a_specific_command_on_a_missing_contact_is_coded() {
        let printer = &mut CapturePrinter::default();
        let id = String::from("AAMk");
        let missing = Some(ErrorCode::CardNotFound);

        let get = MsgraphContactGetCommand {
            contact_id: id.clone(),
        };
        let err = get
            .execute(printer, refusing(404, "ErrorItemNotFound"))
            .unwrap_err();
        assert_eq!(error::code(&err), missing, "{err:#}");
        assert!(format!("{err:#}").contains("Refused"), "{err:#}");

        let delete = |status, code| {
            let delete = MsgraphContactDeleteCommand {
                contact_id: id.clone(),
            };
            let err = delete
                .execute(&mut CapturePrinter::default(), refusing(status, code))
                .unwrap_err();
            error::code(&err)
        };
        assert_eq!(delete(404, "ErrorItemNotFound"), missing);
        assert_eq!(delete(400, "ErrorInvalidIdMalformed"), missing);
        assert_eq!(
            delete(403, "ErrorAccessDenied"),
            Some(ErrorCode::PermissionDenied)
        );
    }

    /// Every specific command naming a contact folder Graph does not hold
    /// carries `addressbook-not-found`, as `card list -k` does, Graph's
    /// message kept; another refusal keeps its own code.
    #[test]
    fn a_specific_command_on_a_missing_folder_is_coded() {
        use clap::Parser;

        use crate::msgraph::{
            contact_folders::{
                child_folders::MsgraphContactFolderChildFoldersCommand,
                delete::MsgraphContactFolderDeleteCommand, get::MsgraphContactFolderGetCommand,
                rename::MsgraphContactFolderRenameCommand,
            },
            contacts::{
                create::MsgraphContactCreateCommand, delta::MsgraphContactDeltaCommand,
                list::MsgraphContactListCommand,
            },
        };

        let run = |line: &str, status, code| {
            let args = line.split_whitespace();
            let client = refusing(status, code);
            let printer = &mut CapturePrinter::default();

            let result = match line.split_whitespace().next().unwrap() {
                "get" => MsgraphContactFolderGetCommand::parse_from(args).execute(printer, client),
                "child-folders" => MsgraphContactFolderChildFoldersCommand::parse_from(args)
                    .execute(printer, client),
                "rename" => {
                    MsgraphContactFolderRenameCommand::parse_from(args).execute(printer, client)
                }
                "delete" => {
                    MsgraphContactFolderDeleteCommand::parse_from(args).execute(printer, client)
                }
                "list" => MsgraphContactListCommand::parse_from(args).execute(printer, client),
                "create" => MsgraphContactCreateCommand::parse_from(args).execute(printer, client),
                "delta" => MsgraphContactDeltaCommand::parse_from(args).execute(printer, client),
                other => panic!("{other}"),
            };

            let err = result.unwrap_err();
            assert!(format!("{err:#}").contains("Refused"), "{line}: {err:#}");
            error::code(&err)
        };

        let missing = Some(ErrorCode::AddressbookNotFound);
        let lines = [
            "get AAMk",
            "child-folders AAMk",
            "rename AAMk Work",
            "delete AAMk",
            "list -f AAMk",
            "create -f AAMk {}",
            "delta -f AAMk",
        ];

        for line in lines {
            assert_eq!(run(line, 404, "ErrorItemNotFound"), missing, "{line}");
            assert_eq!(run(line, 400, "ErrorInvalidIdMalformed"), missing, "{line}");
            let denied = run(line, 403, "ErrorAccessDenied");
            assert_eq!(denied, Some(ErrorCode::PermissionDenied), "{line}");
        }
    }

    /// A Graph listing lifts `@odata.nextLink` into `nextPage`, `null` on
    /// the last page, and `--page-token` gets that link as it is, a link off
    /// the Graph origin refused before anything is sent; the delta's
    /// `@odata.deltaLink` ending a round stays as Graph sent it.
    #[test]
    fn a_graph_listing_pages_by_next_page() {
        use clap::Parser;
        use serde_json::Value;

        use crate::msgraph::{
            contact_folders::{
                child_folders::MsgraphContactFolderChildFoldersCommand,
                list::MsgraphContactFolderListCommand,
            },
            contacts::{delta::MsgraphContactDeltaCommand, list::MsgraphContactListCommand},
        };

        let link = "https://graph.microsoft.com/v1.0/me/contacts?$skip=1";
        let delta = "https://graph.microsoft.com/v1.0/me/contacts/delta?$deltatoken=d1";

        let run = |args: &[&str], body: Value| {
            let response = script::response(200, "application/json", &body.to_string());
            let (script, requests) = Script::new([response]);
            let client = MsgraphClient {
                inner: MsgraphClientStd::new(script, "token", Default::default()),
            };
            let mut printer = CapturePrinter::default();

            let result = match args[0] {
                "list" => MsgraphContactListCommand::parse_from(args).execute(&mut printer, client),
                "folders" => {
                    MsgraphContactFolderListCommand::parse_from(args).execute(&mut printer, client)
                }
                "child-folders" => MsgraphContactFolderChildFoldersCommand::parse_from(args)
                    .execute(&mut printer, client),
                "delta" => {
                    MsgraphContactDeltaCommand::parse_from(args).execute(&mut printer, client)
                }
                other => panic!("{other}"),
            };

            let requests = requests.lock().unwrap();
            let request = requests
                .first()
                .map(|r| String::from_utf8_lossy(r).into_owned());
            (result.map(|()| printer.into_single().unwrap()), request)
        };

        for first in [
            &["list"][..],
            &["folders"],
            &["child-folders", "AAMk"],
            &["delta"],
        ] {
            let (json, _) = run(first, json!({"value": [], "@odata.nextLink": link}));
            let json = json.unwrap();
            assert_eq!(json["nextPage"], link, "{json}");
            assert!(json.get("@odata.nextLink").is_none(), "{json}");

            let next = [first[0], "--page-token", link];
            let (json, request) = run(&next, json!({"value": [], "@odata.deltaLink": delta}));
            let json = json.unwrap();
            let request = request.unwrap();
            assert!(request.starts_with("GET /v1.0/me/contacts?"), "{request}");
            assert_eq!(json["nextPage"], Value::Null, "{json}");

            let off = [
                first[0],
                "--page-token",
                "https://example.org/v1.0/me/contacts",
            ];
            let (result, request) = run(&off, json!({}));
            assert!(result.is_err(), "{off:?}");
            assert_eq!(request, None, "{off:?}");
        }

        let (json, _) = run(&["delta"], json!({"value": [], "@odata.deltaLink": delta}));
        let json = json.unwrap();
        assert_eq!(json["@odata.deltaLink"], delta, "{json}");
        assert_eq!(json["nextPage"], Value::Null, "{json}");
    }
}
