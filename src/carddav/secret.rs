use secrecy::{ExposeSecret, SecretString};
use serde::{de::value::Error, Deserialize, Serialize, Serializer};

/// The secret enum.
///
/// A secret can be retrieved either from a raw string, from a shell
/// command or from a keyring entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(try_from = "SecretSerde")]
#[serde(into = "SecretSerde")]
pub enum Secret {
    /// The secret is contained in a raw string.
    ///
    /// This variant is not safe to use and therefore not
    /// recommended. Yet it works well for testing purpose.
    Plain(SecretString),

    /// The secret is exposed by the given shell command.
    ///
    /// This variant takes the secret from the first line returned by
    /// the given shell command.
    ///
    /// See [process-flows](https://crates.io/crates/process-flows).
    #[cfg(feature = "command")]
    Command(io_process::Command),

    /// The secret is contained in the user's global keyring at the
    /// given entry.
    ///
    /// See [keyring-flows](https://crates.io/crates/keyring-flows).
    #[cfg(feature = "keyring")]
    Keyring(io_keyring::Entry),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecretSerde {
    #[serde(serialize_with = "serialize_secret_string")]
    Raw(SecretString),

    #[cfg(feature = "command")]
    Command(io_process::Command),
    #[cfg(not(feature = "command"))]
    #[serde(with = "missing_command")]
    Command {},

    #[cfg(feature = "keyring")]
    Keyring(io_keyring::Entry),
    #[cfg(not(feature = "keyring"))]
    #[serde(with = "missing_keyring")]
    Keyring {},
}

impl TryFrom<SecretSerde> for Secret {
    type Error = Error;

    fn try_from(secret: SecretSerde) -> Result<Self, Self::Error> {
        match secret {
            SecretSerde::Raw(secret) => Ok(Self::Plain(secret)),

            #[cfg(feature = "command")]
            SecretSerde::Command(cmd) => Ok(Self::Command(cmd)),
            #[cfg(not(feature = "command"))]
            SecretSerde::Command {} => {
                Err(serde::de::Error::custom("missing `command` cargo feature"))
            }

            #[cfg(feature = "keyring")]
            SecretSerde::Keyring(entry) => Ok(Self::Keyring(entry)),
            #[cfg(not(feature = "keyring"))]
            SecretSerde::Keyring {} => {
                Err(serde::de::Error::custom("missing `keyring` cargo feature"))
            }
        }
    }
}

impl Into<SecretSerde> for Secret {
    fn into(self) -> SecretSerde {
        match self {
            Self::Plain(secret) => SecretSerde::Raw(secret),
            #[cfg(feature = "command")]
            Self::Command(cmd) => SecretSerde::Command(cmd),
            #[cfg(feature = "keyring")]
            Self::Keyring(cmd) => SecretSerde::Keyring(cmd),
        }
    }
}

fn serialize_secret_string<S: Serializer>(secret: &SecretString, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(secret.expose_secret())
}

#[cfg(not(feature = "command"))]
mod missing_command {
    const ERR: &'static str = "missing `command` cargo feature";

    pub(crate) fn serialize<S: serde::Serializer>(_: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(Self::ERR))
    }

    pub(crate) fn deserialize<'de, D: serde::Deserializer<'de>>(_: D) -> Result<(), D::Error> {
        Err(serde::de::Error::custom(Self::ERR))
    }
}

#[cfg(not(feature = "keyring"))]
mod missing_keyring {
    const ERR: &'static str = "missing `keyring` cargo feature";

    pub(crate) fn serialize<S: serde::Serializer>(_: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(Self::ERR))
    }

    pub(crate) fn deserialize<'de, D: serde::Deserializer<'de>>(_: D) -> Result<(), D::Error> {
        Err(serde::de::Error::custom(Self::ERR))
    }
}

// #[cfg(test)]
// mod tests {
//     use secrecy::{ExposeSecret, SecretString};

//     use crate::Secret;

//     #[test]
//     fn serialize_raw() {
//         let secret: Secret = toml::from_str("raw = \"password\"").unwrap();
//         let Secret::Raw(secret) = secret else {
//             panic!("should serialize Secret::Raw variant");
//         };

//         assert_eq!(secret.expose_secret(), "password");
//     }

//     #[test]
//     fn deserialize_raw() {
//         let secret = Secret::Raw(SecretString::from("password"));
//         let toml = toml::to_string(&secret).unwrap();

//         assert_eq!(toml.trim(), "raw = \"password\"");
//     }

//     #[cfg(feature = "command")]
//     #[test]
//     fn serialize_command_str() {
//         let secret: Secret = toml::from_str("command = \"echo password\"").unwrap();
//         let Secret::Command(cmd) = secret else {
//             panic!("should serialize Secret::Command variant");
//         };

//         assert_eq!(cmd.program, "echo");

//         let args = cmd.args.unwrap();
//         assert_eq!(1, args.len());

//         let arg = args.into_iter().next().unwrap();
//         assert_eq!(arg, "password");
//     }

//     #[cfg(feature = "command")]
//     #[test]
//     fn serialize_command_seq() {
//         let secret: Secret = toml::from_str("command = [\"echo\", \"password\"]").unwrap();
//         let Secret::Command(cmd) = secret else {
//             panic!("should serialize Secret::Command variant");
//         };

//         assert_eq!(cmd.program, "echo");

//         let args = cmd.args.unwrap();
//         assert_eq!(1, args.len());

//         let arg = args.into_iter().next().unwrap();
//         assert_eq!(arg, "password");
//     }

//     #[cfg(not(feature = "command"))]
//     #[test]
//     fn serialize_command() {
//         let err = toml::from_str::<Secret>("command = \"echo password\"").unwrap_err();
//         assert_eq!(err.message(), "missing `command` cargo feature");
//     }

//     #[cfg(feature = "command")]
//     #[test]
//     fn deserialize_command() {
//         use io_process::Command;

//         let mut cmd = Command::new("echo");
//         cmd.arg("password");

//         let secret = Secret::Command(cmd);
//         let toml = toml::to_string(&secret).unwrap();

//         assert_eq!(toml.trim(), "command = [\"echo\", \"password\"]");
//     }

//     #[cfg(feature = "keyring")]
//     #[test]
//     fn serialize_keyring() {
//         let secret: Secret = toml::from_str("keyring = \"entry\"").unwrap();
//         let Secret::Keyring(entry) = secret else {
//             panic!("should serialize Secret::Keyring variant");
//         };

//         assert_eq!(entry.name, "entry");
//     }

//     #[cfg(not(feature = "keyring"))]
//     #[test]
//     fn serialize_keyring() {
//         let err = toml::from_str::<Secret>("keyring = \"echo password\"").unwrap_err();
//         assert_eq!(err.message(), "missing `keyring` cargo feature");
//     }

//     #[cfg(feature = "keyring")]
//     #[test]
//     fn deserialize_keyring() {
//         use io_keyring::Entry;

//         let entry = Entry::new("entry");
//         let secret = Secret::Keyring(entry);
//         let toml = toml::to_string(&secret).unwrap();

//         assert_eq!(toml.trim(), "keyring = \"entry\"");
//     }
// }
