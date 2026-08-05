use mime::{Mime, Name};
use vstd::prelude::*;

verus! {

// abstract Mime
pub struct MimeView {
    pub type_: Seq<char>,
    pub subtype: Seq<char>,
    pub suffix: Option<Seq<char>>,
    // pub essence: Seq<char>,
    pub params: Map<Seq<char>, Seq<char>>,
}

pub uninterp spec fn view(mt: &Mime) -> MimeView;

// Name
// pub open spec fn image_name() -> Seq<char> { "image"@ }
// pub open spec fn audio_name() -> Seq<char> { "audio"@ }
// pub open spec fn video_name() -> Seq<char> { "video"@ }
// pub open spec fn xml_name() -> Seq<char> { "xml"@ }

pub uninterp spec fn name_identity<'a>(name: &Name<'a>,) -> MimeView;


// ----------------
// Constant
// -----------------
// The Rust Mime type https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#43
// pub struct Mime {
//     source: Source,
//     slash: usize,
//     plus: Option<usize>,
//     params: ParamSource,
// }

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#750
// TEXT_PLAIN, "text/plain", 4;
pub open spec fn text_plain_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "plain"@,
        suffix: None,
        params: Map::empty(),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#751
// TEXT_PLAIN_UTF_8, "text/plain; charset=utf-8", 4, None, 10;
pub open spec fn text_plain_utf8_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "plain"@,
        suffix: None,
        params: Map::empty().insert("charset"@, "utf-8"@),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#757
// TEXT_XML, "text/xml", 4;
pub open spec fn text_xml_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "xml"@,
        suffix: None,
        params: Map::empty(),
    }
}

// pub uninterp spec fn image_identity() -> MimeView;
pub uninterp spec fn audio_identity() -> MimeView;
pub uninterp spec fn video_identity() -> MimeView;
pub uninterp spec fn application_identity() -> MimeView;
pub uninterp spec fn star_identity() -> MimeView;
pub uninterp spec fn text_identity() -> MimeView;
pub uninterp spec fn json_identity() -> MimeView;
pub uninterp spec fn font_identity() -> MimeView;



pub uninterp spec fn mime_identity(mt: &Mime) -> MimeView;
pub uninterp spec fn application_octet_stream_identity() -> MimeView;
pub uninterp spec fn text_css() -> MimeView;
pub uninterp spec fn text_javascript() -> MimeView;

// Mime
// https://docs.rs/mime/latest/mime/struct.Mime.html#method.essence_str
pub open spec fn essence_str(mt: &Mime) -> Seq<char> {
    view(mt).type_ + "/"@ + view(mt).subtype
    // view(mt).essence
}
pub uninterp spec fn name_text<'a>(name: Name<'a>) -> Seq<char>;
pub open spec fn suffix_name(mt: &Mime) -> Option<Seq<char>> {
    view(mt).suffix
}
pub uninterp spec fn subtype_name(mt: &Mime) -> Seq<char>;

// https://docs.rs/mime/latest/mime/struct.Mime.html#method.get_param
// pub open spec fn get_param(mime: &Mime, name: Seq<char>) -> Option<Seq<char>> {
//     let params = view(mime).params;

//     if params.contains_key(name) {
//         Some(params[name])
//     } else {
//         None
//     }
// }



} // verus!