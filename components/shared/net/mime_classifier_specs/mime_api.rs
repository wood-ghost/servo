use mime::{self, Mime, Name};
use vstd::prelude::*;

// use super::model::*;
use vstd::std_specs::cmp::PartialEqSpec;
use vstd::std_specs::fmt::fmt_req_all;
use vstd::utf8::valid_first_scalar;

use core::str::FromStr;

verus! {

// api

pub broadcast axiom fn axiom_fmt_req_all_mime()
    ensures
        #[trigger] fmt_req_all::<Mime>(),
;


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

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#754
// TEXT_CSS, "text/css", 4;
pub open spec fn text_css_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "css"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#756
// TEXT_JAVASCRIPT, "text/javascript", 4;
pub open spec fn text_javascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript"@,
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

// parse from string
// "image/x-icon"
pub open spec fn image_x_icon_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "x-icon"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "image/webp"
pub open spec fn image_webp_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "webp"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "video/webm"
pub open spec fn video_webm_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "webm"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/basic"
pub open spec fn audio_basic_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "basic"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/aiff"
pub open spec fn audio_aiff_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "aiff"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/mpeg"
pub open spec fn audio_mpeg_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "mpeg"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/ogg"
pub open spec fn application_ogg_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "ogg"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/midi"
pub open spec fn audio_midi_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "midi"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "video/avi"
pub open spec fn video_avi_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "avi"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/wave"
pub open spec fn audio_wave_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "wave"@,
        suffix: None,
        params: Map::empty(),
    }
}

