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
pub(crate) use super::preload_machine::{preload, preload_requires};

verus! {

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
pub(crate) open spec fn type_matches_destination<M: LinkProcessingModel>(
    model: M,
    type_: Seq<char>,
    destination: DestinationView,
) -> bool {
    let destination = if destination.as_native() == NativeDestination::None {
        "fetch"@
    } else { destination.as_str() };
    super::match_preload_type::run(model, type_, destination).result
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
    if !type_matches_destination(model, options.type_, options.destination) {
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
