use mime::Mime;
use vstd::prelude::*;
// use verus_state_machines_macros::state_machine;
use verus_state_machines_macros::{
    state_machine,
    case_on_next,
};

use crate::mime_classifier::{
    ApacheBugFlag, 
    NoSniffFlag,
    GroupedClassifier,
    MimeClassifier,
};

// use crate::mime_classifier::MIMEChecker;
use crate::mime_classifier_specs::mime_api::*;
use crate::mime_classifier_specs::predicates::{
    is_xml,
    is_html,
    is_image,
    is_audio_video,
    is_explicit_unknown,
};
use crate::mime_classifier_specs::classifier::checker_trait::{
    MIMECheckerSpec,
    MimeClassifierModel,
    lemma_model_sniff_unknown_type_matches_spec,
    lemma_model_sniff_text_or_data_matches_spec,
    lemma_model_image_type_matches_spec,
    lemma_model_audio_video_type_matches_spec,
};
use crate::mime_classifier_specs::classifier::algorithms::mime_type_sniffing::{
    MimeClassifierAutomaton,
    MimeTypeSniffState,
    mime_type_sniffing_trace,
    mime_type_sniffing_result,
};
use crate::mime_classifier_specs::classifier::algorithms::{
    sniff_unknown_type_spec,
    sniff_text_or_data_spec,
};
use crate::mime_classifier_specs::requires as SpecRequires;

verus! {

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

    lemma_model_sniff_unknown_type_matches_spec(classifier, no_sniff_flag, data);

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

    lemma_model_sniff_unknown_type_matches_spec(classifier, no_sniff_flag, data);

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

    lemma_model_sniff_text_or_data_matches_spec(classifier, data);

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
        SpecRequires::image_classifier_matches_whatwg(classifier, data), // for servo behavior
        SpecRequires::audio_or_video_classifier_matches_whatwg(classifier, data), // for servo behavior
        SpecRequires::font_classifier_matches_whatwg(classifier, data), // for servo behavior
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

    lemma_model_image_type_matches_spec(classifier, data);
    lemma_model_audio_video_type_matches_spec(classifier, data);
    SpecRequires::lemma_image_classifier_matches_whatwg(classifier, data);
    SpecRequires::lemma_audio_or_video_classifier_matches_whatwg(classifier, data);

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

} // verus!
