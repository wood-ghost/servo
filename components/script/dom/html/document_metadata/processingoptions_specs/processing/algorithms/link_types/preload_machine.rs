/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/links.html#preload
//! Snapshot/inspection 2026-10-30.
//! Form: row 1, state_machine!; options/request/controller updates and shared temporaries.
//! Callback definitions are separate algorithms in ../callbacks.rs. This machine
//! produces an operation plan; observing actual native execution remains separate.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use net_traits::request::Destination as NativeDestination;
use super::super::super::model::*;
use super::super::super::types::*;
use super::super::super::model_trait::LinkProcessingModel;
use super::super::request;
use super::preload::{type_matches_destination, create_preload_key, preload_computation};

verus! {
pub closed spec fn type_matches<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> bool {
    type_matches_destination(model, options.type_, options.destination)
}
pub closed spec fn image_with_source_set(options: LinkProcessingOptionsView) -> bool {
    options.destination.as_native() == NativeDestination::Image && options.source_set is Some
}
// FINDING: the linked selection algorithm returns a source and density, while
// this step assigns its result to string href. The primitive exposes the href
// component; URL/string and density correspondence remain an explicit boundary.
pub closed spec fn select_href<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { href: model.select_image_source(options.source_set->Some_0), ..options }
}
pub closed spec fn request_obligation(options: LinkProcessingOptionsView) -> bool { request::create_link_request_requires(options) }
pub closed spec fn create_request<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> Option<RequestView> {
    request::create_link_request(model, options)
}
pub closed spec fn new_entry(options: LinkProcessingOptionsView) -> PreloadEntryView {
    PreloadEntryView { integrity: options.integrity, response: None, on_response_available: None }
}
pub closed spec fn preload_key(request: RequestView) -> PreloadKeyView { create_preload_key(request) }
pub closed spec fn document_is_null(options: LinkProcessingOptionsView) -> bool { options.document is None }
pub closed spec fn set_early_hint(request: RequestView) -> RequestView {
    RequestView { initiator_type: Some("early hint"@), ..request }
}
// Definition marker for callbacks::report_timing, evaluated with shared state at invocation.
pub closed spec fn report_timing_definition() -> PreloadCallback { PreloadCallback::ReportTiming }
// https://fetch.spec.whatwg.org/#concept-fetch
// Definition marker for callbacks::preload_response, not execution of that callback now.
pub closed spec fn fetch_request<M: LinkProcessingModel>(model: M, request: RequestView) -> int {
    model.fetch_request(request, PreloadCallback::ProcessResponseConsumeBody)
}
// Definition marker for callbacks::commit_preload, not a frozen copy of shared state.
pub closed spec fn commit_definition<M: LinkProcessingModel>(
    model: M, options: LinkProcessingOptionsView, entry: PreloadEntryView, key: PreloadKeyView,
    controller: Option<int>, unsafe_end_time: int, process_response: Option<int>,
) -> DocsAlgoView {
    model.commit_algorithm(PreloadCallbackState {
        entry, key, controller, unsafe_end_time, document: options.document, process_response,
    })
}
pub closed spec fn set_on_document_ready(options: LinkProcessingOptionsView, commit: DocsAlgoView) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { on_document_ready: Some(commit), ..options }
}
pub closed spec fn on_document_ready_action() -> PreloadCommit { PreloadCommit::OnDocumentReady }
pub closed spec fn commit_to_document(options: LinkProcessingOptionsView) -> PreloadCommit {
    PreloadCommit::Document { document: options.document->Some_0 }
}
pub enum PreloadState { Step1, Step2, Step3, Step4, Step5, Step6, Step7, Step8, Step9, Step10, Step11, Step12, Step13, Final }
pub enum PreloadOutcome { Running, TypeMismatch, RequestFailure, Fetch }
}

