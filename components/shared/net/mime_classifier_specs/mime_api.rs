use mime::{self, Mime, Name};
use vstd::prelude::*;

use super::model::*;
use vstd::std_specs::cmp::PartialEqSpec;


verus! {

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


// Constant
pub(crate) assume_specification [mime::TEXT_PLAIN] -> (result: Mime)
    ensures
        essence_str(&result) == "text/plain"@,
        mime_identity(&result) == text_plain_identity(),
;

pub(crate) assume_specification [mime::TEXT_PLAIN_UTF_8] -> (result: Mime)
    ensures
        essence_str(&result) == "text/plain"@, //TODO:
        mime_identity(&result) == text_plain_utf8_identity(),
;

pub(crate) assume_specification [mime::TEXT_HTML] -> (result: Mime)
    ensures
        // essence_str(&result) == "text/html"@,
        mime_identity(&result) == text_html_identity(),
;

pub(crate) assume_specification [mime::APPLICATION_OCTET_STREAM] -> (result: Mime)
    ensures
        mime_identity(&result) == application_octet_stream_identity(),
;

pub(crate) assume_specification [mime::TEXT_CSS] -> (result: Mime)
    ensures
        mime_identity(&result) == text_css(),
;

pub(crate) assume_specification [mime::TEXT_JAVASCRIPT] -> (result: Mime)
    ensures
        mime_identity(&result) == text_javascript(),
;

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#769
// IMAGE_BMP, "image/bmp", 5;
pub(crate) assume_specification [mime::IMAGE_BMP] -> (result: Mime)
    ensures
        // essence_str(&result) == "image/bmp"@, //TODO:
        view(&result) == (MimeView {
        type_: "image"@,
        subtype: "bmp"@,
        suffix: None,
        params: Map::empty(),
    }),
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

// Mime
pub assume_specification<'a>
    [Mime::essence_str]
    (mt: &'a Mime) -> (result: &'a str)
    ensures
        result@ == essence_str(mt),
;
pub assume_specification<'a>
    [Mime::suffix]
    (mt: &'a Mime) -> (result: Option<Name<'a>>)
    ensures
        match result {
            // Some(name) => suffix_name(mt) == Some(name_text(name)),
            Some(name) => suffix(mt) == Some(name_identity(&name)),
            None => suffix(mt).is_none(),
        },
;
pub assume_specification<'a>
    [Mime::type_]
    (mt: &'a Mime) -> (result: Name<'a>)
    ensures
        name_identity(&result) == view(mt).type_,
;
pub assume_specification<'a>
    [Mime::subtype]
    (mt: &'a Mime) -> (result: Name<'a>)
    ensures
        name_identity(&result) == view(mt).subtype,
;
pub assume_specification<'a>
    [Name::<'a>::as_str]
    (name: &Name<'a>) -> (result: &'a str)
    ensures
        result@ == name_identity(name),
;


} // verus!