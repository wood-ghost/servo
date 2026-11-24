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
use net_traits::mime_classifier_specs::mime_api::is_http_whitespace;
use net_traits::request::Destination;
use vstd::prelude::*;

use crate::dom::document::Document;
use super::super::super::processingoptions::{LinkHeader, LinkProcessingPhase};
use super::super::postconditions::apply_header_attributes;
use super::super::servo_support::create_source_set;
use super::model::LinkProcessingOptionsView;
use super::types::{DocumentView, LinkFetchPriority, LinkHeaderView, LinkHeaderViewSpec};

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
    // TODO: Specify the effects of processing the extracted links in order.
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

// Step 2.5: CSS media-query-list parsing and evaluation against the user's environment.
// https://drafts.csswg.org/cssom/#parse-a-media-query-list
// https://drafts.csswg.org/mediaqueries/#mq-list
// This guard controls dispatch; the fields passed to dispatch do not depend on its value.
pub uninterp spec fn media_query_list_matches_environment(media: Seq<char>, doc: DocumentView) -> bool;

// https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#matches-the-environment
pub open spec fn whatwg_matches_environment(
    media: Seq<char>,
    doc: DocumentView,
) -> bool {
    // A string matches the environment of the user if it is the empty string
    ||| media == Seq::empty()
    // a string consisting of only ASCII whitespace
    ||| media.all(|c: char| is_http_whitespace(c) || c == '\u{000C}') // TODO: use is_ascii_whitespace later
    // or is a media query list that matches the user's environment according to the definitions given in Media Queries. [MQ]
    ||| media_query_list_matches_environment(media, doc)
}

// https://w3c.github.io/webappsec-referrer-policy/#referrer-policy
// A referrer policy is the empty string, "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", or "unsafe-url".
pub(crate) open spec fn referrer_policy_for_token(value: Seq<char>) -> Option<ReferrerPolicy> {
    let keyword = ascii_lowercase(value);
    if keyword == ""@ { Some(ReferrerPolicy::EmptyString) }
    else if keyword == "no-referrer"@ { Some(ReferrerPolicy::NoReferrer) }
    else if keyword == "no-referrer-when-downgrade"@ { Some(ReferrerPolicy::NoReferrerWhenDowngrade) }
    else if keyword == "same-origin"@ { Some(ReferrerPolicy::SameOrigin) }
    else if keyword == "origin"@ { Some(ReferrerPolicy::Origin) }
    else if keyword == "strict-origin"@ { Some(ReferrerPolicy::StrictOrigin) }
    else if keyword == "origin-when-cross-origin"@ { Some(ReferrerPolicy::OriginWhenCrossOrigin) }
    else if keyword == "strict-origin-when-cross-origin"@ { Some(ReferrerPolicy::StrictOriginWhenCrossOrigin) }
    else if keyword == "unsafe-url"@ { Some(ReferrerPolicy::UnsafeUrl) }
    else { None }
}

// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#fetch-priority-attribute
pub(crate) open spec fn fetch_priority_for_token(value: Seq<char>) -> Option<LinkFetchPriority> {
    let keyword = ascii_lowercase(value);
    if keyword == "high"@ { Some(LinkFetchPriority::High) }
    else if keyword == "low"@ { Some(LinkFetchPriority::Low) }
    else if keyword == "auto"@ { Some(LinkFetchPriority::Auto) }
    else { None }
}

