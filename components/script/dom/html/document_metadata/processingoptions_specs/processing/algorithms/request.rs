/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/semantics.html#create-a-link-request
//! Snapshot/inspection 2026-10-30.
//! Form: row 1, state_machine!; URL/request temporaries and request-field updates.
//! Native comments retain older numbering; executable expressions are unchanged.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::model::*;
use super::super::types::ServoUrlView;
use super::super::model_trait::LinkProcessingModel;
use super::cors::create_a_potential_cors_request;

verus! {

// Closed field adapters support generated public relations without widening
// the invariant-bearing standard views. Each setter changes precisely one field.
pub closed spec fn href(options: LinkProcessingOptionsView) -> Seq<char> { options.href }
// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#encoding-parsing-a-url
pub closed spec fn encoding_parse<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> Option<ServoUrlView> {
    model.encoding_parse_url(options.href, options.base_url)
}
// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#create-a-potential-cors-request
pub closed spec fn potential_cors_request<M: LinkProcessingModel>(model: M, url: ServoUrlView, options: LinkProcessingOptionsView) -> RequestView {
    create_a_potential_cors_request(model, url, options.destination, options.crossorigin, false)
}
pub closed spec fn set_policy_container(request: RequestView, options: LinkProcessingOptionsView) -> RequestView {
    RequestView { policy_container: Some(options.policy_container), ..request }
}
pub closed spec fn set_integrity_metadata(request: RequestView, options: LinkProcessingOptionsView) -> RequestView {
    RequestView { integrity: options.integrity, ..request }
}
pub closed spec fn set_cryptographic_nonce_metadata(request: RequestView, options: LinkProcessingOptionsView) -> RequestView {
    RequestView { nonce: options.cryptographic_nonce_metadata, ..request }
}
pub closed spec fn set_referrer_policy(request: RequestView, options: LinkProcessingOptionsView) -> RequestView {
    RequestView { referrer_policy: options.referrer_policy, ..request }
}
pub closed spec fn set_client(request: RequestView, options: LinkProcessingOptionsView) -> RequestView {
    RequestView { client: Some(options.environment), ..request }
}
pub closed spec fn set_priority(request: RequestView, options: LinkProcessingOptionsView) -> RequestView {
    RequestView { priority: options.fetch_priority, ..request }
}
pub enum CreateLinkRequestState { Step1, Step2, Step3, Step4, Step5, Step6, Step7, Step8, Step9, Step10, Step11, Final }
}

state_machine! {
    create_link_request_machine<M: LinkProcessingModel> {
        fields {
            // Immutable inputs
            pub model: M,
            pub options: LinkProcessingOptionsView,
            // temporary var used by Steps 2–4
            pub url: Option<ServoUrlView>,
            // temporary var used by Steps 4–11; None before request creation
            pub request: Option<RequestView>,
            pub state: CreateLinkRequestState,
            pub result: Option<RequestView>,
        }
        init! {
            initialize(model: M, options: LinkProcessingOptionsView) {
                init model = model;
                init options = options;
                init url = None;
                init request = None;
                init state = CreateLinkRequestState::Step1;
                init result = None;
            }
        }
        // 1. Assert: options's href is not the empty string.
        transition! {
            step1() {
                require(pre.state == CreateLinkRequestState::Step1);
                require(href(pre.options).len() > 0);
                update state = CreateLinkRequestState::Step2;
            }
        }
        // 2. Let url be the result of encoding-parsing a URL given options's href, relative to options's base URL.
        transition! {
            step2() {
                require(pre.state == CreateLinkRequestState::Step2);
                update url = encoding_parse(pre.model, pre.options);
                update state = CreateLinkRequestState::Step3;
            }
        }
        // 3. If url is failure, then return null.
        transition! {
            step3() {
                require(pre.state == CreateLinkRequestState::Step3);
                if pre.url is None {
                    update state = CreateLinkRequestState::Final;
                    update result = None;
                } else {
                    update state = CreateLinkRequestState::Step4;
                }
            }
        }
        // 4. Let request be the result of creating a potential-CORS request given url, options's destination, and options's crossorigin.
        transition! {
            step4() {
                require(pre.state == CreateLinkRequestState::Step4);
                update request = Some(potential_cors_request(pre.model, pre.url->Some_0, pre.options));
                update state = CreateLinkRequestState::Step5;
            }
        }
        // 5. Set request's policy container to options's policy container.
        transition! {
            step5() {
                require(pre.state == CreateLinkRequestState::Step5);
                update request = Some(set_policy_container(pre.request->Some_0, pre.options));
                update state = CreateLinkRequestState::Step6;
            }
        }
        // 6. Set request's integrity metadata to options's integrity.
        transition! {
            step6() {
                require(pre.state == CreateLinkRequestState::Step6);
                update request = Some(set_integrity_metadata(pre.request->Some_0, pre.options));
                update state = CreateLinkRequestState::Step7;
            }
        }
        // 7. Set request's cryptographic nonce metadata to options's cryptographic nonce metadata.
        transition! {
            step7() {
                require(pre.state == CreateLinkRequestState::Step7);
                update request = Some(set_cryptographic_nonce_metadata(pre.request->Some_0, pre.options));
                update state = CreateLinkRequestState::Step8;
            }
        }
        // 8. Set request's referrer policy to options's referrer policy.
        transition! {
            step8() {
                require(pre.state == CreateLinkRequestState::Step8);
                update request = Some(set_referrer_policy(pre.request->Some_0, pre.options));
                update state = CreateLinkRequestState::Step9;
            }
        }
        // 9. Set request's client to options's environment.
        transition! {
            step9() {
                require(pre.state == CreateLinkRequestState::Step9);
                update request = Some(set_client(pre.request->Some_0, pre.options));
                update state = CreateLinkRequestState::Step10;
            }
        }
        // 10. Set request's priority to options's fetch priority.
        transition! {
            step10() {
                require(pre.state == CreateLinkRequestState::Step10);
                update request = Some(set_priority(pre.request->Some_0, pre.options));
                update state = CreateLinkRequestState::Step11;
            }
        }
        // 11. Return request.
        transition! {
            step11() {
                require(pre.state == CreateLinkRequestState::Step11);
                update state = CreateLinkRequestState::Final;
                update result = pre.request;
            }
        }
    }
}

