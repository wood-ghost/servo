use mime::{self, Mime, Name};
use vstd::prelude::*;

// use super::model::*;
use vstd::std_specs::cmp::PartialEqSpec;

use core::str::FromStr;

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

// ----------------
// Constant Identity
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
pub open spec fn text_plain_utf_8_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "plain"@,
        suffix: None,
        params: Map::empty().insert("charset"@, "utf-8"@),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#752
// TEXT_HTML, "text/html", 4;
pub open spec fn text_html_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "html"@,
        suffix: None,
        params: Map::empty(),
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

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#766
// IMAGE_JPEG, "image/jpeg", 5;
pub open spec fn image_jpeg_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "jpeg"@,
        suffix: None,
        params: Map::empty(),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#767
// IMAGE_GIF, "image/gif", 5;
pub open spec fn image_gif_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "gif"@,
        suffix: None,
        params: Map::empty(),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#768
// IMAGE_PNG, "image/png", 5;
pub open spec fn image_png_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "png"@,
        suffix: None,
        params: Map::empty(),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#769
// IMAGE_BMP, "image/bmp", 5;
pub open spec fn image_bmp_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "bmp"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#779
// APPLICATION_OCTET_STREAM, "application/octet-stream", 11;
pub open spec fn application_octet_stream_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "octet-stream"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#781
// APPLICATION_PDF, "application/pdf", 11;
pub open spec fn application_pdf_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "pdf"@,
        suffix: None,
        params: Map::empty(),
    }
}

pub uninterp spec fn video_mp4_identity() -> MimeView;
pub uninterp spec fn text_css() -> MimeView;
pub uninterp spec fn text_javascript() -> MimeView;


// Mime
// pub uninterp spec fn mime_identity(mt: &Mime) -> MimeView;

// https://docs.rs/mime/latest/mime/struct.Mime.html#method.essence_str
pub open spec fn essence_str(mt: &Mime) -> Seq<char> {
    view(mt).type_ + "/"@ + view(mt).subtype
}

pub open spec fn suffix(mt: &Mime) -> Option<Seq<char>> {
    view(mt).suffix
}
pub open spec fn subtype(mt: &Mime) -> Seq<char> {
    view(mt).subtype
}

// --------------------
// str::parse for Mime 
// --------------------
#[verifier::external_trait_specification]
#[verifier::external_trait_extension(FromStrSpec via FromStrSpecImpl)]
pub trait ExFromStr: Sized {
    type ExternalTraitSpecificationFor: FromStr;
    type Err;

    spec fn from_str_ensures(i: Seq<char>, r: Result<Self, Self::Err>) -> bool;

    fn from_str(s: &str) -> (r: Result<Self, Self::Err>)
        ensures
            Self::from_str_ensures(s@, r)
    ;
}

pub assume_specification<F: FromStr>[ str::parse::<F> ](s: &str) -> (
    result: Result<F, <F as FromStr>::Err>
)
    ensures
        call_ensures( <F as FromStr>::from_str, (s,), result),
;

impl FromStrSpecImpl for Mime {
    open spec fn from_str_ensures(
        input: Seq<char>,
        result: Result<Mime, <Mime as FromStr>::Err>,
    ) -> bool {
        &&& (result is Ok ==> {
            essence_str(&result->Ok_0) == input
        })
        // hardcode
        &&& result is Ok
        // &&& (input == "image/x-icon"@) ==> result is Ok
        // &&& (input == "image/webp"@) ==> result is Ok
        // &&& (input == "video/webm"@) ==> result is Ok
        // &&& (input == "audio/basic"@) ==> result is Ok
        // &&& (input == "audio/aiff"@) ==> result is Ok
        // &&& (input == "audio/mpeg"@) ==> result is Ok
        // &&& (input == "application/ogg"@) ==> result is Ok
        // &&& (input == "audio/midi"@) ==> result is Ok
        // &&& (input == "video/avi"@) ==> result is Ok
        // &&& (input == "audio/wave"@) ==> result is Ok
        &&& input == "video/mp4"@ ==> view(&result->Ok_0) == video_mp4_identity()
    } 
}

// --------------------
// PartialEq for Mime 
// --------------------
pub assume_specification[ <Mime as core::cmp::PartialEq<Mime>>::eq ](left: &Mime, right: &Mime) -> (result: bool)
    ensures
        result == (view(left) == view(right)),
;

// insensitive is not considered
pub assume_specification<'a>[ <Name<'a> as core::cmp::PartialEq<Name<'a>>>::eq ](left: &Name<'a>, right: &Name<'a>) -> (result: bool)
    ensures
        result == (name_identity(left) == name_identity(right)),
;

pub broadcast axiom fn axiom_name_obeys_eq_spec<'a>()
    ensures
        #[trigger] <Name<'a> as PartialEqSpec<Name<'a>>>::obeys_eq_spec(),
;