state_machine! {
    preload_machine<M: LinkProcessingModel> {
        fields {
            // Immutable inputs
            pub model: M,
            pub process_response: Option<int>,
            // Mutable input record, updated by Steps 2 and 13.
            pub options: LinkProcessingOptionsView,
            // temporary var used by Steps 3–11
            pub request: Option<RequestView>,
            // temporary var used by Steps 5, 11.3 and callback invocation
            pub unsafe_end_time: int,
            // temporary var used by Steps 6–13 and callback invocation
            pub entry: Option<PreloadEntryView>,
            // temporary var used by Steps 7–13
            pub key: Option<PreloadKeyView>,
            // temporary var used by Steps 9–12 and callback invocation
            pub controller: Option<int>,
            // temporary var used by Steps 10–12 and callback invocation
            pub report_timing: Option<PreloadCallback>,
            // temporary var used by Steps 12–13
            pub commit: Option<DocsAlgoView>,
            // Observation of Step 13's installation or invocation, not another HTML step.
            pub commit_action: Option<PreloadCommit>,
            pub state: PreloadState,
            pub outcome: PreloadOutcome,
        }
        init! {
            initialize(model: M, options: LinkProcessingOptionsView, process_response: Option<int>) {
                init model = model;
                init options = options;
                init process_response = process_response;
                init request = None;
                init unsafe_end_time = 0;
                init entry = None;
                init key = None;
                init controller = None;
                init report_timing = None;
                init commit = None;
                init commit_action = None;
                init state = PreloadState::Step1;
                init outcome = PreloadOutcome::Running;
            }
        }
        // 1. If options's type doesn't match options's destination, then return.
        transition! {
            step1() {
                require(pre.state == PreloadState::Step1);
                if !type_matches(pre.model, pre.options) {
                    update state = PreloadState::Final;
                    update outcome = PreloadOutcome::TypeMismatch;
                } else { update state = PreloadState::Step2; }
            }
        }
        // 2. If options's destination is "image" and options's source set is not null, then set options's href to the result of selecting an image source from options's source set.
        transition! {
            step2() {
                require(pre.state == PreloadState::Step2);
                if image_with_source_set(pre.options) {
                    update options = select_href(pre.model, pre.options);
                    update state = PreloadState::Step3;
                } else { update state = PreloadState::Step3; }
            }
        }
        // 3. Let request be the result of creating a link request given options.
        transition! {
            step3() {
                require(pre.state == PreloadState::Step3);
                // INHERITED from create-a-link-request Step 1, after Step 2's href update.
                require(request_obligation(pre.options));
                update request = create_request(pre.model, pre.options);
                update state = PreloadState::Step4;
            }
        }
        // 4. If request is null, then return.
        transition! {
            step4() {
                require(pre.state == PreloadState::Step4);
                if pre.request is None {
                    update state = PreloadState::Final;
                    update outcome = PreloadOutcome::RequestFailure;
                } else { update state = PreloadState::Step5; }
            }
        }
        // 5. Let unsafeEndTime be 0.
        transition! {
            step5() {
                require(pre.state == PreloadState::Step5);
                update unsafe_end_time = 0;
                update state = PreloadState::Step6;
            }
        }
        // 6. Let entry be a new preload entry whose integrity metadata is options's integrity.
        transition! {
            step6() {
                require(pre.state == PreloadState::Step6);
                update entry = Some(new_entry(pre.options));
                update state = PreloadState::Step7;
            }
        }
        // 7. Let key be the result of creating a preload key given request.
        transition! {
            step7() {
                require(pre.state == PreloadState::Step7);
                update key = Some(preload_key(pre.request->Some_0));
                update state = PreloadState::Step8;
            }
        }
        // 8. If options's document is null, then set request's initiator type to "early hint".
        transition! {
            step8() {
                require(pre.state == PreloadState::Step8);
                if document_is_null(pre.options) {
                    update request = Some(set_early_hint(pre.request->Some_0));
                    update state = PreloadState::Step9;
                } else { update state = PreloadState::Step9; }
            }
        }
        // 9. Let controller be null.
        transition! {
            step9() {
                require(pre.state == PreloadState::Step9);
                update controller = None;
                update state = PreloadState::Step10;
            }
        }
        // 10. Let reportTiming given a Document document be to report timing for controller given document's relevant global object.
        transition! {
            step10() {
                require(pre.state == PreloadState::Step10);
                update report_timing = Some(report_timing_definition());
                update state = PreloadState::Step11;
            }
        }
        // 11. Set controller to the result of fetching request, with processResponseConsumeBody set to the following steps given a response response and null, failure, or a byte sequence bodyBytes:
        transition! {
            step11() {
                require(pre.state == PreloadState::Step11);
                update controller = Some(fetch_request(pre.model, pre.request->Some_0));
                update state = PreloadState::Step12;
            }
        }
        // 12. Let commit be the following steps given a Document document:
        transition! {
            step12() {
                require(pre.state == PreloadState::Step12);
                update commit = Some(commit_definition(pre.model, pre.options, pre.entry->Some_0,
                    pre.key->Some_0, pre.controller, pre.unsafe_end_time, pre.process_response));
                update state = PreloadState::Step13;
            }
        }
        // 13. If options's document is null, then set options's on document ready to commit. Otherwise, call commit with options's document.
        transition! {
            step13() {
                require(pre.state == PreloadState::Step13);
                if document_is_null(pre.options) {
                    update options = set_on_document_ready(pre.options, pre.commit->Some_0);
                    update commit_action = Some(on_document_ready_action());
                    update state = PreloadState::Final;
                    update outcome = PreloadOutcome::Fetch;
                } else {
                    update commit_action = Some(commit_to_document(pre.options));
                    update state = PreloadState::Final;
                    update outcome = PreloadOutcome::Fetch;
                }
            }
        }
    }
}

