/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use mime::Mime;
use net_traits::mime_classifier_specs::{
    mime_api::{self as SpecMime, FromStrSpec},
    requires as MimeRequires,
};
use net_traits::request::Destination;
use vstd::prelude::*;

use super::postconditions::type_matches_destination_ok;

verus! {

/// Inherit MIME requirements along the calls made by type_matches_destination.
/// Empty types and fetch destinations skip parsing. A parse failure imposes no
/// classifier requirement; every allowed successful result must satisfy the
/// existing classifier requirement, without copying its suffix restrictions.
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
    rel == "preload"@ ==> {
        // INHERITED from the type check preceding the early return.
        &&& type_matches_destination_requires(link_type, destination)
        // INHERITED from preload on the path reaching that call.
        &&& (type_matches_destination_ok(link_type, destination)
            ==> preload_requires(link_type, destination))
    }
}

} // verus!
