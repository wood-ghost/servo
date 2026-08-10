use mime::Mime;
use vstd::prelude::*;

use crate::LoadContext;
use crate::mime_classifier::{MediaType, ApacheBugFlag, NoSniffFlag};

use crate::mime_classifier::ByteMatcher;
use super::byte_matcher as SpecByteMatcher;

// use crate::mime_classifier_specs::{
//     is_image,
//     has_xml_suffix,
//     essence_is_text_xml,
//     essence_is_application_xml,
//     is_audio,
//     is_video,
//     essence_is_application_ogg,
// };

verus! {

pub open spec fn classify_input_is_valid<'a>(
    context: LoadContext,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: &'a [u8],
) -> bool {
    // TODO:
    true
}

pub uninterp spec fn dummy_mime() -> Mime;
pub open spec fn classify<'a>(
    context: LoadContext,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: &'a [u8],
) -> Mime {
    // TODO:
    dummy_mime()
}

pub(crate) open spec fn all_valid(matchers: Seq<ByteMatcher>) -> bool {
    forall |i: int| #![trigger matchers[i]] 0 <= i < matchers.len() ==>
            SpecByteMatcher::validate_ok(matchers[i].pattern@, matchers[i].mask@)
}

} // verus!
