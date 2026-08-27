use mime::Mime;
use vstd::prelude::*;
use verus_state_machines_macros::state_machine;

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
use super::byte_matcher as SpecByteMatcher;
use crate::mime_classifier_specs::mime_api::{
    text_plain_identity,
    text_css_identity,
    text_javascript_identity,
    application_octet_stream_identity,
    MimeView,
    essence_str,
    essence_str_view,
    view,
    option_view,
};
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
    mime_essence_str_lemmas,
    is_explicit_unknown,
};

// use crate::mime_classifier_specs::{
//     is_image,
//     has_xml_suffix,
//     essence_is_text_xml,
//     essence_is_application_xml,
//     is_audio,
//     is_video,
//     essence_is_application_ogg,
// };

verus! {

pub(crate) trait MIMECheckerSpec {
    spec fn classify_spec(&self, data: Seq<u8>) -> Option<MimeView>;
    spec fn validate_spec(&self) -> bool;
}

// ------------------------------------
// Binary or Plaintext Classifier
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#binary-data-byte
pub open spec fn is_binary_data_byte(byte: u8) -> bool {
    // A binary data byte is a byte in the range 0x00 to 0x08 (NUL to BS), 
    ||| 0x00u8 <= byte && byte <= 0x08u8
    // the byte 0x0B (VT), 
    ||| byte == 0x0Bu8
    // a byte in the range 0x0E to 0x1A (SO to SUB), 
    ||| 0x0Eu8 <= byte && byte <= 0x1Au8
    // or a byte in the range 0x1C to 0x1F (FS to US). 
    ||| 0x1Cu8 <= byte && byte <= 0x1Fu8
}
pub open spec fn contains_binary_data_byte(data: Seq<u8>) -> bool {
    exists |i: int| #![trigger data[i]] 0 <= i < data.len() && is_binary_data_byte(data[i])
}

// https://mimesniff.spec.whatwg.org/#rules-for-text-or-binary
//  To determine whether a binary resource has been mislabeled as plain text, 
//  execute the following rules for distinguishing if a resource is text or binary:
pub open spec fn bin_or_plain_classify_spec(data: Seq<u8>) -> MimeView {
    // 1. Let length be the number of bytes in the resource header.
    // 2. If length is greater than or equal to 2 and the first 2 bytes of the resource 
    //   header are equal to 0xFE 0xFF (UTF-16BE BOM) or 0xFF 0xFE (UTF-16LE BOM), the 
    //   computed MIME type is "text/plain".
    if data.len() as int >= 2 && ((data[0] == 0xFE && data[1] == 0xFF) || (data[0] == 0xFF && data[1] == 0xFE)) {
    //   Abort these steps.
        text_plain_identity()
    }
    // 3. If length is greater than or equal to 3 and the first 3 bytes of the resource 
    //   header are equal to 0xEF 0xBB 0xBF (UTF-8 BOM), the computed MIME type is "text/plain".
    else if data.len() as int >= 3 && (data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF) {
    //   Abort these steps.
        text_plain_identity()
    }
    // 4. If the resource header contains no binary data bytes, the computed MIME type is "text/plain".
    else if !contains_binary_data_byte(data) {
    //   Abort these steps.
        text_plain_identity()
    }
    // 5. The computed MIME type is "application/octet-stream".
    else {
        application_octet_stream_identity()
    }
}

impl MIMECheckerSpec for BinaryOrPlaintextClassifier {
    open spec fn classify_spec(&self, data: Seq<u8>) -> Option<MimeView> {
        Some(bin_or_plain_classify_spec(data))
    }

    open spec fn validate_spec(&self) -> bool {
        true
    }
}

// pub(crate) open spec fn classify_group_from(
//     classifier: &GroupedClassifier,
//     data: Seq<u8>,
//     index: nat,
// ) -> Option<MimeView>
//     decreases
//         classifier.byte_matchers@.len() - index,
// {
//     if index == classifier.byte_matchers@.len() {
//         None
//     } else {
//         match classifier.byte_matchers@[index as int].classify_spec(data) {
//             Some(content_type) => Some(content_type),
//             None => classify_group_from(
//                 classifier,
//                 data,
//                 index + 1,
//             ),
//         }
//     }
// }

pub(crate) open spec fn classify_group_from(
    matchers: Seq<Box<dyn ThreadSafeMIMEChecker>>,
    data: Seq<u8>,
) -> Option<MimeView>
    decreases
        matchers.len(),
{
    if matchers.len() == 0 {
        None
    } else {
        match matchers.first().classify_spec(data) {
            Some(content_type) => Some(content_type),
            None => classify_group_from(
                matchers.drop_first(),
                data,
            ),
        }
    }
}

pub(crate) broadcast proof fn lemma_classify_group_from_first_some(
    matchers: Seq<Box<dyn ThreadSafeMIMEChecker>>,
    data: Seq<u8>,
    idx: int,
)
    requires
        0 <= idx < matchers.len(),
        forall |j: int|
            #![trigger matchers[j].classify_spec(data)]
            0 <= j < idx ==>
                matchers[j].classify_spec(data).is_none(),
        matchers[idx].classify_spec(data).is_some(),
    ensures
        #![trigger
            classify_group_from(matchers, data),
            matchers[idx].classify_spec(data)
        ]
        classify_group_from(matchers, data) == matchers[idx].classify_spec(data),
    decreases idx
{
    if idx > 0 {
        lemma_classify_group_from_first_some(
            matchers.drop_first(),
            data,
            idx - 1,
        );
    }
}

