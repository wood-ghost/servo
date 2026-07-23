use mime::{Mime, Name};
use vstd::prelude::*;

verus! {

// abstract Mime
pub struct MimeView {
    pub type_: Seq<char>,
    pub subtype: Seq<char>,
    pub suffix: Option<Seq<char>>,
    pub essence: Seq<char>,
}

pub uninterp spec fn view(mt: &Mime) -> MimeView;

// Name
pub open spec fn image_name() -> Seq<char> { "image"@ }
pub open spec fn audio_name() -> Seq<char> { "audio"@ }
pub open spec fn video_name() -> Seq<char> { "video"@ }
pub open spec fn xml_name() -> Seq<char> { "xml"@ }

pub uninterp spec fn name_identity<'a>(name: &Name<'a>,) -> int;
pub uninterp spec fn xml_identity() -> int;
pub uninterp spec fn image_identity() -> int;
pub uninterp spec fn audio_identity() -> int;
pub uninterp spec fn video_identity() -> int;
pub uninterp spec fn application_identity() -> int;
pub uninterp spec fn star_identity() -> int;
pub uninterp spec fn text_identity() -> int;
pub uninterp spec fn json_identity() -> int;
pub uninterp spec fn font_identity() -> int;

// Constant
pub uninterp spec fn text_plain_identity() -> int;
pub uninterp spec fn text_plain_utf8_identity() -> int;
pub uninterp spec fn mime_identity(mt: &Mime) -> int;
pub uninterp spec fn application_octet_stream_identity() -> int;
pub uninterp spec fn text_css() -> int;
pub uninterp spec fn text_javascript() -> int;

// Mime
pub open spec fn essence_str(mt: &Mime) -> Seq<char> {
    view(mt).essence
}
pub uninterp spec fn name_text<'a>(name: Name<'a>) -> Seq<char>;
pub open spec fn suffix_name(mt: &Mime) -> Option<Seq<char>> {
    view(mt).suffix
}
pub uninterp spec fn subtype_name(mt: &Mime) -> Seq<char>;




} // verus!