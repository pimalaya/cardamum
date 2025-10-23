use std::{borrow::Cow, collections::HashSet};

use anyhow::{anyhow, Result};
use http::Uri;

use io_addressbook::{
    addressbook::Addressbook,
    card::Card,
    carddav::{
        coroutines::{
            addressbook_home_set::AddressbookHomeSet,
            create_addressbook::CreateAddressbook,
            create_card::CreateCard,
            current_user_principal::CurrentUserPrincipal,
            delete_addressbook::DeleteAddressbook,
            delete_card::DeleteCard,
            follow_redirects::FollowRedirectsResult,
            list_addressbooks::ListAddressbooks,
            list_cards::ListCards,
            read_card::ReadCard,
            send::SendResult,
            update_addressbook::UpdateAddressbook,
            update_card::UpdateCard,
            well_known::{WellKnown, WellKnownResult},
        },
        request::set_uri_path,
    },
};
use io_stream::runtimes::std::handle;
use pimalaya_toolbox::stream::Stream;

use super::config::CarddavConfig;

#[derive(Debug)]
pub struct CarddavClient<'a> {
    config: io_addressbook::carddav::config::CarddavConfig<'a>,
    stream: Stream,
}

impl<'a> CarddavClient<'a> {
    pub fn new(config: &'a CarddavConfig) -> Result<Self> {
        let tls = &config.tls;

        if let Some(uri) = &config.home_uri {
            let stream = Stream::connect(uri, tls)?;
            return Self::from_home_uri(config, stream, Cow::Borrowed(uri));
        };

        if let Some(uri) = &config.server_uri {
            let stream = Stream::connect(&uri, tls)?;
            return Self::from_server_uri(config, stream, uri.clone());
        }

        if let Some(discover) = &config.discover {
            let hostname = if let Some(port) = discover.port {
                Cow::from(format!("{}:{port}", discover.host))
            } else {
                Cow::from(&discover.host)
            };

            let scheme = match &discover.scheme {
                Some(scheme) => Cow::from(scheme),
                None => Cow::from("https"),
            };

            let uri: Uri = format!("{scheme}://{hostname}/.well-known/carddav")
                .parse()
                .unwrap();

            let mut stream = Stream::connect(&uri, tls)?;

            let remote_config = io_addressbook::carddav::config::CarddavConfig {
                uri: Cow::Borrowed(&uri),
                auth: TryFrom::try_from(&config.auth)?,
            };

            let mut well_known = WellKnown::new(&remote_config, discover.method.clone());
            let mut arg = None;

            let ok = loop {
                match well_known.resume(arg.take()) {
                    WellKnownResult::Ok(ok) => break ok,
                    WellKnownResult::Err(err) => {
                        return Err(anyhow!(err).context("Discover CardDAV server error"));
                    }
                    WellKnownResult::Io(io) => arg = Some(handle(&mut stream, io)?),
                }
            };

            if !ok.keep_alive {
                stream = Stream::connect(&ok.uri, tls)?;
            }

            return Self::from_server_uri(config, stream, ok.uri);
        }

        let ctx = "Cannot discover CardDAV home URI";
        let err = "Missing one of `discover`, `server-uri` or `home-uri` config option";
        Err(anyhow!(err).context(ctx))
    }

    fn from_home_uri(config: &'a CarddavConfig, stream: Stream, uri: Cow<'a, Uri>) -> Result<Self> {
        let auth = TryFrom::try_from(&config.auth)?;
        let config = io_addressbook::carddav::config::CarddavConfig { uri, auth };
        let client = Self { config, stream };

        return Ok(client);
    }

