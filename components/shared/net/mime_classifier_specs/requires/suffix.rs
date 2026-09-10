use mime::Mime;
use vstd::prelude::*;

use crate::LoadContext;
use crate::mime_classifier_specs::mime_api::suffix;
use crate::mime_classifier_specs::predicates::{
    is_xml, is_html, is_image, is_audio_video, is_javascript,
};

verus! {

pub open spec fn is_font_requires(mime: &Mime) -> bool {
    suffix(mime).is_none()
}

pub open spec fn is_javascript_requires(mime: &Mime) -> bool {
    suffix(mime).is_none()
}

pub open spec fn is_explicit_unknown_requires(mime: &Mime) -> bool {
    suffix(mime).is_none()
}

// The executable JavaScript and font predicates require suffix-free inputs
// when their respective branches are reached.
pub open spec fn get_media_type_requires(mime: &Mime) -> bool {
    &&& (!is_xml(mime) && !is_html(mime) && !is_image(mime)
        && !is_audio_video(mime))
        ==> is_javascript_requires(mime)
    &&& (!is_xml(mime) && !is_html(mime) && !is_image(mime)
        && !is_audio_video(mime) && !is_javascript(mime))
        ==> is_font_requires(mime)
}

pub open spec fn maybe_get_media_type_requires(supplied_type: &Option<Mime>) -> bool {
    match supplied_type {
        Some(mt) => get_media_type_requires(mt),
        None => true,
    }
}

pub open spec fn classify_suffix_requires(
    context: LoadContext, supplied_type: &Option<Mime>,
) -> bool {
    &&& (context == LoadContext::Browsing || context == LoadContext::Image
        || context == LoadContext::AudioVideo || context == LoadContext::Font)
        ==> maybe_get_media_type_requires(supplied_type)
    // Browsing checks for an explicitly unknown MIME type after the XML/HTML exit.
    &&& context == LoadContext::Browsing ==> match supplied_type {
        Some(mt) => (!is_xml(mt) && !is_html(mt))
            ==> is_explicit_unknown_requires(mt),
        None => true,
    }
}

} // verus!
