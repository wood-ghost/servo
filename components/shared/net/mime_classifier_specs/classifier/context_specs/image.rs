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
    lemma_model_image_type_matches_spec,
};
use crate::mime_classifier_specs::classifier::algorithms::pattern_matching::image_type_pattern_matching_algo;

verus! {

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
    if supplied_type is Some && is_xml(&supplied_type->Some_0) {
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
{
    lemma_model_image_type_matches_spec(classifier, data);
}

} // verus!