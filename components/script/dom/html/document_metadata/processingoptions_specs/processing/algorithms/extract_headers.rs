/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/semantics.html#extract-links-from-headers
//! Snapshot/inspection 2026-10-30.
//! Form: row 1, state_machine!; list accumulation, loop cursor and continue.
//! FINDING: HTML's singular linkObject conflicts with the linked RFC field parser's
//! list result. This machine keeps the literal singular interface. The native
//! multi-object/relation adapter remains separately specified in headers.rs.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::model::{HeaderListView, ParsedLinkHeader};
use super::super::model_trait::LinkProcessingModel;

verus! {
// https://fetch.spec.whatwg.org/#concept-header-list-get-decode-split
pub closed spec fn get_decode_and_split<M: LinkProcessingModel>(model: M, headers: HeaderListView) -> Seq<Seq<char>> {
    model.get_decode_and_split_link_headers(headers)
}
// https://httpwg.org/specs/rfc8288.html#parse-fv
pub closed spec fn parse_link_object<M: LinkProcessingModel>(model: M, link_header: Seq<char>) -> Option<ParsedLinkHeader> {
    model.parse_standard_link_header(link_header)
}
pub enum ExtractHeadersState { Step1, Step2, Step3, Step3_1, Step3_2, Step3_3, Step4, Final }
}

state_machine! {
    extract_headers_machine<M: LinkProcessingModel> {
        fields {
            // Immutable inputs
            pub model: M,
            pub headers: HeaderListView,
            // temporary var used by Steps 1–4
            pub links: Seq<ParsedLinkHeader>,
            // temporary var used by Steps 2–3.3; never shortened
            pub raw_link_headers: Seq<Seq<char>>,
            // Iteration cursor and loop variable for Step 3.
            pub index: nat,
            pub link_header: Seq<char>,
            // temporary var used by Steps 3.1–3.3; None means missing target_uri
            pub link_object: Option<ParsedLinkHeader>,
            pub state: ExtractHeadersState,
            pub result: Seq<ParsedLinkHeader>,
        }
        init! {
            initialize(model: M, headers: HeaderListView) {
                init model = model;
                init headers = headers;
                init links = Seq::empty();
                init raw_link_headers = Seq::empty();
                init index = 0;
                init link_header = Seq::empty();
                init link_object = None;
                init state = ExtractHeadersState::Step1;
                init result = Seq::empty();
            }
        }
        // 1. Let links be a new list.
        transition! {
            step1() {
                require(pre.state == ExtractHeadersState::Step1);
                update links = Seq::empty();
                update state = ExtractHeadersState::Step2;
            }
        }
        // 2. Let rawLinkHeaders be the result of getting, decoding, and splitting `Link` from headers.
        transition! {
            step2() {
                require(pre.state == ExtractHeadersState::Step2);
                update raw_link_headers = get_decode_and_split(pre.model, pre.headers);
                update state = ExtractHeadersState::Step3;
            }
        }
        // 3. For each linkHeader of rawLinkHeaders:
        transition! {
            step3() {
                require(pre.state == ExtractHeadersState::Step3);
                if pre.index < pre.raw_link_headers.len() {
                    update link_header = pre.raw_link_headers[pre.index as int];
                    update state = ExtractHeadersState::Step3_1;
                } else { update state = ExtractHeadersState::Step4; }
            }
        }
        // 3.1. Let linkObject be the result of parsing linkHeader. [WEBLINK]
        transition! {
            step3_1() {
                require(pre.state == ExtractHeadersState::Step3_1);
                update link_object = parse_link_object(pre.model, pre.link_header);
                update state = ExtractHeadersState::Step3_2;
            }
        }
        // 3.2. If linkObject["target_uri"] does not exist, then continue.
        transition! {
            step3_2() {
                require(pre.state == ExtractHeadersState::Step3_2);
                if pre.link_object is None {
                    update index = pre.index + 1;
                    update state = ExtractHeadersState::Step3;
                } else { update state = ExtractHeadersState::Step3_3; }
            }
        }
        // 3.3. Append linkObject to links.
        transition! {
            step3_3() {
                require(pre.state == ExtractHeadersState::Step3_3);
                update links = pre.links.push(pre.link_object->Some_0);
                update index = pre.index + 1;
                update state = ExtractHeadersState::Step3;
            }
        }
        // 4. Return links.
        transition! {
            step4() {
                require(pre.state == ExtractHeadersState::Step4);
                update state = ExtractHeadersState::Final;
                update result = pre.links;
            }
        }
    }
}

