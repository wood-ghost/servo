/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/semantics.html#apply-link-options-from-parsed-header-attributes
//! Snapshot/inspection 2026-10-30.
//! Form: row 1, state_machine!; destination temporary, options updates and nested returns.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::model::{LinkAttributes, LinkProcessingOptionsView};
use super::super::types::{DestinationView, ReferrerPolicyView};
use super::super::super::predicates::ascii_lowercase;
use super::{cors, options};

verus! {
// Public closed field/keyword adapters keep local invariant-bearing views private.
pub closed spec fn translate_destination(value: Seq<char>) -> Option<DestinationView> {
    super::link_types::preload::translate_a_preload_destination(value)
}
pub closed spec fn set_destination(options: LinkProcessingOptionsView, destination: DestinationView) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { destination, ..options }
}
pub closed spec fn cors_keyword(value: Seq<char>) -> bool { cors::determine_cors_settings_for_token(value) is Some }
pub closed spec fn set_crossorigin(options: LinkProcessingOptionsView, value: Seq<char>) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { crossorigin: cors::determine_cors_settings_for_token(value), ..options }
}
pub closed spec fn set_integrity(options: LinkProcessingOptionsView, value: Seq<char>) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { integrity: value, ..options }
}
pub closed spec fn referrer_policy_keyword(value: Seq<char>) -> bool { ReferrerPolicyView::from_name(ascii_lowercase(value)) is Some }
pub closed spec fn set_referrer_policy(options: LinkProcessingOptionsView, value: Seq<char>) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { referrer_policy: ReferrerPolicyView::from_name(ascii_lowercase(value))->Some_0, ..options }
}
pub closed spec fn set_nonce(options: LinkProcessingOptionsView, value: Seq<char>) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { cryptographic_nonce_metadata: value, ..options }
}
pub closed spec fn set_type(options: LinkProcessingOptionsView, value: Seq<char>) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { type_: value, ..options }
}
pub closed spec fn fetch_priority_keyword(value: Seq<char>) -> bool { options::fetch_priority_keyword_state(value) is Some }
// FINDING: Step 7 says keyword, whereas the record definition says state. The
// keyword's explicitly defined table state is the typed representation used here.
pub closed spec fn set_fetch_priority_keyword(options: LinkProcessingOptionsView, value: Seq<char>) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { fetch_priority: options::fetch_priority_keyword_state(value)->Some_0, ..options }
}
pub enum ApplyAttributesState { Step1, Step1_1, Step1_2, Step1_3, Step1_4, Step2, Step3, Step4, Step5, Step6, Step7, Step8, Final }
}

