use mime::{self, Mime, Name};
use vstd::prelude::*;

// use super::model::*;
use vstd::std_specs::cmp::PartialEqSpec;
use vstd::std_specs::fmt::fmt_req_all;
use vstd::utf8::valid_first_scalar;

use core::str::FromStr;

use super::views::*;
use super::constants::*;

verus! {

// api

pub broadcast axiom fn axiom_fmt_req_all_mime()
    ensures
        #[trigger] fmt_req_all::<Mime>(),
;


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

// https://mimesniff.spec.whatwg.org/#parsing-a-mime-type
// To parse a MIME type, given a string input, run these steps: 
pub open spec fn parse_mime_type_spec(input: Seq<char>) -> Option<MimeView> {
    // 1. Remove any leading and trailing HTTP whitespace from input. 
    let input = remove_http_whitespace(input);
    // 2. Let position be a position variable for input, initially pointing at the start of input.
    // 3. Let type be the result of collecting a sequence of code points that are not U+002F (/) from input, given position. 
    let (type_, position) = collect_a_sequence_of_code_points(input, |c: char| c != '/', 0);
    // 4. If type is the empty string or does not solely contain HTTP token code points, then return failure. 
    if type_.len() == 0 || !solely_contains_http_token_code_points(type_) {
        None
    // 5. If position is past the end of input, then return failure. 
    } else if position >= input.len() {
        None
    } else {
        // 6. Advance position by 1. (This skips past U+002F (/).) 
        let position = position + 1;
        // 7. Let subtype be the result of collecting a sequence of code points that are not U+003B (;) from input, given position. 
        let (subtype, position) = collect_a_sequence_of_code_points(input, |c: char| c != ';', position);
        // 8. Remove any trailing HTTP whitespace from subtype. 
        let subtype = remove_http_whitespace(subtype);
        // 9. If subtype is the empty string or does not solely contain HTTP token code points, then return failure. 
        if subtype.len() == 0 || !solely_contains_http_token_code_points(subtype) {
            None
        } else {
            // Let mimeType be a new MIME type record whose type is type, in ASCII lowercase, and subtype is subtype, in ASCII lowercase. 
            let mime_type = MimeView {
                type_: type_,
                subtype: subtype,
                suffix: None, // TODO:
                params: Map::empty(),
            };
            Some(mime_type) //TODO:
        }
    }

}


pub open spec fn solely_contains_http_token_code_points(input: Seq<char>) -> bool {
    forall |i: int| 0 <= i < input.len() ==> #[trigger] is_http_token_code_point(input[i])
}
// https://mimesniff.spec.whatwg.org/#http-token-code-point
// An HTTP token code point is U+0021 (!), U+0023 (#), U+0024 ($), U+0025 (%), U+0026 (&), U+0027 ('), U+002A (*), U+002B (+), 
// U+002D (-), U+002E (.), U+005E (^), U+005F (_), U+0060 (`), U+007C (|), U+007E (~), or an ASCII alphanumeric.
pub open spec fn is_http_token_code_point(c: char) -> bool {
    ||| (c == '\u{0021}') // !
    ||| (c == '\u{0023}') // #
    ||| (c == '\u{0024}') // $
    ||| (c == '\u{0025}') // %
    ||| (c == '\u{0026}') // &
    ||| (c == '\u{0027}') // '
    ||| (c == '\u{002A}') // *
    ||| (c == '\u{002B}') // +
    ||| (c == '\u{002D}') // -
    ||| (c == '\u{002E}') // .
    ||| (c == '\u{005E}') // ^
    ||| (c == '\u{005F}') // _
    ||| (c == '\u{0060}') // `
    ||| (c == '\u{007C}') // |
    ||| (c == '\u{007E}') // ~
    ||| (is_ascii_alphanumeric_(c))
}
pub open spec fn is_ascii_alphanumeric_(c: char) -> bool {
    ('0' <= c && c <= '9')
        || ('A' <= c && c <= 'Z')
        || ('a' <= c && c <= 'z')
}

// https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points
// To collect a sequence of code points meeting a condition condition from a string input, 
// given a position variable position tracking the position of the calling algorithm within input:
pub open spec fn collect_a_sequence_of_code_points_helper(
    input: Seq<char>,
    condition: spec_fn(char) -> bool,
    result: Seq<char>,
    position: int,
) -> (Seq<char>, int)
    recommends
        0 <= position <= input.len(),
    decreases input.len() - position,
{
    // 2. While position doesn't point past the end of input and the code point at position within input meets the condition condition: 
    if position < input.len() && condition(input[position]) {
        collect_a_sequence_of_code_points_helper(
            input,
            condition,
            // 2.1 Append that code point to the end of result. 
            result.push(input[position]),
            // 2.2 Advance position by 1.
            position + 1,
        )
    } else {
        // 3. Return result. 
        (result, position)
    }
}
pub open spec fn collect_a_sequence_of_code_points(input: Seq<char>, condition: spec_fn(char) -> bool, position: int) -> (Seq<char>, int)
    recommends 0 <= position <= input.len(),
{
    collect_a_sequence_of_code_points_helper(
        input,
        condition,
        // 1. Let result be the empty string. 
        Seq::empty(),
        position,
    )
}

// An HTTP tab or space is U+0009 TAB or U+0020 SPACE.
// HTTP whitespace is U+000A LF, U+000D CR, or an HTTP tab or space. 
pub open spec fn is_http_whitespace(c: char) -> bool {
    ||| c == '\u{000A}' // LF
    ||| c == '\u{000D}' // CR
    ||| c == '\u{0009}' // TAB
    ||| c == '\u{0020}' // SPACE
}
pub open spec fn remove_http_whitespace(input: Seq<char>) -> Seq<char>
    decreases input.len(),
{
    if input.len() == 0 {
        input
    } else if is_http_whitespace(input[0]) {
        // remove prefix
        remove_http_whitespace(
            input.subrange(1, input.len() as int),
        )
    } else if is_http_whitespace(input[input.len() - 1]) {
        // remove suffix
        remove_http_whitespace(
            input.subrange(0, input.len() as int - 1),
        )
    } else {
        input
    }
}

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
;

pub assume_specification [mime::IMAGE] -> (result: Name<'static>)
    ensures
        name_identity(&result) == image_name(),
;

pub assume_specification [mime::AUDIO] -> (result: Name<'static>)
    ensures
        name_identity(&result) == audio_name(),
;

pub assume_specification [mime::VIDEO] -> (result: Name<'static>)
    ensures
        name_identity(&result) == video_name(),
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
