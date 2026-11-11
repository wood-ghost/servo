/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WHATWG preload algorithms and result contracts.
//! https://html.spec.whatwg.org/multipage/links.html#link-type-preload
//! Inspected 2026-10-29; standard last updated 2026-10-26.

use net_traits::request::Destination as NativeDestination;
use net_traits::mime_classifier_specs::{mime_api as SpecMime, predicates as SpecMimePredicates};
use vstd::prelude::*;
use super::super::super::model::*;
use super::super::super::types::DestinationView;
use super::super::super::model_trait::LinkProcessingModel;
use super::super::super::super::predicates::is_preload_destination;
use super::super::destination::translate_a_potential_destination;
use super::super::request::create_link_request;
pub(crate) use super::preload_machine::{preload, preload_requires};

verus! {

broadcast use {
    super::super::super::types::lemma_destination_name_valid,
    super::super::destination::translation_has_destination_type,
};

// https://html.spec.whatwg.org/multipage/links.html#translate-a-preload-destination
// Snapshot/inspection 2026-10-30.
// Form: row 3, direct spec fn; ordered conditional/final returns, no temporary.
// The returned destination string is represented by the native-enum view lookup.
pub(crate) open spec fn translate_a_preload_destination(destination: Seq<char>) -> Option<DestinationView> {
    // 1. If destination is not a preload destination, then return null.
    if !is_preload_destination(destination) {
        None
    } else {
        // 2. Return the result of translating destination.
        DestinationView::from_name(super::super::destination::translate_a_potential_destination(destination))
    }
}

pub(crate) proof fn preload_translation_meets_linked_domain(destination: Seq<char>)
    requires is_preload_destination(destination),
    ensures
        super::super::destination::is_potential_destination(destination),
        translate_a_preload_destination(destination) is Some,
{}

// https://html.spec.whatwg.org/multipage/links.html#match-preload-type
// Native destination adapter to the literal seven-step state machine.
// FINDING: HTML's algorithm says "fetch" but link options carry the translated
// Fetch destination (the empty string). Keep that representation choice here,
// outside the standard's step 2.
// For the purposes of this section, a string type matches a preload destination destination if the following algorithm returns true:
pub(crate) open spec fn type_matches_destination(
    // model: M,
    type_: Seq<char>,
    destination: DestinationView,
) -> bool {
    // 1. If type is an empty string, then return true.
    if type_ == ""@ {
        true
    } else {
        // 2. If destination is "fetch", then return true.
        if destination.as_str() == translate_a_potential_destination("fetch"@) {
            true
        } else {
            // 3. Let mimeTypeRecord be the result of parsing type.
            let mime_type_record = SpecMime::parse_mime_type(type_);
            // 4. If mimeTypeRecord is failure, then return false.
            if mime_type_record is None {
                false
            } else {
                // 5. If mimeTypeRecord is not supported by the user agent, then return false.
                if !SpecMimePredicates::is_supported_by_user_agent(&mime_type_record->Some_0) {
                    false
                } else {
                    // 6. If any of the following are true:
                    if
                        // 6.1 destination is "script" and mimeTypeRecord is a JavaScript MIME type;
                        (destination.as_str() == "script"@ && SpecMimePredicates::is_javascript_view(&mime_type_record->Some_0))
                        // 6.2 destination is "image" and mimeTypeRecord is an image MIME type;
                        || (destination.as_str() == "image"@ && SpecMimePredicates::is_image_view(&mime_type_record->Some_0))
                        // 6.3 destination is "font" and mimeTypeRecord is a font MIME type;
                        || (destination.as_str() == "font"@ && SpecMimePredicates::is_font_view(&mime_type_record->Some_0))
                        // 6.4 destination is "style" and mimeTypeRecord's essence is text/css; or
                        || (destination.as_str() == "style"@ && SpecMime::essence_str_view(&mime_type_record->Some_0) == "text/css"@)
                        // 6.5 destination is "track" and mimeTypeRecord's essence is text/vtt,
                        || (destination.as_str() == "track"@ && SpecMime::essence_str_view(&mime_type_record->Some_0) == "text/vtt"@)
                    {
                        // 6.6 then return true.
                        true
                    } else {
                        // 7. Return false.
                        false
                    }
                }
            }
        }
    }
}

// https://html.spec.whatwg.org/multipage/links.html#create-a-preload-key
// Snapshot/inspection 2026-10-30.
// Form: row 3, a single final return with no carried temporary. The source is
// an unnumbered sentence, so no step number is invented.
// To create a preload key for a request request, return a new preload key whose URL is request's URL, destination is request's destination, mode is request's mode, and credentials mode is request's credentials mode.
pub(crate) open spec fn create_preload_key(request: RequestView) -> PreloadKeyView {
    PreloadKeyView {
        url: request.url,
        destination: request.destination,
        mode: request.mode,
        credentials_mode: request.credentials_mode,
    }
}

// Computation helper. The canonical thirteen-step definition and the theorem
// relating this helper to its generated relations live in preload_machine.rs.
pub(crate) open spec fn preload_computation<M: LinkProcessingModel>(
    model: M,
    options: LinkProcessingOptionsView,
    process_response: Option<int>,
) -> PreloadPlan {
    if !type_matches_destination(options.type_, options.destination) {
        PreloadPlan::TypeMismatch
    } else {
        let options = if options.destination.as_native() == NativeDestination::Image {
            match options.source_set {
                Some(source_set) => LinkProcessingOptionsView {
                    href: model.select_image_source(source_set), ..options
                },
                None => options,
            }
        } else { options };
        let request = create_link_request(model, options);
        match request {
            None => PreloadPlan::RequestFailure,
            Some(request) => {
                let unsafe_end_time = 0int;
                let entry = PreloadEntryView {
                    integrity_metadata: options.integrity,
                    response: None,
                    on_response_available: None,
                };
                let key = create_preload_key(request);
                let request = if options.document is None {
                    RequestView { initiator_type: Some("early hint"@), ..request }
                } else { request };
                let controller = None;
                let report_timing = PreloadCallback::ReportTiming;
                // This plan describes the fetch and its callback; execution is a
                // separate effect/trace obligation, not a spec-mode network operation.
                let fetch = PreloadFetch {
                    request,
                    process_response_consume_body: PreloadCallback::ProcessResponseConsumeBody,
                    controller: model.fetch_request(request, PreloadCallback::ProcessResponseConsumeBody),
                };
                let commit_callback = PreloadCallback::Commit;
                let commit_algorithm = model.commit_algorithm(PreloadCallbackState {
                    entry, key, controller: Some(fetch.controller), unsafe_end_time,
                    document: options.document, process_response,
                });
                let commit = match options.document {
                    None => PreloadCommit::OnDocumentReady,
                    Some(document) => PreloadCommit::Document { document },
                };
                let options = if options.document is None {
                    LinkProcessingOptionsView { on_document_ready: Some(commit_algorithm), ..options }
                } else { options };
                PreloadPlan::Fetch {
                    options,
                    request,
                    initial_unsafe_end_time: unsafe_end_time,
                    entry,
                    key,
                    initial_controller: controller,
                    process_response,
                    report_timing,
                    fetch,
                    commit_callback,
                    commit_algorithm,
                    commit,
                }
            },
        }
    }
}

pub(crate) open spec fn preload_ensures<M: LinkProcessingModel>(
    model: M,
    options: LinkProcessingOptionsView,
    process_response: Option<int>,
    result: PreloadPlan,
) -> bool
    recommends preload_requires(model, options),
{
    result == preload(model, options, process_response)
}

} // verus!
