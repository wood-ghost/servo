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

// https://mimesniff.spec.whatwg.org/#image-type-pattern-matching-algorithm
pub(crate) open spec fn image_type_pattern_matching_algo(
    classifier: &MimeClassifier, 
    data: Seq<u8>,
) -> Option<MimeView> {
    // 1. Execute the following steps for each row row in the following table: 
    // 1.1 Let patternMatched be the result of the pattern matching algorithm given input, 
    // the value in the first column of row, the value in the second column of row, and the 
    // value in the third column of row.
    // 1.2 If patternMatched is true, return the value in the fourth column of row.
    // 00 00 01 00 | FF FF FF FF | None | image/x-icon
    if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x00\x00\x01\x00"@,
        b"\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(image_x_icon_identity())
    // 00 00 02 00 | FF FF FF FF | None | image/x-icon
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x00\x00\x02\x00"@,
        b"\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(image_x_icon_identity())

    // 42 4D | FF FF | None | image/bmp
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x42\x4D"@,
        b"\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(image_bmp_identity())

    // 47 49 46 38 37 61 | FF FF FF FF FF FF | None | image/gif
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x47\x49\x46\x38\x37\x61"@,
        b"\xFF\xFF\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(image_gif_identity())

    // 47 49 46 38 39 61 | FF FF FF FF FF FF | None | image/gif
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x47\x49\x46\x38\x39\x61"@,
        b"\xFF\xFF\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(image_gif_identity())

    // 52 49 46 46 00 00 00 00 57 45 42 50 56 50
    // FF FF FF FF 00 00 00 00 FF FF FF FF FF FF
    // None | image/webp
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x52\x49\x46\x46\x00\x00\x00\x00\x57\x45\x42\x50\x56\x50"@,
        b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(image_webp_identity())

    // 89 50 4E 47 0D 0A 1A 0A
    // FF FF FF FF FF FF FF FF | None | image/png
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"@,
        b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(image_png_identity())

    // FF D8 FF | FF FF FF | None | image/jpeg
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\xFF\xD8\xFF"@,
        b"\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(image_jpeg_identity())

    // 2. Return undefined.
    } else {
        None
    } 
}

// https://mimesniff.spec.whatwg.org/#audio-or-video-type-pattern-matching-algorithm
pub(crate) open spec fn audio_or_video_type_pattern_matching_algo(
    classifier: &MimeClassifier, 
    data: Seq<u8>,
) -> Option<MimeView> {
    // | Byte Pattern                       | Pattern Mask                       | Leading Ignored | MIME Type      |
    // |------------------------------------|------------------------------------|-----------------|----------------|
    // | 46 4F 52 4D 00 00 00 00 41 49 46 46| FF FF FF FF 00 00 00 00 FF FF FF FF| None            | audio/aiff     |
    if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x46\x4F\x52\x4D\x00\x00\x00\x00\x41\x49\x46\x46"@,
        b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(audio_aiff_identity())
    // | 49 44 33                           | FF FF FF                           | None            | audio/mpeg     |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x49\x44\x33"@,
        b"\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(audio_mpeg_identity())
    // | 4F 67 67 53 00                     | FF FF FF FF FF                     | None            | application/ogg|
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x4F\x67\x67\x53\x00"@,
        b"\xFF\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(application_ogg_identity())
    // | 4D 54 68 64 00 00 00 06            | FF FF FF FF FF FF FF FF            | None            | audio/midi     |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x4D\x54\x68\x64\x00\x00\x00\x06"@,
        b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(audio_midi_identity())
    // | 52 49 46 46 00 00 00 00 41 56 49 20| FF FF FF FF 00 00 00 00 FF FF FF FF| None            | video/avi      |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x52\x49\x46\x46\x00\x00\x00\x00\x41\x56\x49\x20"@,
        b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(video_avi_identity())
    // | 52 49 46 46 00 00 00 00 57 41 56 45| FF FF FF FF 00 00 00 00 FF FF FF FF| None            | audio/wave     |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x52\x49\x46\x46\x00\x00\x00\x00\x57\x41\x56\x45"@,
        b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(audio_wave_identity())
    // 2. If input matches the signature for MP4, return "video/mp4". 
    } else if SpecMp4Matcher::matches_mp4_signature(data) {
        Some(video_mp4_identity())
    // 3. If input matches the signature for WebM, return "video/webm". 
    } else if matches_webm_signature(data) { 
        Some(video_webm_identity())
    // 4. If input matches the signature for MP3 without ID3, return "audio/mpeg". 
    } else if matches_mp3_without_id3_signature(data) {
        Some(audio_mpeg_identity())
    // 5. Return undefined. 
    } else {
        None
    }
}

// https://mimesniff.spec.whatwg.org/#font-type-pattern-matching-algorithm
pub(crate) open spec fn font_type_pattern_matching_algo(
    classifier: &MimeClassifier, 
    data: Seq<u8>,
) -> Option<MimeView> {
    // | Byte Pattern     | Pattern Mask     | Leading Ignored | MIME Type                     |
    // |------------------|------------------|-----------------|-------------------------------|
    // | (00 × 34) 4C 50  | (00 × 34) FF FF  | None            | application/vnd.ms-fontobject |
    if SpecByteMatcher::pattern_matching_success(
        data, 
        (seq![0x00u8; 34] + seq![0x4Cu8, 0x50u8]),
        (seq![0x00u8; 34] + seq![0xFFu8, 0xFFu8]),
        Set::<u8>::empty(),
    ) {
        Some(application_vnd_ms_fontobject_identity())
    // | 00 01 00 00      | FF FF FF FF      | None            | font/ttf                      |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x00\x01\x00\x00"@,
        b"\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(application_font_sfnt_identity()) //FIXME: font/ttf
    // | 4F 54 54 4F      | FF FF FF FF      | None            | font/otf                      |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x4F\x54\x54\x4F"@,
        b"\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(application_font_sfnt_identity())// FIXME: font/otf
    // | 74 74 63 66      | FF FF FF FF      | None            | font/collection               |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x74\x74\x63\x66"@,
        b"\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(application_font_sfnt_identity())// FIXME: font/collection
    // | 77 4F 46 46      | FF FF FF FF      | None            | font/woff                     |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x77\x4F\x46\x46"@,
        b"\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(application_font_woff_identity())
    // | 77 4F 46 32      | FF FF FF FF      | None            | font/woff2                    |
    } else if SpecByteMatcher::pattern_matching_success(
        data,
        b"\x77\x4F\x46\x32"@,
        b"\xFF\xFF\xFF\xFF"@,
        Set::<u8>::empty(),
    ) {
        Some(application_font_woff2_identity())
    // 2. Return undefined. 
    } else {
        None
    }
}

} // verus!
