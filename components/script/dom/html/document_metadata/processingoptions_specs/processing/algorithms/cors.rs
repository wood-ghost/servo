/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! HTML CORS-settings keywords and potential-CORS request construction.
//! Enumerated-attribute definition and state algorithm inspected 2026-10-30.

use net_traits::request::CorsSettings as NativeCorsSettings;
use vstd::prelude::*;
use super::super::model::*;
use super::super::types::{CorsSettingsView, DestinationView, ServoUrlView, cors_settings_attribute_definition};
use super::super::model_trait::LinkProcessingModel;
use super::enumerated_attributes::{matching_keyword, determine_the_state_of_an_attribute};

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

// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#create-a-potential-cors-request
pub(crate) open spec fn create_a_potential_cors_request<M: LinkProcessingModel>(
    model: M,
    url: ServoUrlView,
    destination: DestinationView,
    cors_attribute_state: Option<CorsSettingsView>,
    same_origin_fallback: bool,
) -> RequestView {
    // 1. Let mode be "no-cors" if corsAttributeState is No CORS, and "cors" otherwise.
    let mode = if cors_attribute_state is None {
        RequestMode::NoCors
    } else {
        RequestMode::Cors
    };
    // 2. If the same-origin fallback flag is set and mode is "no-cors", set mode to "same-origin".
    let mode = if same_origin_fallback && mode == RequestMode::NoCors {
        RequestMode::SameOrigin
    } else {
        mode
    };
    // 3. Let credentialsMode be "include".
    let credentials_mode = CredentialsMode::Include;
    // 4. If corsAttributeState is Anonymous, set credentialsMode to "same-origin".
    let credentials_mode = if cors_attribute_state is Some
        && cors_attribute_state->Some_0.as_native() == NativeCorsSettings::Anonymous
    {
        CredentialsMode::SameOrigin
    } else {
        credentials_mode
    };
    // 5. Return a new request with URL, destination, mode, credentials mode,
    // and the use-URL-credentials flag set. Other defaults belong to Fetch.
    RequestView {
        url,
        destination,
        mode,
        credentials_mode,
        use_url_credentials: true,
        ..model.new_request()
    }
}

} // verus!
