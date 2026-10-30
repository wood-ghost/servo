/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Trace bookkeeping and observation accessors. WHATWG branching belongs in
//! algorithms/headers.rs, so the outer algorithm can be read in standard order.

use vstd::prelude::*;
use super::model::*;
use super::types::{DocumentView, SourceSetView};

verus! {

pub(crate) open spec fn trace_has_ordered_prefix(
    links: Seq<ParsedLinkHeader>,
    document: LinkHeaderDocumentState,
    trace: ProcessLinkHeadersTrace,
) -> bool {
    let visited = trace.steps.len();
    if visited > links.len() || trace.states.len() != visited + 1 {
        false
    } else {
        &&& trace.states[0] == document
        &&& forall|i: int| 0 <= i < trace.states.len() ==>
            (#[trigger] trace.states[i]).document == document.document
        &&& forall|i: int| 0 <= i < visited ==> {
            let step = #[trigger] trace.steps[i];
            &&& step.header_index == i
            &&& (step.action is ReturnAfterApply ==> i + 1 == visited)
        }
        &&& if visited == links.len() { true }
            else if visited > 0 { trace.steps[visited - 1].action is ReturnAfterApply }
            else { false }
    }
}

pub(crate) open spec fn media_call_matches(
    attributes: LinkAttributes,
    document: DocumentView,
    media: Option<MediaQueryCall>,
) -> bool {
    if attributes.dom().contains("media"@) {
        match media {
            Some(call) => call.document == document && call.query == attributes["media"@],
            None => false,
        }
    } else { media is None }
}

pub(crate) open spec fn observed_media_result(media: Option<MediaQueryCall>) -> bool {
    match media { Some(call) => call.result, None => true }
}

pub(crate) open spec fn apply_call_matches(
    before: LinkProcessingOptionsView,
    attribs: LinkAttributes,
    rel: Seq<char>,
    after: LinkProcessingOptionsView,
    result: bool,
    step: LinkHeaderStep,
) -> bool {
    let call = match step.action {
        LinkHeaderAction::ReturnAfterApply { apply } => Some(apply),
        LinkHeaderAction::Dispatch { apply, .. } => Some(apply),
        _ => None,
    };
    match call {
        Some(call) => {
            &&& call.before == before
            &&& call.attributes == attribs
            &&& call.relation == rel
            &&& call.after == after
            &&& call.result == result
        },
        None => false,
    }
}

pub(crate) open spec fn observed_source_set_call(step: LinkHeaderStep) -> Option<CreateSourceSetCall> {
    match step.action {
        LinkHeaderAction::Dispatch { source_set, .. } => source_set,
        _ => None,
    }
}

/// None denotes a missing/incorrectly recorded call, not a null result from the
/// source-set algorithm (which returns a source set, including an empty one).
pub(crate) open spec fn create_source_set_call_result(
    default_source: Seq<char>,
    srcset: Seq<char>,
    sizes: Seq<char>,
    img: Option<int>,
    step: LinkHeaderStep,
) -> Option<SourceSetView> {
    match observed_source_set_call(step) {
        Some(call) => if call.default_source == default_source
            && call.srcset == srcset && call.sizes == sizes && call.img == img
        { Some(call.result) } else { None },
        None => None,
    }
}

} // verus!
