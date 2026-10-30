/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Link-processing-options construction and attribute application.
//! Source: HTML semantics.html, inspected 2026-10-29 (updated 2026-10-26).

use net_traits::request::Destination as NativeDestination;
use net_traits::ReferrerPolicy as NativeReferrerPolicy;
use vstd::prelude::*;
use super::super::model::*;
use super::super::types::{DestinationView, ReferrerPolicyView, LinkFetchPriorityView, fetch_priority_attribute_definition};
use super::super::super::predicates::ascii_lowercase;
use super::cors::determine_cors_settings_for_token;
use super::enumerated_attributes::matching_keyword;
use super::link_types::preload::translate_a_preload_destination;

verus! {

/// The keyword/state mapping referenced by apply-attributes step 7.
/// Attribute defaults belong to the separate state-determination algorithm.
pub(crate) open spec fn fetch_priority_keyword_state(value: Seq<char>) -> Option<LinkFetchPriorityView> {
    let attribute = fetch_priority_attribute_definition();
    match matching_keyword(attribute, value) {
        Some(keyword) => Some(attribute.keyword_states[keyword]),
        None => None,
    }
}

/// https://html.spec.whatwg.org/multipage/semantics.html#link-processing-options
/// Defaults plus the fields set by process-link-headers step 2.6.
pub(crate) open spec fn initial_options(
    link: ParsedLinkHeader,
    document: LinkHeaderDocumentState,
) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView {
        href: link.target_uri,
        base_url: document.base_url,
        origin: document.origin,
        environment: document.environment,
        policy_container: document.policy_container,
        document: Some(document.document),
        initiator: "link"@,
        integrity: Seq::empty(),
        type_: Seq::empty(),
        cryptographic_nonce_metadata: Seq::empty(),
        destination: DestinationView::from_native(NativeDestination::None),
        crossorigin: None,
        referrer_policy: ReferrerPolicyView::from_native(NativeReferrerPolicy::EmptyString),
        source_set: None,
        on_document_ready: None,
        fetch_priority: LinkFetchPriorityView::Auto,
    }
}

// https://html.spec.whatwg.org/multipage/semantics.html#apply-link-options-from-parsed-header-attributes
// To apply link options from parsed header attributes to a link processing options
// options given attribs and a string rel:
// The pair records the Boolean return and the updated options.
pub(crate) open spec fn apply_link_options_from_parsed_header(
    options: LinkProcessingOptionsView,
    attribs: LinkAttributes,
    rel: Seq<char>,
) -> (bool, LinkProcessingOptionsView) {
    let original_options = options;
    // 1. If rel is "preload":
    let options = if rel == "preload"@ {
        // 1.1. If attribs["as"] does not exist, then return false.
        if !attribs.dom().contains("as"@) {
            None
        } else {
            // 1.2. Let destination be the result of translating attribs["as"].
            let destination = translate_a_preload_destination(attribs["as"@]);
            match destination {
                // 1.3. If destination is null, then return false.
                None => None,
                // 1.4. Set options's destination to destination.
                Some(destination) => Some(LinkProcessingOptionsView { destination, ..options }),
            }
        }
    } else {
        Some(options)
    };
    match options {
        None => (false, original_options),
        Some(options) => {
            // 2. If attribs["crossorigin"] exists and matches a CORS settings keyword,
            // set options's crossorigin to the corresponding state.
            let options = if attribs.dom().contains("crossorigin"@) {
                match determine_cors_settings_for_token(attribs["crossorigin"@]) {
                    Some(crossorigin) => LinkProcessingOptionsView { crossorigin: Some(crossorigin), ..options },
                    None => options,
                }
            } else { options };
            // 3. If attribs["integrity"] exists, set options's integrity to it.
            let options = if attribs.dom().contains("integrity"@) {
                LinkProcessingOptionsView { integrity: attribs["integrity"@], ..options }
            } else { options };
            // 4. If attribs["referrerpolicy"] exists and ASCII-case-insensitively
            // matches a referrer policy, set options's referrer policy to it.
            let options = if attribs.dom().contains("referrerpolicy"@) {
                let policy = ascii_lowercase(attribs["referrerpolicy"@]);
                match ReferrerPolicyView::from_name(policy) {
                    Some(policy) => LinkProcessingOptionsView { referrer_policy: policy, ..options },
                    None => options,
                }
            } else { options };
            // 5. If attribs["nonce"] exists, set options's nonce to it.
            let options = if attribs.dom().contains("nonce"@) {
                LinkProcessingOptionsView { cryptographic_nonce_metadata: attribs["nonce"@], ..options }
            } else { options };
            // 6. If attribs["type"] exists, set options's type to it.
            let options = if attribs.dom().contains("type"@) {
                LinkProcessingOptionsView { type_: attribs["type"@], ..options }
            } else { options };
            // 7. If attribs["fetchpriority"] exists and matches a fetch-priority keyword,
            // set options's fetch priority to the corresponding state.
            let options = if attribs.dom().contains("fetchpriority"@) {
                match fetch_priority_keyword_state(attribs["fetchpriority"@]) {
                    Some(fetch_priority) => LinkProcessingOptionsView { fetch_priority, ..options },
                    None => options,
                }
            } else { options };
            // 8. Return true.
            (true, options)
        },
    }
}

pub(crate) open spec fn apply_link_options_from_parsed_header_ensures(
    before: LinkProcessingOptionsView,
    after: LinkProcessingOptionsView,
    attribs: LinkAttributes,
    rel: Seq<char>,
    result: bool,
) -> bool {
    (result, after) == apply_link_options_from_parsed_header(before, attribs, rel)
}

} // verus!
