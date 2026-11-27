use mime::Mime;
use vstd::prelude::*;
// use verus_state_machines_macros::state_machine;
use verus_state_machines_macros::{
    state_machine,
    case_on_next,
};

use crate::LoadContext;
use crate::mime_classifier::{
    MediaType, 
    ApacheBugFlag, 
    NoSniffFlag,
    ByteMatcher,
    Mp4Matcher,
    BinaryOrPlaintextClassifier,
    GroupedClassifier,
    MimeClassifier,
    ThreadSafeMIMEChecker,
};

// use crate::mime_classifier::MIMEChecker;
use crate::mime_classifier_specs::byte_matcher as SpecByteMatcher;
use crate::mime_classifier_specs::mp4_matcher as SpecMp4Matcher;
use crate::mime_classifier_specs::classifier::checker_trait::MimeClassifierModel;
use crate::verus_specs::external::mime::*;
use crate::mime_classifier_specs::predicates::{
    is_xml,
    is_html,
    is_image,
    is_audio_video,
    is_javascript,
    is_font,
    is_json,
    is_text,
    is_css,
    is_explicit_unknown,
    matches_webm_signature,
    matches_mp3_without_id3_signature,
};

verus! {

// https://mimesniff.spec.whatwg.org/#mime-type-sniffing-algorithm
// To determine the computed MIME type of a resource, user agents must use the following
// MIME type sniffing algorithm:
pub ghost enum MimeTypeSniffState {
    Init,
    State1,
    State2,
    State3,
    State4,
    State5,
    State6,
    State7,
    State8,
    Final,
}

state_machine! {
    mime_classifier_automaton<C: MimeClassifierModel> {
        fields {
            // Immutable inputs for one execution of the algorithm.
            pub classifier: C,
            pub supplied_type: Option<Mime>,
            pub no_sniff_flag: NoSniffFlag,
            pub apache_bug_flag: ApacheBugFlag,
            pub data: Seq<u8>,

            // current state
            pub state: MimeTypeSniffState,

            // temporary var used by Steps 5–8
            pub matched_type: Option<MimeView>,

            // return value
            pub computed_mime_type: Option<MimeView>,
        }

        //// The transitions

        init! {
            initialize(
                classifier: C,
                supplied_type: Option<Mime>,
                no_sniff_flag: NoSniffFlag,
                apache_bug_flag: ApacheBugFlag,
                data: Seq<u8>,
            ) {
                init classifier = classifier;
                init supplied_type = supplied_type;
                init no_sniff_flag = no_sniff_flag;
                init apache_bug_flag = apache_bug_flag;
                init data = data;

                init state = MimeTypeSniffState::Init;
                init matched_type = None;
                init computed_mime_type = None;
            }
        }

        // 1. If the supplied MIME type is an XML MIME type or
        // HTML MIME type, the computed MIME type is the supplied
        // MIME type. Abort these steps.
        transition! {
            step1() {
                require(pre.state == MimeTypeSniffState::Init);

                match pre.supplied_type {
                    Some(mt) => {
                        if is_xml(&mt) || is_html(&mt) {
                            update state = MimeTypeSniffState::Final;
                            update computed_mime_type = Some(view(&mt));
                        } else {
                            update state = MimeTypeSniffState::State1;
                        }
                    },
                    None => {
                        update state = MimeTypeSniffState::State1;
                    },
                }
            }
        }

        // 2. If the supplied MIME type is undefined or if the supplied MIME type’s essence is 
        //    "unknown/unknown", "application/unknown", or "*/*", execute the rules for identifying 
        //    an unknown MIME type with the sniff-scriptable flag equal to the inverse of the no-sniff 
        //    flag and abort these steps.
        transition!{
            step2() {
                require(pre.state == MimeTypeSniffState::State1);

                match &pre.supplied_type {
                    None => {
                        update state = MimeTypeSniffState::Final;
                        update computed_mime_type = Some(
                            pre.classifier.sniff_unknown_type(pre.no_sniff_flag, pre.data)
                        );
                    }
                    Some(mt) => {
                        if is_explicit_unknown(&mt) {
                            update state = MimeTypeSniffState::Final;
                            update computed_mime_type = Some(
                                pre.classifier.sniff_unknown_type(pre.no_sniff_flag, pre.data)
                            );
                        } else {
                            update state = MimeTypeSniffState::State2;
                        }
                    }
                }
            }
        }

        // 3. If the no-sniff flag is set, the computed MIME type is the supplied MIME type.
        //    Abort these steps.
        transition!{
            step3() {
                require(pre.state == MimeTypeSniffState::State2);
                require(pre.supplied_type is Some);

                if pre.no_sniff_flag == NoSniffFlag::On {
                    let supplied_type = pre.supplied_type->Some_0;

                    update state = MimeTypeSniffState::Final;
                    update computed_mime_type = Some(view(&supplied_type));
                } else {
                    update state = MimeTypeSniffState::State3;
                }
            }
        }

        // 4. If the check-for-apache-bug flag is set, execute the rules for distinguishing if a 
        //    resource is text or binary and abort these steps.
        transition!{
            step4() {
                require(pre.state == MimeTypeSniffState::State3);

                if pre.apache_bug_flag == ApacheBugFlag::On {
                    update state = MimeTypeSniffState::Final;
                    update computed_mime_type = Some(
                        pre.classifier.sniff_text_or_data(pre.data)
                    );
                } else {
                    update state = MimeTypeSniffState::State4;
                }
            }
        }

        // 5. If the supplied MIME type is an image MIME type supported by the user agent, let 
        //    matched-type be the result of executing the image type pattern matching algorithm with 
        //    the resource header as the byte sequence to be matched.
        transition!{
            step5() {
                require(pre.state == MimeTypeSniffState::State4);
                require(pre.supplied_type is Some);

                let supplied_type = pre.supplied_type->Some_0;

                if is_image(&supplied_type) {
                    update matched_type = pre.classifier.image_type(pre.data);
                    
                } 
                
                update state = MimeTypeSniffState::State5;
            }
        }

        // 6. If matched-type is not undefined, the computed MIME type is matched-type.
        //    Abort these steps.
        transition! {
            step6() {
                require(pre.state == MimeTypeSniffState::State5);

                if pre.matched_type.is_some() {
                    update state = MimeTypeSniffState::Final;
                    update computed_mime_type = pre.matched_type;
                } else {
                    update state = MimeTypeSniffState::State6;
                }
            }
        }

        // 7. If the supplied MIME type is an audio or video MIME type supported by the user agent, 
        //    let matched-type be the result of executing the audio or video type pattern matching 
        //    algorithm with the resource header as the byte sequence to be matched.
        transition!{
            step7() {
                require(pre.state == MimeTypeSniffState::State6);
                require(pre.supplied_type is Some);

                let supplied_type = pre.supplied_type->Some_0;

                if is_audio_video(&supplied_type) {
                    update matched_type = pre.classifier.audio_video_type(pre.data);
                } 

                update state = MimeTypeSniffState::State7;
            }
        }

        // 8. If matched-type is not undefined, the computed MIME type is matched-type.
        //    Abort these steps.
        transition! {
            step8() {
                require(pre.state == MimeTypeSniffState::State7);

                if pre.matched_type.is_some() {
                    update state = MimeTypeSniffState::Final;
                    update computed_mime_type = pre.matched_type;
                } else {
                    update state = MimeTypeSniffState::State8;
                }
            }
        }

        // 9. The computed MIME type is the supplied MIME type. 
        transition!{
            step9() {
                require(pre.state == MimeTypeSniffState::State8);
                require(pre.supplied_type is Some);

                let supplied_type = pre.supplied_type->Some_0;

                update state = MimeTypeSniffState::Final;
                update computed_mime_type = Some(view(&supplied_type));
            }
        }

        //// Invariants on the state

        #[invariant]
        pub fn supplied_type_defined(&self) -> bool {
            (
                self.state == MimeTypeSniffState::State2
                    || self.state == MimeTypeSniffState::State3
                    || self.state == MimeTypeSniffState::State4
                    || self.state == MimeTypeSniffState::State5
                    || self.state == MimeTypeSniffState::State6
                    || self.state == MimeTypeSniffState::State7
                    || self.state == MimeTypeSniffState::State8
            ) ==> self.supplied_type is Some
        }

        #[invariant]
        pub fn matched_type_is_fresh(&self) -> bool {
            (
                self.state != MimeTypeSniffState::State5
                    && self.state != MimeTypeSniffState::State7
                    && self.state != MimeTypeSniffState::Final
            ) ==> self.matched_type is None
        }

        #[invariant]
        pub fn result_exists_only_at_final(&self) -> bool {
            (self.state == MimeTypeSniffState::Final)
                <==> (self.computed_mime_type is Some)
        }

        //// Proofs that the invariants hold

        #[inductive(initialize)]
        fn initialize_inductive(
            post: Self,
            classifier: C,
            supplied_type: Option<Mime>,
            no_sniff_flag: NoSniffFlag,
            apache_bug_flag: ApacheBugFlag,
            data: Seq<u8>,
        ) {}

        #[inductive(step1)]
        fn step1_inductive(pre: Self, post: Self) {}

        #[inductive(step2)]
        fn step2_inductive(pre: Self, post: Self) {}

        #[inductive(step3)]
        fn step3_inductive(pre: Self, post: Self) {}

        #[inductive(step4)]
        fn step4_inductive(pre: Self, post: Self) {}

        #[inductive(step5)]
        fn step5_inductive(pre: Self, post: Self) {}

        #[inductive(step6)]
        fn step6_inductive(pre: Self, post: Self) {}

        #[inductive(step7)]
        fn step7_inductive(pre: Self, post: Self) {}

        #[inductive(step8)]
        fn step8_inductive(pre: Self, post: Self) {}

        #[inductive(step9)]
        fn step9_inductive(pre: Self, post: Self) {}
    }
}

pub(crate) open spec fn mime_type_sniffing_trace<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: Option<Mime>,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    data: Seq<u8>,
    trace: Seq<mime_classifier_automaton::State<C>>,
) -> bool {
    &&& trace.len() > 0
    &&& mime_classifier_automaton::State::<C>::initialize(
        trace.first(),
        classifier,
        supplied_type,
        no_sniff_flag,
        apache_bug_flag,
        data,
    )
    &&& forall |i: int|
        0 <= i && i + 1 < trace.len() ==>
            #[trigger]
            mime_classifier_automaton::State::<C>::next(
                trace[i],
                trace[i + 1],
            )
    &&& trace.last().state == MimeTypeSniffState::Final
}

