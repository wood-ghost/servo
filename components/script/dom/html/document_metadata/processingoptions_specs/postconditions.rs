/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use mime::{FromStrError, Mime};
use net_traits::mime_classifier::MediaType;
use net_traits::mime_classifier_specs::{classifier as SpecClassifier, mime_api as SpecMime};
use SpecMime::FromStrSpec;
use net_traits::request::{CorsSettings, Destination};
use vstd::prelude::*;

use super::csp_api::{DestinationSpec, translate_a_potential_destination};
use super::processing::{
    DocumentView, LinkProcessingOptionsView, OnResponseAvailableView, PixelDensity, PreloadEntryView,
    PreloadKeyView, RequestView, SourceSetView, preload_key_of,
};
use super::processing::process_link_headers::{
    fetch_priority_for_token, link_object_attribs_get,
    referrer_policy_for_token,
};

verus! {

/// <https://html.spec.whatwg.org/multipage/links.html#preload-destination>
pub(crate) open spec fn is_preload_destination(value: Seq<char>) -> bool {
    ||| value == "fetch"@
    ||| value == "font"@
    ||| value == "image"@
    ||| value == "script"@
    ||| value == "style"@
    ||| value == "track"@
}

/// <https://html.spec.whatwg.org/multipage/links.html#translate-a-preload-destination>
pub(crate) open spec fn translate_a_preload_destination(destination: Seq<char>) -> Option<Destination> {
    // 1. If destination is not a preload destination, then return null.
    if !is_preload_destination(destination) {
        None
    } else {
        // 2. Return the result of translating destination.
        <Destination as DestinationSpec>::from_str(translate_a_potential_destination(destination))
    }
}

// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-settings-attribute
pub(crate) open spec fn cors_settings_for_token(value: Seq<char>) -> Option<CorsSettings> {
    let keyword = SpecMime::ascii_lowercase(value);
    if keyword == "anonymous"@ { Some(CorsSettings::Anonymous) }
    else if keyword == "use-credentials"@ { Some(CorsSettings::UseCredentials) }
    else { None }
}

// None represents failure; Some carries the updated options on success.
// https://html.spec.whatwg.org/multipage/semantics.html#apply-link-options-from-parsed-header-attributes
pub open(crate) spec fn apply_header_attributes(
    options: LinkProcessingOptionsView,
    attribs: Seq<(Seq<char>, Seq<char>)>,
    rel: Seq<char>,
) -> Option<LinkProcessingOptionsView> {
    // 1. If rel is "preload":
    let after_step_1 = if rel == "preload"@ {
        match link_object_attribs_get(attribs, "as"@) {
            // 1.1 If attribs["as"] does not exist, then return false.
            None => None,
            // 1.2 Let destination be the result of translating attribs["as"].
            Some(value) => match translate_a_preload_destination(value) {
                // 1.3 If destination is null, then return false.
                None => None,
                // 1.4 Set options's destination to destination.
                Some(destination) => Some(LinkProcessingOptionsView { destination, ..options }),
            },
        }
    } else {
        Some(options)
    };
    match after_step_1 {
        None => None, // step 1 return
        Some(options) => {
            // 2. If attribs["crossorigin"] exists and is an ASCII case-insensitive match for one of the CORS settings attribute keywords, then set options's crossorigin to the CORS settings attribute state corresponding to that keyword.
            let options = match link_object_attribs_get(attribs, "crossorigin"@) {
                Some(value) => match cors_settings_for_token(value) {
                    Some(cors) => LinkProcessingOptionsView { crossorigin: Some(cors), ..options },
                    None => options,
                },
                None => options,
            };
            // 3. If attribs["integrity"] exists, then set options's integrity to attribs["integrity"].
            let options = match link_object_attribs_get(attribs, "integrity"@) {
                Some(integrity) => LinkProcessingOptionsView { integrity, ..options },
                None => options,
            };
            // 4. If attribs["referrerpolicy"] exists and is an ASCII case-insensitive match for some referrer policy, then set options's referrer policy to that referrer policy.
            let options = match link_object_attribs_get(attribs, "referrerpolicy"@) {
                Some(value) => match referrer_policy_for_token(value) {
                    Some(referrer_policy) => LinkProcessingOptionsView { referrer_policy, ..options },
                    None => options,
                },
                None => options,
            };
            // 5. If attribs["nonce"] exists, then set options's nonce to attribs["nonce"].
            let options = match link_object_attribs_get(attribs, "nonce"@) {
                Some(nonce) => LinkProcessingOptionsView { cryptographic_nonce_metadata: nonce, ..options },
                None => options,
            };
            // 6. If attribs["type"] exists, then set options's type to attribs["type"].
            let options = match link_object_attribs_get(attribs, "type"@) {
                Some(type_) => LinkProcessingOptionsView { type_, ..options },
                None => options,
            };
            // 7. If attribs["fetchpriority"] exists and is an ASCII case-insensitive match for a fetch priority attribute keyword, then set options's fetch priority to that fetch priority attribute keyword.
            let options = match link_object_attribs_get(attribs, "fetchpriority"@) {
                Some(value) => match fetch_priority_for_token(value) {
                    Some(fetch_priority) => LinkProcessingOptionsView { fetch_priority, ..options },
                    None => options,
                },
                None => options,
            };
            // 8. Return true.
            Some(options)
        }
    }
}

pub(crate) open spec fn apply_link_options_from_parsed_header(
    before: LinkProcessingOptionsView,
    after: LinkProcessingOptionsView,
    attribs: Seq<(Seq<char>, Seq<char>)>,
    rel: Seq<char>,
    result: bool,
) -> bool {
    match apply_header_attributes(before, attribs, rel) {
        Some(expected) => result && after == expected,
        None => !result && after == before,
    }
}

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

// TODO: Model image selection and link request creation. These declarations do not
// specify the corresponding native methods.
pub(crate) uninterp spec fn whatwg_select_image_source(
    source_set: SourceSetView, document: DocumentView,
) -> Option<(Seq<char>, PixelDensity)>;

pub(crate) uninterp spec fn whatwg_create_link_request(options: LinkProcessingOptionsView) -> Option<RequestView>;

// https://html.spec.whatwg.org/multipage/links.html#preload
// To preload given a link processing options options and an optional processResponse, which is an algorithm accepting a response:
pub(crate) open spec fn preload_ensures(
    options: LinkProcessingOptionsView,
    process_response: Option<OnResponseAvailableView>, // TODO: model callback execution
    preloaded: Map<PreloadKeyView, PreloadEntryView>,
) -> (Map<PreloadKeyView, PreloadEntryView>, Seq<RequestView>) {
    // 1. If options's type doesn't match options's destination, then return.
    if !type_matches_destination_ok(options.type_, options.destination) {
        (preloaded, Seq::<RequestView>::empty())
    } else {
        // 2. If options's destination is "image" and options's source set is not null:
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
                match whatwg_create_link_request(options) {
                    // 4. If request is null, then return.
                    None => (preloaded, Seq::<RequestView>::empty()),
                    Some(request) => {
                        // 5. Let unsafeEndTime be 0. (skip)
                        // 6. Let entry be a new preload entry whose integrity metadata is options's integrity.
                        let entry = PreloadEntryView {
                            integrity_metadata: options.integrity,
                            response: None,
                            on_response_available: None,
                        };
                        // 7. Let key be the result of creating a preload key given request.
                        let key = preload_key_of(request);
                        // 8. If options's document is null, then set request's initiator type to "early hint".
                        let request = if options.document is None {
                            RequestView { initiator_type: Some("early hint"@), ..request }
                        } else {
                            request
                        };
                        // 9–10. The controller and timing callback are not observed here.
                        // 11. Fetch request.
                        let fetches = seq![request];
                        // 12–13. Commit immediately only when a document is available.
                        let preloaded = if options.document is None {
                            preloaded
                        } else {
                            preloaded.insert(key, entry)
                        };
                        (preloaded, fetches)
                    },
                }
            },
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
