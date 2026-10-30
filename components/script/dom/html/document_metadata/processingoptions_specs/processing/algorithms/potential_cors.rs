/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/urls-and-fetching.html#create-a-potential-cors-request
//! Snapshot/inspection 2026-10-30.
//! Form: row 1, state_machine!; mode and credentialsMode are set across steps.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use net_traits::request::CorsSettings as NativeCorsSettings;
use super::super::model::*;
use super::super::types::{ServoUrlView, DestinationView, CorsSettingsView};
use super::super::model_trait::LinkProcessingModel;

verus! {
// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#attr-crossorigin-anonymous
pub closed spec fn anonymous(state: Option<CorsSettingsView>) -> bool {
    state is Some && state->Some_0.as_native() == NativeCorsSettings::Anonymous
}

// https://fetch.spec.whatwg.org/#concept-request
// This constructor uses the separately supplied Fetch-default record, then the
// five fields explicitly specified by HTML. It does not model network execution.
pub closed spec fn new_request<M: LinkProcessingModel>(
    model: M, url: ServoUrlView, destination: DestinationView,
    mode: Seq<char>, credentials_mode: Seq<char>,
) -> RequestView {
    RequestView {
        url, destination,
        mode: if mode == "no-cors"@ { RequestMode::NoCors }
            else if mode == "same-origin"@ { RequestMode::SameOrigin } else { RequestMode::Cors },
        credentials_mode: if credentials_mode == "same-origin"@ { CredentialsMode::SameOrigin } else { CredentialsMode::Include },
        use_url_credentials: true,
        ..model.new_request()
    }
}
pub enum PotentialCorsState { Step1, Step2, Step3, Step4, Step5, Final }
}

state_machine! {
    potential_cors_machine<M: LinkProcessingModel> {
        fields {
            // Immutable inputs
            pub model: M,
            pub url: ServoUrlView,
            pub destination: DestinationView,
            pub cors_attribute_state: Option<CorsSettingsView>,
            pub same_origin_fallback_flag: bool,
            // temporary var used by Steps 1–5
            pub mode: Seq<char>,
            // temporary var used by Steps 3–5
            pub credentials_mode: Seq<char>,
            pub state: PotentialCorsState,
            pub result: Option<RequestView>,
        }
        init! {
            initialize(model: M, url: ServoUrlView, destination: DestinationView,
                cors_attribute_state: Option<CorsSettingsView>, same_origin_fallback_flag: bool) {
                init model = model;
                init url = url;
                init destination = destination;
                init cors_attribute_state = cors_attribute_state;
                init same_origin_fallback_flag = same_origin_fallback_flag;
                init mode = Seq::empty();
                init credentials_mode = Seq::empty();
                init state = PotentialCorsState::Step1;
                init result = None;
            }
        }
        // 1. Let mode be "no-cors" if corsAttributeState is No CORS, and "cors" otherwise.
        transition! {
            step1() {
                require(pre.state == PotentialCorsState::Step1);
                update mode = if pre.cors_attribute_state is None { "no-cors"@ } else { "cors"@ };
                update state = PotentialCorsState::Step2;
            }
        }
        // 2. If same-origin fallback flag is set and mode is "no-cors", set mode to "same-origin".
        transition! {
            step2() {
                require(pre.state == PotentialCorsState::Step2);
                if pre.same_origin_fallback_flag && pre.mode == "no-cors"@ {
                    update mode = "same-origin"@;
                    update state = PotentialCorsState::Step3;
                } else {
                    update state = PotentialCorsState::Step3;
                }
            }
        }
        // 3. Let credentialsMode be "include".
        transition! {
            step3() {
                require(pre.state == PotentialCorsState::Step3);
                update credentials_mode = "include"@;
                update state = PotentialCorsState::Step4;
            }
        }
        // 4. If corsAttributeState is Anonymous, set credentialsMode to "same-origin".
        transition! {
            step4() {
                require(pre.state == PotentialCorsState::Step4);
                if anonymous(pre.cors_attribute_state) {
                    update credentials_mode = "same-origin"@;
                    update state = PotentialCorsState::Step5;
                } else {
                    update state = PotentialCorsState::Step5;
                }
            }
        }
        // 5. Return a new request whose URL is url, destination is destination, mode is mode, credentials mode is credentialsMode, and whose use-URL-credentials flag is set.
        transition! {
            step5() {
                require(pre.state == PotentialCorsState::Step5);
                update state = PotentialCorsState::Final;
                update result = Some(new_request(pre.model, pre.url, pre.destination, pre.mode, pre.credentials_mode));
            }
        }
    }
}

