/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Native link-element boundary. Observations must eventually be produced by
//! instrumentation around update_source_set/processing_options and effects;
//! these declarations are not evidence that the supplied reference plan ran.

use vstd::prelude::*;
use net_traits::request::Destination;
use crate::dom::types::HTMLLinkElement;
use super::views::StandardModel;
use super::super::processing::model::*;
use super::super::processing::types::*;
use super::super::processing::algorithms::link_types::{preload, preload_element};
use super::super::requires::{type_matches_destination_requires, type_matches_standard_requires};

verus! {

// TRUSTED(scope): not verified yet; remove when HTMLLinkElement preload input observations are verified
pub(crate) uninterp spec fn observations(element: &HTMLLinkElement) -> LinkElementPreloadObservations;
// TRUSTED(scope): not verified yet; remove when HTMLLinkElement preload execution-plan observation is verified
pub(crate) uninterp spec fn observed_plan(element: &HTMLLinkElement) -> LinkElementPreloadPlan;

pub(crate) open spec fn translated(value: Option<Seq<char>>) -> Option<DestinationView> {
    match value { None => None, Some(value) => preload::translate_a_preload_destination(value) }
}

/// External DOM-input assumptions. Do not propagate them as if arbitrary DOM
/// strings were proved to satisfy native parser/decoder/selection requirements.
pub(crate) open spec fn element_preload_requires(element: &HTMLLinkElement) -> bool {
    let input = observations(element);
    // IMPLICIT: native compute_destination_for_attribute translates the raw value instead of an enumerated-state keyword; witness candidate: as="SCRIPT" (source inspection, not an executed browser test).
    &&& translated(input.raw_as) == translated(input.as_keyword)
    // INHERITED from preload/create-link-request after assigning the destination.
    &&& preload_element::element_preload_requires(StandardModel, input)
    &&& match translated(input.as_keyword) {
        None => true,
        Some(destination) => {
            let options = LinkProcessingOptionsView { destination, ..input.options_after_source_update };
            // INHERITED from type_matches_destination.
            &&& type_matches_destination_requires(options.type_, destination.as_native())
            // INHERITED from type_matches_destination; witnesses and model assumptions are in requires.rs.
            &&& type_matches_standard_requires(options.type_, destination.as_native())
            &&& (preload::type_matches_destination(StandardModel, options.type_, destination) ==> {
                // IMPLICIT: native options/request construction lacks fetch priority; witness: fetchpriority="high".
                &&& options.fetch_priority == LinkFetchPriorityView::Auto
                &&& if destination.as_native() == Destination::Image && options.source_set is Some {
                    let sources = options.source_set->Some_0.image_sources.entries;
                    // INHERITED from the contextual native source-selection domain.
                    &&& options.source_set->Some_0.well_formed()
                    // IMPLICIT: native source selection removes index zero and unwraps densities; witnesses: empty set or an entry without a density descriptor.
                    &&& sources.len() > 0
                    &&& (forall|i: int| 0 <= i < sources.len() ==>
                        (#[trigger] sources[i]).descriptor matches Some(ImageSourceDescriptorView::PixelDensity { .. }))
                } else { true }
            })
        },
    }
}

/// Standard-result contract, explicitly trusted until native effect capture is
/// connected. TypeMismatch denotes equivalent no-fetch behavior, not a claim
/// that native code calls preload before its already-hoisted type guard.
pub(crate) open spec fn element_preload_ensures(element: &HTMLLinkElement) -> bool {
    observed_plan(element) == preload_element::element_preload_plan(StandardModel, observations(element))
}

} // verus!
