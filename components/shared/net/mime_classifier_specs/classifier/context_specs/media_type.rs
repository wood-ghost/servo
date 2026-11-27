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
    MimeClassifier,
};

// use crate::mime_classifier::MIMEChecker;
use crate::mime_classifier_specs::byte_matcher as SpecByteMatcher;
use crate::mime_classifier_specs::mp4_matcher as SpecMp4Matcher;
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
};
use crate::mime_classifier_specs::classifier::context_specs::browsing::mime_classify_browsing_result;
use crate::mime_classifier_specs::classifier::context_specs::image::mime_classify_image_result;
use crate::mime_classifier_specs::classifier::context_specs::audio_video::mime_classify_audio_video_result;
use crate::mime_classifier_specs::classifier::context_specs::font::mime_classify_font_result;
use crate::mime_classifier_specs::classifier::context_specs::others::{
    mime_classify_plugin_result,
    mime_classify_style_result,
    mime_classify_script_result,
    mime_classify_text_track_result,
    mime_classify_cache_manifest_result,
};

verus! {

pub open spec fn get_media_type_spec(mime: &Mime) -> Option<MediaType> {
    if is_xml(mime) {
        Some(MediaType::Xml)
    } else if is_html(mime) {
        Some(MediaType::Html)
    } else if is_image(mime) {
        Some(MediaType::Image)
    } else if is_audio_video(mime) {
        Some(MediaType::AudioVideo)
    } else if is_javascript(mime) {
        Some(MediaType::JavaScript)
    } else if is_font(mime) {
        Some(MediaType::Font)
    } else if is_json(mime) {
        Some(MediaType::Json)
    } else if is_text(mime) {
        Some(MediaType::Text)
    } else if is_css(mime) {
        Some(MediaType::Css)
    } else {
        None
    }
}

// ------------------------------------
// MIME Classifier
// ------------------------------------
pub open spec fn mime_classifier_classify_spec(
    classifier: &MimeClassifier,
    context: LoadContext,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    match context {
        LoadContext::Browsing =>
            mime_classify_browsing_result(classifier, no_sniff_flag, apache_bug_flag, supplied_type, data, result),
        LoadContext::Image =>
            mime_classify_image_result(classifier, supplied_type, data, result),
        LoadContext::AudioVideo =>
            mime_classify_audio_video_result(classifier, supplied_type, data, result),
        LoadContext::Plugin =>
            mime_classify_plugin_result(classifier, supplied_type, data, result),
        LoadContext::Style =>
            mime_classify_style_result(classifier, no_sniff_flag, supplied_type, data, result),
        LoadContext::Script =>
            mime_classify_script_result(classifier, supplied_type, data, result),
        LoadContext::Font =>
            mime_classify_font_result(classifier, supplied_type, data, result),
        LoadContext::TextTrack =>
            mime_classify_text_track_result(classifier, supplied_type,  result), // supplied_type for Servo behavior
        LoadContext::CacheManifest =>
            mime_classify_cache_manifest_result(classifier, supplied_type, result), // supplied_type for Servo behavior
    }
}

} // verus!
