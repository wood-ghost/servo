use mime::Mime;
use vstd::prelude::*;

use crate::LoadContext;
use crate::mime_classifier_specs::mime_api::view;
use crate::mime_classifier_specs::predicates::{
    is_xml, is_html, is_image, is_audio_video, is_javascript, is_font,
};

verus! {

pub open spec fn is_xml_requires(mime: &Mime) -> bool {
    &&& view(mime).type_ == "text"@ && view(mime).subtype == "xml"@
        ==> view(mime).suffix.is_none()
    &&& view(mime).type_ == "application"@ && view(mime).subtype == "xml"@
        ==> view(mime).suffix.is_none()
}

pub open spec fn is_html_requires(mime: &Mime) -> bool {
    view(mime).type_ == "text"@ && view(mime).subtype == "html"@
        ==> view(mime).suffix.is_none()
}

pub open spec fn is_font_requires(mime: &Mime) -> bool {
    view(mime).type_ == "application"@
        && (view(mime).subtype == "font-cff"@
            || view(mime).subtype == "font-off"@
            || view(mime).subtype == "font-sfnt"@
            || view(mime).subtype == "font-ttf"@
            || view(mime).subtype == "font-woff"@
            || view(mime).subtype == "vnd.ms-fontobject"@
            || view(mime).subtype == "vnd.ms-opentype"@)
        ==> view(mime).suffix.is_none()
}

pub open spec fn is_javascript_requires(mime: &Mime) -> bool {
    &&& view(mime).type_ == "application"@
        && (view(mime).subtype == "ecmascript"@
            || view(mime).subtype == "javascript"@
            || view(mime).subtype == "x-ecmascript"@
            || view(mime).subtype == "x-javascript"@)
        ==> view(mime).suffix.is_none()
    &&& view(mime).type_ == "text"@
        && (view(mime).subtype == "ecmascript"@
            || view(mime).subtype == "javascript"@
            || view(mime).subtype == "javascript1.0"@
            || view(mime).subtype == "javascript1.1"@
            || view(mime).subtype == "javascript1.2"@
            || view(mime).subtype == "javascript1.3"@
            || view(mime).subtype == "javascript1.4"@
            || view(mime).subtype == "javascript1.5"@
            || view(mime).subtype == "jscript"@
            || view(mime).subtype == "livescript"@
            || view(mime).subtype == "x-ecmascript"@
            || view(mime).subtype == "x-javascript"@)
        ==> view(mime).suffix.is_none()
}

pub open spec fn is_explicit_unknown_requires(mime: &Mime) -> bool {
    &&& view(mime).type_ == "unknown"@ && view(mime).subtype == "unknown"@
        ==> view(mime).suffix.is_none()
    &&& view(mime).type_ == "application"@ && view(mime).subtype == "unknown"@
        ==> view(mime).suffix.is_none()
    &&& view(mime).type_ == "*"@ && view(mime).subtype == "*"@
        ==> view(mime).suffix.is_none()
}

pub open spec fn is_audio_video_requires(mime: &Mime) -> bool {
    view(mime).type_ == "application"@ && view(mime).subtype == "ogg"@ ==> view(mime).suffix.is_none()
}

pub open spec fn is_json_requires(mime: &Mime) -> bool {
    &&& view(mime).type_ == "application"@ && view(mime).subtype == "json"@
        ==> view(mime).suffix.is_none()
    &&& view(mime).type_ == "text"@ && view(mime).subtype == "json"@
        ==> view(mime).suffix.is_none()
}

// Apply each predicate's suffix requirement when its branch is reached.
pub open spec fn get_media_type_requires(mime: &Mime) -> bool {
    &&& is_xml_requires(mime)
    &&& !is_xml(mime) ==> is_html_requires(mime)
    &&& (!is_xml(mime) && !is_html(mime) && !is_image(mime))
        ==> is_audio_video_requires(mime)
    &&& (!is_xml(mime) && !is_html(mime) && !is_image(mime)
        && !is_audio_video(mime))
        ==> is_javascript_requires(mime)
    &&& (!is_xml(mime) && !is_html(mime) && !is_image(mime)
        && !is_audio_video(mime) && !is_javascript(mime))
        ==> is_font_requires(mime)
    &&& (!is_xml(mime) && !is_html(mime) && !is_image(mime)
        && !is_audio_video(mime) && !is_javascript(mime) && !is_font(mime))
        ==> is_json_requires(mime)
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
    // Every context checks XML, then HTML if XML does not match.
    &&& match supplied_type {
        Some(mt) => is_xml_requires(mt)
            && (!is_xml(mt) ==> is_html_requires(mt)),
        None => true,
    }
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
