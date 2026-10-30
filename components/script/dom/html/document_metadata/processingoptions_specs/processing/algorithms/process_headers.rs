/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers
//! Snapshot/inspection 2026-10-30.
//! Form: row 1, state_machine!; loop, shared temporaries, option updates and return.
//! Guards for recorded calls are observation-format constraints, not assertions
//! added to HTML or assumptions that arbitrary external input is well formed.
//! The supplied trace design does not claim native ghost instrumentation exists.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::model::*;
use super::super::types::*;
use super::super::trace::*;
use super::{headers, options};
use super::link_types::process_link_header_call_matches;

verus! {
pub type Extraction = spec_fn() -> Seq<ParsedLinkHeader>;
pub closed spec fn trace_shape(links: Seq<ParsedLinkHeader>, doc: LinkHeaderDocumentState, trace: ProcessLinkHeadersTrace) -> bool {
    trace_has_ordered_prefix(links, doc, trace)
}
pub closed spec fn observed_count(trace: ProcessLinkHeadersTrace) -> nat { trace.steps.len() }
pub closed spec fn before_doc(trace: ProcessLinkHeadersTrace, index: nat) -> LinkHeaderDocumentState { trace.states[index as int] }
pub closed spec fn after_doc(trace: ProcessLinkHeadersTrace, index: nat) -> LinkHeaderDocumentState { trace.states[index as int + 1] }
pub closed spec fn relation(link_object: ParsedLinkHeader) -> Seq<char> { link_object.relation_type }
pub closed spec fn attributes(link_object: ParsedLinkHeader) -> LinkAttributes { link_object.target_attributes }
pub closed spec fn media_phase() -> LinkHeadersPhase { LinkHeadersPhase::Media }
pub closed spec fn pre_media_phase() -> LinkHeadersPhase { LinkHeadersPhase::PreMedia }
pub closed spec fn phase_skip_observed(trace: ProcessLinkHeadersTrace, index: nat, doc: LinkHeaderDocumentState) -> bool {
    trace.steps[index as int].action is SkipPhase && trace.steps[index as int].media is None && after_doc(trace, index) == doc
}
pub closed spec fn media_call_observed(trace: ProcessLinkHeadersTrace, index: nat, attribs: LinkAttributes, doc: LinkHeaderDocumentState) -> bool {
    media_call_matches(attribs, doc.document, trace.steps[index as int].media)
}
// https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#matches-the-environment
// TRUSTED(scope): not verified yet; remove when media-result observation correspondence is verified
pub closed spec fn matches_environment(trace: ProcessLinkHeadersTrace, index: nat) -> bool {
    observed_media_result(trace.steps[index as int].media)
}
pub closed spec fn media_skip_observed(trace: ProcessLinkHeadersTrace, index: nat, doc: LinkHeaderDocumentState) -> bool {
    trace.steps[index as int].action is SkipMedia && after_doc(trace, index) == doc
}
pub closed spec fn new_options(link_object: ParsedLinkHeader, doc: LinkHeaderDocumentState) -> LinkProcessingOptionsView {
    options::initial_options(link_object, doc)
}
pub closed spec fn apply_attributes(options: LinkProcessingOptionsView, attribs: LinkAttributes, rel: Seq<char>) -> (bool, LinkProcessingOptionsView) {
    options::apply_link_options_from_parsed_header(options, attribs, rel)
}
pub closed spec fn application_observed(trace: ProcessLinkHeadersTrace, index: nat, before: LinkProcessingOptionsView,
    attribs: LinkAttributes, rel: Seq<char>, after: LinkProcessingOptionsView, result: bool) -> bool {
    apply_call_matches(before, attribs, rel, after, result, trace.steps[index as int])
}
pub closed spec fn return_observed(trace: ProcessLinkHeadersTrace, index: nat, doc: LinkHeaderDocumentState) -> bool {
    trace.steps[index as int].action is ReturnAfterApply && after_doc(trace, index) == doc
}
// https://html.spec.whatwg.org/multipage/images.html#create-a-source-set
// TRUSTED(scope): not verified yet; remove when source-set-result observation correspondence is verified
pub closed spec fn create_source_set_observed(trace: ProcessLinkHeadersTrace, index: nat, link_object: ParsedLinkHeader,
    attribs: LinkAttributes) -> Option<SourceSetView> {
    create_source_set_call_result(link_object.target_uri, attribs["imagesrcset"@], attribs["imagesizes"@], None, trace.steps[index as int])
}
pub closed spec fn no_source_set_call(trace: ProcessLinkHeadersTrace, index: nat) -> bool { observed_source_set_call(trace.steps[index as int]) is None }
pub closed spec fn set_source_set(options: LinkProcessingOptionsView, source_set: SourceSetView) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { source_set: Some(source_set), ..options }
}
pub closed spec fn dispatch_observed(trace: ProcessLinkHeadersTrace, index: nat, rel: Seq<char>, options: LinkProcessingOptionsView) -> bool {
    process_link_header_call_matches(rel, options, trace.steps[index as int])
}
// https://html.spec.whatwg.org/multipage/semantics.html#process-a-link-header
// The parent checks invocation order/arguments. Per-relation effects have their
// separate contracts; the returned document-state snapshot is supplied by the trace.
// TRUSTED(scope): not verified yet; remove when dispatch document-state observation correspondence is verified
pub closed spec fn process_link_header_observed(trace: ProcessLinkHeadersTrace, index: nat) -> LinkHeaderDocumentState {
    after_doc(trace, index)
}
pub enum ProcessHeadersState { Step1, Step2, Step2_1, Step2_2, Step2_3, Step2_4, Step2_5, Step2_6, Step2_7, Step2_8, Step2_9, Final }
}

