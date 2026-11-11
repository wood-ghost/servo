/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Temporary Verus support for checking link-processing options incrementally.

pub(crate) mod postconditions;
pub(crate) mod processing;
pub(crate) mod requires;
mod servo_support;

pub(crate) use self::postconditions::{preload_ensures, type_matches_destination_ok};
