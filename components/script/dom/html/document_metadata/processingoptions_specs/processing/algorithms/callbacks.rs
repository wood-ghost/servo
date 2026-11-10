/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The callbacks nested in WHATWG's preload algorithm.
//! https://html.spec.whatwg.org/multipage/links.html#preload
//! Numbers 10, 11.1–11.6, and 12.1–12.2 are preserved from that algorithm.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::model::*;
use super::super::types::DocumentView;
use super::super::model_trait::LinkProcessingModel;
pub(crate) use super::preload_response::preload_response;

verus! {

// Body of the algorithm defined by preload Step 10:
// Let reportTiming given a Document document be to report timing for
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

// Derived computation helper for the separate preload_response machine.
// State is supplied at invocation time: controller/entry are not frozen at their
// initial values when the callback is defined.
pub(crate) open spec fn preload_response_computation<M: LinkProcessingModel>(
    model: M,
    state: PreloadCallbackState,
    response: ResponseView,
    body_bytes: BodyResult,
) -> CallbackResult {
    let effects = Seq::empty();
    let response = match body_bytes {
        BodyResult::Bytes { bytes } => ResponseView { body: Some(bytes), ..response },
        _ => model.network_error(),
    };
    let state = PreloadCallbackState { unsafe_end_time: model.unsafe_shared_current_time(), ..state };
    let effects = match state.document {
        Some(document) => effects.push(report_timing(model, state.controller, document)),
        None => effects,
    };
    let (entry, effects) = match state.entry.on_response_available {
        None => (PreloadEntryView { response: Some(response), ..state.entry }, effects),
        Some(callback) => (state.entry, effects.push(CallbackEffect::InvokeResponse { callback, response })),
    };
    let effects = match state.process_response {
        Some(callback) => effects.push(CallbackEffect::InvokeResponse { callback, response }),
        None => effects,
    };
    CallbackResult { state: PreloadCallbackState { entry, ..state }, effects }
}

// Derived computation helper; the canonical commit machine follows below.
pub(crate) open spec fn commit_preload_computation<M: LinkProcessingModel>(
    model: M,
    state: PreloadCallbackState,
    document: DocumentView,
) -> Seq<CallbackEffect> {
    let effects = Seq::empty();
    let effects = if state.entry.response is Some {
        effects.push(report_timing(model, state.controller, document))
    } else { effects };
    effects.push(CallbackEffect::StorePreload { document, key: state.key, entry: state.entry })
}

// links.html#link-type-preload: fetch-and-process-the-linked-resource, step 6.1.
// Also the corresponding prefetch process-response callback.
pub(crate) open spec fn link_response_event(element: int, response: ResponseView) -> CallbackEffect {
    super::link_types::preload_element::element_response_event(element, response)
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

verus! {
/// Representation of itemwise preload-key equality: URL equality is based on
/// serialization, not structural equality including a blob-entry cache.
pub(crate) type PreloadCacheKey = (Seq<char>, Seq<char>, RequestMode, CredentialsMode);

pub closed spec fn cache_key(context: PreloadCallbackState) -> PreloadCacheKey {
    (super::url::serialize_url(context.key.url, false), context.key.destination.as_str(),
        context.key.mode, context.key.credentials_mode)
}
pub closed spec fn entry_for_commit(context: PreloadCallbackState) -> PreloadEntryView { context.entry }
pub closed spec fn entry_has_response(context: PreloadCallbackState) -> bool { context.entry.response is Some }
pub closed spec fn commit_timing<M: LinkProcessingModel>(model: M, context: PreloadCallbackState, document: DocumentView) -> CallbackEffect {
    report_timing(model, context.controller, document)
}
pub closed spec fn commit_store(context: PreloadCallbackState, document: DocumentView) -> CallbackEffect {
    CallbackEffect::StorePreload { document, key: context.key, entry: context.entry }
}
pub enum CommitPreloadState { Step12_1, Step12_2, Final }
}

// https://html.spec.whatwg.org/multipage/links.html#preload
// Snapshot/inspection 2026-10-30; the algorithm defined by Step 12.
// Form: row 1, state_machine!; mutate the preload map and record ordered effects.
state_machine! {
    commit_preload_machine<M: LinkProcessingModel> {
        fields {
            // Immutable inputs / callback-invocation snapshot
            pub model: M,
            pub context: PreloadCallbackState,
            pub document: DocumentView,
            // Mutable document state. Keys use the standard's itemwise equality.
            pub map_of_preloaded_resources: Map<PreloadCacheKey, PreloadEntryView>,
            // Observation log, not a replacement for the map update.
            pub effects: Seq<CallbackEffect>,
            pub state: CommitPreloadState,
        }
        init! {
            initialize(model: M, context: PreloadCallbackState, document: DocumentView,
                map_of_preloaded_resources: Map<PreloadCacheKey, PreloadEntryView>) {
                init model = model;
                init context = context;
                init document = document;
                init map_of_preloaded_resources = map_of_preloaded_resources;
                init effects = Seq::empty();
                init state = CommitPreloadState::Step12_1;
            }
        }
        // 12.1. If entry's response is not null, then call reportTiming given document.
        transition! {
            step12_1() {
                require(pre.state == CommitPreloadState::Step12_1);
                if entry_has_response(pre.context) {
                    update effects = pre.effects.push(commit_timing(pre.model, pre.context, pre.document));
                    update state = CommitPreloadState::Step12_2;
                } else { update state = CommitPreloadState::Step12_2; }
            }
        }
        // 12.2. Set document's map of preloaded resources[key] to entry.
        transition! {
            step12_2() {
                require(pre.state == CommitPreloadState::Step12_2);
                update map_of_preloaded_resources = pre.map_of_preloaded_resources.insert(cache_key(pre.context), entry_for_commit(pre.context));
                update effects = pre.effects.push(commit_store(pre.context, pre.document));
                update state = CommitPreloadState::Final;
            }
        }
    }
}

verus! {
pub(crate) open spec fn commit_run<M: LinkProcessingModel>(model: M, context: PreloadCallbackState, document: DocumentView,
    preloads: Map<PreloadCacheKey, PreloadEntryView>) -> commit_preload_machine::State<M>
{
    let s0 = choose|s| commit_preload_machine::State::initialize(s, model, context, document, preloads);
    let s1 = choose|s| commit_preload_machine::State::step12_1(s0, s);
    choose|s| commit_preload_machine::State::step12_2(s1, s)
}

/// Effect-only adapter. commit_result proves the same effect list for every
/// initial map; this adapter does not assert that a real document's map is empty.
pub(crate) open spec fn commit_preload<M: LinkProcessingModel>(model: M, state: PreloadCallbackState, document: DocumentView) -> Seq<CallbackEffect> {
    commit_run(model, state, document, Map::empty()).effects
}

pub(crate) broadcast proof fn commit_result<M: LinkProcessingModel>(model: M, context: PreloadCallbackState, document: DocumentView,
    preloads: Map<PreloadCacheKey, PreloadEntryView>)
    ensures
        (#[trigger] commit_run(model, context, document, preloads)).state == CommitPreloadState::Final,
        commit_run(model, context, document, preloads).effects == commit_preload_computation(model, context, document),
        commit_run(model, context, document, preloads).map_of_preloaded_resources == preloads.insert(cache_key(context), context.entry),
{
    let s0 = commit_preload_machine::take_step::initialize(model, context, document, preloads);
    assert(s0 == (choose|s| commit_preload_machine::State::initialize(s, model, context, document, preloads)));
    let s1 = commit_preload_machine::take_step::step12_1(s0);
    assert(commit_preload_machine::State::step12_1(s0, s1));
    assert(s1 == (choose|s| commit_preload_machine::State::step12_1(s0, s)));
    let s2 = commit_preload_machine::take_step::step12_2(s1);
    assert(commit_preload_machine::State::step12_2(s1, s2));
    assert(s2 == (choose|s| commit_preload_machine::State::step12_2(s1, s)));
}
} // verus!
