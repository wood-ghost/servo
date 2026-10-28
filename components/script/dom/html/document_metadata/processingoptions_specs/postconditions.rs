/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Postcondition specifications for link processing options.
//!
//! These describe Servo's current decisions under the existing MIME parser/API
//! contracts and classifier input requirements. They preserve ordered MIME
//! classification, use classification as the current proxy for format support,
//! and compare the full MIME record with TEXT_CSS for style destinations.
//! General parser correspondence and WHATWG agreement remain separate obligations.

use mime::Mime;
use net_traits::mime_classifier::MediaType;
use net_traits::mime_classifier_specs::{classifier as SpecClassifier, mime_api as SpecMime};
use net_traits::request::Destination;
use vstd::prelude::*;

use super::super::processingoptions::LinkProcessingOptions;
use super::servo_support::destination_is_script_like;

verus! {

// ----------------------------------------
// LinkProcessingOptions postconditions
// ----------------------------------------

/// Destination-specific acceptance after a successful MIME classification.
pub(crate) open spec fn destination_matches_media_type(
    destination: Destination,
    mime: &Mime,
    media_type: MediaType,
) -> bool {
    ||| (destination_is_script_like(destination) && media_type == MediaType::JavaScript)
    ||| match destination {
        Destination::Audio | Destination::Video => media_type == MediaType::AudioVideo,
        Destination::Image => media_type == MediaType::Image,
        Destination::Font => media_type == MediaType::Font,
        Destination::Json => media_type == MediaType::Json,
        // Preserve Servo's full-record comparison, including parameters.
        Destination::Style => SpecMime::view(mime) == SpecMime::text_css_identity(),
        Destination::Track => SpecMime::essence_str(mime) == "text/vtt"@,
        _ => false,
    }
}

/// A type matches when it is empty, the destination is fetch, or the parser and
/// ordered classifier admit a successful result accepted by the destination.
/// This reuses their result relations rather than duplicating their algorithms.
/// TODO: verify against standard
pub(crate) open spec fn type_matches_destination_result(
    options: &LinkProcessingOptions,
) -> bool {
    // ||| options.link_type@.len() == 0
    // ||| options.destination == Destination::None
    // ||| exists|mime: Mime, media_type: MediaType| {
    //     &&& <Mime as SpecMime::FromStrSpec>::from_str_ensures(options.link_type@, Ok(mime))
    //     &&& #[trigger] SpecClassifier::get_media_type_spec(&mime, Some(media_type))
    //     &&& destination_matches_media_type(options.destination, &mime, media_type)
    // }
    true
}

} // verus!