pub(crate) broadcast proof fn lemma_classify_group_from_all_none(
    matchers: Seq<Box<dyn ThreadSafeMIMEChecker>>,
    data: Seq<u8>,
)
    requires
        forall |j: int|
            #![trigger matchers[j].classify_spec(data)]
            0 <= j < matchers.len() ==>
                matchers[j].classify_spec(data).is_none(),
    ensures
        #[trigger] classify_group_from(matchers, data).is_none(),
    decreases matchers.len()
{
    if matchers.len() > 0 {
        lemma_classify_group_from_all_none(
            matchers.drop_first(),
            data,
        );
    }
}

impl MIMECheckerSpec for GroupedClassifier {
    open spec fn classify_spec(&self, data: Seq<u8>) -> Option<MimeView> {
        classify_group_from(self.byte_matchers@, data)
    }

    open spec fn validate_spec(&self) -> bool {
        forall |i: int|
            0 <= i < self.byte_matchers@.len() ==>
                // #[trigger] self.byte_matchers@[i].validate_spec()
                #[trigger] self.byte_matchers@[i].dyn_validate_spec()
    }
}

// https://mimesniff.spec.whatwg.org/#image-type-pattern-matching-algorithm
pub(crate) open spec fn image_type_pattern_matching_algo(
    classifier: &MimeClassifier, 
    data: Seq<u8>,
) -> Option<MimeView> {
    //TODO:
    classifier.image_classifier.classify_spec(data)
}

// https://mimesniff.spec.whatwg.org/#audio-or-video-type-pattern-matching-algorithm
pub(crate) open spec fn audio_or_video_type_pattern_matching_algo(
    classifier: &MimeClassifier, 
    data: Seq<u8>,
) -> Option<MimeView> {
    //TODO:
    classifier.audio_video_classifier.classify_spec(data)
}

// https://mimesniff.spec.whatwg.org/#font-type-pattern-matching-algorithm
pub(crate) open spec fn font_type_pattern_matching_algo(
    classifier: &MimeClassifier, 
    data: Seq<u8>,
) -> Option<MimeView> {
    //TODO:
    classifier.font_classifier.classify_spec(data)
}

pub closed spec fn mime_classifier_validate_spec(classifier: &MimeClassifier) -> bool {
    &&& classifier.image_classifier.validate_spec()
    &&& classifier.audio_video_classifier.validate_spec()
    &&& classifier.scriptable_classifier.validate_spec()
    &&& classifier.plaintext_classifier.validate_spec()
    &&& classifier.archive_classifier.validate_spec()
    &&& classifier.binary_or_plaintext.validate_spec()
    &&& classifier.font_classifier.validate_spec()
}

// TODO: necessary?
pub(crate) proof fn lemma_mime_classifier_validate_spec(classifier: &MimeClassifier)
    ensures
        mime_classifier_validate_spec(classifier)
            == (
                classifier.image_classifier.validate_spec()
                    && classifier.audio_video_classifier.validate_spec()
                    && classifier.scriptable_classifier.validate_spec()
                    && classifier.plaintext_classifier.validate_spec()
                    && classifier.archive_classifier.validate_spec()
                    && classifier.binary_or_plaintext.validate_spec()
                    && classifier.font_classifier.validate_spec()
            ),
{}

