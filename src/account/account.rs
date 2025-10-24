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

//! # Account configuration
//!
//! Module dedicated to account configuration.

use serde::Deserialize;

#[cfg(feature = "carddav")]
use crate::carddav::config::CarddavConfig;
#[cfg(feature = "vdir")]
use crate::vdir::config::VdirConfig;

use super::de;

/// The account configuration.
#[derive(Clone, Debug, Deserialize)]
#[serde(from = "de::Account")]
pub struct Account {
    pub default: bool,
    #[cfg(feature = "carddav")]
    pub carddav: Option<CarddavConfig>,
    #[cfg(feature = "vdir")]
    pub vdir: Option<VdirConfig>,
}
