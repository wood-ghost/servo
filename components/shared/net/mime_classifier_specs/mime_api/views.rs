use mime::{self, Mime, Name};
use vstd::prelude::*;

verus! {

// api

// abstract Mime
pub struct MimeView {
    pub type_: Seq<char>,
    pub subtype: Seq<char>,
    pub suffix: Option<Seq<char>>,
    // pub essence: Seq<char>,
    pub params: Map<Seq<char>, Seq<char>>,
}

pub uninterp spec fn view(mt: &Mime) -> MimeView;
pub open spec fn option_view(value: &Option<Mime>) -> Option<MimeView> {
    match value {
        Some(mt) => Some(view(mt)),
        None => None,
    }
}

// Name
pub uninterp spec fn name_identity<'a>(name: &Name<'a>,) -> Seq<char>;

pub open spec fn image_name() -> Seq<char> { "image"@ }
pub open spec fn audio_name() -> Seq<char> { "audio"@ }
pub open spec fn video_name() -> Seq<char> { "video"@ }
pub open spec fn xml_name() -> Seq<char> { "xml"@ }
pub open spec fn application_name() -> Seq<char> { "application"@ }
pub open spec fn star_name() -> Seq<char> { "*"@ }
pub open spec fn text_name() -> Seq<char> { "text"@ }
pub open spec fn json_name() -> Seq<char> { "json"@ }
pub open spec fn font_name() -> Seq<char> { "font"@ }

// Mime
// pub uninterp spec fn mime_identity(mt: &Mime) -> MimeView;

// https://docs.rs/mime/latest/mime/struct.Mime.html#method.essence_str
pub open spec fn essence_str(mt: &Mime) -> Seq<char> {
    // match(view(mt).suffix) {
    //     Some(suffix) => view(mt).type_ + "/"@ + view(mt).subtype + "+"@ + suffix,
    //     None => view(mt).type_ + "/"@ + view(mt).subtype,
    // }
    view(mt).type_ + "/"@ + view(mt).subtype // for servo behavior
}
pub open spec fn essence_str_view(mt: &MimeView) -> Seq<char> {
    // match(mt.suffix) {
    //     Some(suffix) => mt.type_ + "/"@ + mt.subtype + "+"@ + suffix,
    //     None => mt.type_ + "/"@ + mt.subtype,
    // }
    mt.type_ + "/"@ + mt.subtype // for servo behavior
}

} // verus!
