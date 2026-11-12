/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use mime::{FromStrError, Mime};
use net_traits::mime_classifier_specs::{mime_api as SpecMime, predicates as SpecMimePredicates};
use SpecMime::FromStrSpec;
use net_traits::request::Destination;
use script_bindings::refcounted::Trusted;
use servo_base::id::WebViewId;
use vstd::prelude::*;

use crate::dom::document::Document;
use crate::dom::types::HTMLLinkElement;
use super::super::processingoptions::LinkProcessingOptions;

verus! {

// UA support depends on MIME contents, including parameters, so representatives
// with equal MimeView values have the same support answer.
// TRUSTED(scope): not verified yet; remove when user-agent decoder support correspondence is verified
pub(crate) uninterp spec fn is_supported_by_user_agent(mt: &SpecMime::MimeView) -> bool;

// <https://html.spec.whatwg.org/multipage/#match-preload-type>
// For the purposes of this section, a string type matches a preload destination destination if the following algorithm returns true:
pub(crate) open spec fn type_matches_destination_ok(
    type_: Seq<char>,
    destination: Destination, 
    // NOTE: preload destination is translated into Destination before LinkProcessingOptions 
) -> bool {
    // 1. If type is an empty string, then return true.
    if type_ == ""@ {
        true
    } else {
        // 2. If destination is "fetch", then return true.
        if destination == Destination::None {
            true
        } else {
            // 3. Let mimeTypeRecord be the result of parsing type.
            let mime_type_record: Result<Mime, FromStrError> = Mime::from_str_spec(type_);
            match mime_type_record {
                // 4. If mimeTypeRecord is failure, then return false.
                Err(_) => false,
                Ok(mtr) => {
                    // 5. If mimeTypeRecord is not supported by the user agent, then return false. TODO:
                    if !is_supported_by_user_agent(&SpecMime::view(&mtr)) {
                        false
                    } else {
                        // 6. If any of the following are true:
                        if
                            // 6.1 destination is "script" and mimeTypeRecord is a JavaScript MIME type;
                            (destination == Destination::Script && SpecMimePredicates::is_javascript(&mtr))
                            // 6.2 destination is "image" and mimeTypeRecord is an image MIME type;
                            || (destination == Destination::Image && SpecMimePredicates::is_image(&mtr))
                            // 6.3 destination is "font" and mimeTypeRecord is a font MIME type;
                            || (destination == Destination::Font && SpecMimePredicates::is_font(&mtr))
                            // 6.4 destination is "style" and mimeTypeRecord's essence is text/css; or
                            || (destination == Destination::Style && SpecMime::essence_str(&mtr) == "text/css"@)
                            // 6.5 estination is "track" and mimeTypeRecord's essence is text/vtt,
                            || (destination == Destination::Track && SpecMime::essence_str(&mtr) == "text/vtt"@)
                        {
                            // 6. then return true
                            true
                        } else {
                            // 7. Return false.
                            false
                        }
                    }
                },
            }
        }
    }
}

/// Unfinished postcondition boundary for the native preload call.
/// This input-only marker observes no document snapshot, resulting state, or
/// fetch trace. Replace it with an invocation-bound state/effect relation when
/// the document model is added; it currently establishes no behavior property.
// TRUSTED(scope): not verified yet; remove when preload's document-state/effect correspondence is verified
pub(crate) uninterp spec fn preload_ensures(
    options: &LinkProcessingOptions,
    webview_id: WebViewId,
    link: &Option<Trusted<HTMLLinkElement>>,
    document: &Document,
) -> bool;

} // verus!