pub(crate) broadcast proof fn lemma_valid_mime_classifier_from_valid_fields(
    classifier: &MimeClassifier,
)
    requires
        classifier.image_classifier.validate_spec(),
        classifier.audio_video_classifier.validate_spec(),
        classifier.scriptable_classifier.validate_spec(),
        classifier.plaintext_classifier.validate_spec(),
        classifier.archive_classifier.validate_spec(),
        classifier.binary_or_plaintext.validate_spec(),
        classifier.font_classifier.validate_spec(),
    ensures
        #[trigger] mime_classifier_validate_spec(classifier)
{}

pub open spec fn get_media_type_spec(
    mime: &Mime, result: Option<MediaType>
) -> bool {
    &&& (result == Some(MediaType::Xml)) == is_xml(mime)
    &&& (result == Some(MediaType::Html)) == (!is_xml(mime) && is_html(mime))
    &&& (result == Some(MediaType::Image)) == (!is_xml(mime) && !is_html(mime) && is_image(mime))
    &&& (result == Some(MediaType::AudioVideo)) == (!is_xml(mime) && !is_html(mime) 
                                                    && !is_image(mime) && is_audio_video(mime))
    &&& (result == Some(MediaType::JavaScript)) == (!is_xml(mime) && !is_html(mime) 
                    && !is_image(mime) && !is_audio_video(mime) && is_javascript(mime))
    &&& (result == Some(MediaType::Font)) == (!is_xml(mime) && !is_html(mime) 
                    && !is_image(mime) && !is_audio_video(mime) && !is_javascript(mime) && is_font(mime))
    &&& (result == Some(MediaType::Json)) == (!is_xml(mime) && !is_html(mime) 
                    && !is_image(mime) && !is_audio_video(mime) && !is_javascript(mime) && !is_font(mime) 
                    && is_json(mime))
    &&& (result == Some(MediaType::Text)) == (!is_xml(mime) && !is_html(mime) 
                    && !is_image(mime) && !is_audio_video(mime) && !is_javascript(mime) && !is_font(mime) 
                    && !is_json(mime) && is_text(mime))
    &&& (result == Some(MediaType::Css)) == (!is_xml(mime) && !is_html(mime) 
                    && !is_image(mime) && !is_audio_video(mime) && !is_javascript(mime) && !is_font(mime) 
                    && !is_json(mime) && !is_text(mime) && is_css(mime))
    // &&& !is_xml(mime) && !is_html(mime) == (
    //             ((result == Some(MediaType::Image))
    //                 == is_image(mime))
    //             &&
    //             ((result == Some(MediaType::AudioVideo))
    //                 == is_audio_video(mime))
    //         )
}

// ------------------------------------
// MIME Classifier
// ------------------------------------
pub trait MimeClassifierModel {
    spec fn sniff_unknown_type(&self, no_sniff_flag: NoSniffFlag, data: Seq<u8>) -> MimeView;

    spec fn sniff_text_or_data(&self, data: Seq<u8>) -> MimeView;

    spec fn image_type(&self, data: Seq<u8>) -> Option<MimeView>;

    spec fn audio_video_type(&self, data: Seq<u8>) -> Option<MimeView>;

    spec fn font_type(&self, data: Seq<u8>) -> Option<MimeView>;
}

impl<'a> MimeClassifierModel for &'a MimeClassifier {
    closed spec fn sniff_unknown_type(&self, no_sniff_flag: NoSniffFlag, data: Seq<u8>) -> MimeView {
        _sniff_unknown_type_spec(*self, no_sniff_flag, data)
    }

    closed spec fn sniff_text_or_data(&self, data: Seq<u8>) -> MimeView {
        sniff_text_or_data_spec(*self, data)
    }

    closed spec fn image_type(&self, data: Seq<u8>) -> Option<MimeView> {
        image_type_pattern_matching_algo(*self, data)
    }

    closed spec fn audio_video_type(&self, data: Seq<u8>) -> Option<MimeView> {
        audio_or_video_type_pattern_matching_algo(*self, data)
    }

    closed spec fn font_type(&self, data: Seq<u8>) -> Option<MimeView> {
        font_type_pattern_matching_algo(*self, data)
    }
}