    fn from_server_uri(
        config: &'a CarddavConfig,
        mut stream: Stream,
        mut uri: Uri,
    ) -> Result<Self> {
        let tls = &config.tls;

        let remote_config = io_addressbook::carddav::config::CarddavConfig {
            uri: Cow::Borrowed(&uri),
            auth: TryFrom::try_from(&config.auth)?,
        };

        let mut principal = CurrentUserPrincipal::new(&remote_config);
        let mut arg = None;

        let ok = loop {
            match principal.resume(arg.take()) {
                FollowRedirectsResult::Ok(ok) => break ok,
                FollowRedirectsResult::Err(err) => {
                    return Err(anyhow!(err).context("Get current user principal error"))
                }
                FollowRedirectsResult::Io(io) => arg = Some(handle(&mut stream, io)?),
                FollowRedirectsResult::Reset(new_uri) => {
                    uri = new_uri;
                    stream = Stream::connect(&uri, tls)?;
                }
            }
        };

        let mut same_scheme = true;
        let mut same_authority = true;

        if let Some(discovered_uri) = ok.body {
            uri = if let Some(auth) = discovered_uri.authority() {
                same_authority = uri.authority() == Some(auth);
                same_scheme = uri.scheme() == discovered_uri.scheme();
                discovered_uri
            } else {
                set_uri_path(uri, discovered_uri.path())
            };
        }

        if !ok.keep_alive || !same_scheme || !same_authority {
            stream = Stream::connect(&uri, tls)?;
        }

        let remote_config = io_addressbook::carddav::config::CarddavConfig {
            uri: Cow::Borrowed(&uri),
            auth: TryFrom::try_from(&config.auth)?,
        };

        let mut home = AddressbookHomeSet::new(&remote_config);
        let mut arg = None;

        let ok = loop {
            match home.resume(arg.take()) {
                FollowRedirectsResult::Ok(ok) => break ok,
                FollowRedirectsResult::Err(err) => {
                    return Err(anyhow!(err).context("Get addressbook home set error"));
                }
                FollowRedirectsResult::Io(io) => arg = Some(handle(&mut stream, io)?),
                FollowRedirectsResult::Reset(new_uri) => {
                    uri = new_uri;
                    stream = Stream::connect(&uri, tls)?;
                }
            }
        };

        let mut same_scheme = true;
        let mut same_authority = true;

        if let Some(discovered_uri) = ok.body {
            uri = if let Some(auth) = discovered_uri.authority() {
                same_authority = uri.authority() == Some(auth);
                same_scheme = uri.scheme() == discovered_uri.scheme();
                discovered_uri
            } else {
                set_uri_path(uri, discovered_uri.path())
            };
        }

        if !ok.keep_alive || !same_scheme || !same_authority {
            stream = Stream::connect(&uri, tls)?;
        }

        Self::from_home_uri(config, stream, Cow::Owned(uri))
    }

    pub fn create_addressbook(&mut self, addressbook: Addressbook) -> Result<()> {
        let mut create = CreateAddressbook::new(&self.config, addressbook);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Creat addressbook error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn list_addressbooks(&mut self) -> Result<HashSet<Addressbook>> {
        let mut list = ListAddressbooks::new(&self.config);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(err) => return Err(anyhow!(err).context("List addressbooks error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn update_addressbook(&mut self, addressbook: Addressbook) -> Result<()> {
        let mut update = UpdateAddressbook::new(&self.config, addressbook);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Update addressbook error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn delete_addressbook(&mut self, id: impl AsRef<str>) -> Result<()> {
        let mut delete = DeleteAddressbook::new(&self.config, id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn create_card(&mut self, card: Card) -> Result<()> {
        let mut create = CreateCard::new(&self.config, card);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn list_cards(&mut self, addressbook_id: impl AsRef<str>) -> Result<HashSet<Card>> {
        let mut list = ListCards::new(&self.config, addressbook_id);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn read_card(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<Card> {
        let mut read = ReadCard::new(&self.config, addressbook_id, card_id);
        let mut arg = None;

        loop {
            match read.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn update_card(&mut self, card: Card) -> Result<()> {
        let mut update = UpdateCard::new(&self.config, card);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn delete_card(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<()> {
        let mut delete = DeleteCard::new(&self.config, addressbook_id, card_id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }
}
