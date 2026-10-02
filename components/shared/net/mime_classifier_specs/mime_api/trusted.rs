use mime::{self, Mime, Name};
use vstd::prelude::*;

// use super::model::*;
use vstd::std_specs::cmp::PartialEqSpec;
use vstd::std_specs::fmt::fmt_req_all;
use vstd::utf8::valid_first_scalar;

use core::str::FromStr;

use super::views::*;
use super::constants::*;
use super::parser::*;

verus! {

// api

pub broadcast axiom fn axiom_fmt_req_all_mime()
    ensures
        #[trigger] fmt_req_all::<Mime>(),
;


// --------------------
// str::parse for Mime 
// --------------------
pub assume_specification<F: FromStr>[ str::parse::<F> ](s: &str) -> (
    result: Result<F, <F as FromStr>::Err>
)
    requires
        call_requires(<F as FromStr>::from_str, (s,)),
    ensures
        call_ensures( <F as FromStr>::from_str, (s,), result),
;

// The precondition is inherited from FromStrSpec::from_str_requires in parser.rs.
pub assume_specification[ <Mime as FromStr>::from_str ](input: &str) -> (
    result: Result<Mime, <Mime as FromStr>::Err>
)
    ensures
        true // TODO:
;

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
            Some(name) => view(mt).suffix == Some(name_identity(&name)),
            None => view(mt).suffix.is_none(),
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
