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
use crate::mime_classifier_specs::mime_api::*;
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
    MimeClassifierAutomaton<C: MimeClassifierModel> {
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
    trace: Seq<MimeClassifierAutomaton::State<C>>,
) -> bool {
    &&& trace.len() > 0
    &&& MimeClassifierAutomaton::State::<C>::initialize(
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
            MimeClassifierAutomaton::State::<C>::next(
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
    exists |trace: Seq<MimeClassifierAutomaton::State<C>>|
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

} // verus!