// https://mimesniff.spec.whatwg.org/#rules-for-identifying-an-unknown-mime-type
// To determine the computed MIME type of a resource resource with an unknown MIME type, 
// execute the following rules for identifying an unknown MIME type: 
pub(crate) open spec fn _sniff_unknown_type_spec(
    classifier: &MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    data: Seq<u8>,
) -> MimeView {
    // 1. If the sniff-scriptable flag is set, execute the following steps for each row row in the following table:
    let matched_type =
        if no_sniff_flag == NoSniffFlag::Off {
            // 1.1 Let patternMatched be the result of the pattern matching algorithm given resource’s resource header, 
            //     the value in the first column of row, the value in the second column of row, and the value in the third 
            //     column of row.
            classifier.scriptable_classifier.classify_spec(data)
        } else {
            None
        };
    let matched_type = match matched_type {
        // 1.2 If patternMatched is true, return the value in the fourth column of row. 
        Some(mt) => Some(mt),
        // 2. Execute the following steps for each row row in the following table: 
        // 2.1 Let patternMatched be the result of the pattern matching algorithm given 
        //     resource’s resource header, the value in the first column of row, the value 
        //     in the second column of row, and the value in the third column of row.
        None => classifier.plaintext_classifier.classify_spec(data),
    };
    let matched_type = match matched_type {
        // 2.2 If patternMatched is true, return the value in the fourth column of row. 
        Some(mt) => Some(mt),
        // 3. Let matchedType be the result of executing the image type pattern matching 
        //    algorithm given resource’s resource header. 
        None => classifier.image_classifier.classify_spec(data),
    };
    let matched_type = match matched_type {
        // 4. If matchedType is not undefined, return matchedType. 
        Some(mt) => Some(mt),
        // 5. Set matchedType to the result of executing the audio or video type pattern 
        //    matching algorithm given resource’s resource header. 
        None => classifier.audio_video_classifier.classify_spec(data),
    };
    let matched_type = match matched_type {
        // 6. If matchedType is not undefined, return matchedType. 
        Some(mt) => Some(mt),
        // 7. Set matchedType to the result of executing the archive type pattern matching 
        //    algorithm given resource’s resource header. 
        None => classifier.archive_classifier.classify_spec(data),
    };

    match matched_type {
        // 8. If matchedType is not undefined, return matchedType. 
        Some(mt) => mt,
        None  => {
            // TODO: https://github.com/servo/servo/issues/47252
            // if !contains_binary_data_byte(data) {
                // 9. If resource’s resource header contains no binary data bytes, return 
                //    "text/plain"
                // text_plain_identity()
            // } else {
                // Return "application/octet-stream".
            //     application_octet_stream_identity()
            // }
            bin_or_plain_classify_spec(data)
        },
    }
}

pub closed spec fn sniff_unknown_type_spec(
    classifier: &MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    data: Seq<u8>,
) -> MimeView {
    _sniff_unknown_type_spec(
        classifier,
        no_sniff_flag,
        data,
    )
}

// https://mimesniff.spec.whatwg.org/#rules-for-text-or-binary
pub(crate) open spec fn sniff_text_or_data_spec(
    classifier: &MimeClassifier,
    data: Seq<u8>,
) -> MimeView {
    bin_or_plain_classify_spec(data)
} 

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

pub closed spec fn mime_classify_browsing_result<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    mime_type_sniffing_result(
        classifier,
        *supplied_type,
        no_sniff_flag,
        apache_bug_flag,
        data,
        result,
    )
}

pub(crate) open spec fn mime_classify_browsing_result_from_trace<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
    trace: Seq<MimeClassifierAutomaton::State<&'a MimeClassifier>>,
) -> bool {
    &&& mime_type_sniffing_trace(
        classifier,
        *supplied_type,
        no_sniff_flag,
        apache_bug_flag,
        data,
        trace
    )
    &&& trace.last().computed_mime_type == Some(result)
}


pub(crate) proof fn lemma_sniff_unknown_type_spec(
    classifier: &MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    data: Seq<u8>,
)
    ensures
        sniff_unknown_type_spec(
            classifier,
            no_sniff_flag,
            data,
        ) == _sniff_unknown_type_spec(
            classifier,
            no_sniff_flag,
            data,
        ),
{}

pub(crate) open spec fn mime_classify_browsing_after_step4(
    classifier: &MimeClassifier,
    supplied_type: &Mime,
    data: Seq<u8>,
) -> MimeView {
    let matched_type =
        if is_image(supplied_type) {
            classifier.image_classifier.classify_spec(data)
        } else {
            None
        };

    match matched_type {
        Some(mt) => mt,
        None => {
            let matched_type =
                if is_audio_video(supplied_type) {
                    classifier.audio_video_classifier.classify_spec(data)
                } else {
                    None
                };

            match matched_type {
                Some(mt) => mt,
                None => view(supplied_type),
            }
        },
    }
}