state_machine! {
    process_headers_machine {
        fields {
            // Immutable inputs. The response adapter supplies the separate
            // extract_links_from_headers spec; Servo's pre-extracted adapter
            // supplies the already-observed result of that operation.
            pub extract_links: Extraction,
            pub phase: LinkHeadersPhase,
            pub trace: ProcessLinkHeadersTrace,
            // Document-state snapshot; identity is invariant across trace states.
            pub doc: LinkHeaderDocumentState,
            // temporary var used by Steps 1–2.9; never shortened
            pub links: Seq<ParsedLinkHeader>,
            // Loop cursor/variable for Step 2.
            pub index: nat,
            pub link_object: Option<ParsedLinkHeader>,
            // temporary vars used by Steps 2.1–2.9
            pub rel: Seq<char>,
            pub attribs: LinkAttributes,
            // temporary var used by Steps 2.3–2.4
            pub expected_phase: Option<LinkHeadersPhase>,
            // temporary var used by Steps 2.6–2.9
            pub options: Option<LinkProcessingOptionsView>,
            pub state: ProcessHeadersState,
        }
        init! {
            initialize(extract_links: Extraction, doc: LinkHeaderDocumentState,
                phase: LinkHeadersPhase, trace: ProcessLinkHeadersTrace) {
                // Observation validity, not a standard input restriction.
                require(trace_shape(extract_links(), doc, trace));
                init extract_links = extract_links;
                init doc = doc;
                init phase = phase;
                init trace = trace;
                init links = Seq::empty();
                init index = 0;
                init link_object = None;
                init rel = Seq::empty();
                init attribs = Map::empty();
                init expected_phase = None;
                init options = None;
                init state = ProcessHeadersState::Step1;
            }
        }
        // 1. Let links be the result of extracting links from response's header list.
        transition! {
            step1() {
                require(pre.state == ProcessHeadersState::Step1);
                update links = (pre.extract_links)();
                update state = ProcessHeadersState::Step2;
            }
        }
        // 2. For each linkObject in links:
        transition! {
            step2() {
                require(pre.state == ProcessHeadersState::Step2);
                if pre.index < pre.links.len() {
                    require(pre.index < observed_count(pre.trace));
                    require(pre.doc == before_doc(pre.trace, pre.index));
                    update link_object = Some(pre.links[pre.index as int]);
                    update state = ProcessHeadersState::Step2_1;
                } else { update state = ProcessHeadersState::Final; }
            }
        }
        // 2.1. Let rel be linkObject["relation_type"].
        transition! {
            step2_1() {
                require(pre.state == ProcessHeadersState::Step2_1);
                update rel = relation(pre.link_object->Some_0);
                update state = ProcessHeadersState::Step2_2;
            }
        }
        // 2.2. Let attribs be linkObject["target_attributes"].
        transition! {
            step2_2() {
                require(pre.state == ProcessHeadersState::Step2_2);
                update attribs = attributes(pre.link_object->Some_0);
                update state = ProcessHeadersState::Step2_3;
            }
        }
        // 2.3. Let expectedPhase be "media" if either "srcset", "imagesrcset", or "media" exist in attribs; otherwise "pre-media".
        transition! {
            step2_3() {
                require(pre.state == ProcessHeadersState::Step2_3);
                update expected_phase = Some(if pre.attribs.dom().contains("srcset"@)
                    || pre.attribs.dom().contains("imagesrcset"@) || pre.attribs.dom().contains("media"@)
                    { media_phase() } else { pre_media_phase() });
                update state = ProcessHeadersState::Step2_4;
            }
        }
        // 2.4. If expectedPhase is not phase, then continue.
        transition! {
            step2_4() {
                require(pre.state == ProcessHeadersState::Step2_4);
                if pre.expected_phase->Some_0 != pre.phase {
                    require(phase_skip_observed(pre.trace, pre.index, pre.doc));
                    update index = pre.index + 1;
                    update state = ProcessHeadersState::Step2;
                } else { update state = ProcessHeadersState::Step2_5; }
            }
        }
        // 2.5. If attribs["media"] exists and attribs["media"] does not match the environment, then continue.
        transition! {
            step2_5() {
                require(pre.state == ProcessHeadersState::Step2_5);
                require(media_call_observed(pre.trace, pre.index, pre.attribs, pre.doc));
                if pre.attribs.dom().contains("media"@) && !matches_environment(pre.trace, pre.index) {
                    require(media_skip_observed(pre.trace, pre.index, pre.doc));
                    update index = pre.index + 1;
                    update state = ProcessHeadersState::Step2;
                } else { update state = ProcessHeadersState::Step2_6; }
            }
        }
        // 2.6. Let options be a new link processing options with
        // href
        // linkObject["target_uri"]
        // base URL
        // doc's document base URL
        // origin
        // doc's origin
        // environment
        // doc's relevant settings object
        // policy container
        // doc's policy container
        // document
        // doc
        transition! {
            step2_6() {
                require(pre.state == ProcessHeadersState::Step2_6);
                update options = Some(new_options(pre.link_object->Some_0, pre.doc));
                update state = ProcessHeadersState::Step2_7;
            }
        }
        // 2.7. Apply link options from parsed header attributes to options given attribs and rel. If that returned false, then return.
        transition! {
            step2_7() {
                require(pre.state == ProcessHeadersState::Step2_7);
                let applied = apply_attributes(pre.options->Some_0, pre.attribs, pre.rel);
                require(application_observed(pre.trace, pre.index, pre.options->Some_0, pre.attribs, pre.rel, applied.1, applied.0));
                update options = Some(applied.1);
                if !applied.0 {
                    require(return_observed(pre.trace, pre.index, pre.doc));
                    update index = pre.index + 1;
                    update state = ProcessHeadersState::Final;
                } else { update state = ProcessHeadersState::Step2_8; }
            }
        }
        // 2.8. If attribs["imagesrcset"] exists and attribs["imagesizes"] exists, then set options's source set to the result of creating a source set given linkObject["target_uri"], attribs["imagesrcset"], attribs["imagesizes"], and null.
        transition! {
            step2_8() {
                require(pre.state == ProcessHeadersState::Step2_8);
                if pre.attribs.dom().contains("imagesrcset"@) && pre.attribs.dom().contains("imagesizes"@) {
                    require(create_source_set_observed(pre.trace, pre.index, pre.link_object->Some_0, pre.attribs) is Some);
                    update options = Some(set_source_set(pre.options->Some_0,
                        create_source_set_observed(pre.trace, pre.index, pre.link_object->Some_0, pre.attribs)->Some_0));
                    update state = ProcessHeadersState::Step2_9;
                } else {
                    require(no_source_set_call(pre.trace, pre.index));
                    update state = ProcessHeadersState::Step2_9;
                }
            }
        }
        // 2.9. Run the process a link header steps for rel given options.
        transition! {
            step2_9() {
                require(pre.state == ProcessHeadersState::Step2_9);
                require(dispatch_observed(pre.trace, pre.index, pre.rel, pre.options->Some_0));
                update doc = process_link_header_observed(pre.trace, pre.index);
                update index = pre.index + 1;
                update state = ProcessHeadersState::Step2;
            }
        }
    }
}

