use vstd::prelude::*;

use mime::Mime;
use crate::LoadContext;
use crate::mime_classifier::{ApacheBugFlag, GroupedClassifier, MimeClassifier, NoSniffFlag};
use crate::mime_classifier_specs::classifier::checker_trait::MIMECheckerSpec;
use crate::mime_classifier_specs::predicates::{is_audio_video, is_explicit_unknown, is_html, is_image, is_xml};
use crate::mime_classifier_specs::classifier::algorithms::{
    image_type_pattern_matching_algo,
    audio_or_video_type_pattern_matching_algo,
    font_type_pattern_matching_algo,
};

verus! {

/// The current `find_map` contract requires the closure's precondition for every
/// iterator element, including elements after the first successful match.
pub(crate) open spec fn grouped_classify_requires(
    classifier: &GroupedClassifier,
    data: Seq<u8>,
) -> bool {
    forall |i: int| 0 <= i < classifier.byte_matchers@.len() ==>
        #[trigger] classifier.byte_matchers@[i].dyn_classify_requires(data)
}

pub(crate) open spec fn sniff_unknown_type_requires(
    classifier: &MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    data: Seq<u8>,
) -> bool {
    &&& no_sniff_flag == NoSniffFlag::Off ==>
        classifier.scriptable_classifier.classify_requires(data)
    &&& (no_sniff_flag == NoSniffFlag::On
        || classifier.scriptable_classifier.classify_spec(data).is_none()) ==> {
        &&& classifier.plaintext_classifier.classify_requires(data)
        &&& classifier.plaintext_classifier.classify_spec(data).is_none() ==> {
            &&& classifier.image_classifier.classify_requires(data)
            &&& classifier.image_classifier.classify_spec(data).is_none() ==> {
                &&& classifier.audio_video_classifier.classify_requires(data)
                &&& classifier.audio_video_classifier.classify_spec(data).is_none() ==>
                    classifier.archive_classifier.classify_requires(data)
            }
        }
    }
}

/// Require input safety only for the classifier groups selected by this call.
/// Configuration validation and WHATWG agreement remain separate requirements.
pub closed spec fn classify_input_requires(
    classifier: &MimeClassifier,
    context: LoadContext,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
) -> bool {
    classify_input_requires_body(
        classifier, context, no_sniff_flag, apache_bug_flag, supplied_type, data,
    )
}

pub(crate) proof fn lemma_classify_input_requires(
    classifier: &MimeClassifier,
    context: LoadContext,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
)
    ensures
        classify_input_requires(classifier, context, no_sniff_flag, apache_bug_flag, supplied_type, data)
            == classify_input_requires_body(classifier, context, no_sniff_flag, apache_bug_flag, supplied_type, data),
{}

pub(crate) open spec fn classify_input_requires_body(
    classifier: &MimeClassifier,
    context: LoadContext,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
) -> bool {
    if match supplied_type {
        Some(mt) => is_xml(mt) || is_html(mt),
        None => false,
    } {
        true
    } else {
        match context {
            LoadContext::Browsing => match supplied_type {
                None => sniff_unknown_type_requires(classifier, no_sniff_flag, data),
                Some(mt) => {
                    if is_explicit_unknown(mt) {
                        sniff_unknown_type_requires(classifier, no_sniff_flag, data)
                    } else if no_sniff_flag == NoSniffFlag::On
                        || apache_bug_flag == ApacheBugFlag::On {
                        true
                    } else if is_image(mt) {
                        classifier.image_classifier.classify_requires(data)
                    } else if is_audio_video(mt) {
                        classifier.audio_video_classifier.classify_requires(data)
                    } else {
                        true
                    }
                },
            },
            LoadContext::Image => classifier.image_classifier.classify_requires(data),
            LoadContext::AudioVideo => classifier.audio_video_classifier.classify_requires(data),
            LoadContext::Font => classifier.font_classifier.classify_requires(data),
            _ => true,
        }
    }
}

// ============================================================================
// Combined classifier requirements and unfolding lemma
// ============================================================================

pub open spec fn classifiers_match_whatwg(
    classifier: &MimeClassifier,
    data: Seq<u8>,
) -> bool {
    &&& image_classifier_matches_whatwg(classifier, data)
    &&& audio_or_video_classifier_matches_whatwg(classifier, data)
    &&& font_classifier_matches_whatwg(classifier, data)
}

pub(crate) broadcast proof fn lemma_classifiers_match_whatwg(
    classifier: &MimeClassifier,
    data: Seq<u8>,
)
    requires
        #[trigger] classifiers_match_whatwg(classifier, data),
    ensures
        classifier.image_classifier.classify_spec(data)
            == image_type_pattern_matching_algo(classifier, data),
        classifier.audio_video_classifier.classify_spec(data)
            == audio_or_video_type_pattern_matching_algo(classifier, data),
        classifier.font_classifier.classify_spec(data)
            == font_type_pattern_matching_algo(classifier, data),
{
    lemma_image_classifier_matches_whatwg(classifier, data);
    lemma_audio_or_video_classifier_matches_whatwg(classifier, data);
    lemma_font_classifier_matches_whatwg(classifier, data);
}

// ============================================================================
// Image classifier: agreement with the WHATWG pattern-matching specification
// ============================================================================

pub closed spec fn image_classifier_matches_whatwg(
    classifier: &MimeClassifier,
    data: Seq<u8>,
) -> bool {
    classifier.image_classifier.classify_spec(data)
        == image_type_pattern_matching_algo(classifier, data)
}

pub(crate) proof fn lemma_image_classifier_matches_whatwg(
    classifier: &MimeClassifier,
    data: Seq<u8>,
)
    requires
        image_classifier_matches_whatwg(classifier, data),
    ensures
        classifier.image_classifier.classify_spec(data)
            == image_type_pattern_matching_algo(classifier, data),
{}

// ============================================================================
// Audio/video classifier: agreement with the WHATWG pattern-matching specification
// ============================================================================

pub closed spec fn audio_or_video_classifier_matches_whatwg(
    classifier: &MimeClassifier,
    data: Seq<u8>,
) -> bool {
    classifier.audio_video_classifier.classify_spec(data)
        == audio_or_video_type_pattern_matching_algo(classifier, data)
}

pub(crate) proof fn lemma_audio_or_video_classifier_matches_whatwg(
    classifier: &MimeClassifier,
    data: Seq<u8>,
)
    requires
        audio_or_video_classifier_matches_whatwg(classifier, data),
    ensures
        classifier.audio_video_classifier.classify_spec(data)
            == audio_or_video_type_pattern_matching_algo(classifier, data),
{}

// ============================================================================
// Font classifier: agreement with the WHATWG pattern-matching specification
// ============================================================================

pub closed spec fn font_classifier_matches_whatwg(
    classifier: &MimeClassifier,
    data: Seq<u8>,
) -> bool {
    classifier.font_classifier.classify_spec(data)
        == font_type_pattern_matching_algo(classifier, data)
}

pub(crate) proof fn lemma_font_classifier_matches_whatwg(
    classifier: &MimeClassifier,
    data: Seq<u8>,
)
    requires
        font_classifier_matches_whatwg(classifier, data),
    ensures
        classifier.font_classifier.classify_spec(data)
            == font_type_pattern_matching_algo(classifier, data),
{}

} // verus!