// ------------------------------------
// lemmas for state machine
// ------------------------------------
pub(crate) proof fn lemma_mime_classify_browsing_step1_result<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
)
    requires
        *supplied_type is Some,
        is_xml(&supplied_type->Some_0) || is_html(&supplied_type->Some_0),
        result == view(&supplied_type->Some_0),

    ensures
        mime_classify_browsing_result(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type,
            data,
            result,
        ),
{
    let state0 =
        MimeClassifierAutomaton::take_step::initialize(
            classifier,
            *supplied_type,
            no_sniff_flag,
            apache_bug_flag,
            data,
        );

    let state1 = MimeClassifierAutomaton::take_step::step1(state0);

    MimeClassifierAutomaton::show::step1(state0, state1);

    let trace = seq![state0, state1];

    assert(
        mime_classify_browsing_result_from_trace(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type,
            data,
            result,
            trace,
        )
    );
}

pub(crate) proof fn lemma_mime_classify_browsing_none_result<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
)
   requires
        *supplied_type is None,
    ensures
        mime_classify_browsing_result(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type,
            data,
            sniff_unknown_type_spec(classifier, no_sniff_flag, data)
        ), 
{
    let state0 =
        MimeClassifierAutomaton::take_step::initialize(
            classifier,
            *supplied_type,
            no_sniff_flag,
            apache_bug_flag,
            data
        );

    let state1 = MimeClassifierAutomaton::take_step::step1(state0);
    let state2 = MimeClassifierAutomaton::take_step::step2(state1);

    MimeClassifierAutomaton::show::step1(state0, state1);
    MimeClassifierAutomaton::show::step2(state1, state2);

    let trace = seq![state0, state1, state2];

    assert(
        mime_classify_browsing_result_from_trace(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type,
            data,
            sniff_unknown_type_spec(classifier, no_sniff_flag, data),
            trace
        )
    );
}

pub(crate) proof fn lemma_mime_classify_browsing_explicit_unknown_result<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type_input: &Option<Mime>,
    supplied_type: &Mime,
    data: Seq<u8>,
)
    requires
        *supplied_type_input == Some(*supplied_type),
        is_explicit_unknown(supplied_type),
        !is_xml(supplied_type),
        !is_html(supplied_type),
    ensures
        mime_classify_browsing_result(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type_input,
            data,
            sniff_unknown_type_spec(classifier, no_sniff_flag, data)
        ),
{
    let state0 =
        MimeClassifierAutomaton::take_step::initialize(
            classifier,
            *supplied_type_input,
            no_sniff_flag,
            apache_bug_flag,
            data
        );

    let state1 = MimeClassifierAutomaton::take_step::step1(state0);
    let state2 = MimeClassifierAutomaton::take_step::step2(state1);

    MimeClassifierAutomaton::show::step1(state0, state1);
    MimeClassifierAutomaton::show::step2(state1, state2);

    let trace = seq![state0, state1, state2];

    assert(
        mime_classify_browsing_result_from_trace(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type_input,
            data,
            sniff_unknown_type_spec(classifier, no_sniff_flag, data),
            trace
        )
    );
}

pub(crate) proof fn lemma_mime_classify_browsing_no_sniff_result<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type_input: &Option<Mime>,
    supplied_type: &Mime,
    data: Seq<u8>,
)
    requires
        *supplied_type_input == Some(*supplied_type),
        !is_xml(supplied_type),
        !is_html(supplied_type),
        !is_explicit_unknown(supplied_type),
        no_sniff_flag == NoSniffFlag::On,
    ensures
        mime_classify_browsing_result(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type_input,
            data,
            view(supplied_type),
        ),
{
    let state0 =
        MimeClassifierAutomaton::take_step::initialize(
            classifier,
            *supplied_type_input,
            no_sniff_flag,
            apache_bug_flag,
            data,
        );

    let state1 = MimeClassifierAutomaton::take_step::step1(state0);
    let state2 = MimeClassifierAutomaton::take_step::step2(state1);
    let state3 = MimeClassifierAutomaton::take_step::step3(state2);

    MimeClassifierAutomaton::show::step1(state0, state1);
    MimeClassifierAutomaton::show::step2(state1, state2);
    MimeClassifierAutomaton::show::step3(state2, state3);

    let trace = seq![state0, state1, state2, state3];

    assert(
        mime_classify_browsing_result_from_trace(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type_input,
            data,
            view(supplied_type),
            trace,
        )
    );
}

