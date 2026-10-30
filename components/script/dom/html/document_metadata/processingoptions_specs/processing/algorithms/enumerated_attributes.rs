/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Reference specification for HTML's enumerated-attribute state algorithm.
//! <https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#enumerated-attribute>
//! Inspected 2026-10-30, Living Standard last updated 2026-10-30.

use vstd::prelude::*;
use super::super::types::{EnumeratedAttributeState, EnumeratedAttributeView};
use super::super::super::predicates::ascii_case_insensitive_match;

verus! {

/// "An ASCII case-insensitive match for one of the keywords defined for the attribute."
/// This selects a keyword, not a default state. Several matching keywords may
/// represent the same state; a well-formed definition makes their states agree.
pub(crate) open spec fn matching_keyword<State>(
    attribute: EnumeratedAttributeView<State>,
    value: Seq<char>,
) -> Option<Seq<char>> {
    if exists|keyword: Seq<char>|
        attribute.keyword_states.dom().contains(keyword)
        && #[trigger] ascii_case_insensitive_match(value, keyword)
    {
        Some(choose|keyword: Seq<char>|
            attribute.keyword_states.dom().contains(keyword)
            && #[trigger] ascii_case_insensitive_match(value, keyword))
    } else {
        None
    }
}

/// To determine the state of an attribute, use the following steps:
/// None means the attribute is not specified; Some("") means a present empty value.
pub(crate) open spec fn determine_the_state_of_an_attribute<State>(
    attribute: EnumeratedAttributeView<State>,
    value: Option<Seq<char>>,
) -> EnumeratedAttributeState<State>
    recommends attribute.well_formed(),
{
    // 1. If the attribute is not specified:
    if value is None {
        // 1.1. If the attribute has a missing value default state defined, then
        // return that missing value default state.
        if attribute.missing_value_default is Some {
            EnumeratedAttributeState::State { value: attribute.missing_value_default->Some_0 }
        } else {
            // 1.2. Otherwise, return no state.
            EnumeratedAttributeState::NoState
        }
    } else {
        let value = value->Some_0;
        // 2. If the attribute's value is an ASCII case-insensitive match for one
        // of the keywords defined for the attribute, then return the state
        // represented by that keyword.
        let keyword = matching_keyword(attribute, value);
        if keyword is Some {
            EnumeratedAttributeState::State { value: attribute.keyword_states[keyword->Some_0] }
        // 3. If the attribute has an empty value default state defined and the
        // attribute's value is the empty string, then return that empty value default state.
        } else if attribute.empty_value_default is Some && value == ""@ {
            EnumeratedAttributeState::State { value: attribute.empty_value_default->Some_0 }
        // 4. If the attribute has an invalid value default state defined, then
        // return that invalid value default state.
        } else if attribute.invalid_value_default is Some {
            EnumeratedAttributeState::State { value: attribute.invalid_value_default->Some_0 }
        } else {
            // 5. Return no state.
            EnumeratedAttributeState::NoState
        }
    }
}

} // verus!