/// One iteration of <https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers>.
/// Returns the options passed at step 2.9. Both a skip and an early return yield None.
pub(crate) open spec fn process_link_headers_loop_body(
    link: LinkObject,
    doc: DocumentView,
    phase: LinkProcessingPhase,
) -> Option<LinkProcessingOptionsView> {
    // 1. Let rel be linkObject["relation_type"].
    let rel = link.relation_type;
    // 2. Let attribs be linkObject["target_attributes"].
    let attribs = link.target_attributes;
    // 3. Let expectedPhase be "media" if either "srcset", "imagesrcset", or "media" exist in attribs; otherwise "pre-media".
    let media = link_object_attribs_get(attribs, "media"@);
    let expected_phase = if link_object_attribs_get(attribs, "srcset"@) is Some
        || link_object_attribs_get(attribs, "imagesrcset"@) is Some || media is Some {
        LinkProcessingPhase::Media
    } else {
        LinkProcessingPhase::PreMedia
    };
    // 4. If expectedPhase is not phase, then continue.
    if expected_phase != phase {
        None
    } else {
        // 5. If attribs["media"] exists and attribs["media"] does not match the environment, then continue.
        if media is Some && !whatwg_matches_environment(media->Some_0, doc) {
            None
        } else {
            // 6. Let options be a new link processing options with
            let options = LinkProcessingOptionsView {
                href: link.target_uri, // href: linkObject["target_uri"]
                base_url: doc.base_url, // base URL: doc's document base URL
                origin: doc.origin, // origin: doc's origin
                environment: doc.environment, // environment: doc's relevant settings object
                policy_container: doc.policy_container, // policy container: doc's policy container
                document: Some(doc), // document: doc
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
            };
            // 7. Apply link options from parsed header attributes to options given attribs and rel. If that returned false, then return.
            match apply_header_attributes(options, attribs, rel) {
                None => None,
                Some(applied) => {
                    // 8. If attribs["imagesrcset"] exists and attribs["imagesizes"] exists, then set options's source set to the result of creating a source set given linkObject["target_uri"], attribs["imagesrcset"], attribs["imagesizes"], and null.
                    let options = match (link_object_attribs_get(attribs, "imagesrcset"@), link_object_attribs_get(attribs, "imagesizes"@)) {
                        (Some(srcset), Some(sizes)) => LinkProcessingOptionsView {
                            source_set: Some(create_source_set(link.target_uri, srcset, sizes, doc)@),
                            ..applied
                        },
                        _ => applied,
                    };
                    // 9. These are the options passed to process a link header for rel.
                    Some(options)
                },
            }
        }
    }
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

proof fn first_index_after_push(keys: Seq<Seq<char>>, last: Seq<char>, key: Seq<char>)
    ensures keys.push(last).index_of_first(key) == match keys.index_of_first(key) {
        Some(i) => Some(i),
        None if last == key => Some(keys.len() as int),
        None => None,
    },
{
    let all = keys.push(last);
    keys.index_of_first_ensures(key);
    all.index_of_first_ensures(key);
    if let Some(i) = keys.index_of_first(key) {
        assert(all[i] == key);
        let j = all.index_of_first(key)->Some_0;
        if j < i { assert(keys[j] == all[j]); }
        if i < j { assert(all[i] != key); }
    } else if let Some(j) = all.index_of_first(key) {
        if j < keys.len() { assert(keys[j] == key); }
    } else if last == key {
        assert(all[keys.len() as int] == key);
    }
}

// get_attributes preserves the first matching parameter; only a missing value becomes "".
pub(crate) proof fn first_normalized_attribute(header: LinkHeaderView, key: Seq<char>)
    requires
        super::super::requires::link_header_key_is_lowercase(header, key),
        ascii_lowercase(key) == key, key != "rel"@, key != "anchor"@,
    ensures
        link_object_attribs_get(get_attributes(header.params), key) ==
            (if super::types::link_header_has_key(header, key) {
                Some(super::types::link_header_value_for_key(header, key).unwrap_or(Seq::empty()))
            } else { None }),
    decreases header.params.len(),
{
    let p = header.params;
    let keys = p.map_values(|entry: (Seq<char>, Option<Seq<char>>)| entry.0);
    super::types::link_header_lookup_properties(header, key);
    if p.len() == 0 {
        let names = get_attributes(p).map_values(|entry: (Seq<char>, Seq<char>)| entry.0);
        names.index_of_first_ensures(key);
    } else {
        let prefix = LinkHeaderView { params: p.drop_last(), ..header };
        let (name, value) = p.last();
        let prior = prefix.params.map_values(|entry: (Seq<char>, Option<Seq<char>>)| entry.0);
        assert(keys =~= prior.push(name));
        first_index_after_push(prior, name, key);
        super::types::link_header_lookup_properties(prefix, key);
        assert forall|i: int| 0 <= i < prefix.params.len() implies
            (ascii_lowercase((#[trigger] prefix.params[i]).0) == key ==> prefix.params[i].0 == key) by {
            assert(prefix.params[i] == p[i]);
        }
        first_normalized_attribute(prefix, key);
        assert((ascii_lowercase(name) == key) == (name == key)) by { assert(name == p[p.len()-1].0); }
        if ascii_lowercase(name) != "rel"@ && ascii_lowercase(name) != "anchor"@ {
            let attrs = get_attributes(prefix.params);
            let names = attrs.map_values(|entry: (Seq<char>, Seq<char>)| entry.0);
            let all = get_attributes(p).map_values(|entry: (Seq<char>, Seq<char>)| entry.0);
            assert(all =~= names.push(ascii_lowercase(name)));
            first_index_after_push(names, ascii_lowercase(name), key);
            names.index_of_first_ensures(key);
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