pub(crate) proof fn lemma_mime_classify_browsing_apache_bug_result<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type_input: &Option<Mime>,
    supplied_type: &Mime,
    data: Seq<u8>,
)
    requires
        *supplied_type_input == Some(*supplied_type),
        !is_xml(supplied_type),
        !is_html(supplied_type),
        !is_explicit_unknown(supplied_type),
        no_sniff_flag == NoSniffFlag::Off,
        apache_bug_flag == ApacheBugFlag::On,
    ensures
        mime_classify_browsing_result(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type_input,
            data,
            sniff_text_or_data_spec(classifier, data),
        ),
{
    let state0 =
        MimeClassifierAutomaton::take_step::initialize(
            classifier,
            *supplied_type_input,
            no_sniff_flag,
            apache_bug_flag,
            data,
        );

    let state1 = MimeClassifierAutomaton::take_step::step1(state0);
    let state2 = MimeClassifierAutomaton::take_step::step2(state1);
    let state3 = MimeClassifierAutomaton::take_step::step3(state2);
    let state4 = MimeClassifierAutomaton::take_step::step4(state3);

    MimeClassifierAutomaton::show::step1(state0, state1);
    MimeClassifierAutomaton::show::step2(state1, state2);
    MimeClassifierAutomaton::show::step3(state2, state3);
    MimeClassifierAutomaton::show::step4(state3, state4);

    let trace = seq![state0, state1, state2, state3, state4];

    assert(
        mime_classify_browsing_result_from_trace(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            supplied_type_input,
            data,
            sniff_text_or_data_spec(classifier, data),
            trace,
        )
    );

}

pub(crate) proof fn lemma_mime_classify_browsing_after_step4_trace<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Mime,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    data: Seq<u8>,
)
    requires
        !is_xml(supplied_type),
        !is_html(supplied_type),
        !is_explicit_unknown(supplied_type),
        no_sniff_flag == NoSniffFlag::Off,
        apache_bug_flag == ApacheBugFlag::Off,
    ensures
        mime_classify_browsing_result(
            classifier,
            no_sniff_flag,
            apache_bug_flag,
            &Some(*supplied_type),
            data,
            mime_classify_browsing_after_step4(
                classifier,
                supplied_type,
                data
            )
    ),
{
    let result = mime_classify_browsing_after_step4(classifier, supplied_type, data);
    let supplied_type_input = Some(*supplied_type);

    let state0 =
        MimeClassifierAutomaton::take_step::initialize(
            classifier,
            supplied_type_input,
            no_sniff_flag,
            apache_bug_flag,
            data
        );

    let state1 = MimeClassifierAutomaton::take_step::step1(state0);
    let state2 = MimeClassifierAutomaton::take_step::step2(state1);
    let state3 = MimeClassifierAutomaton::take_step::step3(state2);
    let state4 = MimeClassifierAutomaton::take_step::step4(state3);
    let state5 = MimeClassifierAutomaton::take_step::step5(state4);
    let state6 = MimeClassifierAutomaton::take_step::step6(state5);

    MimeClassifierAutomaton::show::step1(state0, state1);
    MimeClassifierAutomaton::show::step2(state1, state2);
    MimeClassifierAutomaton::show::step3(state2, state3);
    MimeClassifierAutomaton::show::step4(state3, state4);
    MimeClassifierAutomaton::show::step5(state4, state5);
    MimeClassifierAutomaton::show::step6(state5, state6);

    if state6.state == MimeTypeSniffState::Final {
        let trace = seq![state0, state1, state2, state3, state4, state5, state6];

        assert(
            mime_classify_browsing_result_from_trace(
                classifier,
                no_sniff_flag,
                apache_bug_flag,
                &supplied_type_input,
                data,
                result,
                trace
            )
        );
    } else {
        let state7 = MimeClassifierAutomaton::take_step::step7(state6);
        let state8 = MimeClassifierAutomaton::take_step::step8(state7);

        MimeClassifierAutomaton::show::step7(state6, state7);
        MimeClassifierAutomaton::show::step8(state7, state8);

        if state8.state == MimeTypeSniffState::Final {
            let trace = seq![state0, state1, state2, state3, state4, state5, state6, state7, state8];
            assert(
                mime_classify_browsing_result_from_trace(
                    classifier,
                    no_sniff_flag,
                    apache_bug_flag,
                    &supplied_type_input,
                    data,
                    result,
                    trace
                )
            );
        } else {
            let state9 = MimeClassifierAutomaton::take_step::step9(state8);

            MimeClassifierAutomaton::show::step9(state8, state9);
            
            let trace = seq![state0, state1, state2, state3, state4, state5, state6, state7, state8, state9];

            assert(
                mime_classify_browsing_result_from_trace(
                    classifier,
                    no_sniff_flag,
                    apache_bug_flag,
                    &supplied_type_input,
                    data,
                    result,
                    trace
                )
            );
        }
    }
}

