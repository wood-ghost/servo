/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WHATWG "process link headers", as a specification-only small-step machine.
//! <https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers>
//! Standard text inspected 2026-11-19.
//!
//! The normal entry is Step 1. The response is represented by its header list,
//! the only response field this algorithm reads. Later function contracts may
//! begin at an invariant-valid middle state, such as S2_1 after extraction.
//! Atomic calls are deterministic but otherwise uninterpreted until their own
//! models exist. There is no abstraction from Servo LinkHeader values here.

use verus_state_machines_macros::state_machine;
use vstd::prelude::*;

use super::model::LinkProcessingOptionsView;
use super::types::{DocumentView, SourceSetView};

verus! {

pub struct LinkObject {
    pub target_uri: Seq<char>,
    pub relation_type: Seq<char>,
    // RFC 8288 Appendix B representation: ASCII-lowercased names, "" for a
    // valueless parameter, and parser order and duplicates retained.
    pub target_attributes: Seq<(Seq<char>, Seq<char>)>,
}

pub enum LinkHeadersPhase {
    PreMedia,
    Media,
}

#[allow(non_camel_case_types)]
pub enum ProcessLinkHeadersPc {
    S1,
    S2_1,
    S2_2,
    S2_3,
    S2_4,
    S2_5,
    S2_6,
    S2_7,
    S2_8,
    S2_9,
    Final,
}

// The final control state is shared; the output distinguishes exhaustion of
// the links from Step 2.7's early return. The invocation log remains in calls.
pub enum ProcessLinkHeadersOutput {
    Completed,
    Returned,
}

// The atomic extraction boundary is expected to supply normalized names.
// Initialization admits only inputs satisfying that representation requirement;
// the extraction/normalization algorithm itself is still abstract.
pub open spec fn link_objects_have_lowercase_attribute_names(links: Seq<LinkObject>) -> bool {
    forall|i: int, j: int, k: int|
        0 <= i < links.len()
            && 0 <= j < links[i].target_attributes.len()
            && 0 <= k < links[i].target_attributes[j].0.len()
        ==> !('A' <= #[trigger] links[i].target_attributes[j].0[k]
            && links[i].target_attributes[j].0[k] <= 'Z')
}

// Step 1: atomic in this model; TODO: own model.
// Fetch header lists contain byte-sequence names and values. Extraction returns
// the standard link objects, including their normalized attribute-list encoding.
pub uninterp spec fn whatwg_extract_links_from_headers(
    headers: Seq<(Seq<u8>, Seq<u8>)>,
) -> Seq<LinkObject>;

// Modelling choice: HTML indexes the RFC 8288 attribute list by key without
// specifying which duplicate it means. Use the FIRST matching tuple, including
// an empty value; never skip to a later duplicate with a nonempty value.
pub open spec fn link_object_attribs_get(
    attribs: Seq<(Seq<char>, Seq<char>)>,
    name: Seq<char>,
) -> Option<Seq<char>> {
    let names = attribs.map_values(|entry: (Seq<char>, Seq<char>)| entry.0);
    match names.index_of_first(name) {
        Some(i) => Some(attribs[i].1),
        None => None,
    }
}

// Step 2.5: atomic in this model; TODO: own model.
pub uninterp spec fn whatwg_matches_environment(
    media: Seq<char>,
    doc: DocumentView,
) -> bool;

// Step 2.6: atomic in this model; TODO: own model.
// Represents the options defaults and the specified href, document base URL,
// origin, relevant settings object, policy container, and document fields.
pub uninterp spec fn whatwg_link_options_from_document(
    doc: DocumentView,
    target_uri: Seq<char>,
) -> LinkProcessingOptionsView;

// Step 2.7: atomic in this model; TODO: own model.
// None represents failure; Some carries the updated options on success.
pub uninterp spec fn whatwg_apply_link_options_from_parsed_header_attributes(
    options: LinkProcessingOptionsView,
    attribs: Seq<(Seq<char>, Seq<char>)>,
    rel: Seq<char>,
) -> Option<LinkProcessingOptionsView>;

// Step 2.8: atomic in this model; TODO: own model.
// The standard's img argument is fixed to null here. The abstract document is
// the evaluation context, not a replacement for that img argument.
pub uninterp spec fn whatwg_create_source_set_with_null_img(
    default_source: Seq<char>,
    srcset: Seq<char>,
    sizes: Seq<char>,
    doc: DocumentView,
) -> SourceSetView;

// The DSL generates public transition relations. Keep this ordinary record
// update closed so they can reuse the existing crate-private options record
// without changing that record's visibility or introducing another atomic call.
pub closed spec fn link_options_with_source_set(
    options: LinkProcessingOptionsView,
    source_set: SourceSetView,
) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { source_set: Some(source_set), ..options }
}