verus! {
/// Computable observation predicate. processing_trace below establishes a complete
/// fine-grained run for every accepted observation; it is not native trace capture.
pub(crate) open spec fn process_link_headers_ensures(links: Seq<ParsedLinkHeader>, doc: LinkHeaderDocumentState,
    phase: LinkHeadersPhase, trace: ProcessLinkHeadersTrace) -> bool {
    headers::processing_trace_computation(links, doc, phase, trace)
}

pub(crate) open spec fn processing_steps(fine: Seq<process_headers_machine::State>) -> bool {
    forall|i: int| 0 <= i && i + 1 < fine.len() ==>
        #[trigger] process_headers_machine::State::next(fine[i], fine[i + 1])
}

pub(crate) open spec fn complete_processing_trace(links: Seq<ParsedLinkHeader>, doc: LinkHeaderDocumentState,
    phase: LinkHeadersPhase, trace: ProcessLinkHeadersTrace, fine: Seq<process_headers_machine::State>) -> bool {
    &&& fine.len() > 0
    &&& process_headers_machine::State::initialize(fine.first(), || links, doc, phase, trace)
    &&& processing_steps(fine)
    &&& fine.last().state == ProcessHeadersState::Final
    &&& fine.last().index == trace.steps.len()
    &&& fine.last().doc == trace.states.last()
}

proof fn join_processing_traces(prefix: Seq<process_headers_machine::State>, rest: Seq<process_headers_machine::State>)
    -> (fine: Seq<process_headers_machine::State>)
    requires prefix.len() > 0, rest.len() > 0, prefix.last() == rest.first(), processing_steps(prefix), processing_steps(rest),
    ensures fine.len() > 0, fine.first() == prefix.first(), fine.last() == rest.last(), processing_steps(fine),
{
    let fine = prefix.drop_last() + rest;
    assert forall|i: int| 0 <= i && i + 1 < fine.len() implies
        #[trigger] process_headers_machine::State::next(fine[i], fine[i + 1]) by {
        if i < prefix.len() - 1 {
            assert(fine[i] == prefix[i]);
            assert(fine[i + 1] == prefix[i + 1]);
        } else {
            let j = i - (prefix.len() - 1);
            assert(fine[i] == rest[j]);
            assert(fine[i + 1] == rest[j + 1]);
        }
    }
    fine
}

