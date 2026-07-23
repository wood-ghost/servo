use mime::Mime;
use vstd::prelude::*;

use crate::LoadContext;

verus! {
// check if the resource is retrieved via HTTP
pub(crate) open spec fn is_xml(mt: &Mime) -> bool {
    !SpecMime::is_image(mt) && (
        SpecMime::has_xml_suffix(mt)
            || SpecMime::essence_is_text_xml(mt)
            || SpecMime::essence_is_application_xml(mt))
} 

pub(crate) open spec fn is_html(mt: &Mime) -> bool {
    SpecMime::essence_is_text_html(mt)
}

pub(crate) open spec fn is_image(mt: &Mime) -> bool {
    SpecMime::is_image(mt)
}

pub(crate) open spec fn is_audio_video(mt: &Mime) -> bool {
    SpecMime::is_audio(mt) || SpecMime::is_video(mt) || SpecMime::essence_is_application_ogg(mt)
}

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

pub open spec fn classify<'a>(
    context: LoadContext,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
    supplied_type: &Option<Mime>,
    data: &'a [u8],
) -> Mime {
    // TODO:
    SpecMime::dummy_mime()
}

} // verus!
