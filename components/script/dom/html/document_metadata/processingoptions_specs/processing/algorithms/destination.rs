/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Fetch potential-destination translation; snapshot 2026-10-19,
//! inspected 2026-10-30.

use vstd::prelude::*;
use super::super::types::destination_types;

verus! {

// https://fetch.spec.whatwg.org/#concept-potential-destination
// A potential destination is "fetch" or a destination which is not the empty string.
pub(crate) open spec fn is_potential_destination(potential_destination: Seq<char>) -> bool {
    potential_destination == "fetch"@ || (destination_types().contains(potential_destination) && potential_destination != ""@)
}

// https://fetch.spec.whatwg.org/#concept-potential-destination-translate
// Snapshot 2026-10-19, inspected 2026-10-30.
// Form: row 3, direct spec fn; ordered returns without carried variables.
// The intervening Assert is exported as a domain obligation under rule 6.3;
// it is neither a failure branch nor a conjunct of the result.
pub(crate) open spec fn translate_a_potential_destination(potential_destination: Seq<char>) -> Seq<char>
    // 2. Assert: potentialDestination is a destination.
    recommends potential_destination == "fetch"@ || destination_types().contains(potential_destination),
{
    // 1. If potentialDestination is "fetch", then return the empty string.
    if potential_destination == "fetch"@ {
        Seq::empty()
    } else {
        // 3. Return potentialDestination.
        potential_destination
    }
}

pub(crate) proof fn translation_has_destination_type(potential_destination: Seq<char>)
    requires is_potential_destination(potential_destination),
    ensures destination_types().contains(translate_a_potential_destination(potential_destination)),
{
    assert(Seq::<char>::empty() =~= ""@);
}

} // verus!