pub(crate) open spec fn mime_type_sniffing_result<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: Option<Mime>,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    exists |trace: Seq<mime_classifier_automaton::State<C>>|
        #[trigger]
        mime_type_sniffing_trace(
            classifier,
            supplied_type,
            no_sniff_flag,
            apache_bug_flag,
            data,
            trace,
        )
        && trace.last().computed_mime_type == Some(result)
}

// Execution through the generated state-machine relations, following mime_parse_automaton.

pub open spec fn mime_type_sniffing_final_state<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: Option<Mime>,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    data: Seq<u8>,
) -> mime_classifier_automaton::State<C> {
    let initial = choose|state: mime_classifier_automaton::State<C>|
        mime_classifier_automaton::State::<C>::initialize(
            state, classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
        );
    mime_type_sniffing_run(initial)
}

pub open spec fn mime_type_sniffing_terminal(state: MimeTypeSniffState) -> bool {
    state == MimeTypeSniffState::Final
}

// Select a successor without duplicating the DSL's field updates. Progress is
// proved for invariant-valid states; terminal states are left unchanged.
pub open spec fn mime_type_sniffing_step<C: MimeClassifierModel>(
    pre: mime_classifier_automaton::State<C>,
) -> mime_classifier_automaton::State<C> {
    if mime_type_sniffing_terminal(pre.state) {
        pre
    } else {
        choose|post: mime_classifier_automaton::State<C>|
            mime_classifier_automaton::State::<C>::next(pre, post)
    }
}

