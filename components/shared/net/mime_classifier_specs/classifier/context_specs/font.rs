use mime::Mime;
use vstd::prelude::*;

use crate::mime_classifier::{
    MimeClassifier,
};

// use crate::mime_classifier::MIMEChecker;
use crate::mime_classifier_specs::mime_api::*;
use crate::mime_classifier_specs::predicates::{
    is_xml,
    is_html,
};
use crate::mime_classifier_specs::classifier::checker_trait::{
    MimeClassifierModel,
    lemma_model_font_type_matches_spec,
};
use crate::mime_classifier_specs::classifier::algorithms::pattern_matching::font_type_pattern_matching_algo;

verus! {

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
    if supplied_type is Some && is_xml(&supplied_type->Some_0) {
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
{
    lemma_model_font_type_matches_spec(classifier, data);
}

} // verus!