verus! {
/// Computation helper; literal_loop_trace proves its complete execution witness
/// against the numbered transitions, including continue and accumulation.
pub(crate) open spec fn literal_extract_values<M: LinkProcessingModel>(model: M, raw: Seq<Seq<char>>, index: nat, links: Seq<ParsedLinkHeader>)
    -> Seq<ParsedLinkHeader>
    decreases raw.len() - index,
{
    if index >= raw.len() { links } else {
        let next = match model.parse_standard_link_header(raw[index as int]) {
            None => links, Some(object) => links.push(object),
        };
        literal_extract_values(model, raw, index + 1, next)
    }
}

pub(crate) open spec fn extract_links_from_headers<M: LinkProcessingModel>(model: M, headers: HeaderListView) -> Seq<ParsedLinkHeader> {
    literal_extract_values(model, model.get_decode_and_split_link_headers(headers), 0, Seq::empty())
}

pub(crate) open spec fn extraction_steps<M: LinkProcessingModel>(trace: Seq<extract_headers_machine::State<M>>) -> bool {
    forall|i: int| 0 <= i && i + 1 < trace.len() ==>
        #[trigger] extract_headers_machine::State::next(trace[i], trace[i + 1])
}

pub(crate) open spec fn complete_extraction_trace<M: LinkProcessingModel>(model: M, headers: HeaderListView,
    result: Seq<ParsedLinkHeader>, trace: Seq<extract_headers_machine::State<M>>) -> bool
{
    &&& trace.len() > 0
    &&& extract_headers_machine::State::initialize(trace.first(), model, headers)
    &&& extraction_steps(trace)
    &&& trace.last().state == ExtractHeadersState::Final
    &&& trace.last().result == result
}

proof fn join_extraction_traces<M: LinkProcessingModel>(prefix: Seq<extract_headers_machine::State<M>>, rest: Seq<extract_headers_machine::State<M>>)
    -> (trace: Seq<extract_headers_machine::State<M>>)
    requires prefix.len() > 0, rest.len() > 0, prefix.last() == rest.first(), extraction_steps(prefix), extraction_steps(rest),
    ensures trace.len() > 0, trace.first() == prefix.first(), trace.last() == rest.last(), extraction_steps(trace),
{
    let trace = prefix.drop_last() + rest;
    assert forall|i: int| 0 <= i && i + 1 < trace.len() implies
        #[trigger] extract_headers_machine::State::next(trace[i], trace[i + 1]) by {
        if i < prefix.len() - 1 {
            assert(trace[i] == prefix[i]);
            assert(trace[i + 1] == prefix[i + 1]);
        } else {
            let j = i - (prefix.len() - 1);
            assert(trace[i] == rest[j]);
            assert(trace[i + 1] == rest[j + 1]);
        }
    }
    trace
}

proof fn literal_loop_trace<M: LinkProcessingModel>(pre: extract_headers_machine::State<M>)
    -> (trace: Seq<extract_headers_machine::State<M>>)
    requires pre.state == ExtractHeadersState::Step3, pre.index <= pre.raw_link_headers.len(),
    ensures
        trace.len() > 0, trace.first() == pre, extraction_steps(trace),
        trace.last().state == ExtractHeadersState::Final,
        trace.last().result == literal_extract_values(pre.model, pre.raw_link_headers, pre.index, pre.links),
    decreases pre.raw_link_headers.len() - pre.index,
{
    let head = extract_headers_machine::take_step::step3(pre);
    extract_headers_machine::show::step3(pre, head);
    if pre.index >= pre.raw_link_headers.len() {
        let finished = extract_headers_machine::take_step::step4(head);
        extract_headers_machine::show::step4(head, finished);
        seq![pre, head, finished]
    } else {
        let parsed = extract_headers_machine::take_step::step3_1(head);
        extract_headers_machine::show::step3_1(head, parsed);
        let checked = extract_headers_machine::take_step::step3_2(parsed);
        extract_headers_machine::show::step3_2(parsed, checked);
        if parsed.link_object is None {
            let rest = literal_loop_trace(checked);
            join_extraction_traces(seq![pre, head, parsed, checked], rest)
        } else {
            let appended = extract_headers_machine::take_step::step3_3(checked);
            extract_headers_machine::show::step3_3(checked, appended);
            let rest = literal_loop_trace(appended);
            join_extraction_traces(seq![pre, head, parsed, checked, appended], rest)
        }
    }
}

pub(crate) proof fn extraction_trace<M: LinkProcessingModel>(model: M, headers: HeaderListView)
    -> (trace: Seq<extract_headers_machine::State<M>>)
    ensures complete_extraction_trace(model, headers, extract_links_from_headers(model, headers), trace),
{
    let s0 = extract_headers_machine::take_step::initialize(model, headers);
    let s1 = extract_headers_machine::take_step::step1(s0);
    extract_headers_machine::show::step1(s0, s1);
    let s2 = extract_headers_machine::take_step::step2(s1);
    extract_headers_machine::show::step2(s1, s2);
    let rest = literal_loop_trace(s2);
    join_extraction_traces(seq![s0, s1, s2], rest)
}
} // verus!