pub broadcast axiom fn axiom_name_eq_spec<'a>(
    left: &Name<'a>,
    right: &Name<'a>,
)
    ensures
        #[trigger]
        <Name<'a> as PartialEqSpec<Name<'a>>>::eq_spec(left, right)
            == (name_identity(left) =~= name_identity(right)),
;

pub broadcast group group_name_partial_eq_axioms {
    axiom_name_obeys_eq_spec,
    axiom_name_eq_spec,
}


#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExMime(Mime);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExName<'a>(Name<'a>);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExFromStrError(mime::FromStrError);

pub assume_specification[ <Mime as Clone>::clone ](mt: &Mime) -> (result: Mime)
    ensures
        view(&result) == view(mt),
;

// ----------------
// Constant
// -----------------
pub(crate) assume_specification [mime::TEXT_PLAIN] -> (result: Mime)
    ensures
        view(&result) == text_plain_identity(),
;

pub(crate) assume_specification [mime::TEXT_PLAIN_UTF_8] -> (result: Mime)
    ensures
        view(&result) == text_plain_utf_8_identity(),
;

pub(crate) assume_specification [mime::TEXT_HTML] -> (result: Mime)
    ensures
        view(&result) == text_html_identity(),
;

pub(crate) assume_specification [mime::APPLICATION_OCTET_STREAM] -> (result: Mime)
    ensures
        view(&result) == application_octet_stream_identity(),
;

pub(crate) assume_specification [mime::APPLICATION_PDF] -> (result: Mime)
    ensures
        view(&result) == application_pdf_identity(),
;

pub(crate) assume_specification [mime::TEXT_CSS] -> (result: Mime)
    ensures
        view(&result) == text_css(),
;

pub(crate) assume_specification [mime::TEXT_JAVASCRIPT] -> (result: Mime)
    ensures
        view(&result) == text_javascript(),
;

pub(crate) assume_specification [mime::IMAGE_JPEG] -> (result: Mime)
    ensures
        view(&result) == image_jpeg_identity(),
;

pub(crate) assume_specification [mime::IMAGE_GIF] -> (result: Mime)
    ensures
        view(&result) == image_gif_identity(),
;

pub(crate) assume_specification [mime::IMAGE_PNG] -> (result: Mime)
    ensures
        view(&result) == image_png_identity(),
;

pub(crate) assume_specification [mime::IMAGE_BMP] -> (result: Mime)
    ensures
        view(&result) == image_bmp_identity(),
;


// NAME
pub assume_specification [mime::XML] -> (result: Name<'static>)
    ensures
        name_identity(&result) == xml_name(),
        // name_text(result) == "xml"@,
;

pub assume_specification [mime::IMAGE] -> (result: Name<'static>)
    ensures
        name_identity(&result) == image_name(),
        // name_text(result) == "image"@,
;

pub assume_specification [mime::AUDIO] -> (result: Name<'static>)
    ensures
        name_identity(&result) == audio_name(),
        // name_text(result) == "audio"@,
;

pub assume_specification [mime::VIDEO] -> (result: Name<'static>)
    ensures
        name_identity(&result) == video_name(),
        // name_text(result) == "video"@,
;

pub assume_specification [mime::APPLICATION] -> (result: Name<'static>)
    ensures
        name_identity(&result) == application_name(),
;

pub assume_specification [mime::STAR] -> (result: Name<'static>)
    ensures
        name_identity(&result) == star_name(),
;

pub assume_specification [mime::TEXT] -> (result: Name<'static>)
    ensures
        name_identity(&result) == text_name(),
;

pub assume_specification [mime::JSON] -> (result: Name<'static>)
    ensures
        name_identity(&result) == json_name(),
;

pub assume_specification [mime::FONT] -> (result: Name<'static>)
    ensures
        name_identity(&result) == font_name(),
;

pub assume_specification [mime::TEXT_XML] -> (result: Mime)
    ensures
        view(&result) == text_xml_identity(),
;

// Mime
pub assume_specification<'a> [Mime::essence_str](mt: &'a Mime) -> (result: &'a str)
    ensures
        result@ == essence_str(mt),
;
pub assume_specification<'a> [Mime::suffix] (mt: &'a Mime) -> (result: Option<Name<'a>>)
    ensures
        match result {
            // Some(name) => suffix_name(mt) == Some(name_text(name)),
            Some(name) => suffix(mt) == Some(name_identity(&name)),
            None => suffix(mt).is_none(),
        },
;
pub assume_specification<'a> [Mime::type_] (mt: &'a Mime) -> (result: Name<'a>)
    ensures
        name_identity(&result) == view(mt).type_,
;
pub assume_specification<'a> [Mime::subtype] (mt: &'a Mime) -> (result: Name<'a>)
    ensures
        name_identity(&result) == view(mt).subtype,
;
pub assume_specification<'a> [Name::<'a>::as_str] (name: &Name<'a>) -> (result: &'a str)
    ensures
        result@ == name_identity(name),
;


} // verus!