// Every transition advances a phase or finishes; no input-dependent fuel is needed.
pub open spec fn mime_type_sniffing_phase_rank(state: MimeTypeSniffState) -> nat {
    match state {
        MimeTypeSniffState::Init => 9,
        MimeTypeSniffState::State1 => 8,
        MimeTypeSniffState::State2 => 7,
        MimeTypeSniffState::State3 => 6,
        MimeTypeSniffState::State4 => 5,
        MimeTypeSniffState::State5 => 4,
        MimeTypeSniffState::State6 => 3,
        MimeTypeSniffState::State7 => 2,
        MimeTypeSniffState::State8 => 1,
        MimeTypeSniffState::Final => 0,
    }
}

pub proof fn mime_type_sniffing_next_exists<C: MimeClassifierModel>(
    pre: mime_classifier_automaton::State<C>,
) -> (post: mime_classifier_automaton::State<C>)
    requires pre.invariant(), !mime_type_sniffing_terminal(pre.state),
    ensures
        mime_classifier_automaton::State::<C>::next(pre, post),
        post.invariant(),
{
    match pre.state {
        MimeTypeSniffState::Init => {
            let post = mime_classifier_automaton::take_step::step1(pre);
            mime_classifier_automaton::show::step1(pre, post);
            post
        },
        MimeTypeSniffState::State1 => {
            let post = mime_classifier_automaton::take_step::step2(pre);
            mime_classifier_automaton::show::step2(pre, post);
            post
        },
        MimeTypeSniffState::State2 => {
            let post = mime_classifier_automaton::take_step::step3(pre);
            mime_classifier_automaton::show::step3(pre, post);
            post
        },
        MimeTypeSniffState::State3 => {
            let post = mime_classifier_automaton::take_step::step4(pre);
            mime_classifier_automaton::show::step4(pre, post);
            post
        },
        MimeTypeSniffState::State4 => {
            let post = mime_classifier_automaton::take_step::step5(pre);
            mime_classifier_automaton::show::step5(pre, post);
            post
        },
        MimeTypeSniffState::State5 => {
            let post = mime_classifier_automaton::take_step::step6(pre);
            mime_classifier_automaton::show::step6(pre, post);
            post
        },
        MimeTypeSniffState::State6 => {
            let post = mime_classifier_automaton::take_step::step7(pre);
            mime_classifier_automaton::show::step7(pre, post);
            post
        },
        MimeTypeSniffState::State7 => {
            let post = mime_classifier_automaton::take_step::step8(pre);
            mime_classifier_automaton::show::step8(pre, post);
            post
        },
        MimeTypeSniffState::State8 => {
            let post = mime_classifier_automaton::take_step::step9(pre);
            mime_classifier_automaton::show::step9(pre, post);
            post
        },
        MimeTypeSniffState::Final => pre,
    }
}

