/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/links.html#match-preload-type
//! Snapshot inspected 2026-10-30; HTML last updated 2026-10-30.
//! Form: row 1, state_machine!; mimeTypeRecord is carried from step 3 to steps 4–6.
//! A once-bound temporary qualifies under the user's approved clarification.
//! MIME parsing, category predicates and UA support are separately named model
//! operations. Their native correspondence is not established by this machine.

use vstd::prelude::*;
use verus_state_machines_macros::{state_machine, case_on_next};
use super::super::super::model_trait::LinkProcessingModel;

verus! {
pub enum MatchPreloadTypeState { Step1, Step2, Step3, Step4, Step5, Step6, Step7, Final }

/// Named linked operations, supplied as mathematical functions. This avoids
/// widening all crate-private views to satisfy generated public DSL relations.
#[verifier::reject_recursive_types(T)]
pub struct MimeOperations<T> {
    pub parse: spec_fn(Seq<char>) -> Option<T>,
    pub supported: spec_fn(T) -> bool,
    pub javascript: spec_fn(T) -> bool,
    pub image: spec_fn(T) -> bool,
    pub font: spec_fn(T) -> bool,
    pub essence: spec_fn(T) -> Seq<char>,
}

impl<T> MimeOperations<T> {
    // https://mimesniff.spec.whatwg.org/#parse-a-mime-type
    pub open spec fn parse_mime_type(self, type_: Seq<char>) -> Option<T> { (self.parse)(type_) }
    // https://mimesniff.spec.whatwg.org/#supported-by-the-user-agent
    pub open spec fn mime_supported(self, mime: T) -> bool { (self.supported)(mime) }
    // https://mimesniff.spec.whatwg.org/#javascript-mime-type
    pub open spec fn is_javascript_mime(self, mime: T) -> bool { (self.javascript)(mime) }
    // https://mimesniff.spec.whatwg.org/#image-mime-type
    pub open spec fn is_image_mime(self, mime: T) -> bool { (self.image)(mime) }
    // https://mimesniff.spec.whatwg.org/#font-mime-type
    pub open spec fn is_font_mime(self, mime: T) -> bool { (self.font)(mime) }
    // https://mimesniff.spec.whatwg.org/#mime-type-essence
    pub open spec fn mime_essence(self, mime: T) -> Seq<char> { (self.essence)(mime) }
}

pub(crate) open spec fn operations<M: LinkProcessingModel>(model: M) -> MimeOperations<M::Mime> {
    MimeOperations {
        parse: |type_| model.parse_mime_type(type_),
        supported: |mime| model.mime_supported(mime),
        javascript: |mime| model.is_javascript_mime(mime),
        image: |mime| model.is_image_mime(mime),
        font: |mime| model.is_font_mime(mime),
        essence: |mime| model.mime_essence(mime),
    }
}
}