verus! {
pub(crate) open spec fn preload_requires<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> bool {
    // INHERITED from create-a-link-request Step 1 on the path reaching that call.
    type_matches_destination(model, options.type_, options.destination) ==> {
        let selected_href = if options.destination.as_native() == NativeDestination::Image {
            match options.source_set {
                Some(source_set) => model.select_image_source(source_set), None => options.href,
            }
        } else { options.href };
        selected_href.len() > 0
    }
}

pub(crate) open spec fn preload_run<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView, process_response: Option<int>)
    -> preload_machine::State<M>
    recommends preload_requires(model, options),
{
    let s0 = choose|s| preload_machine::State::initialize(s, model, options, process_response);
    let s1 = choose|s| preload_machine::State::step1(s0, s);
    if s1.state == PreloadState::Final { s1 } else {
        let s2 = choose|s| preload_machine::State::step2(s1, s);
        let s3 = choose|s| preload_machine::State::step3(s2, s);
        let s4 = choose|s| preload_machine::State::step4(s3, s);
        if s4.state == PreloadState::Final { s4 } else {
            let s5 = choose|s| preload_machine::State::step5(s4, s);
            let s6 = choose|s| preload_machine::State::step6(s5, s);
            let s7 = choose|s| preload_machine::State::step7(s6, s);
            let s8 = choose|s| preload_machine::State::step8(s7, s);
            let s9 = choose|s| preload_machine::State::step9(s8, s);
            let s10 = choose|s| preload_machine::State::step10(s9, s);
            let s11 = choose|s| preload_machine::State::step11(s10, s);
            let s12 = choose|s| preload_machine::State::step12(s11, s);
            choose|s| preload_machine::State::step13(s12, s)
        }
    }
}

/// Observation projection from a completed standard run; not another algorithm step.
pub(crate) open spec fn preload_plan<M: LinkProcessingModel>(s: preload_machine::State<M>) -> PreloadPlan
    recommends s.state == PreloadState::Final,
{
    match s.outcome {
        PreloadOutcome::TypeMismatch => PreloadPlan::TypeMismatch,
        PreloadOutcome::RequestFailure => PreloadPlan::RequestFailure,
        PreloadOutcome::Fetch => PreloadPlan::Fetch {
            options: s.options, request: s.request->Some_0, initial_unsafe_end_time: s.unsafe_end_time,
            entry: s.entry->Some_0, key: s.key->Some_0, initial_controller: None,
            process_response: s.process_response, report_timing: s.report_timing->Some_0,
            fetch: PreloadFetch { request: s.request->Some_0,
                process_response_consume_body: PreloadCallback::ProcessResponseConsumeBody,
                controller: s.controller->Some_0 },
            commit_callback: PreloadCallback::Commit, commit_algorithm: s.commit->Some_0,
            commit: s.commit_action->Some_0,
        },
        // Unreachable for a completed run, as proved by preload_result.
        PreloadOutcome::Running => PreloadPlan::TypeMismatch,
    }
}

