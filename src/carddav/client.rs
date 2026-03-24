// This file is part of Cardamum, a CLI to manage contacts.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::{
    borrow::Cow,
    collections::HashSet,
    fmt::Write as _,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use http::{StatusCode, Uri};
use io_http::v1_1::coroutines::send::{SendHttp, SendHttpResult};

use io_addressbook::{
    addressbook::Addressbook,
    card::{Card, VCardValue},
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
            send::{Send, SendError, SendResult},
            update_addressbook::UpdateAddressbook,
            update_card::UpdateCard,
            well_known::{WellKnown, WellKnownResult},
        },
        request::{push_uri_path, set_uri_path, Request},
        response::{Multistatus, PropstatResponse, Value},
    },
};
use io_stream::runtimes::std::handle;
use log::{debug, trace};
use pimalaya_toolbox::stream::Stream;
use serde::Deserialize;

use super::config::CarddavConfig;

const LIST_CARDS_PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<propfind xmlns="DAV:">
  <prop>
    <getetag />
  </prop>
</propfind>
"#;

const READ_CARD_PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<propfind xmlns="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
  <prop>
    <C:address-data />
  </prop>
</propfind>
"#;

const MULTIGET_CARDS_REPORT_CHUNK_SIZE: usize = 500;

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

    fn is_netease_contacts(&self) -> bool {
        self.config.uri.host() == Some("contacts.163.com")
    }

    fn next_netease_card_id() -> String {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        millis.to_string()
    }

    fn netease_card_text_values(card: &Card, name: &str) -> Vec<String> {
        card.entries()
            .filter(|entry| entry.name.as_str().eq_ignore_ascii_case(name))
            .flat_map(|entry| entry.values.iter())
            .filter_map(|value| match value {
                VCardValue::Text(text) if !text.is_empty() => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    fn is_netease_group_card(card: &Card) -> bool {
        Self::netease_card_text_values(card, "X-ADDRESSBOOKSERVER-KIND")
            .into_iter()
            .any(|value| value.eq_ignore_ascii_case("group"))
            || Self::netease_card_text_values(card, "X-TYPE")
                .into_iter()
                .any(|value| value.eq_ignore_ascii_case("group"))
            || Self::netease_card_text_values(card, "KIND")
                .into_iter()
                .any(|value| value.eq_ignore_ascii_case("group"))
            || card
                .vcard
                .uid()
                .is_some_and(|uid| uid.starts_with("MacGroup-"))
    }

    fn extract_netease_group_cid(card: &Card) -> Option<String> {
        Self::netease_card_text_values(card, "CID")
            .into_iter()
            .next()
            .or_else(|| {
                card.vcard
                    .uid()
                    .and_then(|uid| uid.strip_prefix("MacGroup-"))
                    .map(str::to_owned)
            })
    }

    fn normalize_netease_vcard(content: String, id: &str) -> String {
        let mut lines = content
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .split('\n')
            .map(str::to_owned)
            .collect::<Vec<_>>();

        let mut has_uid = false;
        let mut has_cid = false;

        for line in &mut lines {
            if line
                .split_once(':')
                .map(|(key, _)| key.eq_ignore_ascii_case("UID"))
                == Some(true)
            {
                *line = format!("UID:{id}");
                has_uid = true;
                continue;
            }

            if line
                .split_once(':')
                .map(|(key, _)| key.eq_ignore_ascii_case("CID"))
                == Some(true)
            {
                *line = format!("CID:{id}");
                has_cid = true;
            }
        }

        let end_idx = lines
            .iter()
            .position(|line| line.eq_ignore_ascii_case("END:VCARD"))
            .unwrap_or(lines.len());

        if !has_uid {
            lines.insert(end_idx, format!("UID:{id}"));
        }

        if !has_cid {
            let end_idx = lines
                .iter()
                .position(|line| line.eq_ignore_ascii_case("END:VCARD"))
                .unwrap_or(lines.len());
            lines.insert(end_idx, format!("CID:{id}"));
        }

        if !lines
            .iter()
            .any(|line| line.eq_ignore_ascii_case("END:VCARD"))
        {
            lines.push("END:VCARD".to_string());
        }

        lines.join("\r\n")
    }

    fn normalize_netease_group_vcard(content: String, cid: &str) -> String {
        let group_id = format!("MacGroup-{cid}");
        let mut lines = content
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .split('\n')
            .map(str::to_owned)
            .collect::<Vec<_>>();

        let mut has_uid = false;
        let mut has_cid = false;
        let mut has_x_type = false;
        let mut has_kind = false;

        for line in &mut lines {
            if line
                .split_once(':')
                .map(|(key, _)| key.eq_ignore_ascii_case("UID"))
                == Some(true)
            {
                *line = format!("UID:{group_id}");
                has_uid = true;
                continue;
            }

            if line
                .split_once(':')
                .map(|(key, _)| key.eq_ignore_ascii_case("CID"))
                == Some(true)
            {
                *line = format!("CID:{cid}");
                has_cid = true;
                continue;
            }

            if line.split_once(':').map(|(key, value)| {
                key.eq_ignore_ascii_case("X-TYPE") && value.eq_ignore_ascii_case("GROUP")
            }) == Some(true)
            {
                *line = "X-TYPE:GROUP".to_string();
                has_x_type = true;
                continue;
            }

            if line.split_once(':').map(|(key, value)| {
                key.eq_ignore_ascii_case("X-ADDRESSBOOKSERVER-KIND")
                    && value.eq_ignore_ascii_case("group")
            }) == Some(true)
            {
                *line = "X-ADDRESSBOOKSERVER-KIND:group".to_string();
                has_kind = true;
            }
        }

        let mut end_idx = lines
            .iter()
            .position(|line| line.eq_ignore_ascii_case("END:VCARD"))
            .unwrap_or(lines.len());

        if !has_uid {
            lines.insert(end_idx, format!("UID:{group_id}"));
            end_idx += 1;
        }

        if !has_cid {
            lines.insert(end_idx, format!("CID:{cid}"));
            end_idx += 1;
        }

        if !has_x_type {
            lines.insert(end_idx, "X-TYPE:GROUP".to_string());
            end_idx += 1;
        }

        if !has_kind {
            lines.insert(end_idx, "X-ADDRESSBOOKSERVER-KIND:group".to_string());
        }

        if !lines
            .iter()
            .any(|line| line.eq_ignore_ascii_case("END:VCARD"))
        {
            lines.push("END:VCARD".to_string());
        }

        lines.join("\r\n")
    }

    fn normalize_card_for_upload(&self, card: Card) -> Result<Card> {
        if !self.is_netease_contacts() {
            return Ok(card);
        }

        if Self::is_netease_group_card(&card) {
            let cid =
                Self::extract_netease_group_cid(&card).unwrap_or_else(Self::next_netease_card_id);
            let id = format!("MacGroup-{cid}");
            let content = Self::normalize_netease_group_vcard(card.to_string(), &cid);
            let vcard = Card::parse(content).context("Parse normalized group vCard")?;

            return Ok(Card {
                id,
                addressbook_id: card.addressbook_id,
                vcard,
            });
        }

        let id = if card.id.chars().all(|ch| ch.is_ascii_digit()) {
            card.id.clone()
        } else {
            Self::next_netease_card_id()
        };

        let content = Self::normalize_netease_vcard(card.to_string(), &id);
        let vcard = Card::parse(content).context("Parse normalized vCard")?;

        Ok(Card {
            id,
            addressbook_id: card.addressbook_id,
            vcard,
        })
    }

    fn list_card_ids_via_propfind(
        &mut self,
        addressbook_id: impl AsRef<str>,
    ) -> Result<Vec<String>> {
        let addressbook_id = addressbook_id.as_ref();
        let request = Request::propfind(&self.config, format!("{addressbook_id}/"))
            .content_type_xml()
            .depth(1);
        let mut send = Send::<Multistatus<GetetagProp>>::new(
            request,
            LIST_CARDS_PROPFIND_BODY.as_bytes().to_vec(),
        );
        let mut arg = None;

        let ok = loop {
            match send.resume(arg.take()) {
                SendResult::Ok(ok) => break ok,
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("List cards via PROPFIND error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        };

        let mut ids = HashSet::new();

        if let Some(responses) = ok.body.responses {
            for response in responses {
                trace!("process multistatus");

                let Some(propstats) = response.propstats else {
                    continue;
                };

                let has_getetag = propstats.into_iter().any(|propstat| {
                    propstat.status.is_success() && propstat.prop.getetag.is_some()
                });

                if !has_getetag {
                    debug!("multistatus propstat response error");
                    continue;
                }

                let Some(id) = card_id_from_href(&response.href.value) else {
                    continue;
                };

                ids.insert(id.to_string());
            }
        }

        Ok(ids.into_iter().collect())
    }

    fn card_href(&self, addressbook_id: &str, card_id: &str) -> String {
        push_uri_path(
            self.config.uri.clone().into_owned(),
            format!("/{addressbook_id}/{card_id}.vcf"),
        )
        .path()
        .to_string()
    }

    fn read_cards_via_multiget(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_ids: &[String],
    ) -> Result<HashSet<Card>> {
        let addressbook_id = addressbook_id.as_ref();
        let mut cards = HashSet::new();

        for chunk in card_ids.chunks(MULTIGET_CARDS_REPORT_CHUNK_SIZE) {
            let hrefs = chunk
                .iter()
                .map(|card_id| self.card_href(addressbook_id, card_id))
                .collect::<Vec<_>>();
            let request = Request::report(&self.config, format!("{addressbook_id}/"))
                .content_type_xml()
                .depth(1);
            let mut send = Send::<Multistatus<AddressDataProp>>::new(
                request,
                build_addressbook_multiget_body(hrefs.iter().map(String::as_str)).into_bytes(),
            );
            let mut arg = None;

            let ok = loop {
                match send.resume(arg.take()) {
                    SendResult::Ok(ok) => break ok,
                    SendResult::Err(err) => {
                        return Err(
                            anyhow!(err).context("List cards via addressbook-multiget error")
                        )
                    }
                    SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
                }
            };

            if let Some(responses) = ok.body.responses {
                for response in responses {
                    trace!("process multistatus");

                    let Some(card) = parse_address_data_card(response, addressbook_id) else {
                        continue;
                    };

                    cards.insert(card);
                }
            }
        }

        Ok(cards)
    }

    fn read_card_via_propfind(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<Card> {
        let addressbook_id = addressbook_id.as_ref();
        let card_id = card_id.as_ref();
        let request = Request::propfind(&self.config, format!("{addressbook_id}/{card_id}.vcf"))
            .content_type_xml()
            .depth(0);
        let mut send = Send::<Multistatus<AddressDataProp>>::new(
            request,
            READ_CARD_PROPFIND_BODY.as_bytes().to_vec(),
        );
        let mut arg = None;

        let ok = loop {
            match send.resume(arg.take()) {
                SendResult::Ok(ok) => break ok,
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Read card via PROPFIND error"))
                }
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        };

        let Some(responses) = ok.body.responses else {
            return Err(anyhow!("Missing multistatus response"));
        };

        for response in responses {
            if let Some(status) = response.status {
                if !status.is_success() {
                    continue;
                }
            }

            let Some(propstats) = response.propstats else {
                continue;
            };

            for propstat in propstats {
                if !propstat.status.is_success() {
                    continue;
                }

                let Some(content) = propstat.prop.address_data else {
                    continue;
                };

                let vcard = Card::parse(content.value).context("Parse CardDAV address-data")?;
                return Ok(Card {
                    id: card_id.to_string(),
                    addressbook_id: addressbook_id.to_string(),
                    vcard,
                });
            }
        }

        Err(anyhow!("Missing CardDAV address-data"))
    }

    fn delete_card_raw(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<()> {
        let addressbook_id = addressbook_id.as_ref();
        let card_id = card_id.as_ref();
        let path = format!("/{addressbook_id}/{card_id}.vcf");
        let request = Request::delete(&self.config, path).body(Vec::<u8>::new());
        let mut send = SendHttp::new(request);
        let mut arg = None;

        loop {
            match send.resume(arg.take()) {
                SendHttpResult::Ok(ok) => {
                    let status = ok.response.status();
                    if status.is_success() {
                        return Ok(());
                    }

                    let body = String::from_utf8_lossy(ok.response.body()).to_string();
                    return Err(anyhow!("HTTP response error {status}: {body}"));
                }
                SendHttpResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete card raw HTTP error"))
                }
                SendHttpResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn create_addressbook(&mut self, addressbook: Addressbook) -> Result<()> {
        let mut create = CreateAddressbook::new(&self.config, addressbook);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => {
                    return Err(anyhow!(err).context("Create addressbook error"))
                }
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
        let card = self.normalize_card_for_upload(card)?;
        let mut create = CreateCard::new(&self.config, card);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Create card error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn list_cards(&mut self, addressbook_id: impl AsRef<str>) -> Result<HashSet<Card>> {
        let addressbook_id = addressbook_id.as_ref().to_owned();
        let mut list = ListCards::new(&self.config, &addressbook_id);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(SendError::Response(StatusCode::NOT_IMPLEMENTED, _))
                    if self.is_netease_contacts() =>
                {
                    let ids = self.list_card_ids_via_propfind(&addressbook_id)?;
                    break match self.read_cards_via_multiget(&addressbook_id, &ids) {
                        Ok(cards) => Ok(cards),
                        Err(err) => {
                            debug!(
                                "cannot list cards via addressbook-multiget: {err:#}; falling back to per-card PROPFIND"
                            );

                            let mut cards = HashSet::new();

                            for id in ids {
                                match self.read_card_via_propfind(&addressbook_id, &id) {
                                    Ok(card) => {
                                        cards.insert(card);
                                    }
                                    Err(err) => {
                                        debug!("cannot read card {id} via PROPFIND: {err:#}");
                                    }
                                }
                            }

                            Ok(cards)
                        }
                    };
                }
                SendResult::Err(err) => return Err(anyhow!(err).context("List cards error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn read_card(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<Card> {
        let addressbook_id = addressbook_id.as_ref().to_owned();
        let card_id = card_id.as_ref().to_owned();
        let mut read = ReadCard::new(&self.config, &addressbook_id, &card_id);
        let mut arg = None;

        loop {
            match read.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(SendError::Response(StatusCode::NOT_IMPLEMENTED, _))
                    if self.is_netease_contacts() =>
                {
                    break self.read_card_via_propfind(&addressbook_id, &card_id);
                }
                SendResult::Err(err) => return Err(anyhow!(err).context("Read card error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn update_card(&mut self, card: Card) -> Result<()> {
        let card = self.normalize_card_for_upload(card)?;
        let mut update = UpdateCard::new(&self.config, card);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Update card error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn delete_card(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<()> {
        if self.is_netease_contacts() {
            return self.delete_card_raw(addressbook_id, card_id);
        }

        let mut delete = DeleteCard::new(&self.config, addressbook_id, card_id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Delete card error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct GetetagProp {
    getetag: Option<Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct AddressDataProp {
    address_data: Option<Value>,
}

fn card_id_from_href(href: &str) -> Option<&str> {
    let href = href.trim_end_matches('/');
    if !href.to_ascii_lowercase().ends_with(".vcf") {
        return None;
    }

    let mut parts = href.rsplit(['.', '/']);
    parts.next();
    parts.next()
}

fn build_addressbook_multiget_body<'a>(hrefs: impl IntoIterator<Item = &'a str>) -> String {
    let mut body = String::from(
        r#"<?xml version="1.0" encoding="utf-8" ?>
<C:addressbook-multiget xmlns="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
  <prop>
    <C:address-data />
  </prop>
"#,
    );

    for href in hrefs {
        let _ = writeln!(body, "  <href>{href}</href>");
    }

    body.push_str("</C:addressbook-multiget>\n");
    body
}

fn parse_address_data_card(
    response: PropstatResponse<AddressDataProp>,
    addressbook_id: &str,
) -> Option<Card> {
    if let Some(status) = response.status {
        if !status.is_success() {
            debug!("multistatus response error");
            return None;
        }
    }

    let Some(propstats) = response.propstats else {
        return None;
    };

    let id = card_id_from_href(&response.href.value)?.to_string();

    for propstat in propstats {
        if !propstat.status.is_success() {
            debug!("multistatus propstat error");
            continue;
        }

        let Some(content) = propstat.prop.address_data else {
            continue;
        };

        let Ok(vcard) = Card::parse(content.value) else {
            continue;
        };

        return Some(Card {
            id,
            addressbook_id: addressbook_id.to_string(),
            vcard,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_card_id_from_href() {
        assert_eq!(
            card_id_from_href("/carddav/hash/contacts/1773979575174.vcf"),
            Some("1773979575174")
        );
        assert_eq!(card_id_from_href("/carddav/hash/contacts/"), None);
    }

    #[test]
    fn builds_multiget_body_with_all_hrefs() {
        let body = build_addressbook_multiget_body([
            "/carddav/hash/contacts/1.vcf",
            "/carddav/hash/contacts/2.vcf",
        ]);

        assert!(body.contains("<C:addressbook-multiget"));
        assert!(body.contains("<href>/carddav/hash/contacts/1.vcf</href>"));
        assert!(body.contains("<href>/carddav/hash/contacts/2.vcf</href>"));
    }
}
