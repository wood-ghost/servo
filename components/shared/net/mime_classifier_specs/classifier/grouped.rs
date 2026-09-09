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
use super::checker_trait::MIMECheckerSpec;
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

pub closed spec fn mime_classifier_validate_spec(classifier: &MimeClassifier) -> bool {
    &&& classifier.image_classifier.validate_spec()
    &&& classifier.audio_video_classifier.validate_spec()
    &&& classifier.scriptable_classifier.validate_spec()
    &&& classifier.plaintext_classifier.validate_spec()
    &&& classifier.archive_classifier.validate_spec()
    &&& classifier.binary_or_plaintext.validate_spec()
    &&& classifier.font_classifier.validate_spec()
}

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

} // verus!
