//! `Mime` abstraction, external API contracts, and essence proofs.
//!
//! The external type/operation specifications and formatting axiom are trusted.
//! The pure view definitions and proof lemmas are grouped separately below.

use ::mime::{Mime, Name};
use vstd::prelude::*;
use vstd::std_specs::fmt::fmt_req_all;

// use super::model::*;
use super::name::name_identity;

verus! {

// api

// abstract Mime
// The Rust Mime type https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#43
// pub struct Mime {
//     source: Source,
//     slash: usize,
//     plus: Option<usize>,
//     params: ParamSource,
// }
pub struct MimeView {
    pub type_: Seq<char>,
    // mime 0.3's subtype() excludes the separately stored structured suffix.
    // WHATWG's subtype includes both, joined by '+'.
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

// Mime
// pub uninterp spec fn mime_identity(mt: &Mime) -> MimeView;

// https://docs.rs/mime/latest/mime/struct.Mime.html#method.essence_str
pub open spec fn essence_str(mt: &Mime) -> Seq<char> {
    essence_str_view(&view(mt))
}
pub open spec fn essence_str_view(mt: &MimeView) -> Seq<char> {
    match(mt.suffix) {
        Some(suffix) => mt.type_ + "/"@ + mt.subtype + "+"@ + suffix,
        None => mt.type_ + "/"@ + mt.subtype,
    }
}

// Trusted external type and operation contracts.

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExMime(Mime);

pub broadcast axiom fn axiom_fmt_req_all_mime()
    ensures
        #[trigger] fmt_req_all::<Mime>(),
;

// --------------------
// PartialEq for Mime
// --------------------
pub assume_specification[ <Mime as core::cmp::PartialEq<Mime>>::eq ](left: &Mime, right: &Mime) -> (result: bool)
    ensures
        result == (view(left) == view(right)),
;

pub assume_specification[ <Mime as Clone>::clone ](mt: &Mime) -> (result: Mime)
    ensures
        view(&result) == view(mt),
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

// Proof lemmas over the abstract view.

/// Reconstruct the full essence from the type, base subtype, and optional suffix.
pub(crate) broadcast proof fn lemma_essence_str(mt: &Mime, expected: &str)
    requires
        expected@.len() == view(mt).type_.len() + 1 + view(mt).subtype.len()
            + match view(mt).suffix {
                Some(suffix) => 1 + suffix.len(),
                None => 0,
            },
        forall|i: int| 0 <= i < expected@.len() ==> #[trigger] expected@[i] ==
            if i < view(mt).type_.len() {
                view(mt).type_[i]
            } else if i == view(mt).type_.len() {
                '/'
            } else if i < view(mt).type_.len() + 1 + view(mt).subtype.len() {
                view(mt).subtype[i - view(mt).type_.len() - 1]
            } else if i == view(mt).type_.len() + 1 + view(mt).subtype.len() {
                '+'
            } else {
                view(mt).suffix->Some_0[i - view(mt).type_.len() - 2 - view(mt).subtype.len()]
            },
    ensures
        #![trigger essence_str(mt), expected@]
        essence_str(mt) =~= expected@,
{
    assert(essence_str(mt) =~= expected@);
}

pub(crate) broadcast group mime_essence_str_lemmas {
    lemma_essence_str,
}

} // verus!
