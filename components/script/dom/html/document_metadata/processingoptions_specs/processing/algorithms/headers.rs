/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WHATWG extraction and processing of HTTP Link headers.
//! https://html.spec.whatwg.org/multipage/semantics.html#processing-link-headers
//! The numbered outer structure follows the standard, inspected 2026-10-29.

use vstd::prelude::*;
use super::super::model::*;
use super::super::model_trait::LinkProcessingModel;
use super::super::trace::*;
use super::options::{initial_options, apply_link_options_from_parsed_header};
use super::link_types::process_link_header_call_matches;
pub(crate) use super::extract_headers::extract_links_from_headers;
pub(crate) use super::process_headers::process_link_headers_ensures;

verus! {

// Native multi-object parser adapter, not the literal HTML extraction algorithm.
// FINDING: RFC 8288 B.2 returns a list, while HTML Step 3.1 names one linkObject.
// Keep this expansion behavior separate; extract_headers.rs models HTML literally.
pub(crate) open spec fn extract_links_from_values<M: LinkProcessingModel>(
    model: M,
    raw_link_headers: Seq<Seq<char>>,
    index: nat,
    links: Seq<ParsedLinkHeader>,
) -> Seq<ParsedLinkHeader>
    decreases raw_link_headers.len() - index,
{
    if index >= raw_link_headers.len() {
        links
    } else {
        let link_header = raw_link_headers[index as int];
        let link_objects = model.parse_link_header(link_header);
        let links = append_parsed_link_objects(link_objects, 0, links);
        extract_links_from_values(model, raw_link_headers, index + 1, links)
    }
}

pub(crate) open spec fn append_parsed_link_objects(
    link_objects: Seq<Option<ParsedLinkHeader>>,
    index: nat,
    links: Seq<ParsedLinkHeader>,
) -> Seq<ParsedLinkHeader>
    decreases link_objects.len() - index,
{
    if index >= link_objects.len() {
        links
    } else {
        let links = match link_objects[index as int] {
            None => links,
            Some(link_object) => links.push(link_object),
        };
        append_parsed_link_objects(link_objects, index + 1, links)
    }
}

pub(crate) open spec fn extract_links_from_headers_ensures<M: LinkProcessingModel>(
    model: M,
    headers: HeaderListView,
    result: Seq<ParsedLinkHeader>,
) -> bool {
    result == extract_links_from_headers(model, headers)
}

// Observation-validator computation. process_headers::processing_trace proves
// every accepted coarse trace has a complete run of the numbered machine.
pub(crate) open spec fn processing_trace_computation(
    extracted_links: Seq<ParsedLinkHeader>,
    document: LinkHeaderDocumentState,
    phase: LinkHeadersPhase,
    trace: ProcessLinkHeadersTrace,
) -> bool {
    let links = extracted_links;
    if !trace_has_ordered_prefix(links, document, trace) {
        false
    } else {
        forall|i: int| 0 <= i < trace.steps.len() ==> {
            let link_object = links[i];
            let step = #[trigger] trace.steps[i];
            let before = trace.states[i];
            let after = trace.states[i + 1];
            let rel = link_object.relation_type;
            let attribs = link_object.target_attributes;
            let expected_phase = if attribs.dom().contains("srcset"@)
                || attribs.dom().contains("imagesrcset"@)
                || attribs.dom().contains("media"@)
            {
                LinkHeadersPhase::Media
            } else {
                LinkHeadersPhase::PreMedia
            };
            if expected_phase != phase {
                &&& step.action is SkipPhase
                &&& step.media is None
                &&& after == before
            } else {
                &&& media_call_matches(attribs, before.document, step.media)
                &&& if attribs.dom().contains("media"@) && !observed_media_result(step.media) {
                    &&& step.action is SkipMedia
                    &&& after == before
                } else {
                    let options = initial_options(link_object, before);
                    let (applied, updated_options) = apply_link_options_from_parsed_header(
                        options, attribs, rel,
                    );
                    &&& apply_call_matches(options, attribs, rel, updated_options, applied, step)
                    &&& if !applied {
                        &&& step.action is ReturnAfterApply
                        &&& after == before
                        // trace_has_ordered_prefix requires this to be the final step.
                    } else {
                        let options = updated_options;
                        let options = if attribs.dom().contains("imagesrcset"@)
                            && attribs.dom().contains("imagesizes"@)
                        {
                            match create_source_set_call_result(
                                link_object.target_uri,
                                attribs["imagesrcset"@],
                                attribs["imagesizes"@],
                                None,
                                step,
                            ) {
                                Some(source_set) => Some(LinkProcessingOptionsView {
                                    source_set: Some(source_set), ..options
                                }),
                                None => None, // Invalid trace: the required call was not recorded.
                            }
                        } else if observed_source_set_call(step) is None {
                            Some(options)
                        } else {
                            None
                        };
                        match options {
                            Some(options) => process_link_header_call_matches(rel, options, step),
                            None => false,
                        }
                    }
                }
            }
        }
    }
}

pub(crate) open spec fn process_link_headers_from_response_ensures<M: LinkProcessingModel>(
    model: M,
    response_headers: HeaderListView,
    document: LinkHeaderDocumentState,
    phase: LinkHeadersPhase,
    trace: ProcessLinkHeadersTrace,
) -> bool {
    // Response adapter to the machine's supplied extraction operation.
    let links = extract_links_from_headers(model, response_headers);
    process_link_headers_ensures(links, document, phase, trace)
}

} // verus!
