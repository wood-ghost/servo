/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! HTML CORS-settings keywords and potential-CORS request construction.
//! Enumerated-attribute definition and state algorithm inspected 2026-10-30.

use vstd::prelude::*;
use super::super::types::{CorsSettingsView, cors_settings_attribute_definition};
use super::enumerated_attributes::{matching_keyword, determine_the_state_of_an_attribute};
pub(crate) use super::potential_cors::create_a_potential_cors_request;

verus! {

// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-settings-attribute
// The keyword table has "anonymous" and "use-credentials". Empty/invalid value
// defaults are attribute-state rules, not additional keyword-table entries.
pub(crate) open spec fn determine_cors_settings_for_token(token: Seq<char>) -> Option<CorsSettingsView> {
    let attribute = cors_settings_attribute_definition();
    match matching_keyword(attribute, token) {
        Some(keyword) => attribute.keyword_states[keyword],
        None => None,
    }
}

pub(crate) open spec fn cors_settings_attribute(value: Option<Seq<char>>) -> Option<CorsSettingsView> {
    let state = determine_the_state_of_an_attribute(cors_settings_attribute_definition(), value);
    // The CORS definition supplies missing and invalid defaults, so it always
    // determines a state. No CORS is State { value: None }, not NoState.
    // The regression proof cors_attribute_always_has_a_state establishes this projection's domain.
    state->State_value
}

} // verus!