pub open spec fn process_link_headers_terminal(state: process_link_headers_automaton::State) -> bool {
    state.output is Some
}

// https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers
state_machine! {
    process_link_headers_automaton {
        fields {
            // To process link headers given a Document doc, a response response, and a "pre-media" or "media" phase:
            pub response_headers: Seq<(Seq<u8>, Seq<u8>)>,
            pub doc: DocumentView,
            pub phase: LinkHeadersPhase,

            // Step 1 assigns links. Its initial empty placeholder is
            // distinguished from a completed empty extraction by pc == S1.
            pub links: Seq<LinkObject>,
            // The current link is links[i]; pc names the next numbered step.
            pub i: nat,
            pub pc: ProcessLinkHeadersPc,

            // Step-local values are absent until assigned. Advancing to another
            // link (including normal completion) resets all four to None. An
            // early return retains the failing link's locals and its
            // pre-application options.
            pub rel: Option<Seq<char>>,
            pub attribs: Option<Seq<(Seq<char>, Seq<char>)>>,
            pub expected_phase: Option<LinkHeadersPhase>,
            pub options: Option<LinkProcessingOptionsView>,

            // Step 2.9 invocations only; their effects are not modelled.
            pub calls: Seq<(Seq<char>, LinkProcessingOptionsView)>,

            // None while processing; Some records why Final was reached.
            pub output: Option<ProcessLinkHeadersOutput>,
        }

        // Begin the whole standard algorithm, before Step 1 has run.
        init! {
            initialize(
                response_headers: Seq<(Seq<u8>, Seq<u8>)>,
                doc: DocumentView,
                phase: LinkHeadersPhase,
            ) {
                require(link_objects_have_lowercase_attribute_names(
                    whatwg_extract_links_from_headers(response_headers),
                ));
                init response_headers = response_headers;
                init doc = doc;
                init phase = phase;
                init links = Seq::empty();
                init i = 0;
                init pc = ProcessLinkHeadersPc::S1;
                init rel = None;
                init attribs = None;
                init expected_phase = None;
                init options = None;
                init calls = Seq::empty();
                init output = None;
            }
        }

        // 1. Let links be the result of extracting links from response's header list.
        transition! {
            s1() {
                require(pre.pc == ProcessLinkHeadersPc::S1);
                let links = whatwg_extract_links_from_headers(pre.response_headers);
                update links = links;
                update i = 0;
                if links.len() == 0 {
                    update pc = ProcessLinkHeadersPc::Final;
                    update output = Some(ProcessLinkHeadersOutput::Completed);
                } else {
                    update pc = ProcessLinkHeadersPc::S2_1;
                    update output = None;
                }
                update rel = None;
                update attribs = None;
                update expected_phase = None;
                update options = None;
            }
        }

        // 2. For each linkObject in links:
        // S1 selects the first link; the continue and S2_9 edges select the next
        // link or Final with a Completed output. This iteration bookkeeping is
        // not a separate transition.

        // 2.1. Let rel be linkObject["relation_type"].
        transition! {
            s2_1() {
                require(pre.pc == ProcessLinkHeadersPc::S2_1);
                require(pre.i < pre.links.len());
                update rel = Some(pre.links[pre.i as int].relation_type);
                update pc = ProcessLinkHeadersPc::S2_2;
            }
        }

        // 2.2. Let attribs be linkObject["target_attributes"].
        transition! {
            s2_2() {
                require(pre.pc == ProcessLinkHeadersPc::S2_2);
                require(pre.i < pre.links.len());
                update attribs = Some(pre.links[pre.i as int].target_attributes);
                update pc = ProcessLinkHeadersPc::S2_3;
            }
        }

        // 2.3. Let expectedPhase be "media" if either "srcset", "imagesrcset", or
        //      "media" exist in attribs; otherwise "pre-media".
        transition! {
            s2_3() {
                require(pre.pc == ProcessLinkHeadersPc::S2_3);
                require(pre.attribs is Some);
                let attribs = pre.attribs->Some_0;
                update expected_phase = Some(
                    if link_object_attribs_get(attribs, "srcset"@) is Some
                        || link_object_attribs_get(attribs, "imagesrcset"@) is Some
                        || link_object_attribs_get(attribs, "media"@) is Some {
                        LinkHeadersPhase::Media
                    } else {
                        LinkHeadersPhase::PreMedia
                    }
                );
                update pc = ProcessLinkHeadersPc::S2_4;
            }
        }

        // 2.4. If expectedPhase is not phase, then continue.
        transition! {
            s2_4() {
                require(pre.pc == ProcessLinkHeadersPc::S2_4);
                require(pre.expected_phase is Some);
                if pre.expected_phase->Some_0 != pre.phase {
                    update i = pre.i + 1;
                    if pre.i + 1 < pre.links.len() {
                        update pc = ProcessLinkHeadersPc::S2_1;
                        update output = None;
                    } else {
                        update pc = ProcessLinkHeadersPc::Final;
                        update output = Some(ProcessLinkHeadersOutput::Completed);
                    }
                    update rel = None;
                    update attribs = None;
                    update expected_phase = None;
                    update options = None;
                } else {
                    update pc = ProcessLinkHeadersPc::S2_5;
                }
            }
        }

        // 2.5. If attribs["media"] exists and attribs["media"] does not match the
        //      environment, then continue.
        transition! {
            s2_5() {
                require(pre.pc == ProcessLinkHeadersPc::S2_5);
                require(pre.attribs is Some);
                let media = link_object_attribs_get(pre.attribs->Some_0, "media"@);
                if media is Some && !whatwg_matches_environment(media->Some_0, pre.doc) {
                    update i = pre.i + 1;
                    if pre.i + 1 < pre.links.len() {
                        update pc = ProcessLinkHeadersPc::S2_1;
                        update output = None;
                    } else {
                        update pc = ProcessLinkHeadersPc::Final;
                        update output = Some(ProcessLinkHeadersOutput::Completed);
                    }
                    update rel = None;
                    update attribs = None;
                    update expected_phase = None;
                    update options = None;
                } else {
                    update pc = ProcessLinkHeadersPc::S2_6;
                }
            }
        }

        // 2.6. Let options be a new link processing options with
        //      href: linkObject["target_uri"];
        //      base URL: doc's document base URL;
        //      origin: doc's origin;
        //      environment: doc's relevant settings object;
        //      policy container: doc's policy container;
        //      document: doc.
        transition! {
            s2_6() {
                require(pre.pc == ProcessLinkHeadersPc::S2_6);
                require(pre.i < pre.links.len());
                update options = Some(whatwg_link_options_from_document(
                    pre.doc, pre.links[pre.i as int].target_uri,
                ));
                update pc = ProcessLinkHeadersPc::S2_7;
            }
        }

        // 2.7. Apply link options from parsed header attributes to options given
        //      attribs and rel. If that returned false, then return.
        transition! {
            s2_7() {
                require(pre.pc == ProcessLinkHeadersPc::S2_7);
                require(pre.rel is Some);
                require(pre.attribs is Some);
                require(pre.options is Some);
                match whatwg_apply_link_options_from_parsed_header_attributes(
                    pre.options->Some_0, pre.attribs->Some_0, pre.rel->Some_0,
                ) {
                    None => {
                        update pc = ProcessLinkHeadersPc::Final;
                        update output = Some(ProcessLinkHeadersOutput::Returned);
                    },
                    Some(options) => {
                        update options = Some(options);
                        update pc = ProcessLinkHeadersPc::S2_8;
                    },
                }
            }
        }

        // 2.8. If attribs["imagesrcset"] exists and attribs["imagesizes"] exists,
        //      then set options's source set to the result of creating a source
        //      set given linkObject["target_uri"], attribs["imagesrcset"],
        //      attribs["imagesizes"], and null.
        transition! {
            s2_8() {
                require(pre.pc == ProcessLinkHeadersPc::S2_8);
                require(pre.i < pre.links.len());
                require(pre.attribs is Some);
                require(pre.options is Some);
                let srcset = link_object_attribs_get(pre.attribs->Some_0, "imagesrcset"@);
                let sizes = link_object_attribs_get(pre.attribs->Some_0, "imagesizes"@);
                if srcset is Some && sizes is Some {
                    let options = pre.options->Some_0;
                    update options = Some(link_options_with_source_set(
                        options,
                        whatwg_create_source_set_with_null_img(
                            pre.links[pre.i as int].target_uri,
                            srcset->Some_0,
                            sizes->Some_0,
                            pre.doc,
                        ),
                    ));
                }
                update pc = ProcessLinkHeadersPc::S2_9;
            }
        }

        // 2.9. Run the process a link header steps for rel given options.
        transition! {
            s2_9() {
                require(pre.pc == ProcessLinkHeadersPc::S2_9);
                require(pre.rel is Some);
                require(pre.options is Some);
                // Only record the invocation. TODO: own model of the callee.
                update calls = pre.calls.push((pre.rel->Some_0, pre.options->Some_0));
                update i = pre.i + 1;
                if pre.i + 1 < pre.links.len() {
                    update pc = ProcessLinkHeadersPc::S2_1;
                    update output = None;
                } else {
                    update pc = ProcessLinkHeadersPc::Final;
                    update output = Some(ProcessLinkHeadersOutput::Completed);
                }
                update rel = None;
                update attribs = None;
                update expected_phase = None;
                update options = None;
            }
        }

        // The output is the terminal marker. This correspondence rules out a
        // Final control state with no output in the progress proof.
        #[invariant]
        pub fn output_is_final(&self) -> bool {
            (self.pc == ProcessLinkHeadersPc::Final) <==> (self.output is Some)
        }

        // Cursor/local availability invariants needed for progress.
        #[invariant]
        pub fn current_link_exists(&self) -> bool {
            self.pc != ProcessLinkHeadersPc::S1 && !process_link_headers_terminal(*self)
                ==> self.i < self.links.len()
        }

        #[invariant]
        pub fn locals_available(&self) -> bool {
            match self.pc {
                ProcessLinkHeadersPc::S1 | ProcessLinkHeadersPc::S2_1 | ProcessLinkHeadersPc::Final => true,
                ProcessLinkHeadersPc::S2_2 => self.rel is Some,
                ProcessLinkHeadersPc::S2_3 | ProcessLinkHeadersPc::S2_5
                    | ProcessLinkHeadersPc::S2_6 => self.rel is Some && self.attribs is Some,
                ProcessLinkHeadersPc::S2_4 => {
                    &&& self.rel is Some
                    &&& self.attribs is Some
                    &&& self.expected_phase is Some
                },
                ProcessLinkHeadersPc::S2_7 | ProcessLinkHeadersPc::S2_8 => {
                    &&& self.rel is Some
                    &&& self.attribs is Some
                    &&& self.options is Some
                },
                ProcessLinkHeadersPc::S2_9 => self.rel is Some && self.options is Some,
            }
        }

        #[inductive(initialize)]
        fn initialize_inductive(
            post: Self,
            response_headers: Seq<(Seq<u8>, Seq<u8>)>,
            doc: DocumentView,
            phase: LinkHeadersPhase,
        ) {}

        #[inductive(s1)]
        fn s1_inductive(pre: Self, post: Self) {}

        #[inductive(s2_1)]
        fn s2_1_inductive(pre: Self, post: Self) {}

        #[inductive(s2_2)]
        fn s2_2_inductive(pre: Self, post: Self) {}

        #[inductive(s2_3)]
        fn s2_3_inductive(pre: Self, post: Self) {}

        #[inductive(s2_4)]
        fn s2_4_inductive(pre: Self, post: Self) {}

        #[inductive(s2_5)]
        fn s2_5_inductive(pre: Self, post: Self) {}

        #[inductive(s2_6)]
        fn s2_6_inductive(pre: Self, post: Self) {}

        #[inductive(s2_7)]
        fn s2_7_inductive(pre: Self, post: Self) {}

        #[inductive(s2_8)]
        fn s2_8_inductive(pre: Self, post: Self) {}

        #[inductive(s2_9)]
        fn s2_9_inductive(pre: Self, post: Self) {}
    }
}