// Invariant preservation and strict rank decrease come from the actual transitions.
pub proof fn mime_type_sniffing_next_properties<C: MimeClassifierModel>(
    pre: mime_classifier_automaton::State<C>,
    post: mime_classifier_automaton::State<C>,
)
    requires pre.invariant(), mime_classifier_automaton::State::<C>::next(pre, post),
    ensures
        post.invariant(),
        !mime_type_sniffing_terminal(pre.state),
        mime_type_sniffing_phase_rank(post.state) < mime_type_sniffing_phase_rank(pre.state),
{
    case_on_next! { pre, post, mime_classifier_automaton::<C> => {
        step1() => {}
        step2() => {}
        step3() => {}
        step4() => {}
        step5() => {}
        step6() => {}
        step7() => {}
        step8() => {}
        step9() => {}
    }}
}

// Determinism is independent of invariant validity, the runner, and termination.
pub proof fn mime_type_sniffing_next_is_deterministic<C: MimeClassifierModel>(
    pre: mime_classifier_automaton::State<C>,
    left: mime_classifier_automaton::State<C>,
    right: mime_classifier_automaton::State<C>,
)
    requires
        mime_classifier_automaton::State::<C>::next(pre, left),
        mime_classifier_automaton::State::<C>::next(pre, right),
    ensures left == right,
{
    reveal(mime_classifier_automaton::State::next);
    reveal(mime_classifier_automaton::State::next_by);
}