state_machine! {
    #[verifier::reject_recursive_types(M)]
    match_preload_type<M> {
        fields {
            // Immutable inputs: set only by initialize.
            pub model: MimeOperations<M>,
            pub type_: Seq<char>,
            pub destination: Seq<char>,
            // temporary var used by Steps 3–6; None denotes MIME parsing failure.
            pub mime_type_record: Option<M>,
            pub state: MatchPreloadTypeState,
            pub result: bool,
        }

        init! {
            initialize(model: MimeOperations<M>, type_: Seq<char>, destination: Seq<char>) {
                init model = model;
                init type_ = type_;
                init destination = destination;
                init mime_type_record = None;
                init state = MatchPreloadTypeState::Step1;
                init result = false;
            }
        }

        // 1. If type is an empty string, then return true.
        transition! {
            step1() {
                require(pre.state == MatchPreloadTypeState::Step1);
                if pre.type_ == ""@ {
                    update state = MatchPreloadTypeState::Final;
                    update result = true;
                } else {
                    update state = MatchPreloadTypeState::Step2;
                }
            }
        }

        // 2. If destination is "fetch", then return true.
        transition! {
            step2() {
                require(pre.state == MatchPreloadTypeState::Step2);
                if pre.destination == "fetch"@ {
                    update state = MatchPreloadTypeState::Final;
                    update result = true;
                } else {
                    update state = MatchPreloadTypeState::Step3;
                }
            }
        }

        // 3. Let mimeTypeRecord be the result of parsing type.
        transition! {
            step3() {
                require(pre.state == MatchPreloadTypeState::Step3);
                update mime_type_record = pre.model.parse_mime_type(pre.type_);
                update state = MatchPreloadTypeState::Step4;
            }
        }

        // 4. If mimeTypeRecord is failure, then return false.
        transition! {
            step4() {
                require(pre.state == MatchPreloadTypeState::Step4);
                if pre.mime_type_record is None {
                    update state = MatchPreloadTypeState::Final;
                    update result = false;
                } else {
                    update state = MatchPreloadTypeState::Step5;
                }
            }
        }

        // 5. If mimeTypeRecord is not supported by the user agent, then return false.
        transition! {
            step5() {
                require(pre.state == MatchPreloadTypeState::Step5);
                if !pre.model.mime_supported(pre.mime_type_record->Some_0) {
                    update state = MatchPreloadTypeState::Final;
                    update result = false;
                } else {
                    update state = MatchPreloadTypeState::Step6;
                }
            }
        }

        // 6. If any of the following are true:
        //    • destination is "script" and mimeTypeRecord is a JavaScript MIME type;
        //    • destination is "image" and mimeTypeRecord is an image MIME type;
        //    • destination is "font" and mimeTypeRecord is a font MIME type;
        //    • destination is "style" and mimeTypeRecord's essence is text/css; or
        //    • destination is "track" and mimeTypeRecord's essence is text/vtt,
        //    then return true.
        transition! {
            step6() {
                require(pre.state == MatchPreloadTypeState::Step6);
                if (pre.destination == "script"@ && pre.model.is_javascript_mime(pre.mime_type_record->Some_0))
                    || (pre.destination == "image"@ && pre.model.is_image_mime(pre.mime_type_record->Some_0))
                    || (pre.destination == "font"@ && pre.model.is_font_mime(pre.mime_type_record->Some_0))
                    || (pre.destination == "style"@ && pre.model.mime_essence(pre.mime_type_record->Some_0) == "text/css"@)
                    || (pre.destination == "track"@ && pre.model.mime_essence(pre.mime_type_record->Some_0) == "text/vtt"@)
                {
                    update state = MatchPreloadTypeState::Final;
                    update result = true;
                } else {
                    update state = MatchPreloadTypeState::Step7;
                }
            }
        }

        // 7. Return false.
        transition! {
            step7() {
                require(pre.state == MatchPreloadTypeState::Step7);
                update state = MatchPreloadTypeState::Final;
                update result = false;
            }
        }
    }
}

