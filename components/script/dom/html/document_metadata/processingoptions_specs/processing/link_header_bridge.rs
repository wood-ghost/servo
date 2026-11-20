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

use vstd::prelude::*;

use super::process_link_headers::LinkObject;
use super::types::LinkHeaderView;

// ASCII lowercase, reused from mime_api.
pub(crate) use net_traits::mime_classifier_specs::mime_api::ascii_lowercase;

verus! {

// Representation | RFC B.2 steps 13-14; B.3 steps 8.1, 9-10 | ASCII name case normalization -> ordered string attributes.
// Exclude rel/anchor after lowercasing names; a valueless parameter becomes "".
// Keep order and duplicates. RFC B.2 step 14.2's first-occurrence rule for
// media/title/title*/type is not applied: link_object_attribs_get already takes
// the first match, so retaining those duplicates does not change its lookups.
pub(crate) open spec fn attributes_of(
    params: Seq<(Seq<char>, Option<Seq<char>>)>,
) -> Seq<(Seq<char>, Seq<char>)>
    decreases params.len(),
{
    if params.len() == 0 {
        Seq::empty()
    } else {
        let attributes = attributes_of(params.drop_last());
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
pub(crate) open spec fn link_object_of(h: LinkHeaderView) -> Option<LinkObject> {
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
                    target_attributes: attributes_of(h.params),
                })
            }
        },
    }
}

// Representation | RFC B.1 step 3.2; B.2 step 17.2 | Parsed headers -> ordered, optional-link compaction (R1).
// Recurse on the prefix and append the last contribution, preserving input order.
pub(crate) open spec fn links_of(hs: Seq<LinkHeaderView>) -> Seq<LinkObject>
    decreases hs.len(),
{
    if hs.len() == 0 {
        Seq::empty()
    } else {
        links_of(hs.drop_last()) + match link_object_of(hs.last()) {
            Some(o) => seq![o],
            None => Seq::empty(),
        }
    }
}

// Representation | RFC B.1 step 3.2; B.2 step 17.2 | One appended header contributes zero or one link (R1).
pub(crate) proof fn links_of_push(hs: Seq<LinkHeaderView>, h: LinkHeaderView)
    ensures
        links_of(hs.push(h)) == links_of(hs) + (match link_object_of(h) {
            Some(o) => seq![o],
            None => seq![],
        }),
{
    assert(hs.push(h).drop_last() =~= hs);
}

// Representation | RFC B.1 step 3.2; B.2 step 17.2 | Advancing a header prefix may leave the link prefix unchanged.
pub(crate) proof fn links_of_take_succ(hs: Seq<LinkHeaderView>, j: int)
    requires 0 <= j < hs.len(),
    ensures
        links_of(hs.take(j + 1)) == links_of(hs.take(j)) + (match link_object_of(hs[j]) {
            Some(o) => seq![o],
            None => seq![],
        }),
{
    hs.lemma_take_succ_push(j);
    links_of_push(hs.take(j), hs[j]);
}

} // verus!
