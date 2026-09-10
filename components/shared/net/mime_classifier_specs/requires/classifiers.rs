use vstd::prelude::*;

use crate::mime_classifier::MimeClassifier;
use crate::mime_classifier_specs::classifier::checker_trait::MIMECheckerSpec;
use crate::mime_classifier_specs::classifier::algorithms::{
    image_type_pattern_matching_algo,
    audio_or_video_type_pattern_matching_algo,
    font_type_pattern_matching_algo,
};

verus! {

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