state_machine! {
    apply_attributes_machine {
        fields {
            // Immutable inputs
            pub attribs: LinkAttributes,
            pub rel: Seq<char>,
            // Mutable input record, updated by Steps 1.4 and 2–7.
            pub options: LinkProcessingOptionsView,
            // temporary var used by Steps 1.2–1.4
            pub destination: Option<DestinationView>,
            pub state: ApplyAttributesState,
            pub result: bool,
        }
        init! {
            initialize(options: LinkProcessingOptionsView, attribs: LinkAttributes, rel: Seq<char>) {
                init options = options;
                init attribs = attribs;
                init rel = rel;
                init destination = None;
                init state = ApplyAttributesState::Step1;
                init result = false;
            }
        }
        // 1. If rel is "preload":
        transition! {
            step1() {
                require(pre.state == ApplyAttributesState::Step1);
                if pre.rel == "preload"@ { update state = ApplyAttributesState::Step1_1; }
                else { update state = ApplyAttributesState::Step2; }
            }
        }
        // 1.1. If attribs["as"] does not exist, then return false.
        transition! {
            step1_1() {
                require(pre.state == ApplyAttributesState::Step1_1);
                if !pre.attribs.dom().contains("as"@) {
                    update state = ApplyAttributesState::Final;
                    update result = false;
                } else { update state = ApplyAttributesState::Step1_2; }
            }
        }
        // 1.2. Let destination be the result of translating attribs["as"].
        transition! {
            step1_2() {
                require(pre.state == ApplyAttributesState::Step1_2);
                update destination = translate_destination(pre.attribs["as"@]);
                update state = ApplyAttributesState::Step1_3;
            }
        }
        // 1.3. If destination is null, then return false.
        transition! {
            step1_3() {
                require(pre.state == ApplyAttributesState::Step1_3);
                if pre.destination is None {
                    update state = ApplyAttributesState::Final;
                    update result = false;
                } else { update state = ApplyAttributesState::Step1_4; }
            }
        }
        // 1.4. Set options's destination to destination.
        transition! {
            step1_4() {
                require(pre.state == ApplyAttributesState::Step1_4);
                update options = set_destination(pre.options, pre.destination->Some_0);
                update state = ApplyAttributesState::Step2;
            }
        }
        // 2. If attribs["crossorigin"] exists and is an ASCII case-insensitive match for one of the CORS settings attribute keywords, then set options's crossorigin to the CORS settings attribute state corresponding to that keyword.
        transition! {
            step2() {
                require(pre.state == ApplyAttributesState::Step2);
                if pre.attribs.dom().contains("crossorigin"@) && cors_keyword(pre.attribs["crossorigin"@]) {
                    update options = set_crossorigin(pre.options, pre.attribs["crossorigin"@]);
                    update state = ApplyAttributesState::Step3;
                } else { update state = ApplyAttributesState::Step3; }
            }
        }
        // 3. If attribs["integrity"] exists, then set options's integrity to attribs["integrity"].
        transition! {
            step3() {
                require(pre.state == ApplyAttributesState::Step3);
                if pre.attribs.dom().contains("integrity"@) {
                    update options = set_integrity(pre.options, pre.attribs["integrity"@]);
                    update state = ApplyAttributesState::Step4;
                } else { update state = ApplyAttributesState::Step4; }
            }
        }
        // 4. If attribs["referrerpolicy"] exists and is an ASCII case-insensitive match for some referrer policy, then set options's referrer policy to that referrer policy.
        transition! {
            step4() {
                require(pre.state == ApplyAttributesState::Step4);
                if pre.attribs.dom().contains("referrerpolicy"@) && referrer_policy_keyword(pre.attribs["referrerpolicy"@]) {
                    update options = set_referrer_policy(pre.options, pre.attribs["referrerpolicy"@]);
                    update state = ApplyAttributesState::Step5;
                } else { update state = ApplyAttributesState::Step5; }
            }
        }
        // 5. If attribs["nonce"] exists, then set options's nonce to attribs["nonce"].
        transition! {
            step5() {
                require(pre.state == ApplyAttributesState::Step5);
                if pre.attribs.dom().contains("nonce"@) {
                    update options = set_nonce(pre.options, pre.attribs["nonce"@]);
                    update state = ApplyAttributesState::Step6;
                } else { update state = ApplyAttributesState::Step6; }
            }
        }
        // 6. If attribs["type"] exists, then set options's type to attribs["type"].
        transition! {
            step6() {
                require(pre.state == ApplyAttributesState::Step6);
                if pre.attribs.dom().contains("type"@) {
                    update options = set_type(pre.options, pre.attribs["type"@]);
                    update state = ApplyAttributesState::Step7;
                } else { update state = ApplyAttributesState::Step7; }
            }
        }
        // 7. If attribs["fetchpriority"] exists and is an ASCII case-insensitive match for a fetch priority attribute keyword, then set options's fetch priority to that fetch priority attribute keyword.
        transition! {
            step7() {
                require(pre.state == ApplyAttributesState::Step7);
                if pre.attribs.dom().contains("fetchpriority"@) && fetch_priority_keyword(pre.attribs["fetchpriority"@]) {
                    update options = set_fetch_priority_keyword(pre.options, pre.attribs["fetchpriority"@]);
                    update state = ApplyAttributesState::Step8;
                } else { update state = ApplyAttributesState::Step8; }
            }
        }
        // 8. Return true.
        transition! {
            step8() {
                require(pre.state == ApplyAttributesState::Step8);
                update state = ApplyAttributesState::Final;
                update result = true;
            }
        }
    }
}

