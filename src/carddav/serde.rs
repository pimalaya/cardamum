use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Multistatus<T> {
    #[serde(rename = "response")]
    pub responses: Vec<Response<T>>,
}

impl Multistatus<AddressbookProp> {
    pub fn get_addressbook_hrefs(&self) -> impl Iterator<Item = &str> {
        self.responses
            .iter()
            .filter_map(Response::get_addressbook_href)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Response<T> {
    pub href: Href,
    #[serde(rename = "propstat")]
    pub propstats: Vec<Propstat<T>>,
}

impl Response<AddressbookProp> {
    pub fn get_addressbook_href(&self) -> Option<&str> {
        for propstat in &self.propstats {
            if propstat.prop.resourcetype.addressbook.is_some() {
                return Some(self.href.value.as_str());
            }
        }

        None
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Propstat<T> {
    pub prop: T,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Href {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Status {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Ctag {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Etag {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LastModified {
    #[serde(with = "date_parser", rename = "$value", default)]
    pub value: DateTime<Utc>,
}

mod date_parser {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc2822(&s)
            .map(|d| d.into())
            .map_err(serde::de::Error::custom)
    }
}

// Current user principal structs

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CurrentUserPrincipalProp {
    pub current_user_principal: CurrentUserPrincipal,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CurrentUserPrincipal {
    pub href: Href,
}

// Addressbook home set structs

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddressbookHomeSetProp {
    pub addressbook_home_set: AddressbookHomeSet,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AddressbookHomeSet {
    pub href: Href,
}

// Addressbook structs

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddressbookProp {
    pub resourcetype: AddressbookResourceType,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AddressbookResourceType {
    pub addressbook: Option<Addressbook>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Addressbook {}

// Address data structs

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddressDataProp {
    pub address_data: Option<AddressData>,
    pub getetag: Option<Etag>,
    pub getlastmodified: Option<LastModified>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AddressData {
    #[serde(rename = "$value")]
    pub value: String,
}

// Ctag structs

#[derive(Clone, Debug, Deserialize)]
pub struct CtagProp {
    pub getctag: Ctag,
}
