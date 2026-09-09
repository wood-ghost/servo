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
use crate::mime_classifier_specs::classifier::checker_trait::MIMECheckerSpec;
use crate::mime_classifier_specs::classifier::grouped::bin_or_plain_classify_spec;
use super::pattern_matching::{
    image_type_pattern_matching_algo,
    audio_or_video_type_pattern_matching_algo,
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

// https://mimesniff.spec.whatwg.org/#rules-for-identifying-an-unknown-mime-type
// To determine the computed MIME type of a resource resource with an unknown MIME type, 
// execute the following rules for identifying an unknown MIME type: 
pub(crate) open spec fn sniff_unknown_type_spec(
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
        None => image_type_pattern_matching_algo(classifier, data), 
    };
    let matched_type = match matched_type {
        // 4. If matchedType is not undefined, return matchedType. 
        Some(mt) => Some(mt),
        // 5. Set matchedType to the result of executing the audio or video type pattern 
        //    matching algorithm given resource’s resource header. 
        // None => classifier.audio_video_classifier.classify_spec(data),
        None => audio_or_video_type_pattern_matching_algo(classifier, data),
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

// https://mimesniff.spec.whatwg.org/#rules-for-text-or-binary
pub(crate) open spec fn sniff_text_or_data_spec(
    classifier: &MimeClassifier,
    data: Seq<u8>,
) -> MimeView {
    bin_or_plain_classify_spec(data)
} 

} // verus!
