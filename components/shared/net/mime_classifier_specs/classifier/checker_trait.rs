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
use super::algorithms::{
    sniff_unknown_type_spec,
    sniff_text_or_data_spec,
    image_type_pattern_matching_algo,
    audio_or_video_type_pattern_matching_algo,
    font_type_pattern_matching_algo,
};
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

pub(crate) trait MIMECheckerSpec {
    spec fn classify_spec(&self, data: Seq<u8>) -> Option<MimeView>;
    spec fn validate_spec(&self) -> bool;
}

pub trait MimeClassifierModel {
    spec fn sniff_unknown_type(&self, no_sniff_flag: NoSniffFlag, data: Seq<u8>) -> MimeView;

    spec fn sniff_text_or_data(&self, data: Seq<u8>) -> MimeView;

    spec fn image_type(&self, data: Seq<u8>) -> Option<MimeView>;

    spec fn audio_video_type(&self, data: Seq<u8>) -> Option<MimeView>;

    spec fn font_type(&self, data: Seq<u8>) -> Option<MimeView>;
}

impl<'a> MimeClassifierModel for &'a MimeClassifier {
    closed spec fn sniff_unknown_type(&self, no_sniff_flag: NoSniffFlag, data: Seq<u8>) -> MimeView {
        sniff_unknown_type_spec(*self, no_sniff_flag, data)
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

pub(crate) proof fn lemma_model_sniff_unknown_type_matches_spec<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    data: Seq<u8>,
)
    ensures
        <&'a MimeClassifier as MimeClassifierModel>::sniff_unknown_type(&classifier, no_sniff_flag, data)
            == sniff_unknown_type_spec(classifier, no_sniff_flag, data),
{
}

pub(crate) proof fn lemma_model_sniff_text_or_data_matches_spec<'a>(
    classifier: &'a MimeClassifier,
    data: Seq<u8>,
)
    ensures
        <&'a MimeClassifier as MimeClassifierModel>::sniff_text_or_data(&classifier, data)
            == sniff_text_or_data_spec(classifier, data),
{
}

pub(crate) proof fn lemma_model_image_type_matches_spec<'a>(
    classifier: &'a MimeClassifier,
    data: Seq<u8>,
)
    ensures
        <&'a MimeClassifier as MimeClassifierModel>::image_type(&classifier, data)
            == image_type_pattern_matching_algo(classifier, data),
{
}

pub(crate) proof fn lemma_model_audio_video_type_matches_spec<'a>(
    classifier: &'a MimeClassifier,
    data: Seq<u8>,
)
    ensures
        <&'a MimeClassifier as MimeClassifierModel>::audio_video_type(&classifier, data)
            == audio_or_video_type_pattern_matching_algo(classifier, data),
{
}

pub(crate) proof fn lemma_model_font_type_matches_spec<'a>(
    classifier: &'a MimeClassifier,
    data: Seq<u8>,
)
    ensures
        <&'a MimeClassifier as MimeClassifierModel>::font_type(&classifier, data)
            == font_type_pattern_matching_algo(classifier, data),
{
}

} // verus!