// "application/postscript"
pub open spec fn application_postscript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "postscript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/x-gzip"
pub open spec fn application_x_gzip_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-gzip"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/zip"
pub open spec fn application_zip_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "zip"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/x-rar-compressed"
pub open spec fn application_x_rar_compressed_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-rar-compressed"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/font-woff"
pub open spec fn application_font_woff_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "font-woff"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/font-sfnt"
pub open spec fn application_font_sfnt_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "font-sfnt"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/vnd.ms-fontobject"
pub open spec fn application_vnd_ms_fontobject_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "vnd.ms-fontobject"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "video/mp4"
pub open spec fn video_mp4_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "mp4"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/vtt"
pub open spec fn text_vtt_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "vtt"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/cache-manifest"
pub open spec fn text_cache_manifest_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "cache-manifest"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/ecmascript"
pub open spec fn application_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "ecmascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/javascript"
pub open spec fn application_javascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "javascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/x-ecmascript"
pub open spec fn application_x_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-ecmascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/x-javascript"
pub open spec fn application_x_javascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-javascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/ecmascript"
pub open spec fn text_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "ecmascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.0"
pub open spec fn text_javascript1_0_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.0"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.1"
pub open spec fn text_javascript1_1_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.1"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.2"
pub open spec fn text_javascript1_2_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.2"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.3"
pub open spec fn text_javascript1_3_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.3"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.4"
pub open spec fn text_javascript1_4_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.4"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.5"
pub open spec fn text_javascript1_5_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.5"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/jscript"
pub open spec fn text_jscript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "jscript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/livescript"
pub open spec fn text_livescript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "livescript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/x-ecmascript"
pub open spec fn text_x_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "x-ecmascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/x-javascript"
pub open spec fn text_x_javascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "x-javascript"@,
        suffix: None,
        params: Map::empty(),
    }
}



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
        // &&& result is Ok
        &&& (input == "image/x-icon"@) ==> result is Ok
        &&& (input == "image/webp"@) ==> result is Ok
        &&& (input == "video/webm"@) ==> result is Ok
        &&& (input == "audio/basic"@) ==> result is Ok
        &&& (input == "audio/aiff"@) ==> result is Ok
        &&& (input == "audio/mpeg"@) ==> result is Ok
        &&& (input == "application/ogg"@) ==> result is Ok
        &&& (input == "audio/midi"@) ==> result is Ok
        &&& (input == "video/avi"@) ==> result is Ok
        &&& (input == "audio/wave"@) ==> result is Ok

        &&& (input == "application/postscript"@) ==> result is Ok
        &&& (input == "application/x-gzip"@) ==> result is Ok
        &&& (input == "application/zip"@) ==> result is Ok
        &&& (input == "application/x-rar-compressed"@) ==> result is Ok
        &&& (input == "application/font-woff"@) ==> result is Ok
        &&& (input == "application/font-sfnt"@) ==> result is Ok
        &&& (input == "application/vnd.ms-fontobject"@) ==> result is Ok
        &&& (input == "video/mp4"@) ==> result is Ok
        &&& (input == "text/vtt"@) ==> result is Ok
        &&& (input == "text/cache-manifest"@) ==> result is Ok

        &&& (input == "application/ecmascript"@) ==> result is Ok
        &&& (input == "application/javascript"@) ==> result is Ok
        &&& (input == "application/x-ecmascript"@) ==> result is Ok
        &&& (input == "application/x-javascript"@) ==> result is Ok
        &&& (input == "text/ecmascript"@) ==> result is Ok
        &&& (input == "text/javascript"@) ==> result is Ok
        &&& (input == "text/javascript1.0"@) ==> result is Ok
        &&& (input == "text/javascript1.1"@) ==> result is Ok
        &&& (input == "text/javascript1.2"@) ==> result is Ok
        &&& (input == "text/javascript1.3"@) ==> result is Ok
        &&& (input == "text/javascript1.4"@) ==> result is Ok
        &&& (input == "text/javascript1.5"@) ==> result is Ok
        &&& (input == "text/jscript"@) ==> result is Ok
        &&& (input == "text/livescript"@) ==> result is Ok
        &&& (input == "text/x-ecmascript"@) ==> result is Ok
        &&& (input == "text/x-javascript"@) ==> result is Ok

        &&& (input == "image/x-icon"@) ==> view(&result->Ok_0) == image_x_icon_identity()
        &&& (input == "image/webp"@) ==> view(&result->Ok_0) == image_webp_identity()
        &&& (input == "video/webm"@) ==> view(&result->Ok_0) == video_webm_identity()
        &&& (input == "audio/basic"@) ==> view(&result->Ok_0) == audio_basic_identity()
        &&& (input == "audio/aiff"@) ==> view(&result->Ok_0) == audio_aiff_identity()
        &&& (input == "audio/mpeg"@) ==> view(&result->Ok_0) == audio_mpeg_identity()
        &&& (input == "application/ogg"@) ==> view(&result->Ok_0) == application_ogg_identity()
        &&& (input == "audio/midi"@) ==> view(&result->Ok_0) == audio_midi_identity()
        &&& (input == "video/avi"@) ==> view(&result->Ok_0) == video_avi_identity()
        &&& (input == "audio/wave"@) ==> view(&result->Ok_0) == audio_wave_identity()

        &&& (input == "application/postscript"@) ==> view(&result->Ok_0) == application_postscript_identity()
        &&& (input == "application/x-gzip"@) ==> view(&result->Ok_0) == application_x_gzip_identity()
        &&& (input == "application/zip"@) ==> view(&result->Ok_0) == application_zip_identity()
        &&& (input == "application/x-rar-compressed"@) ==> view(&result->Ok_0) == application_x_rar_compressed_identity()
        &&& (input == "application/font-woff"@) ==> view(&result->Ok_0) == application_font_woff_identity()
        &&& (input == "application/font-sfnt"@) ==> view(&result->Ok_0) == application_font_sfnt_identity()
        &&& (input == "application/vnd.ms-fontobject"@) ==> view(&result->Ok_0) == application_vnd_ms_fontobject_identity()
        &&& (input == "video/mp4"@) ==> view(&result->Ok_0) == video_mp4_identity()
        &&& (input == "text/vtt"@) ==> view(&result->Ok_0) == text_vtt_identity()
        &&& (input == "text/cache-manifest"@) ==> view(&result->Ok_0) == text_cache_manifest_identity()

        &&& (input == "application/ecmascript"@) ==> view(&result->Ok_0) == application_ecmascript_identity()
        &&& (input == "application/javascript"@) ==> view(&result->Ok_0) == application_javascript_identity()
        &&& (input == "application/x-ecmascript"@) ==> view(&result->Ok_0) == application_x_ecmascript_identity()
        &&& (input == "application/x-javascript"@) ==> view(&result->Ok_0) == application_x_javascript_identity()
        &&& (input == "text/ecmascript"@) ==> view(&result->Ok_0) == text_ecmascript_identity()
        &&& (input == "text/javascript"@) ==> view(&result->Ok_0) == text_javascript_identity()
        &&& (input == "text/javascript1.0"@) ==> view(&result->Ok_0) == text_javascript1_0_identity()
        &&& (input == "text/javascript1.1"@) ==> view(&result->Ok_0) == text_javascript1_1_identity()
        &&& (input == "text/javascript1.2"@) ==> view(&result->Ok_0) == text_javascript1_2_identity()
        &&& (input == "text/javascript1.3"@) ==> view(&result->Ok_0) == text_javascript1_3_identity()
        &&& (input == "text/javascript1.4"@) ==> view(&result->Ok_0) == text_javascript1_4_identity()
        &&& (input == "text/javascript1.5"@) ==> view(&result->Ok_0) == text_javascript1_5_identity()
        &&& (input == "text/jscript"@) ==> view(&result->Ok_0) == text_jscript_identity()
        &&& (input == "text/livescript"@) ==> view(&result->Ok_0) == text_livescript_identity()
        &&& (input == "text/x-ecmascript"@) ==> view(&result->Ok_0) == text_x_ecmascript_identity()
        &&& (input == "text/x-javascript"@) ==> view(&result->Ok_0) == text_x_javascript_identity()
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
        view(&result) == text_css_identity(),
;

pub(crate) assume_specification [mime::TEXT_JAVASCRIPT] -> (result: Mime)
    ensures
        view(&result) == text_javascript_identity(),
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