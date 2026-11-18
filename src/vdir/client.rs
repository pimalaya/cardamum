//! # Vdir client
//!
//! Wraps [`io_vdir::client::VdirClient`], resolving collection names to
//! their paths under the vdir root.

use std::{
    ops::{Deref, DerefMut},
    path::{Component, Path},
};

use anyhow::{Result, anyhow};
use io_vdir::{
    client::{VdirClient as Inner, VdirClientError},
    item::{
        copy::VdirItemCopyError, delete::VdirItemDeleteError, get::VdirItemGetError,
        locate::VdirItemLocateError, r#move::VdirItemMoveError,
    },
    path::VdirPath,
};
use pimalaya_cli::error::CodedError;

use crate::{
    config::{AccountConfig, VdirConfig},
    error::ErrorCode,
};

/// The vdir client a command runs against.
pub struct VdirClient {
    inner: Inner,
}

impl VdirClient {
    /// Opens the client rooted at the configured home directory.
    pub fn new(config: VdirConfig) -> Self {
        let inner = Inner::new(config.root);
        Self { inner }
    }

    /// Resolves a collection name to its path under the vdir root.
    ///
    /// A name the root holds no directory of is rejected here, io-vdir
    /// would otherwise surface a raw OS error, or remove whatever a name
    /// that is not one directory name reaches.
    pub fn collection_path(&self, name: &str) -> Result<VdirPath> {
        collection_dir(self.root(), name).ok_or_else(|| {
            let message = format!("Collection `{name}` not found");
            CodedError::new(ErrorCode::AddressbookNotFound, message).into()
        })
    }
}

/// The directory of the collection `id` under `root`, when `id` is one
/// directory name naming a directory that exists.
pub(crate) fn collection_dir(root: &VdirPath, id: &str) -> Option<VdirPath> {
    let path = root.join(id);
    (is_dir_name(id) && Path::new(path.as_str()).is_dir()).then_some(path)
}

/// Refuses to create the collection `id` when `root` already holds it,
/// coded `addressbook-duplicate`: io-vdir creates its directory over an
/// existing one and rewrites its metadata files.
pub(crate) fn check_collection_absent(root: &VdirPath, id: &str) -> Result<()> {
    match collection_dir(root, id) {
        Some(_) => {
            let message = format!("Cannot create address book `{id}`: it exists already");
            Err(CodedError::new(ErrorCode::AddressbookDuplicate, message).into())
        }
        None => Ok(()),
    }
}

/// Refuses to move or copy the item `id` into `target` when `target`
/// holds an item of that id already, coded `item-duplicate`: io-vdir
/// replaces the target's file, and its card would be lost.
pub(crate) fn check_item_absent(
    client: &VdirClient,
    target: &VdirPath,
    target_name: &str,
    id: &str,
    verb: &str,
) -> Result<()> {
    match client.locate_item(target.clone(), id) {
        Ok(_) => {
            let message = format!(
                "Cannot {verb} item `{id}` into `{target_name}`: it holds an item of that id already"
            );
            Err(CodedError::new(ErrorCode::ItemDuplicate, message).into())
        }
        Err(VdirClientError::VdirItemLocate(VdirItemLocateError::NotFound(_))) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// Whether `id` is one directory name:a single normal path component
/// that is the whole of `id`, so neither empty, `.` nor `..`, and holding
/// no separator, a trailing one included.
pub(crate) fn is_dir_name(id: &str) -> bool {
    let mut components = Path::new(id).components();

    match (components.next(), components.next()) {
        (Some(Component::Normal(name)), None) => name == id,
        _ => false,
    }
}

impl Deref for VdirClient {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VdirClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Opens the vdir client of the account.
///
/// Bails when the account declares no `[vdir]` block.
pub fn build_vdir_client(name: String, mut account_config: AccountConfig) -> Result<VdirClient> {
    let vdir_config = account_config
        .vdir
        .take()
        .ok_or_else(|| anyhow!("Vdir config is missing for account `{name}`"))?;
    Ok(VdirClient::new(vdir_config))
}

/// The code of a vdir failure in an error chain: an item io-vdir did not
/// find in its collection is `card-not-found`, cardamum's items being
/// cards.
pub(crate) fn missing_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    let locate = match cause.downcast_ref::<VdirClientError>()? {
        VdirClientError::VdirItemLocate(err)
        | VdirClientError::VdirItemGet(VdirItemGetError::Locate(err))
        | VdirClientError::VdirItemDelete(VdirItemDeleteError::Locate(err))
        | VdirClientError::VdirItemCopy(VdirItemCopyError::Locate(err))
        | VdirClientError::VdirItemMove(VdirItemMoveError::Locate(err)) => err,
        _ => return None,
    };

    match locate {
        VdirItemLocateError::NotFound(_) => Some(ErrorCode::CardNotFound),
        _ => None,
    }
}
