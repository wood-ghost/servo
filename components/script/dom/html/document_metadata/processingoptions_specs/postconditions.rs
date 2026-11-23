/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use mime::{FromStrError, Mime};
use net_traits::mime_classifier::MediaType;
use net_traits::mime_classifier_specs::{classifier as SpecClassifier, mime_api as SpecMime};
use SpecMime::FromStrSpec;
use net_traits::request::Destination;
use vstd::prelude::*;

use super::processing::{DocumentLinkState, DocumentView, LinkProcessingOptionsView, ProcessLinkHeaderOutput};

verus! {

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
                    // Model actual decoder support; Servo currently approximates it with MIME classification.
                    match SpecClassifier::get_media_type_spec(&mtr) {
                        None => false,
                        Some(mt) => {
                            // Destination-specific preconditions exclude known MIME-group priority conflicts.
                            // 6. If any of the following are true:
                            if
                                // 6.1 destination is "script" and mimeTypeRecord is a JavaScript MIME type;
                                (destination == Destination::Script && mt == MediaType::JavaScript)
                                // 6.2 destination is "image" and mimeTypeRecord is an image MIME type;
                                || (destination == Destination::Image && mt == MediaType::Image)
                                // 6.3 destination is "font" and mimeTypeRecord is a font MIME type;
                                || (destination == Destination::Font && mt == MediaType::Font)
                                // 6.4 destination is "style" and mimeTypeRecord's essence is text/css; or
                                || (destination == Destination::Style && SpecMime::essence_str(&mtr) == "text/css"@)
                                // 6.5 destination is "track" and mimeTypeRecord's essence is text/vtt,
                                || (destination == Destination::Track && SpecMime::essence_str(&mtr) == "text/vtt"@)
                                // The following features are included additionally in Servo
                                // destination is "audio" or "video", and mimeTypeRecord is an audio or video MIME type;
                                || ((destination == Destination::Audio || destination == Destination::Video)
                                    && mt == MediaType::AudioVideo)
                                // destination is a script-like destination and mimeTypeRecord is a JavaScript MIME type;
                                || (destination.is_script_like() && mt == MediaType::JavaScript)
                                // destination is "json" and mimeTypeRecord is a JSON MIME type;
                                || (destination == Destination::Json && mt == MediaType::Json)
                            {
                                // 6. then return true
                                true
                            } else {
                                // 7. Return false.
                                false
                            }
                        },
                    }
                },
            }
        }
    }
}

// https://html.spec.whatwg.org/multipage/links.html#preload
// To preload given a link processing options options and an optional processResponse, which is an algorithm accepting a response:
pub(crate) open spec fn preload_ensures(
    options: LinkProcessingOptionsView,
    process_response: Option<AlgorithmView>, // TODO:
    preloaded: Map<PreloadKeyView, PreloadEntryView>,
) -> (Map<PreloadKeyView, PreloadEntryView>, Seq<RequestView>) {
    // 1. If options's type doesn't match options's destination, then return.
    if !type_matches_destination_ok(options.type_, options.destination) {
        (preloaded, Seq::<RequestView>::empty())
    } else {
        // 2. If options's destination is "image" and options's source set is not null:
        if options.destination == Destination::Image && options.source_set != None {
            let selected = select_image_source_from_source_set(options.source_set, options.document);
            if selected is None {
                new_options
            } else {
                // 2.4 Set options's href to selected[0].
                let new_options.href = selected[0]; 
            }
        }

        let after2: Option<LinkProcessingOptionsView> =
            if options.destination == Destination::Image && options.source_set is Some {
                // 2.1 Assert: options's document is not null. TODO: add to preload_requires
                // 2.2 Let selected be the result of selecting an image source from a source set given options's source set and options's document.
                let selected = whatwg_select_image_source(
                    options.source_set->Some_0,
                    options.document->Some_0,
                );
                match selected {
                    // 2.3 If selected is null, then return.
                    None => None,
                    // 2.4 Set options's href to selected[0].
                    Some(selected) => Some(LinkProcessingOptionsView { href: selected.0, ..options }),
                }
            } else {
                Some(options)
            };
        match after2 {
            // Returned at 2.3.
            None => (preloaded, Seq::<RequestView>::empty()),
            Some(options) => {
            // 3. Let request be the result of creating a link request given options.
            let request = whatwg_create_link_request(options);
            match request {
            // 4. If request is null, then return.
                None => (preloaded, Seq::<RequestView>::empty()),
                Some(request) =>{
                    // 5. Let unsafeEndTime be 0. (skip)
                    // 6. Let entry be a new preload entry whose integrity metadata is options's integrity.
                    // TODO: add PreloadEntryView https://html.spec.whatwg.org/multipage/links.html#preload-entry
                    let entry = PreloadEntryView {
                        integrity_metadata: options.integrity,
                        response: None,                 // default: null
                        on_response_available: None,    // default: null
                    };
                    // 7. Let key be the result of creating a preload key given request.
                    let key = new_preload_key(request);
                    // 8. If options's document is null, then set request's initiator type to "early hint".
                    let request = if options.document is None {
                        RequestView { initiator_type: Some("early hint"@), ..request }
                    } else {
                        request
                    };
                    // 9. Let controller be null. (skip)
                    // 10. Let reportTiming given a Document document be to report timing for controller given document's relevant global object. (skip)
                    // 11. Set controller to the result of fetching request, with processResponseConsumeBody set to the following steps given a response response and null, failure, or a byte sequence bodyBytes:
                    let fetches = seq![request];
                    // 12. Let commit be the following steps given a Document document:i ...
                    // 13. If options's document is null, then set options's on document ready to commit. Otherwise, call commit with options's document. 
                    let preloaded = if options.document is None {
                        preloaded
                    } else {
                        preloaded.insert(key, entry)
                    };

                    (preloaded, fetches)
            }
        }
    }
}

/// Step 2.9 effect: the 'process a link header' steps ran with these arguments,
/// starting from this document snapshot. Effects of several calls (fetches, the
/// map of preloaded resources) are not composed here.
// TRUSTED(scope): not verified yet; remove when process_link_header's document-state/effect correspondence is verified
pub uninterp spec fn process_link_header_ensures(
    rel: Seq<char>,
    options: LinkProcessingOptionsView,
    doc: DocumentView,
) -> bool;

} // verus!