verus! {

pub(crate) open spec fn create_link_request_requires(options: LinkProcessingOptionsView) -> bool {
    options.href.len() > 0
}

/// Scheduling infrastructure: choose only successors satisfying the generated
/// relations, stop after the early return, otherwise visit each remaining step.
pub(crate) open spec fn request_run<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView)
    -> create_link_request_machine::State<M>
    recommends create_link_request_requires(options),
{
    let s0 = choose|s| create_link_request_machine::State::initialize(s, model, options);
    let s1 = choose|s| create_link_request_machine::State::step1(s0, s);
    let s2 = choose|s| create_link_request_machine::State::step2(s1, s);
    let s3 = choose|s| create_link_request_machine::State::step3(s2, s);
    if s3.state == CreateLinkRequestState::Final { s3 } else {
        let s4 = choose|s| create_link_request_machine::State::step4(s3, s);
        let s5 = choose|s| create_link_request_machine::State::step5(s4, s);
        let s6 = choose|s| create_link_request_machine::State::step6(s5, s);
        let s7 = choose|s| create_link_request_machine::State::step7(s6, s);
        let s8 = choose|s| create_link_request_machine::State::step8(s7, s);
        let s9 = choose|s| create_link_request_machine::State::step9(s8, s);
        let s10 = choose|s| create_link_request_machine::State::step10(s9, s);
        choose|s| create_link_request_machine::State::step11(s10, s)
    }
}

pub(crate) open spec fn create_link_request<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView)
    -> Option<RequestView>
    recommends create_link_request_requires(options),
{
    request_run(model, options).result
}

pub(crate) open spec fn create_link_request_ensures<M: LinkProcessingModel>(
    model: M, options: LinkProcessingOptionsView, result: Option<RequestView>,
) -> bool
    recommends create_link_request_requires(options),
{
    result == create_link_request(model, options)
}

/// Derived computation law, not a second definition of the standard algorithm.
pub(crate) broadcast proof fn request_result<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView)
    requires create_link_request_requires(options),
    ensures
        (#[trigger] request_run(model, options)).state == CreateLinkRequestState::Final,
        request_run(model, options).options == options,
        request_run(model, options).result == match model.encoding_parse_url(options.href, options.base_url) {
            None => None,
            Some(url) => Some(RequestView {
                policy_container: Some(options.policy_container), integrity: options.integrity,
                nonce: options.cryptographic_nonce_metadata, referrer_policy: options.referrer_policy,
                client: Some(options.environment), priority: options.fetch_priority,
                ..create_a_potential_cors_request(model, url, options.destination, options.crossorigin, false)
            }),
        },
{
    let s0 = create_link_request_machine::take_step::initialize(model, options);
    assert(s0 == (choose|s| create_link_request_machine::State::initialize(s, model, options)));
    let s1 = create_link_request_machine::take_step::step1(s0);
    assert(create_link_request_machine::State::step1(s0, s1));
    assert(s1 == (choose|s| create_link_request_machine::State::step1(s0, s)));
    let s2 = create_link_request_machine::take_step::step2(s1);
    assert(create_link_request_machine::State::step2(s1, s2));
    assert(s2 == (choose|s| create_link_request_machine::State::step2(s1, s)));
    let s3 = create_link_request_machine::take_step::step3(s2);
    assert(create_link_request_machine::State::step3(s2, s3));
    assert(s3 == (choose|s| create_link_request_machine::State::step3(s2, s)));
    if s3.state != CreateLinkRequestState::Final {
        let s4 = create_link_request_machine::take_step::step4(s3);
        assert(create_link_request_machine::State::step4(s3, s4));
        assert(s4 == (choose|s| create_link_request_machine::State::step4(s3, s)));
        let s5 = create_link_request_machine::take_step::step5(s4);
        assert(create_link_request_machine::State::step5(s4, s5));
        assert(s5 == (choose|s| create_link_request_machine::State::step5(s4, s)));
        let s6 = create_link_request_machine::take_step::step6(s5);
        assert(create_link_request_machine::State::step6(s5, s6));
        assert(s6 == (choose|s| create_link_request_machine::State::step6(s5, s)));
        let s7 = create_link_request_machine::take_step::step7(s6);
        assert(create_link_request_machine::State::step7(s6, s7));
        assert(s7 == (choose|s| create_link_request_machine::State::step7(s6, s)));
        let s8 = create_link_request_machine::take_step::step8(s7);
        assert(create_link_request_machine::State::step8(s7, s8));
        assert(s8 == (choose|s| create_link_request_machine::State::step8(s7, s)));
        let s9 = create_link_request_machine::take_step::step9(s8);
        assert(create_link_request_machine::State::step9(s8, s9));
        assert(s9 == (choose|s| create_link_request_machine::State::step9(s8, s)));
        let s10 = create_link_request_machine::take_step::step10(s9);
        assert(create_link_request_machine::State::step10(s9, s10));
        assert(s10 == (choose|s| create_link_request_machine::State::step10(s9, s)));
        let s11 = create_link_request_machine::take_step::step11(s10);
        assert(create_link_request_machine::State::step11(s10, s11));
        assert(s11 == (choose|s| create_link_request_machine::State::step11(s10, s)));
    }
}

} // verus!