pub broadcast proof fn mime_type_sniffing_step_correct<C: MimeClassifierModel>(
    pre: mime_classifier_automaton::State<C>,
)
    requires pre.invariant(), !mime_type_sniffing_terminal(pre.state),
    ensures
        mime_classifier_automaton::State::<C>::next(pre, #[trigger] mime_type_sniffing_step(pre)),
        mime_type_sniffing_step(pre).invariant(),
        mime_type_sniffing_phase_rank(mime_type_sniffing_step(pre).state)
            < mime_type_sniffing_phase_rank(pre.state),
{
    mime_type_sniffing_next_exists(pre);
    mime_type_sniffing_next_properties(pre, mime_type_sniffing_step(pre));
}

pub open spec fn mime_type_sniffing_run<C: MimeClassifierModel>(
    pre: mime_classifier_automaton::State<C>,
) -> mime_classifier_automaton::State<C>
    decreases mime_type_sniffing_phase_rank(pre.state),
{
    decreases_by(mime_type_sniffing_run_decreases::<C>);
    // Totalize on malformed internal states. Initialized runs satisfy the
    // invariant, and step_correct preserves it, so they never use that fallback.
    if !pre.invariant() || mime_type_sniffing_terminal(pre.state) {
        pre
    } else {
        mime_type_sniffing_run(mime_type_sniffing_step(pre))
    }
}

#[verifier::decreases_by]
proof fn mime_type_sniffing_run_decreases<C: MimeClassifierModel>(
    pre: mime_classifier_automaton::State<C>,
) {
    if pre.invariant() && !mime_type_sniffing_terminal(pre.state) {
        mime_type_sniffing_step_correct(pre);
    }
}

pub open spec fn mime_type_sniffing_steps<C: MimeClassifierModel>(
    trace: Seq<mime_classifier_automaton::State<C>>,
) -> bool {
    forall|i: int| 0 <= i && i + 1 < trace.len() ==>
        #[trigger] mime_classifier_automaton::State::<C>::next(trace[i], trace[i + 1])
}

pub proof fn mime_type_sniffing_step_at<C: MimeClassifierModel>(
    trace: Seq<mime_classifier_automaton::State<C>>,
    i: int,
)
    requires mime_type_sniffing_steps(trace), 0 <= i, i + 1 < trace.len(),
    ensures mime_classifier_automaton::State::<C>::next(trace[i], trace[i + 1]),
{
    reveal(mime_type_sniffing_steps);
}

// Termination supplies a completed trace for every invariant-valid starting state.
// This construction does not depend on the determinism theorem.
pub proof fn mime_type_sniffing_run_trace<C: MimeClassifierModel>(
    pre: mime_classifier_automaton::State<C>,
) -> (trace: Seq<mime_classifier_automaton::State<C>>)
    requires pre.invariant(),
    ensures
        trace.len() > 0,
        trace.first() == pre,
        mime_type_sniffing_steps(trace),
        trace.last() == mime_type_sniffing_run(pre),
        trace.last().invariant(),
        mime_type_sniffing_terminal(trace.last().state),
    decreases mime_type_sniffing_phase_rank(pre.state),
{
    if mime_type_sniffing_terminal(pre.state) {
        seq![pre]
    } else {
        mime_type_sniffing_step_correct(pre);
        let post = mime_type_sniffing_step(pre);
        let rest = mime_type_sniffing_run_trace(post);
        let trace = seq![pre] + rest;
        assert forall|i: int| 0 <= i && i + 1 < trace.len() implies
            #[trigger] mime_classifier_automaton::State::<C>::next(trace[i], trace[i + 1]) by {
            if i > 0 {
                mime_type_sniffing_step_at(rest, i - 1);
            }
        }
        trace
    }
}

