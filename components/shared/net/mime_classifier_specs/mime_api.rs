use mime::{self, Mime, Name};
use vstd::prelude::*;

use super::model::*;


verus! {

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
        mime_identity(&result) == text_plain_identity(),
;

pub(crate) assume_specification [mime::TEXT_PLAIN_UTF_8] -> (result: Mime)
    ensures
        mime_identity(&result) == text_plain_utf8_identity(),
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


// NAME
pub assume_specification [mime::XML] -> (result: Name<'static>)
    ensures
        name_identity(&result) == xml_identity(),
        name_text(result) =~= xml_name()
;

pub assume_specification [mime::IMAGE] -> (result: Name<'static>)
    ensures
        name_identity(&result) == image_identity(),
        name_text(result) =~= image_name(),
;

pub assume_specification [mime::AUDIO] -> (result: Name<'static>)
    ensures
        name_identity(&result) == audio_identity(),
        name_text(result) =~= audio_name(),
;

pub assume_specification [mime::VIDEO] -> (result: Name<'static>)
    ensures
        name_identity(&result) == video_identity(),
        name_text(result) =~= video_name(),
;

pub assume_specification [mime::APPLICATION] -> (result: Name<'static>)
    ensures
        name_identity(&result) == application_identity(),
;

pub assume_specification [mime::STAR] -> (result: Name<'static>)
    ensures
        name_identity(&result) == star_identity(),
;

pub assume_specification [mime::TEXT] -> (result: Name<'static>)
    ensures
        name_identity(&result) == text_identity(),
;

pub assume_specification [mime::JSON] -> (result: Name<'static>)
    ensures
        name_identity(&result) == json_identity(),
;

pub assume_specification [mime::FONT] -> (result: Name<'static>)
    ensures
        name_identity(&result) == font_identity(),
;

// Mime
pub assume_specification<'a>
    [Mime::essence_str]
    (mt: &'a Mime) -> (result: &'a str)
    ensures
        result@ =~= essence_str(mt),
;
pub assume_specification<'a>
    [Mime::suffix]
    (mt: &'a Mime) -> (result: Option<Name<'a>>)
    ensures
        match result {
            Some(name) => suffix_name(mt) == Some(name_text(name)),
            None => suffix_name(mt).is_none(),
        },
;
pub assume_specification<'a>
    [Mime::type_]
    (mt: &'a Mime) -> (result: Name<'a>)
    ensures
        name_text(result) =~= view(mt).type_,
;
pub assume_specification<'a>
    [Mime::subtype]
    (mt: &'a Mime) -> (result: Name<'a>)
    ensures
        name_text(result) == subtype_name(mt),
;
pub assume_specification<'a>
    [Name::<'a>::as_str]
    (name: &Name<'a>) -> (result: &'a str)
    ensures
        result@ == name_text(*name),
;


} // verus!