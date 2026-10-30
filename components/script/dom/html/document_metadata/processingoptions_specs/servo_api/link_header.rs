/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo's ordered LinkParamOwned representation. These are storage adapters,
//! not a replacement for RFC 8288 parsing or WHATWG parameter normalization.

use nom_rfc8288::complete::LinkParamOwned;
use vstd::prelude::*;
use super::super::processing::model::LinkAttributes;

verus! {

pub(crate) open spec fn has_key(params: Seq<LinkParamOwned>, key: Seq<char>) -> bool {
    exists|i: int| 0 <= i < params.len() && (#[trigger] params[i]).key@ == key
}

pub(crate) open spec fn first_value(params: Seq<LinkParamOwned>, key: Seq<char>) -> Option<Seq<char>>
    decreases params.len(),
{
    if params.len() == 0 {
        None
    } else if params[0].key@ == key {
        match params[0].val {
            Some(value) => Some(value@),
            None => None,
        }
    } else {
        first_value(params.drop_first(), key)
    }
}

/// Matches value_for_key_in_link_header's access semantics, including first-match
/// None. It is deliberately not claimed to be RFC 8288's normalized attribute map.
pub(crate) open spec fn valued_attributes(params: Seq<LinkParamOwned>) -> LinkAttributes {
    Map::new(
        params.map_values(|param: LinkParamOwned| param.key@)
            .to_set().filter(|key: Seq<char>| first_value(params, key) is Some),
        |key: Seq<char>| match first_value(params, key) {
            Some(value) => value,
            None => Seq::empty(),
        },
    )
}

} // verus!