verus! {
pub(crate) open spec fn potential_cors_run<M: LinkProcessingModel>(
    model: M, url: ServoUrlView, destination: DestinationView,
    cors_attribute_state: Option<CorsSettingsView>, same_origin_fallback_flag: bool,
) -> potential_cors_machine::State<M> {
    let s0 = choose|s| potential_cors_machine::State::initialize(s, model, url, destination, cors_attribute_state, same_origin_fallback_flag);
    let s1 = choose|s| potential_cors_machine::State::step1(s0, s);
    let s2 = choose|s| potential_cors_machine::State::step2(s1, s);
    let s3 = choose|s| potential_cors_machine::State::step3(s2, s);
    let s4 = choose|s| potential_cors_machine::State::step4(s3, s);
    choose|s| potential_cors_machine::State::step5(s4, s)
}

pub(crate) open spec fn create_a_potential_cors_request<M: LinkProcessingModel>(
    model: M, url: ServoUrlView, destination: DestinationView,
    cors_attribute_state: Option<CorsSettingsView>, same_origin_fallback: bool,
) -> RequestView {
    potential_cors_run(model, url, destination, cors_attribute_state, same_origin_fallback).result->Some_0
}

pub(crate) broadcast proof fn potential_cors_result<M: LinkProcessingModel>(
    model: M, url: ServoUrlView, destination: DestinationView,
    cors_attribute_state: Option<CorsSettingsView>, same_origin_fallback_flag: bool,
)
    ensures
        (#[trigger] potential_cors_run(model, url, destination, cors_attribute_state, same_origin_fallback_flag)).state == PotentialCorsState::Final,
        potential_cors_run(model, url, destination, cors_attribute_state, same_origin_fallback_flag).result == Some(RequestView {
            url, destination,
            mode: if cors_attribute_state is None {
                if same_origin_fallback_flag { RequestMode::SameOrigin } else { RequestMode::NoCors }
            } else { RequestMode::Cors },
            credentials_mode: if cors_attribute_state is Some
                && cors_attribute_state->Some_0.as_native() == NativeCorsSettings::Anonymous
            { CredentialsMode::SameOrigin } else { CredentialsMode::Include },
            use_url_credentials: true, ..model.new_request()
        }),
{
    let s0 = potential_cors_machine::take_step::initialize(model, url, destination, cors_attribute_state, same_origin_fallback_flag);
    assert(s0 == (choose|s| potential_cors_machine::State::initialize(s, model, url, destination, cors_attribute_state, same_origin_fallback_flag)));
    let s1 = potential_cors_machine::take_step::step1(s0);
    assert(potential_cors_machine::State::step1(s0, s1));
    assert(s1 == (choose|s| potential_cors_machine::State::step1(s0, s)));
    let s2 = potential_cors_machine::take_step::step2(s1);
    assert(potential_cors_machine::State::step2(s1, s2));
    assert(s2 == (choose|s| potential_cors_machine::State::step2(s1, s)));
    let s3 = potential_cors_machine::take_step::step3(s2);
    assert(potential_cors_machine::State::step3(s2, s3));
    assert(s3 == (choose|s| potential_cors_machine::State::step3(s2, s)));
    let s4 = potential_cors_machine::take_step::step4(s3);
    assert(potential_cors_machine::State::step4(s3, s4));
    assert(s4 == (choose|s| potential_cors_machine::State::step4(s3, s)));
    let s5 = potential_cors_machine::take_step::step5(s4);
    assert(potential_cors_machine::State::step5(s4, s5));
    assert(s5 == (choose|s| potential_cors_machine::State::step5(s4, s)));
}
} // verus!