// ------------------------------------
// Image Content Type Sniffing
// ------------------------------------
// To determine the computed MIME type of a resource with an image MIME type, execute the following 
// rules for sniffing images specifically: 
pub open spec fn sniff_image_context<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
) -> Option<MimeView> {
    // 1. If the supplied MIME type is an XML MIME type, the computed MIME type is the supplied MIME type.
    // Abort these steps.
    // if supplied_type is Some && is_xml(&supplied_type->Some_0) {
    if supplied_type is Some 
        && (is_xml(&supplied_type->Some_0) || is_html(&supplied_type->Some_0)) { //FIXME: Servo behavior
        option_view(supplied_type)
    } else {
        // 2. Let image-type-matched be the result of executing the image type pattern matching algorithm 
        //    with the resource header as the byte sequence to be matched. 
        let image_type_matched = classifier.image_type(data);
        // 3. If image-type-matched is not undefined, the computed MIME type is image-type-matched
        // Abort these steps.
        match image_type_matched {
            Some(mt) => Some(mt),
            None => {
                // 4. The computed MIME type is the supplied MIME type. 
                option_view(supplied_type)
            },
        }
    }
}

pub open spec fn mime_classify_image_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    // Based on Servo Behavior, we assume:
    // If the supplied MIME type is undefined, the computed MIME type is "application/octet-stream". 
    result == match sniff_image_context(classifier, supplied_type, data) {
        Some(mt) => mt,
        None => {
            match supplied_type {
                Some(mt) => view(mt),
                None => application_octet_stream_identity(),
            }
        },
    }
}

pub(crate) proof fn lemma_mime_classify_image_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
)
    requires
        match supplied_type {
            Some(mt) => !is_xml(mt) && !is_html(mt),
            None => true,
        },
    ensures
        mime_classify_image_result(
            classifier,
            supplied_type,
            data,
            result,
        )
        ==
        (
            result ==
                match image_type_pattern_matching_algo(classifier, data) {
                    Some(mt) => mt,
                    None => {
                        match supplied_type {
                            Some(mt) => view(mt),
                            None => application_octet_stream_identity(),
                        }
                    },
                }
        ),
{}

// ------------------------------------
// Audio/Video Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-an-audio-or-video-context
// To determine the computed MIME type of a resource with an audio or video MIME 
// type, execute the following rules for sniffing audio and video specifically: 
pub open spec fn sniff_audio_video_context<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
) -> Option<MimeView> {
    // 1. If the supplied MIME type is an XML MIME type, the computed MIME type is the supplied MIME type.
    //    Abort these steps.
    if supplied_type is Some && 
        (is_xml(&supplied_type->Some_0) || is_html(&supplied_type->Some_0)) {//FIXME: Servo behavior
        option_view(supplied_type)
    } else {
        // 2. Let audio-or-video-type-matched be the result of executing the audio or video type pattern 
        //    matching algorithm with the resource header as the byte sequence to be matched.
        let audio_video_type_matched = classifier.audio_video_type(data);
        // 3. If audio-or-video-type-matched is not undefined, the computed MIME type is 
        //    audio-or-video-type-matched. 
        // Abort these steps.
        match audio_video_type_matched {
            Some(mt) => Some(mt),
            None => {
                // 4. The computed MIME type is the supplied MIME type. 
                option_view(supplied_type)
            },
        }
    }
}

pub open spec fn mime_classify_audio_video_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    // Based on Servo Behavior, we assume:
    // If the supplied MIME type is undefined, the computed MIME type is "application/octet-stream". 
    result == match sniff_audio_video_context(classifier, supplied_type, data) {
        Some(mt) => mt,
        None => {
            match supplied_type {
                Some(mt) => view(mt),
                None => application_octet_stream_identity(),
            }
        },
    }
}

pub(crate) proof fn lemma_mime_classify_audio_video_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
)
    requires
        match supplied_type {
            Some(mt) => !is_xml(mt) && !is_html(mt),
            None => true,
        },
    ensures
        mime_classify_audio_video_result(
            classifier,
            supplied_type,
            data,
            result,
        )
        ==
        (
            result ==
                match audio_or_video_type_pattern_matching_algo(classifier, data) {
                    Some(mt) => mt,
                    None => {
                        match supplied_type {
                            Some(mt) => view(mt),
                            None => application_octet_stream_identity(),
                        }
                    },
                }
        ),
{}

