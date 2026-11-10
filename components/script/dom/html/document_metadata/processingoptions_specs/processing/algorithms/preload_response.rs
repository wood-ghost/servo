/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/links.html#preload
//! Snapshot/inspection 2026-10-30, callback Steps 11.1–11.6.
//! Form: row 1, state_machine!; response/entry/time updates and ordered effects.
//! The callback receives shared state at invocation, not frozen definition-time values.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::model::*;
use super::super::model_trait::LinkProcessingModel;
use super::callbacks::{report_timing, preload_response_computation};

verus! {
pub closed spec fn initial_entry(context: PreloadCallbackState) -> PreloadEntryView { context.entry }
pub closed spec fn initial_time(context: PreloadCallbackState) -> int { context.unsafe_end_time }
pub closed spec fn body_is_bytes(body_bytes: BodyResult) -> bool { body_bytes is Bytes }
// https://fetch.spec.whatwg.org/#byte-sequence-as-a-body
// Projection of a Fetch body onto the bytes observed by this model. Stream,
// source and length correspondence belong to the separate Fetch body model.
// TRUSTED(scope): not verified yet; remove when Fetch byte-sequence-as-a-body correspondence is verified
pub closed spec fn bytes_as_body(body_bytes: BodyResult) -> Option<Seq<u8>> { Some(body_bytes->Bytes_bytes) }
pub closed spec fn set_response_body(response: ResponseView, body_bytes: BodyResult) -> ResponseView {
    ResponseView { body: bytes_as_body(body_bytes), ..response }
}
// https://fetch.spec.whatwg.org/#concept-network-error
pub closed spec fn network_error<M: LinkProcessingModel>(model: M) -> ResponseView { model.network_error() }
// https://w3c.github.io/hr-time/#dfn-unsafe-shared-current-time
pub closed spec fn unsafe_shared_current_time<M: LinkProcessingModel>(model: M) -> int { model.unsafe_shared_current_time() }
pub closed spec fn document_is_not_null(context: PreloadCallbackState) -> bool { context.document is Some }
pub closed spec fn report_timing_effect<M: LinkProcessingModel>(model: M, context: PreloadCallbackState) -> CallbackEffect {
    report_timing(model, context.controller, context.document->Some_0)
}
pub closed spec fn no_on_response_available(entry: PreloadEntryView) -> bool { entry.on_response_available is None }
pub closed spec fn set_entry_response(entry: PreloadEntryView, response: ResponseView) -> PreloadEntryView {
    PreloadEntryView { response: Some(response), ..entry }
}
pub closed spec fn call_on_response_available(entry: PreloadEntryView, response: ResponseView) -> CallbackEffect {
    CallbackEffect::InvokeResponse { callback: entry.on_response_available->Some_0, response }
}
pub closed spec fn process_response_is_given(context: PreloadCallbackState) -> bool { context.process_response is Some }
pub closed spec fn call_process_response(context: PreloadCallbackState, response: ResponseView) -> CallbackEffect {
    CallbackEffect::InvokeResponse { callback: context.process_response->Some_0, response }
}
pub enum PreloadResponseState { Step11_1, Step11_2, Step11_3, Step11_4, Step11_5, Step11_6, Final }
}

state_machine! {
    preload_response_machine<M: LinkProcessingModel> {
        fields {
            // Immutable inputs / invocation context
            pub model: M,
            pub context: PreloadCallbackState,
            pub body_bytes: BodyResult,
            // Mutable argument/shared variables used by Steps 11.1–11.6.
            pub response: ResponseView,
            pub entry: PreloadEntryView,
            pub unsafe_end_time: int,
            // Observation log of the algorithm's calls, in order.
            pub effects: Seq<CallbackEffect>,
            pub state: PreloadResponseState,
        }
        init! {
            initialize(model: M, context: PreloadCallbackState, response: ResponseView, body_bytes: BodyResult) {
                init model = model;
                init context = context;
                init body_bytes = body_bytes;
                init response = response;
                init entry = initial_entry(context);
                init unsafe_end_time = initial_time(context);
                init effects = Seq::empty();
                init state = PreloadResponseState::Step11_1;
            }
        }
        // 11.1. If bodyBytes is a byte sequence, then set response's body to bodyBytes as a body.
        transition! {
            step11_1() {
                require(pre.state == PreloadResponseState::Step11_1);
                if body_is_bytes(pre.body_bytes) {
                    update response = set_response_body(pre.response, pre.body_bytes);
                    update state = PreloadResponseState::Step11_3;
                } else { update state = PreloadResponseState::Step11_2; }
            }
        }
        // 11.2. Otherwise, set response to a network error.
        transition! {
            step11_2() {
                require(pre.state == PreloadResponseState::Step11_2);
                update response = network_error(pre.model);
                update state = PreloadResponseState::Step11_3;
            }
        }
        // 11.3. Set unsafeEndTime to the unsafe shared current time.
        transition! {
            step11_3() {
                require(pre.state == PreloadResponseState::Step11_3);
                update unsafe_end_time = unsafe_shared_current_time(pre.model);
                update state = PreloadResponseState::Step11_4;
            }
        }
        // 11.4. If options's document is not null, then call reportTiming given options's document.
        transition! {
            step11_4() {
                require(pre.state == PreloadResponseState::Step11_4);
                if document_is_not_null(pre.context) {
                    update effects = pre.effects.push(report_timing_effect(pre.model, pre.context));
                    update state = PreloadResponseState::Step11_5;
                } else { update state = PreloadResponseState::Step11_5; }
            }
        }
        // 11.5. If entry's on response available is null, then set entry's response to response; otherwise call entry's on response available given response.
        transition! {
            step11_5() {
                require(pre.state == PreloadResponseState::Step11_5);
                if no_on_response_available(pre.entry) {
                    update entry = set_entry_response(pre.entry, pre.response);
                    update state = PreloadResponseState::Step11_6;
                } else {
                    update effects = pre.effects.push(call_on_response_available(pre.entry, pre.response));
                    update state = PreloadResponseState::Step11_6;
                }
            }
        }
        // 11.6. If processResponse is given, then call processResponse with response.
        transition! {
            step11_6() {
                require(pre.state == PreloadResponseState::Step11_6);
                if process_response_is_given(pre.context) {
                    update effects = pre.effects.push(call_process_response(pre.context, pre.response));
                    update state = PreloadResponseState::Final;
                } else { update state = PreloadResponseState::Final; }
            }
        }
    }
}

