/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! External Servo API specifications, parameter lookup, and specification views.

// assume_specification generates unsafe scaffolding in this verifier-only module.
#![allow(unsafe_code)]

pub(crate) mod external_specs;
pub(crate) mod link_header;
pub(crate) mod preload;
pub(crate) mod views;
pub(crate) mod url;
pub(crate) mod environment;
pub(crate) mod policy_container;
pub(crate) mod link_element;

pub(crate) use self::external_specs::*;