// ------------------------------------
// Plugin Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-plugin-context
// To determine the computed MIME type of a resource fetched in a plugin context, 
// execute the following rules for sniffing in a plugin context: 
pub open spec fn mime_classify_plugin_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    result == match supplied_type {
        // 1. If the supplied MIME type is undefined, the computed MIME type is "application/octet-stream". 
        Some(mt) => view(mt),
        // 2. The computed MIME type is the supplied MIME type. 
        None => application_octet_stream_identity(),
    }
}

// ------------------------------------
// Style Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-style-context
// To determine the computed MIME type of a resource fetched in a style context, execute the following 
// rules for sniffing in a style context: 
pub open spec fn mime_classify_style_result<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    result == match supplied_type {
        // 1. If the supplied MIME type is undefined, …. (follow Servo behavior) 
        None => {
            if no_sniff_flag == NoSniffFlag::On {
                application_octet_stream_identity()
            } else {
                text_css_identity() 
            }
        } 
        // 2. The computed MIME type is the supplied MIME type. 
        Some(mt) => view(mt),
    }
}

// ------------------------------------
// Script Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-script-context
// To determine the computed MIME type of a resource fetched in a script context, 
// execute the following rules for sniffing in a script context: 
pub open spec fn mime_classify_script_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    result == match supplied_type {
        // 1. If the supplied MIME type is undefined, …. (follow Servo behavior) 
        None => {
           text_javascript_identity() 
        } 
        // 2. The computed MIME type is the supplied MIME type. 
        Some(mt) => view(mt),
    }
}

// ------------------------------------
// Font Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-font-context
// To determine the computed MIME type of a resource with a font MIME type, 
// execute the following rules for sniffing fonts specifically: 
pub open spec fn sniff_font_context<C: MimeClassifierModel>(
    classifier: C,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
) -> Option<MimeView> {
    // If the supplied MIME type is an XML MIME type, the computed MIME type is the supplied MIME type.
    // Abort these steps.
    if supplied_type is Some 
        && (is_xml(&supplied_type->Some_0) || is_html(&supplied_type->Some_0)) { //FIXME: Servo behavior
        option_view(supplied_type)
    } else {
        // 2. Let font-type-matched be the result of executing the font type pattern matching 
        //    algorithm with the resource header as the byte sequence to be matched.
        let font_type_matched = classifier.font_type(data);
        match font_type_matched {
            // 3. If font-type-matched is not undefined, the computed MIME type is font-type-matched. 
            // Abort these steps.
            Some(mt) => Some(mt),
            None => {
                // 4. The computed MIME type is the supplied MIME type. 
                option_view(supplied_type)
            },
        }
    }
}

pub open spec fn mime_classify_font_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    // Based on Servo Behavior, we assume:
    // If the supplied MIME type is undefined, the computed MIME type is "application/octet-stream". 
    result == match sniff_font_context(classifier, supplied_type, data) {
        Some(mt) => mt,
        None => {
            match supplied_type {
                Some(mt) => view(mt),
                None => application_octet_stream_identity(),
            }
        },
    }
}

pub(crate) proof fn lemma_mime_classify_font_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
)
    requires
        match supplied_type {
            Some(mt) => !is_xml(mt) && !is_html(mt),
            None => true,
        },
    ensures
        mime_classify_font_result(
            classifier,
            supplied_type,
            data,
            result,
        )
        ==
        (
            result ==
                match font_type_pattern_matching_algo(classifier, data) {
                    Some(mt) => mt,
                    None => {
                        match supplied_type {
                            Some(mt) => view(mt),
                            None => application_octet_stream_identity(),
                        }
                    },
                }
        ),
{}

// ------------------------------------
// TextTrack Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-text-track-context
pub open spec fn mime_classify_text_track_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    result: MimeView,
) -> bool {
    if supplied_type is Some && 
        (is_xml(&supplied_type->Some_0) || is_html(&supplied_type->Some_0)) { // FIXME: Servo behavior
        view(&supplied_type->Some_0) == result
    } else {
        // The computed MIME type is "text/vtt". 
        essence_str_view(&result) == "text/vtt"@
    }
}

// ------------------------------------
// Cache Manifest Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-cache-manifest-context
pub open spec fn mime_classify_cache_manifest_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    result: MimeView,
) -> bool {
    if supplied_type is Some && 
        (is_xml(&supplied_type->Some_0) || is_html(&supplied_type->Some_0)) { // FIXME: Servo behavior
        view(&supplied_type->Some_0) == result
    } else {
        // The computed MIME type is "text/cache-manifest". 
        essence_str_view(&result) == "text/cache-manifest"@
    }
}

} // verus!