pub(crate) open spec fn preload<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView, process_response: Option<int>) -> PreloadPlan
    recommends preload_requires(model, options),
{
    preload_plan(preload_run(model, options, process_response))
}

pub(crate) broadcast proof fn preload_result<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView, process_response: Option<int>)
    requires preload_requires(model, options),
    ensures
        (#[trigger] preload_run(model, options, process_response)).state == PreloadState::Final,
        preload_run(model, options, process_response).outcome != PreloadOutcome::Running,
        preload_plan(preload_run(model, options, process_response)) == preload_computation(model, options, process_response),
{
    let s0 = preload_machine::take_step::initialize(model, options, process_response);
    assert(s0 == (choose|s| preload_machine::State::initialize(s, model, options, process_response)));
    let s1 = preload_machine::take_step::step1(s0);
    assert(preload_machine::State::step1(s0, s1));
    assert(s1 == (choose|s| preload_machine::State::step1(s0, s)));
    if s1.state != PreloadState::Final {
        let s2 = preload_machine::take_step::step2(s1);
        assert(preload_machine::State::step2(s1, s2));
        assert(s2 == (choose|s| preload_machine::State::step2(s1, s)));
        let s3 = preload_machine::take_step::step3(s2);
        assert(preload_machine::State::step3(s2, s3));
        assert(s3 == (choose|s| preload_machine::State::step3(s2, s)));
        let s4 = preload_machine::take_step::step4(s3);
        assert(preload_machine::State::step4(s3, s4));
        assert(s4 == (choose|s| preload_machine::State::step4(s3, s)));
        if s4.state != PreloadState::Final {
            let s5 = preload_machine::take_step::step5(s4);
            assert(preload_machine::State::step5(s4, s5));
            assert(s5 == (choose|s| preload_machine::State::step5(s4, s)));
            let s6 = preload_machine::take_step::step6(s5);
            assert(preload_machine::State::step6(s5, s6));
            assert(s6 == (choose|s| preload_machine::State::step6(s5, s)));
            let s7 = preload_machine::take_step::step7(s6);
            assert(preload_machine::State::step7(s6, s7));
            assert(s7 == (choose|s| preload_machine::State::step7(s6, s)));
            let s8 = preload_machine::take_step::step8(s7);
            assert(preload_machine::State::step8(s7, s8));
            assert(s8 == (choose|s| preload_machine::State::step8(s7, s)));
            let s9 = preload_machine::take_step::step9(s8);
            assert(preload_machine::State::step9(s8, s9));
            assert(s9 == (choose|s| preload_machine::State::step9(s8, s)));
            let s10 = preload_machine::take_step::step10(s9);
            assert(preload_machine::State::step10(s9, s10));
            assert(s10 == (choose|s| preload_machine::State::step10(s9, s)));
            let s11 = preload_machine::take_step::step11(s10);
            assert(preload_machine::State::step11(s10, s11));
            assert(s11 == (choose|s| preload_machine::State::step11(s10, s)));
            let s12 = preload_machine::take_step::step12(s11);
            assert(preload_machine::State::step12(s11, s12));
            assert(s12 == (choose|s| preload_machine::State::step12(s11, s)));
            let s13 = preload_machine::take_step::step13(s12);
            assert(preload_machine::State::step13(s12, s13));
            assert(s13 == (choose|s| preload_machine::State::step13(s12, s)));
        }
    }
}
} // verus!
