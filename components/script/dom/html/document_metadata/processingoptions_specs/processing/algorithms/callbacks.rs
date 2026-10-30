/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The callbacks nested in WHATWG's preload algorithm.
//! https://html.spec.whatwg.org/multipage/links.html#preload
//! Numbers 10, 11.1–11.6, and 12.1–12.2 are preserved from that algorithm.

use vstd::prelude::*;
use super::super::model::*;
use super::super::types::DocumentView;
use super::super::model_trait::LinkProcessingModel;

verus! {

// 10. Let reportTiming given a Document document be to report timing for
// controller given document's relevant global object.
pub(crate) open spec fn report_timing<M: LinkProcessingModel>(
    model: M,
    controller: Option<int>,
    document: DocumentView,
) -> CallbackEffect {
    CallbackEffect::ReportTiming {
        controller,
        global: model.document_relevant_global(document),
    }
}

// 11. processResponseConsumeBody, given response and bodyBytes.
// State is supplied at invocation time: controller/entry are not frozen at their
// initial values when the callback is defined.
pub(crate) open spec fn preload_response<M: LinkProcessingModel>(
    model: M,
    state: PreloadCallbackState,
    response: ResponseView,
    body_bytes: BodyResult,
) -> CallbackResult {
    let effects = Seq::empty();
    // 11.1. If bodyBytes is a byte sequence, set response's body to those bytes as a body.
    // 11.2. Otherwise, set response to a network error.
    let response = match body_bytes {
        BodyResult::Bytes { bytes } => ResponseView { body: Some(bytes), ..response },
        _ => model.network_error(),
    };
    // 11.3. Set unsafeEndTime to the unsafe shared current time.
    let state = PreloadCallbackState { unsafe_end_time: model.unsafe_shared_current_time(), ..state };
    // 11.4. If document is non-null, call reportTiming given document.
    let effects = match state.document {
        Some(document) => effects.push(report_timing(model, state.controller, document)),
        None => effects,
    };
    // 11.5. If entry's on-response-available is null, set entry's response;
    // otherwise, call on-response-available with response.
    let (entry, effects) = match state.entry.on_response_available {
        None => (PreloadEntryView { response: Some(response), ..state.entry }, effects),
        Some(callback) => (state.entry, effects.push(CallbackEffect::InvokeResponse { callback, response })),
    };
    // 11.6. If processResponse was given, call it with response.
    let effects = match state.process_response {
        Some(callback) => effects.push(CallbackEffect::InvokeResponse { callback, response }),
        None => effects,
    };
    CallbackResult { state: PreloadCallbackState { entry, ..state }, effects }
}

// 12. Let commit be the following steps given a Document document:
pub(crate) open spec fn commit_preload<M: LinkProcessingModel>(
    model: M,
    state: PreloadCallbackState,
    document: DocumentView,
) -> Seq<CallbackEffect> {
    let effects = Seq::empty();
    // 12.1. If entry's response is non-null, call reportTiming given document.
    let effects = if state.entry.response is Some {
        effects.push(report_timing(model, state.controller, document))
    } else { effects };
    // 12.2. Set document's map of preloaded resources[key] to entry.
    effects.push(CallbackEffect::StorePreload { document, key: state.key, entry: state.entry })
}

// links.html#link-type-preload: fetch-and-process-the-linked-resource, step 6.1.
// Also the corresponding prefetch process-response callback.
pub(crate) open spec fn link_response_event(element: int, response: ResponseView) -> CallbackEffect {
    // If response is a network error, fire "error"; otherwise fire "load".
    CallbackEffect::FireEvent {
        element,
        name: if response.network_error { "error"@ } else { "load"@ },
    }
}

pub(crate) open spec fn preload_response_ensures<M: LinkProcessingModel>(
    model: M,
    before: PreloadCallbackState,
    response: ResponseView,
    body: BodyResult,
    result: CallbackResult,
) -> bool {
    result == preload_response(model, before, response, body)
}

pub(crate) open spec fn commit_preload_ensures<M: LinkProcessingModel>(
    model: M,
    state: PreloadCallbackState,
    document: DocumentView,
    effects: Seq<CallbackEffect>,
) -> bool {
    effects == commit_preload(model, state, document)
}

} // verus!
