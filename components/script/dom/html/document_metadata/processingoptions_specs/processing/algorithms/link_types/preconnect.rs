/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/links.html#preconnect
//! Snapshot/inspection 2026-10-30.
//! Form: row 1, state_machine!; URL/partitionKey temporaries and useCredentials update.
//! The user agent should attempt to initiate a preconnect and perform the full
//! connection handshake (DNS+TCP for HTTP, and DNS+TCP+TLS for HTTPS origins)
//! whenever possible, but is allowed to elect to perform a partial handshake
//! (DNS only for HTTP, and DNS or DNS+TCP for HTTPS origins), or skip it entirely,
//! due to resource constraints or other reasons.

use net_traits::request::CorsSettings as NativeCorsSettings;
use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::super::model::*;
use super::super::super::types::ServoUrlView;
use super::super::super::model_trait::LinkProcessingModel;
use super::super::request::{href, encoding_parse};
use super::super::origin::same_origin;

verus! {
// https://fetch.spec.whatwg.org/#http-scheme
pub closed spec fn http_or_https(url: ServoUrlView) -> bool { url.scheme == "http"@ || url.scheme == "https"@ }
// https://fetch.spec.whatwg.org/#determine-the-network-partition-key
pub closed spec fn determine_partition_key<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> int {
    model.network_partition_key(options.environment)
}
pub closed spec fn anonymous_cross_origin<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView, url: ServoUrlView) -> bool {
    options.crossorigin is Some && options.crossorigin->Some_0.as_native() == NativeCorsSettings::Anonymous
        && !same_origin(options.origin, model.url_origin(url))
}
pub closed spec fn no_connection() -> PreconnectPlan { PreconnectPlan::Noop }
// https://fetch.spec.whatwg.org/#concept-connection-obtain
pub closed spec fn obtain_connection_plan<M: LinkProcessingModel>(model: M, partition_key: int, url: ServoUrlView, use_credentials: bool) -> PreconnectPlan {
    PreconnectPlan::MayConnect { partition_key, origin: model.url_origin(url), use_credentials }
}
pub enum PreconnectState { Step1, Step2, Step3, Step4, Step5, Step6, Step7, Step8, Final }
}

state_machine! {
    preconnect_machine<M: LinkProcessingModel> {
        fields {
            // Immutable inputs
            pub model: M,
            pub options: LinkProcessingOptionsView,
            // temporary var used by Steps 2–8
            pub url: Option<ServoUrlView>,
            // temporary var used by Steps 5–8
            pub partition_key: int,
            // temporary var used by Steps 6–8
            pub use_credentials: bool,
            pub state: PreconnectState,
            pub result: PreconnectPlan,
        }
        init! {
            initialize(model: M, options: LinkProcessingOptionsView) {
                init model = model;
                init options = options;
                init url = None;
                init partition_key = 0;
                init use_credentials = false;
                init state = PreconnectState::Step1;
                init result = no_connection();
            }
        }
        // 1. If options's href is an empty string, return.
        transition! {
            step1() {
                require(pre.state == PreconnectState::Step1);
                if href(pre.options).len() == 0 {
                    update state = PreconnectState::Final;
                    update result = no_connection();
                } else { update state = PreconnectState::Step2; }
            }
        }
        // 2. Let url be the result of encoding-parsing a URL given options's href, relative to options's base URL.
        transition! {
            step2() {
                require(pre.state == PreconnectState::Step2);
                update url = encoding_parse(pre.model, pre.options);
                update state = PreconnectState::Step3;
            }
        }
        // 3. If url is failure, then return.
        transition! {
            step3() {
                require(pre.state == PreconnectState::Step3);
                if pre.url is None {
                    update state = PreconnectState::Final;
                    update result = no_connection();
                } else { update state = PreconnectState::Step4; }
            }
        }
        // 4. If url's scheme is not an HTTP(S) scheme, then return.
        transition! {
            step4() {
                require(pre.state == PreconnectState::Step4);
                if !http_or_https(pre.url->Some_0) {
                    update state = PreconnectState::Final;
                    update result = no_connection();
                } else { update state = PreconnectState::Step5; }
            }
        }
        // 5. Let partitionKey be the result of determining the network partition key given options's environment.
        transition! {
            step5() {
                require(pre.state == PreconnectState::Step5);
                update partition_key = determine_partition_key(pre.model, pre.options);
                update state = PreconnectState::Step6;
            }
        }
        // 6. Let useCredentials be true.
        transition! {
            step6() {
                require(pre.state == PreconnectState::Step6);
                update use_credentials = true;
                update state = PreconnectState::Step7;
            }
        }
        // 7. If options's crossorigin is Anonymous and options's origin does not have the same origin as url's origin, then set useCredentials to false.
        transition! {
            step7() {
                require(pre.state == PreconnectState::Step7);
                if anonymous_cross_origin(pre.model, pre.options, pre.url->Some_0) {
                    update use_credentials = false;
                    update state = PreconnectState::Step8;
                } else { update state = PreconnectState::Step8; }
            }
        }
        // 8. The user agent should obtain a connection given partitionKey, url's origin, and useCredentials.
        transition! {
            step8() {
                require(pre.state == PreconnectState::Step8);
                update state = PreconnectState::Final;
                update result = obtain_connection_plan(pre.model, pre.partition_key, pre.url->Some_0, pre.use_credentials);
            }
        }
    }
}

