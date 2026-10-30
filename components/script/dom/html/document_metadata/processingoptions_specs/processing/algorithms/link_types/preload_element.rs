/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/links.html#link-type-preload:fetch-and-process-the-linked-resource-2
//! Snapshot/inspection 2026-10-30.
//! Form: row 1, state_machine!; options/destination temporaries and destination update.
//! Source-set update and element option construction are explicitly supplied
//! linked observations. This is a reference operation plan, not a native trace.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::super::model::*;
use super::super::super::types::DestinationView;
use super::super::super::model_trait::LinkProcessingModel;
use super::preload::{translate_a_preload_destination, preload, preload_requires};

verus! {
// https://html.spec.whatwg.org/multipage/images.html#update-the-source-set
// TRUSTED(scope): not verified yet; remove when element source-set-update observation correspondence is verified
pub closed spec fn update_source_set_observed(observations: LinkElementPreloadObservations) -> bool { true }
// https://html.spec.whatwg.org/multipage/semantics.html#create-link-options-from-element
// TRUSTED(scope): not verified yet; remove when create-link-options-from-element observation correspondence is verified
pub closed spec fn create_link_options_observed(observations: LinkElementPreloadObservations) -> LinkProcessingOptionsView {
    observations.options_after_source_update
}
pub closed spec fn translate_as_keyword(observations: LinkElementPreloadObservations) -> Option<DestinationView> {
    match observations.as_keyword { None => None, Some(keyword) => translate_a_preload_destination(keyword) }
}
pub closed spec fn set_destination(options: LinkProcessingOptionsView, destination: DestinationView) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { destination, ..options }
}
pub closed spec fn preload_obligation<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> bool { preload_requires(model, options) }
pub closed spec fn preload_with_response<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView, observations: LinkElementPreloadObservations) -> PreloadPlan {
    preload(model, options, Some(observations.response_callback))
}
pub enum PreloadElementState { Step1, Step2, Step3, Step4, Step5, Step6, Final }
}

state_machine! {
    preload_element_machine<M: LinkProcessingModel> {
        fields {
            // Immutable inputs and supplied linked-operation observations
            pub model: M,
            pub observations: LinkElementPreloadObservations,
            // temporary var used by Steps 2–6
            pub options: Option<LinkProcessingOptionsView>,
            // temporary var used by Steps 3–5
            pub destination: Option<DestinationView>,
            pub source_set_updated: bool,
            pub state: PreloadElementState,
            pub result: Option<PreloadPlan>,
        }
        init! {
            initialize(model: M, observations: LinkElementPreloadObservations) {
                init model = model;
                init observations = observations;
                init options = None;
                init destination = None;
                init source_set_updated = false;
                init state = PreloadElementState::Step1;
                init result = None;
            }
        }
        // 1. Update the source set for el.
        transition! {
            step1() {
                require(pre.state == PreloadElementState::Step1);
                update source_set_updated = update_source_set_observed(pre.observations);
                update state = PreloadElementState::Step2;
            }
        }
        // 2. Let options be the result of creating link options from el.
        transition! {
            step2() {
                require(pre.state == PreloadElementState::Step2);
                update options = Some(create_link_options_observed(pre.observations));
                update state = PreloadElementState::Step3;
            }
        }
        // 3. Let destination be the result of translating the keyword representing the state of el's as attribute.
        transition! {
            step3() {
                require(pre.state == PreloadElementState::Step3);
                update destination = translate_as_keyword(pre.observations);
                update state = PreloadElementState::Step4;
            }
        }
        // 4. If destination is null, then return.
        transition! {
            step4() {
                require(pre.state == PreloadElementState::Step4);
                if pre.destination is None {
                    update state = PreloadElementState::Final;
                    update result = None;
                } else { update state = PreloadElementState::Step5; }
            }
        }
        // 5. Set options's destination to destination.
        transition! {
            step5() {
                require(pre.state == PreloadElementState::Step5);
                update options = Some(set_destination(pre.options->Some_0, pre.destination->Some_0));
                update state = PreloadElementState::Step6;
            }
        }
        // 6. Preload options, with the following steps given a response response:
        transition! {
            step6() {
                require(pre.state == PreloadElementState::Step6);
                // INHERITED from preload and create-link-request, after destination assignment.
                require(preload_obligation(pre.model, pre.options->Some_0));
                update state = PreloadElementState::Final;
                update result = Some(preload_with_response(pre.model, pre.options->Some_0, pre.observations));
            }
        }
    }
}