/// Loop invariant: the consumed prefix is reflected in index and the current
/// document snapshot. Earlier iterations' effects are included in trace.states.
proof fn processing_loop_trace(pre: process_headers_machine::State, initial_doc: LinkHeaderDocumentState)
    -> (fine: Seq<process_headers_machine::State>)
    requires
        pre.state == ProcessHeadersState::Step2,
        headers::processing_trace_computation(pre.links, initial_doc, pre.phase, pre.trace),
        pre.index <= pre.trace.steps.len(),
        pre.index == pre.trace.steps.len() ==> pre.index == pre.links.len(),
        pre.doc == pre.trace.states[pre.index as int],
    ensures
        fine.len() > 0, fine.first() == pre, processing_steps(fine),
        fine.last().state == ProcessHeadersState::Final,
        fine.last().index == pre.trace.steps.len(),
        fine.last().doc == pre.trace.states.last(),
    decreases pre.links.len() - pre.index,
{
    let s = process_headers_machine::take_step::step2(pre);
    process_headers_machine::show::step2(pre, s);
    if pre.index == pre.links.len() {
        seq![pre, s]
    } else {
        let s21 = process_headers_machine::take_step::step2_1(s);
        process_headers_machine::show::step2_1(s, s21);
        let s22 = process_headers_machine::take_step::step2_2(s21);
        process_headers_machine::show::step2_2(s21, s22);
        let s23 = process_headers_machine::take_step::step2_3(s22);
        process_headers_machine::show::step2_3(s22, s23);
        let s24 = process_headers_machine::take_step::step2_4(s23);
        process_headers_machine::show::step2_4(s23, s24);
        let prefix = seq![pre, s, s21, s22, s23, s24];
        if s24.state == ProcessHeadersState::Step2 {
            let rest = processing_loop_trace(s24, initial_doc);
            join_processing_traces(prefix, rest)
        } else {
            let s25 = process_headers_machine::take_step::step2_5(s24);
            process_headers_machine::show::step2_5(s24, s25);
            let prefix = prefix.push(s25);
            if s25.state == ProcessHeadersState::Step2 {
                let rest = processing_loop_trace(s25, initial_doc);
                join_processing_traces(prefix, rest)
            } else {
                let s26 = process_headers_machine::take_step::step2_6(s25);
                process_headers_machine::show::step2_6(s25, s26);
                let s27 = process_headers_machine::take_step::step2_7(s26);
                process_headers_machine::show::step2_7(s26, s27);
                let prefix = prefix.push(s26).push(s27);
                if s27.state == ProcessHeadersState::Final {
                    prefix
                } else {
                    let s28 = process_headers_machine::take_step::step2_8(s27);
                    process_headers_machine::show::step2_8(s27, s28);
                    let s29 = process_headers_machine::take_step::step2_9(s28);
                    process_headers_machine::show::step2_9(s28, s29);
                    let rest = processing_loop_trace(s29, initial_doc);
                    join_processing_traces(prefix.push(s28).push(s29), rest)
                }
            }
        }
    }
}

pub(crate) proof fn processing_trace(links: Seq<ParsedLinkHeader>, doc: LinkHeaderDocumentState,
    phase: LinkHeadersPhase, trace: ProcessLinkHeadersTrace) -> (fine: Seq<process_headers_machine::State>)
    requires process_link_headers_ensures(links, doc, phase, trace),
    ensures complete_processing_trace(links, doc, phase, trace, fine),
{
    let initial = process_headers_machine::take_step::initialize(|| links, doc, phase, trace);
    let s1 = process_headers_machine::take_step::step1(initial);
    process_headers_machine::show::step1(initial, s1);
    let rest = processing_loop_trace(s1, doc);
    join_processing_traces(seq![initial, s1], rest)
}
} // verus!
