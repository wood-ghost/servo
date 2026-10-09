//! `Name` abstraction and external API contracts.
//!
//! External specifications and equality axioms are trusted. The existing
//! text-only equality model still needs the sensitivity-fidelity review
//! documented by the MIME API regression cases.

use ::mime::Name;
use vstd::prelude::*;
use vstd::std_specs::cmp::PartialEqSpec;

verus! {

// api

// Name
pub uninterp spec fn name_identity<'a>(name: &Name<'a>,) -> Seq<char>;

// Trusted external type and operation contracts.

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExName<'a>(Name<'a>);

pub assume_specification<'a> [Name::<'a>::as_str] (name: &Name<'a>) -> (result: &'a str)
    ensures
        result@ == name_identity(name),
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

} // verus!