verus! {
pub(crate) open spec fn element_preload_requires<M: LinkProcessingModel>(model: M, observations: LinkElementPreloadObservations) -> bool {
    match observations.as_keyword {
        None => true,
        Some(keyword) => match translate_a_preload_destination(keyword) {
            None => true,
            Some(destination) => preload_requires(model, LinkProcessingOptionsView { destination, ..observations.options_after_source_update }),
        },
    }
}

pub(crate) open spec fn element_preload_run<M: LinkProcessingModel>(model: M, observations: LinkElementPreloadObservations) -> preload_element_machine::State<M>
    recommends element_preload_requires(model, observations),
{
    let s0 = choose|s| preload_element_machine::State::initialize(s, model, observations);
    let s1 = choose|s| preload_element_machine::State::step1(s0, s);
    let s2 = choose|s| preload_element_machine::State::step2(s1, s);
    let s3 = choose|s| preload_element_machine::State::step3(s2, s);
    let s4 = choose|s| preload_element_machine::State::step4(s3, s);
    if s4.state == PreloadElementState::Final { s4 } else {
        let s5 = choose|s| preload_element_machine::State::step5(s4, s);
        choose|s| preload_element_machine::State::step6(s5, s)
    }
}

pub(crate) open spec fn element_preload_plan<M: LinkProcessingModel>(model: M, observations: LinkElementPreloadObservations) -> LinkElementPreloadPlan
    recommends element_preload_requires(model, observations),
{
    let result = element_preload_run(model, observations);
    LinkElementPreloadPlan { source_set_updated: result.source_set_updated, preload: result.result }
}

pub(crate) proof fn element_preload_result<M: LinkProcessingModel>(model: M, observations: LinkElementPreloadObservations)
    requires element_preload_requires(model, observations),
    ensures
        element_preload_run(model, observations).state == PreloadElementState::Final,
        element_preload_plan(model, observations).source_set_updated,
        element_preload_plan(model, observations).preload == match observations.as_keyword {
            None => None,
            Some(keyword) => match translate_a_preload_destination(keyword) {
                None => None,
                Some(destination) => Some(preload(model, LinkProcessingOptionsView { destination, ..observations.options_after_source_update }, Some(observations.response_callback))),
            },
        },
{
    let s0 = preload_element_machine::take_step::initialize(model, observations);
    assert(s0 == (choose|s| preload_element_machine::State::initialize(s, model, observations)));
    let s1 = preload_element_machine::take_step::step1(s0);
    assert(preload_element_machine::State::step1(s0, s1));
    assert(s1 == (choose|s| preload_element_machine::State::step1(s0, s)));
    let s2 = preload_element_machine::take_step::step2(s1);
    assert(preload_element_machine::State::step2(s1, s2));
    assert(s2 == (choose|s| preload_element_machine::State::step2(s1, s)));
    let s3 = preload_element_machine::take_step::step3(s2);
    assert(preload_element_machine::State::step3(s2, s3));
    assert(s3 == (choose|s| preload_element_machine::State::step3(s2, s)));
    let s4 = preload_element_machine::take_step::step4(s3);
    assert(preload_element_machine::State::step4(s3, s4));
    assert(s4 == (choose|s| preload_element_machine::State::step4(s3, s)));
    if s4.state != PreloadElementState::Final {
        let s5 = preload_element_machine::take_step::step5(s4);
        assert(preload_element_machine::State::step5(s4, s5));
        assert(s5 == (choose|s| preload_element_machine::State::step5(s4, s)));
        let s6 = preload_element_machine::take_step::step6(s5);
        assert(preload_element_machine::State::step6(s5, s6));
        assert(s6 == (choose|s| preload_element_machine::State::step6(s5, s)));
    }
}

// https://html.spec.whatwg.org/multipage/links.html#link-type-preload:fetch-and-process-the-linked-resource-2
// Snapshot/inspection 2026-10-30. Direct specification of the single callback
// effect; no temporary is carried between steps (row 3).
pub(crate) open spec fn element_response_event(element: int, response: ResponseView) -> CallbackEffect {
    // 6.1. If response is a network error, fire an event named error at el. Otherwise, fire an event named load at el.
    if response.network_error {
        CallbackEffect::FireEvent { element, name: "error"@ }
    } else {
        CallbackEffect::FireEvent { element, name: "load"@ }
    }
}
} // verus!
