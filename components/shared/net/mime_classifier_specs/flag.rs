use mime::Mime;
use vstd::prelude::*;
use vstd::std_specs::convert::FromSpecImpl;
use vstd::std_specs::cmp::PartialEqSpec;

use crate::mime_classifier::ApacheBugFlag;
use crate::mime_classifier::NoSniffFlag;
use crate::mime_classifier_specs::is_text_plain;

verus! {

pub open spec fn from_content_type(mime_type: Option<&Mime>) -> ApacheBugFlag {
    match mime_type {
        Some(mt) => {
            if is_text_plain(mt)
            {
                ApacheBugFlag::On
            } else {
                ApacheBugFlag::Off
            }
        },
        None => ApacheBugFlag::Off,
    }
}
pub open spec fn from(boolean: bool) -> NoSniffFlag {
    if boolean {
        NoSniffFlag::On
    } else {
        NoSniffFlag::Off
    }
}

impl FromSpecImpl<bool> for NoSniffFlag {
    open spec fn obeys_from_spec() -> bool {
        true
    }

    open spec fn from_spec(boolean: bool) -> NoSniffFlag {
        from(boolean)
    }
}

pub assume_specification[
    <NoSniffFlag as core::cmp::PartialEq<NoSniffFlag>>::eq
](
    left: &NoSniffFlag,
    right: &NoSniffFlag,
) -> (result: bool)
    ensures
        result == (*left == *right),
;

pub assume_specification[
    <ApacheBugFlag as core::cmp::PartialEq<ApacheBugFlag>>::eq
](
    left: &ApacheBugFlag,
    right: &ApacheBugFlag,
) -> (result: bool)
    ensures
        result == (*left == *right),
;

pub broadcast axiom fn axiom_no_sniff_flag_obeys_eq_spec()
    ensures
        #[trigger]
        <NoSniffFlag as PartialEqSpec<NoSniffFlag>>::obeys_eq_spec(),
;

pub broadcast axiom fn axiom_no_sniff_flag_eq_spec(
    left: &NoSniffFlag,
    right: &NoSniffFlag,
)
    ensures
        #[trigger]
        <NoSniffFlag as PartialEqSpec<NoSniffFlag>>::eq_spec(left, right)
            == (*left == *right),
;

pub broadcast group group_flag_partial_eq_axioms {
    axiom_no_sniff_flag_obeys_eq_spec,
    axiom_no_sniff_flag_eq_spec,
}

} // verus!