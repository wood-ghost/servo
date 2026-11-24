/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use mime::Mime;
use net_traits::ReferrerPolicy;
use net_traits::mime_classifier_specs::{
    mime_api::{self as SpecMime, FromStrSpec},
    predicates as SpecMimePredicates,
    requires as MimeRequires,
};
use net_traits::request::Destination;
use vstd::prelude::*;

use crate::dom::document::Document;
use super::super::processingoptions::{LinkHeader, LinkProcessingPhase};
use super::postconditions::{cors_settings_for_token, translate_a_preload_destination, type_matches_destination_ok};
use super::processing::{LinkHeaderView, LinkHeaderViewSpec, LinkProcessingOptionsView, link_header_has_key, link_header_value_for_key};
use super::processing::process_link_headers::{get_attributes, link_object_attribs_get};
use super::servo_support::referrer_policy_for_token;

verus! {

// Servo looks up each lowercase key case-sensitively. Check every name before normalization.
pub(crate) open spec fn link_header_key_is_lowercase(header: LinkHeaderView, key: Seq<char>) -> bool {
    forall|i: int| 0 <= i < header.params.len() ==>
        (SpecMime::ascii_lowercase((#[trigger] header.params[i]).0) == key ==> header.params[i].0 == key)
}

// An absent key is fine. If present, its first occurrence must carry a value, which may be empty.
pub(crate) open spec fn link_header_key_has_value_if_present(header: LinkHeaderView, key: Seq<char>) -> bool {
    link_header_has_key(header, key) ==> link_header_value_for_key(header, key) is Some
}

pub(crate) open spec fn apply_link_options_header_requires(header: LinkHeaderView) -> bool {
    // Servo does not implement step 7. Exclude fetchpriority from the normalized attributes.
    &&& link_object_attribs_get(get_attributes(header.params), "fetchpriority"@) is None
    &&& link_header_key_is_lowercase(header, "as"@)
    &&& link_header_key_is_lowercase(header, "crossorigin"@)
    &&& link_header_key_is_lowercase(header, "integrity"@)
    &&& link_header_key_is_lowercase(header, "referrerpolicy"@)
    &&& link_header_key_is_lowercase(header, "nonce"@)
    &&& link_header_key_is_lowercase(header, "type"@)
    &&& link_header_key_has_value_if_present(header, "integrity"@)
    &&& link_header_key_has_value_if_present(header, "referrerpolicy"@)
    &&& link_header_key_has_value_if_present(header, "nonce"@)
    &&& link_header_key_has_value_if_present(header, "type"@)
}

pub(crate) open spec fn apply_link_options_before_requires(
    before: LinkProcessingOptionsView,
    header: LinkHeaderView,
) -> bool {
    let cors = link_header_value_for_key(header, "crossorigin"@);
    let policy = link_header_value_for_key(header, "referrerpolicy"@);
    // On an unrecognized token Servo writes the default, while the spec preserves the old field.
    &&& ((cors is Some && cors_settings_for_token(cors->Some_0) is None) ==> before.crossorigin is None)
    &&& ((policy is Some && referrer_policy_for_token(policy->Some_0) is None)
        ==> before.referrer_policy == ReferrerPolicy::EmptyString)
}

pub(crate) open spec fn apply_link_options_from_parsed_header_requires(
    before: LinkProcessingOptionsView,
    header: LinkHeaderView,
) -> bool {
    &&& apply_link_options_header_requires(header)
    &&& apply_link_options_before_requires(before, header)
}

/// MIME call requirements and compatibility restrictions for type matching.
/// Empty types and fetch destinations skip parsing. A parse failure imposes no
/// classifier requirement; every allowed successful result must satisfy the
/// existing classifier requirement, without copying its suffix restrictions.
/// Audio/video and font matches must not be shadowed by XML. JSON matches must
/// not be shadowed by image, audio/video, or font classification.
/// Style inputs matching the CSS essence additionally require empty parameters
/// and no structured suffix for compatibility with the native CSS comparison.
pub(crate) open spec fn type_matches_destination_requires(
    link_type: Seq<char>,
    destination: Destination,
) -> bool {
    if link_type.len() == 0 || destination == Destination::None {
        true
    } else {
        &&& <Mime as FromStrSpec>::from_str_requires(link_type)
        &&& forall|mt: Mime|
            #[trigger] <Mime as FromStrSpec>::from_str_ensures(link_type, Ok(mt))
                ==> {
                    &&& MimeRequires::get_media_type_requires(&mt)
                    // IMPLICIT: Servo selects XML before AudioVideo; excluded witnesses:
                    // type="audio/foo+xml", destination=Audio;
                    // type="video/foo+xml", destination=Video.
                    &&& (((destination == Destination::Audio || destination == Destination::Video)
                        && SpecMimePredicates::is_audio_video(&mt))
                        ==> !SpecMimePredicates::is_xml(&mt))
                    // IMPLICIT: Servo selects XML before Font; excluded witness:
                    // type="font/foo+xml", destination=Font.
                    &&& ((destination == Destination::Font && SpecMimePredicates::is_font(&mt))
                        ==> !SpecMimePredicates::is_xml(&mt))
                    // IMPLICIT: Servo selects Image, AudioVideo, and Font before Json.
                    // Excluded witnesses for destination=Json: type="image/foo+json",
                    // type="audio/foo+json", and type="font/foo+json".
                    &&& ((destination == Destination::Json && SpecMimePredicates::is_json(&mt)) ==> {
                        &&& !SpecMimePredicates::is_image(&mt)
                        &&& !SpecMimePredicates::is_audio_video(&mt)
                        &&& !SpecMimePredicates::is_font(&mt)
                    })
                    &&& ((destination == Destination::Style
                        && SpecMime::essence_str(&mt) == "text/css"@) ==> {
                        // IMPLICIT: Servo compares the full CSS record; excluded witness:
                        // destination=Style, type="text/css; charset=utf-8".
                        &&& SpecMime::view(&mt).params.len() == 0
                        // Explicitly retain the requested no-suffix condition.
                        // It is already implied by the CSS essence comparison above.
                        &&& SpecMime::view(&mt).suffix == None
                    })
                }
    }
}

/// Input requirements of preload's currently exposed contract.
pub(crate) open spec fn preload_requires(
    link_type: Seq<char>,
    destination: Destination,
) -> bool {
    // INHERITED from type_matches_destination.
    &&& type_matches_destination_requires(link_type, destination)
    // IMPLICIT: native preload asserts a match instead of returning; witness: type="text/plain", destination=Style.
    &&& type_matches_destination_ok(link_type, destination)
}

/// Only preload link headers call the type checker. Its requirements are needed
/// before the guard; preload's requirements apply only after a successful match.
pub(crate) open spec fn process_link_header_requires(
    rel: Seq<char>,
    link_type: Seq<char>,
    destination: Destination,
) -> bool {
    rel == "preload"@ ==> type_matches_destination_requires(link_type, destination)
}

// Step 2.4 skips headers from the other phase. For the current phase we allow either
// outcome of the media check, so the precondition does not depend on the environment.
pub(crate) open spec fn link_header_dispatch_requires(h: LinkHeaderView, phase: LinkProcessingPhase) -> bool {
    let rel = link_header_value_for_key(h, "rel"@);
    let expected_phase = if link_header_has_key(h, "srcset"@) || link_header_has_key(h, "imagesrcset"@)
        || link_header_has_key(h, "media"@) {
        LinkProcessingPhase::Media
    } else {
        LinkProcessingPhase::PreMedia
    };
    let attribs = get_attributes(h.params);
    let as_ = link_object_attribs_get(attribs, "as"@);
    (rel is Some && expected_phase == phase) ==> {
        // IMPLICIT: restrictions on author-supplied header attributes inherited from apply.
        &&& apply_link_options_header_requires(h)
        // IMPLICIT: type and as are author-controlled.
        &&& (rel->Some_0 == "preload"@ && as_ is Some && translate_a_preload_destination(as_->Some_0) is Some)
            ==> type_matches_destination_requires(
                link_object_attribs_get(attribs, "type"@).unwrap_or(""@),
                translate_a_preload_destination(as_->Some_0)->Some_0,
            )
    }
}

// <https://html.spec.whatwg.org/multipage/#process-link-headers>
// To process link headers given a Document doc, a response response, and a "pre-media" or "media" phase:
pub(crate) open spec fn process_link_headers_requires(
    link_headers: Seq<LinkHeader>,
    document: &Document,
    phase: LinkProcessingPhase,
) -> bool {
    &&& (phase == LinkProcessingPhase::Media || phase == LinkProcessingPhase::PreMedia)
    // IMPLICIT: each author-supplied header must be in the dispatch domain.
    &&& forall|i: int| 0 <= i < link_headers.len() ==> link_header_dispatch_requires(#[trigger] link_headers[i]@, phase)
    // 1. Let links be the result of extracting links from response's header list.
}

} // verus!
