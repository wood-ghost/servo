use mime::Mime;
use vstd::prelude::*;

use super::model::*;

verus! {

pub(crate) open spec fn essence_is_text_xml(mt: &Mime) -> bool {
    // view(mt).essence =~= "text/xml"@
    essence_str(mt) == "text/xml"@
}

pub(crate) open spec fn essence_is_application_ogg(mt: &Mime) -> bool {
    // view(mt).essence =~= "application/ogg"@
    essence_str(mt) == "application/ogg"@
}

pub(crate) uninterp spec fn has_html_suffix(mt: &Mime) -> bool;
pub open spec fn is_text_plain(mt: &Mime) -> bool {
    view(mt) == text_plain_identity()
}
pub open spec fn is_text_plain_utf8(mt: &Mime) -> bool {
    view(mt) == text_plain_utf8_identity()
}
pub open spec fn is_image(mt: &Mime) -> bool {
    view(mt).type_ == "image"@ 
}
pub open spec fn is_audio(mt: &Mime) -> bool {
    view(mt).type_ == "audio"@ 
}
pub open spec fn is_video(mt: &Mime) -> bool {
    view(mt).type_ == "video"@ 
}
pub open spec fn has_xml_suffix(mt: &Mime) -> bool {
    match view(mt).suffix {
        Some(suffix) => suffix == "xml"@,
        None => false,
    }
}

pub(crate) open spec fn is_xml(mt: &Mime) -> bool {
    !is_image(mt) && (
        has_xml_suffix(mt)
            || essence_is_text_xml(mt)
            || (essence_str(mt) == "application/xml"@)
    )
} 

pub(crate) open spec fn is_html(mt: &Mime) -> bool {
    essence_str(mt) =~= "text/html"@
}

pub(crate) open spec fn is_audio_video(mt: &Mime) -> bool {
    is_audio(mt) || is_video(mt) || essence_is_application_ogg(mt)
}


} // verus!