verus! {
pub(crate) open spec fn preconnect_run<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> preconnect_machine::State<M> {
    let s0 = choose|s| preconnect_machine::State::initialize(s, model, options);
    let s1 = choose|s| preconnect_machine::State::step1(s0, s);
    if s1.state == PreconnectState::Final { s1 } else {
        let s2 = choose|s| preconnect_machine::State::step2(s1, s);
        let s3 = choose|s| preconnect_machine::State::step3(s2, s);
        if s3.state == PreconnectState::Final { s3 } else {
            let s4 = choose|s| preconnect_machine::State::step4(s3, s);
            if s4.state == PreconnectState::Final { s4 } else {
                let s5 = choose|s| preconnect_machine::State::step5(s4, s);
                let s6 = choose|s| preconnect_machine::State::step6(s5, s);
                let s7 = choose|s| preconnect_machine::State::step7(s6, s);
                choose|s| preconnect_machine::State::step8(s7, s)
            }
        }
    }
}

pub(crate) open spec fn preconnect<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> PreconnectPlan {
    preconnect_run(model, options).result
}

/// Derived computation helper, not another numbered standard definition.
pub(crate) open spec fn preconnect_computation<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView) -> PreconnectPlan {
    if options.href.len() == 0 { PreconnectPlan::Noop } else {
        match model.encoding_parse_url(options.href, options.base_url) {
            None => PreconnectPlan::Noop,
            Some(url) => if url.scheme != "http"@ && url.scheme != "https"@ { PreconnectPlan::Noop }
                else { PreconnectPlan::MayConnect {
                    partition_key: model.network_partition_key(options.environment), origin: model.url_origin(url),
                    use_credentials: !(options.crossorigin is Some && options.crossorigin->Some_0.as_native() == NativeCorsSettings::Anonymous
                        && !same_origin(options.origin, model.url_origin(url))),
                } },
        }
    }
}

pub(crate) broadcast proof fn preconnect_result<M: LinkProcessingModel>(model: M, options: LinkProcessingOptionsView)
    ensures
        (#[trigger] preconnect_run(model, options)).state == PreconnectState::Final,
        preconnect_run(model, options).options == options,
        preconnect_run(model, options).result == preconnect_computation(model, options),
{
    super::super::request::request_input_projections(model, options);
    let s0 = preconnect_machine::take_step::initialize(model, options);
    assert(s0 == (choose|s| preconnect_machine::State::initialize(s, model, options)));
    let s1 = preconnect_machine::take_step::step1(s0);
    assert(preconnect_machine::State::step1(s0, s1));
    assert(s1 == (choose|s| preconnect_machine::State::step1(s0, s)));
    if s1.state != PreconnectState::Final {
        let s2 = preconnect_machine::take_step::step2(s1);
        assert(preconnect_machine::State::step2(s1, s2));
        assert(s2 == (choose|s| preconnect_machine::State::step2(s1, s)));
        let s3 = preconnect_machine::take_step::step3(s2);
        assert(preconnect_machine::State::step3(s2, s3));
        assert(s3 == (choose|s| preconnect_machine::State::step3(s2, s)));
        if s3.state != PreconnectState::Final {
            let s4 = preconnect_machine::take_step::step4(s3);
            assert(preconnect_machine::State::step4(s3, s4));
            assert(s4 == (choose|s| preconnect_machine::State::step4(s3, s)));
            if s4.state != PreconnectState::Final {
                let s5 = preconnect_machine::take_step::step5(s4);
                assert(preconnect_machine::State::step5(s4, s5));
                assert(s5 == (choose|s| preconnect_machine::State::step5(s4, s)));
                let s6 = preconnect_machine::take_step::step6(s5);
                assert(preconnect_machine::State::step6(s5, s6));
                assert(s6 == (choose|s| preconnect_machine::State::step6(s5, s)));
                let s7 = preconnect_machine::take_step::step7(s6);
                assert(preconnect_machine::State::step7(s6, s7));
                assert(s7 == (choose|s| preconnect_machine::State::step7(s6, s)));
                let s8 = preconnect_machine::take_step::step8(s7);
                assert(preconnect_machine::State::step8(s7, s8));
                assert(s8 == (choose|s| preconnect_machine::State::step8(s7, s)));
            }
        }
    }
}
} // verus!
