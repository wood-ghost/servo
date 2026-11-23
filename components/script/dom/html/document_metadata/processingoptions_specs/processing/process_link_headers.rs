/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Structural, specification-only bridge from parsed library data to LinkObject.
//! <https://www.rfc-editor.org/rfc/rfc8288.html#appendix-B>
//! <https://html.spec.whatwg.org/multipage/semantics.html#extract-links-from-headers>
//! <https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers>
//! Text inspected 2026-11-20. B.2/B.3 step numbers below refer to their inner loops.
//!
//! Modelling choice R1: at most one LinkObject per LinkHeaderView. HTML calls
//! "the result of parsing linkHeader" one linkObject; RFC B.2 steps 10 and 17
//! instead produce one link per relation type. Here rel is not split on whitespace.
//! This boundary starts with parsed fields: h.url and present attribute values
//! are carried through, without URI/context resolution or RFC 8187 decoding.
//! No equivalence with Servo's case-sensitive lookups or processing is assumed.

#![verifier::auto_reveal_strlit]

use net_traits::ReferrerPolicy;
use net_traits::request::Destination;
use verus_state_machines_macros::state_machine;
use vstd::prelude::*;

use crate::dom::document::Document;
use super::super::super::processingoptions::{LinkHeader, LinkProcessingPhase};
use super::super::postconditions::process_link_header_ensures;
use super::model::LinkProcessingOptionsView;
use super::types::{
    DocumentView, ElementView, LinkFetchPriority, LinkHeaderView, LinkHeaderViewSpec, SourceSetView,
};

// ASCII lowercase, reused from mime_api.
pub(crate) use net_traits::mime_classifier_specs::mime_api::ascii_lowercase;

