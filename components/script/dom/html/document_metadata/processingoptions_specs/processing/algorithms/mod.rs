/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Numbered WHATWG algorithms. The modules follow the call hierarchy:
//! headers -> options / link_types -> preload -> request / callbacks.

pub(crate) mod headers;
pub(crate) mod extract_headers;
pub(crate) mod process_headers;
pub(crate) mod options;
pub(crate) mod apply_attributes;
pub(crate) mod origin;
pub(crate) mod url;
pub(crate) mod enumerated_attributes;
pub(crate) mod cors;
pub(crate) mod potential_cors;
pub(crate) mod destination;
pub(crate) mod request;
pub(crate) mod callbacks;
pub(crate) mod preload_response;
pub(crate) mod link_types;