verus! {
/// Scheduling helpers use only generated relations; they are not extra HTML steps.
pub(crate) open spec fn destination_steps(s1: apply_attributes_machine::State) -> apply_attributes_machine::State {
    if s1.state == ApplyAttributesState::Step1_1 {
        let s11 = choose|s| apply_attributes_machine::State::step1_1(s1, s);
        if s11.state == ApplyAttributesState::Final { s11 } else {
            let s12 = choose|s| apply_attributes_machine::State::step1_2(s11, s);
            let s13 = choose|s| apply_attributes_machine::State::step1_3(s12, s);
            if s13.state == ApplyAttributesState::Final { s13 }
            else { choose|s| apply_attributes_machine::State::step1_4(s13, s) }
        }
    } else { s1 }
}

pub(crate) open spec fn attribute_run(options: LinkProcessingOptionsView, attribs: LinkAttributes, rel: Seq<char>) -> apply_attributes_machine::State {
    let s0 = choose|s| apply_attributes_machine::State::initialize(s, options, attribs, rel);
    let s1 = choose|s| apply_attributes_machine::State::step1(s0, s);
    let d = destination_steps(s1);
    if d.state == ApplyAttributesState::Final { d } else {
        let s2 = choose|s| apply_attributes_machine::State::step2(d, s);
        let s3 = choose|s| apply_attributes_machine::State::step3(s2, s);
        let s4 = choose|s| apply_attributes_machine::State::step4(s3, s);
        let s5 = choose|s| apply_attributes_machine::State::step5(s4, s);
        let s6 = choose|s| apply_attributes_machine::State::step6(s5, s);
        let s7 = choose|s| apply_attributes_machine::State::step7(s6, s);
        choose|s| apply_attributes_machine::State::step8(s7, s)
    }
}

pub(crate) open spec fn apply_link_options_from_parsed_header(options: LinkProcessingOptionsView, attribs: LinkAttributes, rel: Seq<char>)
    -> (bool, LinkProcessingOptionsView)
{
    let final_state = attribute_run(options, attribs, rel);
    (final_state.result, final_state.options)
}

proof fn destination_steps_correct(s1: apply_attributes_machine::State)
    requires s1.state == ApplyAttributesState::Step1_1 || s1.state == ApplyAttributesState::Step2,
    ensures
        destination_steps(s1).state == ApplyAttributesState::Final || destination_steps(s1).state == ApplyAttributesState::Step2,
        destination_steps(s1).attribs == s1.attribs,
        destination_steps(s1).rel == s1.rel,
        (destination_steps(s1).state == ApplyAttributesState::Final ==> !destination_steps(s1).result),
        destination_steps(s1).options == if s1.state == ApplyAttributesState::Step1_1
            && s1.attribs.dom().contains("as"@) && translate_destination(s1.attribs["as"@]) is Some {
                set_destination(s1.options, translate_destination(s1.attribs["as"@])->Some_0)
            } else { s1.options },
        (destination_steps(s1).state == ApplyAttributesState::Step2) == (s1.state == ApplyAttributesState::Step2
            || (s1.attribs.dom().contains("as"@) && translate_destination(s1.attribs["as"@]) is Some)),
{
    if s1.state == ApplyAttributesState::Step1_1 {
        let s11 = apply_attributes_machine::take_step::step1_1(s1);
        assert(apply_attributes_machine::State::step1_1(s1, s11));
        assert(s11 == (choose|s| apply_attributes_machine::State::step1_1(s1, s)));
        if s11.state != ApplyAttributesState::Final {
            let s12 = apply_attributes_machine::take_step::step1_2(s11);
            assert(apply_attributes_machine::State::step1_2(s11, s12));
            assert(s12 == (choose|s| apply_attributes_machine::State::step1_2(s11, s)));
            let s13 = apply_attributes_machine::take_step::step1_3(s12);
            assert(apply_attributes_machine::State::step1_3(s12, s13));
            assert(s13 == (choose|s| apply_attributes_machine::State::step1_3(s12, s)));
            if s13.state != ApplyAttributesState::Final {
                let s14 = apply_attributes_machine::take_step::step1_4(s13);
                assert(apply_attributes_machine::State::step1_4(s13, s14));
                assert(s14 == (choose|s| apply_attributes_machine::State::step1_4(s13, s)));
            }
        }
    }
}

pub(crate) broadcast proof fn attribute_result(options: LinkProcessingOptionsView, attribs: LinkAttributes, rel: Seq<char>)
    ensures
        (#[trigger] attribute_run(options, attribs, rel)).state == ApplyAttributesState::Final,
        (attribute_run(options, attribs, rel).result, attribute_run(options, attribs, rel).options)
            == options::attribute_application_computation(options, attribs, rel),
{
    let s0 = apply_attributes_machine::take_step::initialize(options, attribs, rel);
    assert(s0 == (choose|s| apply_attributes_machine::State::initialize(s, options, attribs, rel)));
    let s1 = apply_attributes_machine::take_step::step1(s0);
    assert(apply_attributes_machine::State::step1(s0, s1));
    assert(s1 == (choose|s| apply_attributes_machine::State::step1(s0, s)));
    destination_steps_correct(s1);
    let d = destination_steps(s1);
    if d.state != ApplyAttributesState::Final {
        let s2 = apply_attributes_machine::take_step::step2(d);
        assert(apply_attributes_machine::State::step2(d, s2));
        assert(s2 == (choose|s| apply_attributes_machine::State::step2(d, s)));
        let s3 = apply_attributes_machine::take_step::step3(s2);
        assert(apply_attributes_machine::State::step3(s2, s3));
        assert(s3 == (choose|s| apply_attributes_machine::State::step3(s2, s)));
        let s4 = apply_attributes_machine::take_step::step4(s3);
        assert(apply_attributes_machine::State::step4(s3, s4));
        assert(s4 == (choose|s| apply_attributes_machine::State::step4(s3, s)));
        let s5 = apply_attributes_machine::take_step::step5(s4);
        assert(apply_attributes_machine::State::step5(s4, s5));
        assert(s5 == (choose|s| apply_attributes_machine::State::step5(s4, s)));
        let s6 = apply_attributes_machine::take_step::step6(s5);
        assert(apply_attributes_machine::State::step6(s5, s6));
        assert(s6 == (choose|s| apply_attributes_machine::State::step6(s5, s)));
        let s7 = apply_attributes_machine::take_step::step7(s6);
        assert(apply_attributes_machine::State::step7(s6, s7));
        assert(s7 == (choose|s| apply_attributes_machine::State::step7(s6, s)));
        let s8 = apply_attributes_machine::take_step::step8(s7);
        assert(apply_attributes_machine::State::step8(s7, s8));
        assert(s8 == (choose|s| apply_attributes_machine::State::step8(s7, s)));
    }
}
} // verus!