pub(crate) proof fn mime_type_sniffing_final_state_properties<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: Option<Mime>,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    data: Seq<u8>,
)
    ensures
        mime_type_sniffing_final_state(
            classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
        ).invariant(),
        mime_type_sniffing_terminal(mime_type_sniffing_final_state(
            classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
        ).state),
        mime_type_sniffing_final_state(
            classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
        ).computed_mime_type is Some,
        exists|trace: Seq<mime_classifier_automaton::State<C>>|
            #[trigger] mime_type_sniffing_trace(
                classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, trace,
            ) && trace.last() == mime_type_sniffing_final_state(
                classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
            ),
{
    let initial = mime_classifier_automaton::take_step::initialize(
        classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
    );
    let trace = mime_type_sniffing_run_trace(initial);
    assert(mime_type_sniffing_trace(
        classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, trace,
    ));
}

// Determinism makes every completed sequence of transitions agree with the runner.
// The sequence may start at any invariant-valid state, including a terminal state.
pub proof fn mime_type_sniffing_steps_run<C: MimeClassifierModel>(
    trace: Seq<mime_classifier_automaton::State<C>>,
)
    requires
        trace.len() > 0,
        trace.first().invariant(),
        mime_type_sniffing_steps(trace),
        mime_type_sniffing_terminal(trace.last().state),
    ensures
        trace.last() == mime_type_sniffing_run(trace.first()),
        trace.last().invariant(),
    decreases trace.len(),
{
    if trace.len() > 1 {
        mime_type_sniffing_step_at(trace, 0);
        mime_type_sniffing_next_properties(trace[0], trace[1]);
        mime_type_sniffing_step_correct(trace[0]);
        mime_type_sniffing_next_is_deterministic(
            trace[0], trace[1], mime_type_sniffing_step(trace[0]),
        );
        let rest = trace.subrange(1, trace.len() as int);
        assert forall|i: int| 0 <= i && i + 1 < rest.len() implies
            #[trigger] mime_classifier_automaton::State::<C>::next(rest[i], rest[i + 1]) by {
            mime_type_sniffing_step_at(trace, i + 1);
        }
        mime_type_sniffing_steps_run(rest);
    }
}

// Connect the existing existential result relation to the terminating runner.
// Forward: determinism identifies the given trace's endpoint.
// Reverse: termination supplies a trace witnessing the runner's result.
pub(crate) proof fn mime_type_sniffing_result_iff_run<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: Option<Mime>,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    data: Seq<u8>,
    result: MimeView,
)
    ensures
        mime_type_sniffing_result(
            classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, result,
        ) <==> mime_type_sniffing_final_state(
            classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
        ).computed_mime_type == Some(result),
{
    if mime_type_sniffing_result(
        classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, result,
    ) {
        let trace = choose|trace: Seq<mime_classifier_automaton::State<C>>|
            #[trigger] mime_type_sniffing_trace(
                classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, trace,
            ) && trace.last().computed_mime_type == Some(result);
        mime_type_sniffing_steps_run(trace);
    }
    if mime_type_sniffing_final_state(
        classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
    ).computed_mime_type == Some(result) {
        mime_type_sniffing_final_state_properties(
            classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
        );
        let trace = choose|trace: Seq<mime_classifier_automaton::State<C>>|
            #[trigger] mime_type_sniffing_trace(
                classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, trace,
            ) && trace.last() == mime_type_sniffing_final_state(
                classifier, supplied_type, no_sniff_flag, apache_bug_flag, data,
            );
        assert(trace.last().computed_mime_type == Some(result));
    }
}

pub(crate) proof fn mime_type_sniffing_result_unique<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: Option<Mime>,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    data: Seq<u8>,
    left: MimeView,
    right: MimeView,
)
    requires
        mime_type_sniffing_result(
            classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, left,
        ),
        mime_type_sniffing_result(
            classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, right,
        ),
    ensures left == right,
{
    mime_type_sniffing_result_iff_run(
        classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, left,
    );
    mime_type_sniffing_result_iff_run(
        classifier, supplied_type, no_sniff_flag, apache_bug_flag, data, right,
    );
}

} // verus!