// Initialization establishes the invariants, and the inductive proofs above
// preserve them. Thus this stronger invariant-based progress result covers
// every nonterminal state reachable from an initialized invocation.
pub(crate) proof fn process_link_headers_next_exists(
    pre: process_link_headers_automaton::State,
) -> (post: process_link_headers_automaton::State)
    requires pre.invariant(), !process_link_headers_terminal(pre),
    ensures
        process_link_headers_automaton::State::next(pre, post),
        post.invariant(),
{
    match pre.pc {
        ProcessLinkHeadersPc::S1 => {
            let post = process_link_headers_automaton::take_step::s1(pre);
            process_link_headers_automaton::show::s1(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_1 => {
            let post = process_link_headers_automaton::take_step::s2_1(pre);
            process_link_headers_automaton::show::s2_1(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_2 => {
            let post = process_link_headers_automaton::take_step::s2_2(pre);
            process_link_headers_automaton::show::s2_2(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_3 => {
            let post = process_link_headers_automaton::take_step::s2_3(pre);
            process_link_headers_automaton::show::s2_3(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_4 => {
            let post = process_link_headers_automaton::take_step::s2_4(pre);
            process_link_headers_automaton::show::s2_4(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_5 => {
            let post = process_link_headers_automaton::take_step::s2_5(pre);
            process_link_headers_automaton::show::s2_5(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_6 => {
            let post = process_link_headers_automaton::take_step::s2_6(pre);
            process_link_headers_automaton::show::s2_6(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_7 => {
            let post = process_link_headers_automaton::take_step::s2_7(pre);
            process_link_headers_automaton::show::s2_7(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_8 => {
            let post = process_link_headers_automaton::take_step::s2_8(pre);
            process_link_headers_automaton::show::s2_8(pre, post);
            post
        },
        ProcessLinkHeadersPc::S2_9 => {
            let post = process_link_headers_automaton::take_step::s2_9(pre);
            process_link_headers_automaton::show::s2_9(pre, post);
            post
        },
        ProcessLinkHeadersPc::Final => pre,
    }
}

pub(crate) proof fn process_link_headers_next_is_deterministic(
    pre: process_link_headers_automaton::State,
    left: process_link_headers_automaton::State,
    right: process_link_headers_automaton::State,
)
    requires
        process_link_headers_automaton::State::next(pre, left),
        process_link_headers_automaton::State::next(pre, right),
    ensures left == right,
{
    reveal(process_link_headers_automaton::State::next);
    reveal(process_link_headers_automaton::State::next_by);
}

// TODO: termination measure and termination proof.
// TODO: big-step run function and its relationship to the small-step machine.
// TODO: abstraction from LinkHeader and checkpoint/bridge lemmas to Servo.

} // verus!
