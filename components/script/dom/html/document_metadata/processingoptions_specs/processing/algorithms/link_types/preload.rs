/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WHATWG preload algorithms and result contracts.
//! https://html.spec.whatwg.org/multipage/links.html#link-type-preload
//! Inspected 2026-10-29; standard last updated 2026-10-26.

use net_traits::request::Destination as NativeDestination;
use vstd::prelude::*;
use super::super::super::model::*;
use super::super::super::types::DestinationView;
use super::super::super::model_trait::LinkProcessingModel;
use super::super::super::super::predicates::is_preload_destination;
use super::super::request::create_link_request;

verus! {

// https://html.spec.whatwg.org/multipage/links.html#translate-a-preload-destination
pub(crate) open spec fn translate_a_preload_destination(destination: Seq<char>) -> Option<DestinationView> {
    // 1. If destination is not a preload destination, then return null.
    if !is_preload_destination(destination) {
        None
    } else {
        // 2. Return the result of translating destination.
        // Fetch's potential destination "fetch" translates to the empty destination.
        Some(DestinationView::from_native(
            if destination == "fetch"@ { NativeDestination::None }
            else if destination == "font"@ { NativeDestination::Font }
            else if destination == "image"@ { NativeDestination::Image }
            else if destination == "script"@ { NativeDestination::Script }
            else if destination == "style"@ { NativeDestination::Style }
            else { NativeDestination::Track }
        ))
    }
}

// https://html.spec.whatwg.org/multipage/links.html#match-preload-type
// For a preload destination, return whether the type matches that destination.
pub(crate) open spec fn type_matches_destination<M: LinkProcessingModel>(
    model: M,
    type_: Seq<char>,
    destination: DestinationView,
) -> bool {
    // Inspect the existing Servo enum carried by the local wrapper.
    let destination = destination.as_native();
    // 1. If type is an empty string, then return true.
    if type_.len() == 0 {
        true
    // 2. If destination is "fetch", then return true.
    // The translated Fetch destination represents this as NativeDestination::None.
    } else if destination == NativeDestination::None {
        true
    } else {
        // 3. Let mimeTypeRecord be the result of parsing type.
        let mime_type_record = model.parse_mime_type(type_);
        match mime_type_record {
            // 4. If mimeTypeRecord is failure, then return false.
            None => false,
            Some(mime_type_record) => {
                // 5. If mimeTypeRecord is not supported by the user agent, return false.
                if !model.mime_supported(mime_type_record) {
                    false
                } else {
                    // 6. If any of the following are true, then return true:
                    let matches =
                        // destination is "script" and the MIME type is JavaScript;
                        (destination == NativeDestination::Script && model.is_javascript_mime(mime_type_record))
                        // destination is "image" and the MIME type is an image;
                        || (destination == NativeDestination::Image && model.is_image_mime(mime_type_record))
                        // destination is "font" and the MIME type is a font;
                        || (destination == NativeDestination::Font && model.is_font_mime(mime_type_record))
                        // destination is "style" and the MIME essence is text/css;
                        || (destination == NativeDestination::Style && model.mime_essence(mime_type_record) == "text/css"@)
                        // destination is "track" and the MIME essence is text/vtt.
                        || (destination == NativeDestination::Track && model.mime_essence(mime_type_record) == "text/vtt"@);
                    if matches { true } else {
                        // 7. Return false.
                        false
                    }
                }
            },
        }
    }
}

// https://html.spec.whatwg.org/multipage/links.html#create-a-preload-key
pub(crate) open spec fn create_preload_key(request: RequestView) -> PreloadKeyView {
    // Return a new preload key with the request's URL, destination, mode,
    // and credentials mode. Key equality is itemwise equality.
    PreloadKeyView {
        url: request.url,
        destination: request.destination,
        mode: request.mode,
        credentials_mode: request.credentials_mode,
    }
}

// https://html.spec.whatwg.org/multipage/links.html#preload
pub(crate) open spec fn preload<M: LinkProcessingModel>(
    model: M,
    options: LinkProcessingOptionsView,
    process_response: Option<int>,
) -> PreloadPlan {
    // 1. If options's type doesn't match options's destination, then return.
    if !type_matches_destination(model, options.type_, options.destination) {
        PreloadPlan::TypeMismatch
    } else {
        // 2. If destination is image and source set is non-null, select the href.
        let options = if options.destination.as_native() == NativeDestination::Image {
            match options.source_set {
                Some(source_set) => LinkProcessingOptionsView {
                    href: model.select_image_source(source_set), ..options
                },
                None => options,
            }
        } else { options };
        // 3. Let request be the result of creating a link request given options.
        let request = create_link_request(model, options);
        match request {
            // 4. If request is null, then return.
            None => PreloadPlan::RequestFailure,
            Some(request) => {
                // 5. Let unsafeEndTime be 0.
                let unsafe_end_time = 0int;
                // 6. Let entry be a new preload entry with the integrity metadata.
                let entry = PreloadEntryView {
                    integrity: options.integrity,
                    response: None,
                    on_response_available: None,
                };
                // 7. Let key be the result of creating a preload key for request.
                let key = create_preload_key(request);
                // 8. If document is null, set request's initiator type to "early hint".
                let request = if options.document is None {
                    RequestView { initiator_type: Some("early hint"@), ..request }
                } else { request };
                // 9. Let controller be null.
                let controller = None;
                // 10. Let reportTiming be the following algorithm: callbacks::report_timing.
                let report_timing = PreloadCallback::ReportTiming;
                // 11. Fetch request and assign the returned controller. The body of
                // processResponseConsumeBody is callbacks::preload_response (11.1–11.6).
                // This plan describes the fetch and its callback; execution is a
                // separate effect/trace obligation, not a spec-mode network operation.
                let fetch = PreloadFetch {
                    request,
                    process_response_consume_body: PreloadCallback::ProcessResponseConsumeBody,
                };
                // 12. Let commit be the following algorithm: callbacks::commit_preload.
                let commit_callback = PreloadCallback::Commit;
                // 13. If document is null, install commit as on-document-ready;
                // otherwise, call commit with document.
                let commit = match options.document {
                    None => PreloadCommit::OnDocumentReady,
                    Some(document) => PreloadCommit::Document { document },
                };
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
) -> bool {
    // Preserve the create-link-request assertion on paths that reach it.
    let selected_href = if options.destination.as_native() == NativeDestination::Image {
        match options.source_set {
            Some(source_set) => model.select_image_source(source_set),
            None => options.href,
        }
    } else { options.href };
    &&& (type_matches_destination(model, options.type_, options.destination) ==> selected_href.len() > 0)
    &&& result == preload(model, options, process_response)
}

} // verus!