verus! {
pub(crate) open spec fn response_run<M: LinkProcessingModel>(model: M, context: PreloadCallbackState, response: ResponseView, body_bytes: BodyResult)
    -> preload_response_machine::State<M>
{
    let s0 = choose|s| preload_response_machine::State::initialize(s, model, context, response, body_bytes);
    let s1 = choose|s| preload_response_machine::State::step11_1(s0, s);
    let s2 = if s1.state == PreloadResponseState::Step11_2 {
        choose|s| preload_response_machine::State::step11_2(s1, s)
    } else { s1 };
    let s3 = choose|s| preload_response_machine::State::step11_3(s2, s);
    let s4 = choose|s| preload_response_machine::State::step11_4(s3, s);
    let s5 = choose|s| preload_response_machine::State::step11_5(s4, s);
    choose|s| preload_response_machine::State::step11_6(s5, s)
}

pub(crate) open spec fn preload_response<M: LinkProcessingModel>(model: M, state: PreloadCallbackState, response: ResponseView, body_bytes: BodyResult) -> CallbackResult {
    let finished = response_run(model, state, response, body_bytes);
    CallbackResult {
        state: PreloadCallbackState { entry: finished.entry, unsafe_end_time: finished.unsafe_end_time, ..state },
        effects: finished.effects,
    }
}

pub(crate) broadcast proof fn response_result<M: LinkProcessingModel>(model: M, context: PreloadCallbackState, response: ResponseView, body_bytes: BodyResult)
    ensures
        (#[trigger] response_run(model, context, response, body_bytes)).state == PreloadResponseState::Final,
        preload_response(model, context, response, body_bytes) == preload_response_computation(model, context, response, body_bytes),
{
    let s0 = preload_response_machine::take_step::initialize(model, context, response, body_bytes);
    assert(s0 == (choose|s| preload_response_machine::State::initialize(s, model, context, response, body_bytes)));
    let s1 = preload_response_machine::take_step::step11_1(s0);
    assert(preload_response_machine::State::step11_1(s0, s1));
    assert(s1 == (choose|s| preload_response_machine::State::step11_1(s0, s)));
    let s2 = if s1.state == PreloadResponseState::Step11_2 {
        let s2 = preload_response_machine::take_step::step11_2(s1);
        assert(preload_response_machine::State::step11_2(s1, s2));
        assert(s2 == (choose|s| preload_response_machine::State::step11_2(s1, s)));
        s2
    } else { s1 };
    let s3 = preload_response_machine::take_step::step11_3(s2);
    assert(preload_response_machine::State::step11_3(s2, s3));
    assert(s3 == (choose|s| preload_response_machine::State::step11_3(s2, s)));
    let s4 = preload_response_machine::take_step::step11_4(s3);
    assert(preload_response_machine::State::step11_4(s3, s4));
    assert(s4 == (choose|s| preload_response_machine::State::step11_4(s3, s)));
    let s5 = preload_response_machine::take_step::step11_5(s4);
    assert(preload_response_machine::State::step11_5(s4, s5));
    assert(s5 == (choose|s| preload_response_machine::State::step11_5(s4, s)));
    let s6 = preload_response_machine::take_step::step11_6(s5);
    assert(preload_response_machine::State::step11_6(s5, s6));
    assert(s6 == (choose|s| preload_response_machine::State::step11_6(s5, s)));
}
} // verus!