verus! {

/// Scheduling adapter, not an additional standard step. A terminal state has no
/// outgoing transition; keeping it unchanged here merely stops the bounded runner.
pub(crate) open spec fn advance<M>(s: match_preload_type::State<M>)
    -> match_preload_type::State<M>
{
    match s.state {
        MatchPreloadTypeState::Step1 => choose|post| match_preload_type::State::step1(s, post),
        MatchPreloadTypeState::Step2 => choose|post| match_preload_type::State::step2(s, post),
        MatchPreloadTypeState::Step3 => choose|post| match_preload_type::State::step3(s, post),
        MatchPreloadTypeState::Step4 => choose|post| match_preload_type::State::step4(s, post),
        MatchPreloadTypeState::Step5 => choose|post| match_preload_type::State::step5(s, post),
        MatchPreloadTypeState::Step6 => choose|post| match_preload_type::State::step6(s, post),
        MatchPreloadTypeState::Step7 => choose|post| match_preload_type::State::step7(s, post),
        MatchPreloadTypeState::Final => s,
    }
}

/// At most seven transitions; early returns stop scheduling transitions.
pub(crate) open spec fn run<M: LinkProcessingModel>(model: M, type_: Seq<char>, destination: Seq<char>)
    -> match_preload_type::State<M::Mime>
{
    let s = choose|s| match_preload_type::State::initialize(s, operations(model), type_, destination);
    advance(advance(advance(advance(advance(advance(advance(s)))))))
}

pub(crate) proof fn advance_correct<M>(s: match_preload_type::State<M>)
    ensures
        s.state == MatchPreloadTypeState::Step1 ==> match_preload_type::State::step1(s, advance(s)),
        s.state == MatchPreloadTypeState::Step2 ==> match_preload_type::State::step2(s, advance(s)),
        s.state == MatchPreloadTypeState::Step3 ==> match_preload_type::State::step3(s, advance(s)),
        s.state == MatchPreloadTypeState::Step4 ==> match_preload_type::State::step4(s, advance(s)),
        s.state == MatchPreloadTypeState::Step5 ==> match_preload_type::State::step5(s, advance(s)),
        s.state == MatchPreloadTypeState::Step6 ==> match_preload_type::State::step6(s, advance(s)),
        s.state == MatchPreloadTypeState::Step7 ==> match_preload_type::State::step7(s, advance(s)),
        s.state == MatchPreloadTypeState::Final ==> advance(s) == s,
{
    match s.state {
        MatchPreloadTypeState::Step1 => {
            let post = match_preload_type::take_step::step1(s);
            assert(match_preload_type::State::step1(s, post));
        },
        MatchPreloadTypeState::Step2 => {
            let post = match_preload_type::take_step::step2(s);
            assert(match_preload_type::State::step2(s, post));
        },
        MatchPreloadTypeState::Step3 => {
            let post = match_preload_type::take_step::step3(s);
            assert(match_preload_type::State::step3(s, post));
        },
        MatchPreloadTypeState::Step4 => {
            let post = match_preload_type::take_step::step4(s);
            assert(match_preload_type::State::step4(s, post));
        },
        MatchPreloadTypeState::Step5 => {
            let post = match_preload_type::take_step::step5(s);
            assert(match_preload_type::State::step5(s, post));
        },
        MatchPreloadTypeState::Step6 => {
            let post = match_preload_type::take_step::step6(s);
            assert(match_preload_type::State::step6(s, post));
        },
        MatchPreloadTypeState::Step7 => {
            let post = match_preload_type::take_step::step7(s);
            assert(match_preload_type::State::step7(s, post));
        },
        MatchPreloadTypeState::Final => {},
    }
}

/// Derived result theorem for clients; the transition system is the definition.
pub(crate) broadcast proof fn runner_finishes<M: LinkProcessingModel>(model: M, type_: Seq<char>, destination: Seq<char>)
    ensures
        (#[trigger] run(model, type_, destination)).state == MatchPreloadTypeState::Final,
        run(model, type_, destination).model == operations(model),
        run(model, type_, destination).type_ == type_,
        run(model, type_, destination).destination == destination,
        run(model, type_, destination).result == (type_ == ""@ || destination == "fetch"@ ||
            match model.parse_mime_type(type_) {
                None => false,
                Some(mime) => model.mime_supported(mime) && (
                    (destination == "script"@ && model.is_javascript_mime(mime))
                    || (destination == "image"@ && model.is_image_mime(mime))
                    || (destination == "font"@ && model.is_font_mime(mime))
                    || (destination == "style"@ && model.mime_essence(mime) == "text/css"@)
                    || (destination == "track"@ && model.mime_essence(mime) == "text/vtt"@)),
            }),
{
    let initial = match_preload_type::take_step::initialize(operations(model), type_, destination);
    assert(initial == (choose|s| match_preload_type::State::initialize(s, operations(model), type_, destination)));
    advance_correct(initial);
    let s = advance(initial);
    advance_correct(s);
    let s = advance(s);
    advance_correct(s);
    let s = advance(s);
    advance_correct(s);
    let s = advance(s);
    advance_correct(s);
    let s = advance(s);
    advance_correct(s);
    let s = advance(s);
    advance_correct(s);
}

/// Every actual machine edge preserves the inputs. Successful parsing dominates
/// steps 5 and 6, so accessing the MIME temporary is justified on reachable states.
pub(crate) proof fn transition_properties<M>(
    pre: match_preload_type::State<M>, post: match_preload_type::State<M>,
)
    requires
        match_preload_type::State::next(pre, post),
        (pre.state == MatchPreloadTypeState::Step5 || pre.state == MatchPreloadTypeState::Step6)
            ==> pre.mime_type_record is Some,
    ensures
        post.model == pre.model,
        post.type_ == pre.type_,
        post.destination == pre.destination,
        pre.state != MatchPreloadTypeState::Final,
        (post.state == MatchPreloadTypeState::Step5 || post.state == MatchPreloadTypeState::Step6)
            ==> post.mime_type_record is Some,
{
    case_on_next! { pre, post, match_preload_type::<M> => {
        step1() => {}
        step2() => {}
        step3() => {}
        step4() => {}
        step5() => {}
        step6() => {}
        step7() => {}
    }}
}

} // verus!
