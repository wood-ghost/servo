use mime::Mime;
use vstd::prelude::*;
use vstd::assert_seqs_equal;

use crate::mime_classifier_specs::mime_api::*;

verus! {

/// Characterize either component using its position in the current essence
/// model. The component string is arbitrary; no MIME table or fixed offset
/// is needed to instantiate this lemma.
pub(crate) broadcast proof fn lemma_mime_essence_parts_str(mt: &Mime, part: &str)
    ensures
        #![trigger essence_str(mt), part@]
        essence_str(mt).len() == view(mt).type_.len() + 1 + view(mt).subtype.len(),
        essence_str(mt)[view(mt).type_.len() as int] == '/',
        (view(mt).type_.len() == part@.len()
            && (forall|i: int| 0 <= i < part@.len() ==>
                #[trigger] part@[i] == essence_str(mt)[i])) ==> view(mt).type_ == part@,
        (view(mt).subtype.len() == part@.len()
            && (forall|i: int| 0 <= i < part@.len() ==>
                #[trigger] part@[i] == essence_str(mt)[view(mt).type_.len() + 1 + i]))
            ==> view(mt).subtype == part@,
{
    if view(mt).type_.len() == part@.len()
        && (forall|i: int| 0 <= i < part@.len() ==>
            #[trigger] part@[i] == essence_str(mt)[i]) {
        assert_seqs_equal!(view(mt).type_ == part@);
    }
    if view(mt).subtype.len() == part@.len()
        && (forall|i: int| 0 <= i < part@.len() ==>
            #[trigger] part@[i] == essence_str(mt)[view(mt).type_.len() + 1 + i]) {
        assert_seqs_equal!(view(mt).subtype == part@);
    }
}

pub(crate) broadcast group mime_essence_parts_str_lemmas {
    lemma_mime_essence_parts_str,
}

pub open spec fn essence_is_text_xml(mt: &Mime) -> bool {
    essence_str(mt) == "text/xml"@
}

pub open spec fn essence_is_application_ogg(mt: &Mime) -> bool {
    essence_str(mt) == "application/ogg"@
}

pub uninterp spec fn has_html_suffix(mt: &Mime) -> bool;

pub open spec fn is_text_plain(mt: &Mime) -> bool {
    essence_str(mt) == "text/plain"@
}
pub open spec fn is_image(mt: &Mime) -> bool {
    view(mt).type_ == image_name()
}
pub open spec fn is_audio(mt: &Mime) -> bool {
    view(mt).type_ == audio_name()
}
pub open spec fn is_video(mt: &Mime) -> bool {
    view(mt).type_ == video_name()
}
pub open spec fn has_xml_suffix(mt: &Mime) -> bool {
    view(mt).suffix == Some(xml_name())
}

pub open spec fn is_xml(mt: &Mime) -> bool {
    !is_image(mt) && (
        has_xml_suffix(mt)
            || essence_is_text_xml(mt)
            || (essence_str(mt) == "application/xml"@)
    )
} 

pub open spec fn is_html(mt: &Mime) -> bool {
    essence_str(mt) == "text/html"@
}

pub open spec fn is_audio_video(mt: &Mime) -> bool {
    is_audio(mt) || is_video(mt) || essence_is_application_ogg(mt)
}

pub(crate) broadcast proof fn lemma_image_audio_video_disjoint(mt: &Mime)
    ensures
        #[trigger] is_image(mt) ==> !is_audio_video(mt),
        #[trigger] is_audio_video(mt) ==> !is_image(mt),
{
    assert("image"@ != "audio"@) by {
        if "image"@ == "audio"@ {
            assert("image"@[0] == "audio"@[0]);
            assert(false);
        }
    }

    assert("image"@ != "video"@) by {
        if "image"@ == "video"@ {
            assert("image"@[0] == "video"@[0]);
            assert(false);
        }
    }

    if is_image(mt) {
        assert(!essence_is_application_ogg(mt)) by {
            if essence_is_application_ogg(mt) {
                assert(
                    (view(mt).type_ + "/"@ + view(mt).subtype)[0]
                        == "application/ogg"@[0]
                );

                assert(false);
            }
        }

        assert(!is_audio_video(mt));
    }

    if is_audio_video(mt) {
        if is_image(mt) {
            assert(false);
        }
    }
}

/// <https://mimesniff.spec.whatwg.org/#javascript-mime-type>
pub open spec fn is_javascript(mt: &Mime) -> bool {
    ||| essence_str(mt) == "application/ecmascript"@
    ||| essence_str(mt) == "application/javascript"@
    ||| essence_str(mt) == "application/x-ecmascript"@
    ||| essence_str(mt) == "application/x-javascript"@
    ||| essence_str(mt) == "text/ecmascript"@
    ||| essence_str(mt) == "text/javascript"@
    ||| essence_str(mt) == "text/javascript1.0"@
    ||| essence_str(mt) == "text/javascript1.1"@
    ||| essence_str(mt) == "text/javascript1.2"@
    ||| essence_str(mt) == "text/javascript1.3"@
    ||| essence_str(mt) == "text/javascript1.4"@
    ||| essence_str(mt) == "text/javascript1.5"@
    ||| essence_str(mt) == "text/jscript"@
    ||| essence_str(mt) == "text/livescript"@
    ||| essence_str(mt) == "text/x-ecmascript"@
    ||| essence_str(mt) == "text/x-javascript"@
}

/// <https://mimesniff.spec.whatwg.org/#font-mime-type>
pub open spec fn is_font(mt: &Mime) -> bool { 
    ||| view(mt).type_ == font_name()
    ||| essence_str(mt) == "application/font-cff"@
    // https://github.com/servo/servo/issues/47903
    ||| essence_str(mt) == "application/font-off"@ //TODO: new version is font-otf
    ||| essence_str(mt) == "application/font-sfnt"@
    ||| essence_str(mt) == "application/font-ttf"@
    ||| essence_str(mt) == "application/font-woff"@
    ||| essence_str(mt) == "application/vnd.ms-fontobject"@
    ||| essence_str(mt) == "application/vnd.ms-opentype"@
}

/// <https://mimesniff.spec.whatwg.org/#json-mime-type>
pub open spec fn is_json(mt: &Mime) -> bool { 
    ||| view(mt).suffix == Some(json_name())@ 
    ||| essence_str(mt) == "application/json"@
    ||| essence_str(mt) == "text/json"@
}

pub open spec fn is_text(mt: &Mime) -> bool { 
    view(mt) == text_plain_identity()
    || essence_str(mt) == "text/vtt"@
}

pub open spec fn is_css(mt: &Mime) -> bool {
    essence_str(mt) == "text/css"@
}

pub open spec fn is_explicit_unknown(mt: &Mime) -> bool {
    ||| essence_str(mt) == "application/unknown"@
    ||| essence_str(mt) == "unknown/unknown"@
    ||| essence_str(mt) == "*/*"@
}


} // verus!