verus! {

// <https://html.spec.whatwg.org/multipage/#process-link-headers>
// To process link headers given a Document doc, a response response, and a "pre-media" or "media" phase:
pub(crate) open spec fn process_link_headers_ensures(
    link_headers: Seq<LinkHeader>,
    document: &Document,
    phase: LinkProcessingPhase,
) -> bool {
    // 1. Let links be the result of extracting links from response's header list.
    // Here we translate link_headers into whatwg domain
    let links = to_link_objects(link_headers.map_values(|header: LinkHeader| header@));
    // 2. For each linkObject in links:
    // TODO: run the state machine from init to final state. then says that the changing data fits the property described in final state. if fits, return true; if not return false
    true
}

pub struct LinkObject {
    pub target_uri: Seq<char>,
    pub relation_type: Seq<char>,
    // RFC 8288 Appendix B representation: ASCII-lowercased names, "" for a
    // valueless parameter, and parser order and duplicates retained.
    // This is a representation convention, not an initialization precondition.
    pub target_attributes: Seq<(Seq<char>, Seq<char>)>,
}

pub enum LinkHeadersPhase {
    PreMedia,
    Media,
}

pub enum ProcessLinkHeadersState {
    Init,
    // The current link is selected and the loop body is ready to begin.
    LoopHead,
    RelRead,
    AttribsRead,
    ExpectedPhaseRead,
    PhaseChecked,
    MediaChecked,
    OptionsCreated,
    OptionsApplied,
    SourceSetChecked,
    Final,
}

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

// Step 2.7: atomic in this model; TODO: own model.
// None represents failure; Some carries the updated options on success.
pub uninterp spec fn whatwg_apply_link_options_from_parsed_header_attributes(
    options: LinkProcessingOptionsView,
    attribs: Seq<(Seq<char>, Seq<char>)>,
    rel: Seq<char>,
) -> Option<LinkProcessingOptionsView>;

// Step 2.8: atomic in this model; TODO: own model.
pub uninterp spec fn whatwg_create_source_set(
    default_source: Seq<char>,
    srcset: Seq<char>,
    sizes: Seq<char>,
    img: Option<ElementView>,
    doc: DocumentView,
) -> SourceSetView;

// Loop head and Steps 2.1-2.9.
// https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers
state_machine! {
    process_link_headers_automaton {
        fields {
            // Inputs after Step 1 has been performed by the caller.
            pub links: Seq<LinkObject>,
            pub doc: DocumentView,
            pub phase: LinkHeadersPhase,

            // i identifies the current link; continue and Step 2.9 advance it.
            pub i: nat,
            pub state: ProcessLinkHeadersState,
            pub link_object: Option<LinkObject>,
            pub rel: Option<Seq<char>>,
            pub attribs: Option<Seq<(Seq<char>, Seq<char>)>>,
            pub expected_phase: Option<LinkHeadersPhase>,
            pub options: Option<LinkProcessingOptionsView>,

            // Instrumentation (not in WHATWG): records each Step 2.9 invocation (rel, options)
            // in order, for comparison with Servo's calls to process_link_header.
            pub calls: Seq<(Seq<char>, LinkProcessingOptionsView)>,
            // Instrumentation (not in WHATWG): every Step 2.9 invocation of
            // this run has had its effect.
            pub effects: bool,
        }

        // Begin with the extracted links, without restricting their attributes.
        init! {
            initialize(
                links: Seq<LinkObject>,
                doc: DocumentView,
                phase: LinkHeadersPhase,
            ) {
                init links = links;
                init doc = doc;
                init phase = phase;
                init i = 0;
                init state = ProcessLinkHeadersState::Init;
                init link_object = None;
                init rel = None;
                init attribs = None;
                init expected_phase = None;
                init options = None;
                init calls = Seq::empty();
                init effects = true;
            }
        }

        // 2. For each linkObject in links:
        transition! {
            init_loop_head() {
                require(pre.state == ProcessLinkHeadersState::Init);
                if pre.i < pre.links.len() {
                    update link_object = Some(pre.links[pre.i as int]);
                    update state = ProcessLinkHeadersState::LoopHead;
                } else {
                    update link_object = None;
                    update state = ProcessLinkHeadersState::Final;
                }
            }
        }

        // Each getter changes only its local and the control-state tag.
        // 2.1 Let rel be linkObject["relation_type"].
        transition! {
            get_rel() {
                require(pre.state == ProcessLinkHeadersState::LoopHead);
                update rel = Some((pre.link_object->Some_0).relation_type);
                update state = ProcessLinkHeadersState::RelRead;
            }
        }

        // 2.2 Let attribs be linkObject["target_attributes"].
        transition! {
            get_attribs() {
                require(pre.state == ProcessLinkHeadersState::RelRead);
                update attribs = Some((pre.link_object->Some_0).target_attributes);
                update state = ProcessLinkHeadersState::AttribsRead;
            }
        }

        // 2.3 Let expectedPhase be "media" if either "srcset", "imagesrcset", or "media" exist in attribs; otherwise "pre-media".
        transition! {
            get_exspecterd_phase() {
                require(pre.state == ProcessLinkHeadersState::AttribsRead);
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
                update state = ProcessLinkHeadersState::ExpectedPhaseRead;
            }
        }

        // 2.4 If expectedPhase is not phase, then continue.
        // Continue advances and selects the next link before entering LoopHead.
        transition! {
            check_phase() {
                require(pre.state == ProcessLinkHeadersState::ExpectedPhaseRead);
                if pre.expected_phase->Some_0 != pre.phase {
                    update i = pre.i + 1;
                    if pre.i + 1 < pre.links.len() {
                        update link_object = Some(pre.links[(pre.i + 1) as int]);
                        update state = ProcessLinkHeadersState::LoopHead;
                    } else {
                        update link_object = None;
                        update state = ProcessLinkHeadersState::Final;
                    }
                } else {
                    update state = ProcessLinkHeadersState::PhaseChecked;
                }
            }
        }

        // 2.5 If attribs["media"] exists and attribs["media"] does not match the environment, then continue.
        transition! {
            check_media() {
                require(pre.state == ProcessLinkHeadersState::PhaseChecked);
                let media = link_object_attribs_get(pre.attribs->Some_0, "media"@);
                if media is Some && !whatwg_matches_environment(media->Some_0, pre.doc) {
                    update i = pre.i + 1;
                    if pre.i + 1 < pre.links.len() {
                        update link_object = Some(pre.links[(pre.i + 1) as int]);
                        update state = ProcessLinkHeadersState::LoopHead;
                    } else {
                        update link_object = None;
                        update state = ProcessLinkHeadersState::Final;
                    }
                } else {
                    update state = ProcessLinkHeadersState::MediaChecked;
                }
            }
        }

        // 2.6 Let options be a new link processing options with
        transition! {
            create_new_option() {
                require(pre.state == ProcessLinkHeadersState::MediaChecked);
                update options = Some(LinkProcessingOptionsView {
                    href: (pre.link_object->Some_0).target_uri, // href: linkObject["target_uri"]
                    base_url: pre.doc.base_url, // base URL: doc's document base URL
                    origin: pre.doc.origin, // origin: doc's origin
                    environment: pre.doc.environment, // environment: doc's relevant settings object
                    policy_container: pre.doc.policy_container, // policy container: doc's policy container
                    document: Some(pre.doc), // document: doc
                    // Defaults for the remaining link-processing-options fields.
                    initiator: "link"@,
                    integrity: Seq::empty(),
                    type_: Seq::empty(),
                    cryptographic_nonce_metadata: Seq::empty(),
                    destination: Destination::None,
                    crossorigin: None,
                    referrer_policy: ReferrerPolicy::EmptyString,
                    source_set: None,
                    on_document_ready: None,
                    fetch_priority: LinkFetchPriority::Auto,
                });
                update state = ProcessLinkHeadersState::OptionsCreated;
            }
        }

        // 2.7 Apply link options from parsed header attributes to options given attribs and rel. If that returned false, then return.
        transition! {
            apply_link_options() {
                require(pre.state == ProcessLinkHeadersState::OptionsCreated);
                match whatwg_apply_link_options_from_parsed_header_attributes(
                    pre.options->Some_0, pre.attribs->Some_0, pre.rel->Some_0,
                ) {
                    None => {
                        update state = ProcessLinkHeadersState::Final;
                    },
                    Some(options) => {
                        update options = Some(options);
                        update state = ProcessLinkHeadersState::OptionsApplied;
                    },
                }
            }
        }

        // 2.8 If attribs["imagesrcset"] exists and attribs["imagesizes"] exists, then set options's source set to the result of creating a source set given linkObject["target_uri"], attribs["imagesrcset"], attribs["imagesizes"], and null.
        transition! {
            set_option_source_set() {
                require(pre.state == ProcessLinkHeadersState::OptionsApplied);
                let srcset = link_object_attribs_get(pre.attribs->Some_0, "imagesrcset"@);
                let sizes = link_object_attribs_get(pre.attribs->Some_0, "imagesizes"@);
                if srcset is Some && sizes is Some {
                    let options = pre.options->Some_0;
                    let source_set = whatwg_create_source_set(
                        (pre.link_object->Some_0).target_uri,
                        srcset->Some_0,
                        sizes->Some_0,
                        None, // null img
                        pre.doc,
                    );
                    update options = Some(LinkProcessingOptionsView {
                        source_set: Some(source_set),
                        ..options
                    });
                }
                update state = ProcessLinkHeadersState::SourceSetChecked;
            }
        }

        // 2.9 Run the process a link header steps for rel given options.
        // Record each invocation and its abstract postcondition at the starting document snapshot.
        transition! {
            process_link_header() {
                require(pre.state == ProcessLinkHeadersState::SourceSetChecked);
                let rel = pre.rel->Some_0;
                let options = pre.options->Some_0;
                update effects = pre.effects && process_link_header_ensures(rel, options, pre.doc);
                update calls = pre.calls.push((rel, options));
                update i = pre.i + 1;
                if pre.i + 1 < pre.links.len() {
                    update link_object = Some(pre.links[(pre.i + 1) as int]);
                    update state = ProcessLinkHeadersState::LoopHead;
                } else {
                    update link_object = None;
                    update state = ProcessLinkHeadersState::Final;
                }
            }
        }

        // Locals are required only after their assignment in the current iteration.
        // Before then, old values are allowed and will be overwritten.
        #[invariant]
        pub fn locals_available(&self) -> bool {
            match self.state {
                ProcessLinkHeadersState::Init | ProcessLinkHeadersState::Final => true,
                ProcessLinkHeadersState::LoopHead => self.link_object is Some,
                ProcessLinkHeadersState::RelRead => {
                    &&& self.link_object is Some
                    &&& self.rel is Some
                },
                ProcessLinkHeadersState::AttribsRead | ProcessLinkHeadersState::PhaseChecked
                    | ProcessLinkHeadersState::MediaChecked => {
                    &&& self.link_object is Some
                    &&& self.rel is Some
                    &&& self.attribs is Some
                },
                ProcessLinkHeadersState::ExpectedPhaseRead => {
                    &&& self.link_object is Some
                    &&& self.rel is Some
                    &&& self.attribs is Some
                    &&& self.expected_phase is Some
                },
                ProcessLinkHeadersState::OptionsCreated | ProcessLinkHeadersState::OptionsApplied
                    | ProcessLinkHeadersState::SourceSetChecked => {
                    &&& self.link_object is Some
                    &&& self.rel is Some
                    &&& self.attribs is Some
                    &&& self.options is Some
                },
            }
        }

        #[invariant]
        pub fn effects_agree_with_calls(&self) -> bool {
            self.effects == (forall|i: int| 0 <= i < self.calls.len() ==> {
                let (rel, options) = #[trigger] self.calls[i];
                process_link_header_ensures(rel, options, self.doc)
            })
        }

        #[inductive(initialize)]
        fn initialize_inductive(
            post: Self,
            links: Seq<LinkObject>,
            doc: DocumentView,
            phase: LinkHeadersPhase,
        ) {}

        #[inductive(init_loop_head)]
        fn init_loop_head_inductive(pre: Self, post: Self) {}

        #[inductive(get_rel)]
        fn get_rel_inductive(pre: Self, post: Self) {}

        #[inductive(get_attribs)]
        fn get_attribs_inductive(pre: Self, post: Self) {}

        #[inductive(get_exspecterd_phase)]
        fn get_exspecterd_phase_inductive(pre: Self, post: Self) {}

        #[inductive(check_phase)]
        fn check_phase_inductive(pre: Self, post: Self) {}

        #[inductive(check_media)]
        fn check_media_inductive(pre: Self, post: Self) {}

        #[inductive(create_new_option)]
        fn create_new_option_inductive(pre: Self, post: Self) {}

        #[inductive(apply_link_options)]
        fn apply_link_options_inductive(pre: Self, post: Self) {}

        #[inductive(set_option_source_set)]
        fn set_option_source_set_inductive(pre: Self, post: Self) {}

        #[inductive(process_link_header)]
        fn process_link_header_inductive(pre: Self, post: Self) {
            let n = pre.calls.len() as int;
            assert(pre.effects_agree_with_calls());
            assert(post.calls[n] == (pre.rel->Some_0, pre.options->Some_0));
            if post.effects {
                assert forall|i: int| 0 <= i < post.calls.len() implies
                    process_link_header_ensures((#[trigger] post.calls[i]).0, post.calls[i].1, post.doc) by {
                    if i < n {
                        assert(post.calls[i] == pre.calls[i]);
                    }
                }
            } else if !pre.effects {
                // A previous failed conjunct is still in the prefix.
                let i = choose|i: int| 0 <= i < n && !process_link_header_ensures(
                    (#[trigger] pre.calls[i]).0, pre.calls[i].1, pre.doc,
                );
                assert(post.calls[i] == pre.calls[i]);
                assert(!process_link_header_ensures(post.calls[i].0, post.calls[i].1, post.doc));
            } else {
                // The newly appended invocation is the failed conjunct.
                assert(!process_link_header_ensures(post.calls[n].0, post.calls[n].1, post.doc));
            }
            assert(post.effects_agree_with_calls());
        }
    }
}

// Every invariant-valid, nonfinal state has a successor, even when effects is false.
pub(crate) proof fn process_link_headers_next_exists(
    pre: process_link_headers_automaton::State,
) -> (post: process_link_headers_automaton::State)
    requires pre.invariant(), pre.state != ProcessLinkHeadersState::Final,
    ensures process_link_headers_automaton::State::next(pre, post), post.invariant(),
{
    use process_link_headers_automaton::{show, take_step};
    match pre.state {
        ProcessLinkHeadersState::Init => {
            let post = take_step::init_loop_head(pre);
            show::init_loop_head(pre, post);
            post
        },
        ProcessLinkHeadersState::LoopHead => {
            let post = take_step::get_rel(pre);
            show::get_rel(pre, post);
            post
        },
        ProcessLinkHeadersState::RelRead => {
            let post = take_step::get_attribs(pre);
            show::get_attribs(pre, post);
            post
        },
        ProcessLinkHeadersState::AttribsRead => {
            let post = take_step::get_exspecterd_phase(pre);
            show::get_exspecterd_phase(pre, post);
            post
        },
        ProcessLinkHeadersState::ExpectedPhaseRead => {
            let post = take_step::check_phase(pre);
            show::check_phase(pre, post);
            post
        },
        ProcessLinkHeadersState::PhaseChecked => {
            let post = take_step::check_media(pre);
            show::check_media(pre, post);
            post
        },
        ProcessLinkHeadersState::MediaChecked => {
            let post = take_step::create_new_option(pre);
            show::create_new_option(pre, post);
            post
        },
        ProcessLinkHeadersState::OptionsCreated => {
            let post = take_step::apply_link_options(pre);
            show::apply_link_options(pre, post);
            post
        },
        ProcessLinkHeadersState::OptionsApplied => {
            let post = take_step::set_option_source_set(pre);
            show::set_option_source_set(pre, post);
            post
        },
        ProcessLinkHeadersState::SourceSetChecked => {
            let post = take_step::process_link_header(pre);
            show::process_link_header(pre, post);
            post
        },
        ProcessLinkHeadersState::Final => pre,
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

// Representation | RFC B.2 steps 13-14; B.3 steps 8.1, 9-10 | ASCII name case normalization -> ordered string attributes.
// Exclude rel/anchor after lowercasing names; a valueless parameter becomes "".
// Keep order and duplicates. RFC B.2 step 14.2's first-occurrence rule for
// media/title/title*/type is not applied: link_object_attribs_get already takes
// the first match, so retaining those duplicates does not change its lookups.
pub(crate) open spec fn get_attributes(
    params: Seq<(Seq<char>, Option<Seq<char>>)>,
) -> Seq<(Seq<char>, Seq<char>)>
    decreases params.len(),
{
    if params.len() == 0 {
        Seq::empty()
    } else {
        let attributes = get_attributes(params.drop_last());
        let (name, value) = params.last();
        let name = ascii_lowercase(name);
        if name == "rel"@ || name == "anchor"@ {
            attributes
        } else {
            attributes.push((name, match value {
                Some(value) => value,
                None => Seq::empty(),
            }))
        }
    }
}

// Representation | RFC B.2 step 10 | A relation type exists only if rel contains a non-RWS character.
// RWS characters are exactly SP and HTAB, not CR/LF or other Unicode whitespace.
pub(crate) open spec fn contains_non_rws_char(s: Seq<char>) -> bool {
    exists|i: int| 0 <= i < s.len() && (#[trigger] s[i]) != ' ' && s[i] != '\t'
}

// Representation | RFC B.2 steps 9-10, 17.1; B.3 steps 8.1, 9 | ASCII key/relation case normalization -> optional link (R1).
// RFC B.2 step 10 yields no relation types for an empty or SP/HTAB-only rel.
// Under R1, a multi-token rel (e.g. "preload prefetch") stays one relation_type
// string, lowercased without trimming or splitting. Missing rel also yields None.
// For an absent or valueless lowercase rel, None matches Servo's continue when
// value_for_key("rel") is None: that placement of the skip is a representation
// difference. An explicit empty or RWS-only value, or differently cased key,
// need not agree with Servo; do not assume it.
pub(crate) open spec fn to_link_object(h: LinkHeaderView) -> Option<LinkObject> {
    let names = h.params.map_values(|param: (Seq<char>, Option<Seq<char>>)|
        ascii_lowercase(param.0));
    match names.index_of_first("rel"@) {
        None => None,
        Some(i) => {
            let rel = match h.params[i].1 {
                Some(value) => value,
                None => Seq::empty(),
            };
            if !contains_non_rws_char(rel) {
                None
            } else {
                Some(LinkObject {
                    target_uri: h.url,
                    relation_type: ascii_lowercase(rel),
                    target_attributes: get_attributes(h.params),
                })
            }
        },
    }
}

// Representation | RFC B.1 step 3.2; B.2 step 17.2 | Parsed headers -> ordered, optional-link compaction (R1).
// Recurse on the prefix and append the last contribution, preserving input order.
pub(crate) open spec fn to_link_objects(hs: Seq<LinkHeaderView>) -> Seq<LinkObject>
    decreases hs.len(),
{
    if hs.len() == 0 {
        Seq::empty()
    } else {
        to_link_objects(hs.drop_last()) + match to_link_object(hs.last()) {
            Some(o) => seq![o],
            None => Seq::empty(),
        }
    }
}

} // verus!
