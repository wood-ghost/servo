/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! URL equivalence, snapshot 2026-10-19, inspected 2026-10-30.
//! https://url.spec.whatwg.org/#concept-url-equals
//! Form: row 1, state_machine!; serializedA and serializedB are carried to step 3.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::types::{ServoUrlView, HostView, UrlPathView, BlobUrlEntryView};

verus! {

// https://url.spec.whatwg.org/#concept-url-serializer
// Only serialized components are supplied: blob-cache identity cannot affect
// serialization. Host/path/integer serialization are still linked boundaries.
// TRUSTED(scope): not verified yet; remove when the URL serializer is verified
pub(crate) uninterp spec fn serialize_url_components(
    scheme: Seq<char>, username: Seq<char>, password: Seq<char>, host: Option<HostView>,
    port: Option<u16>, path: UrlPathView, query: Option<Seq<char>>, fragment: Option<Seq<char>>,
) -> Seq<char>;

/// Interface to the linked serializer with its optional exclude-fragment input.
pub closed spec fn serialize_url(url: ServoUrlView, exclude_fragment: bool) -> Seq<char> {
    serialize_url_components(url.scheme, url.username, url.password, url.host,
        url.port, url.path, url.query, if exclude_fragment { None } else { url.fragment })
}

pub enum UrlEqualityState { Step1, Step2, Step3, Final }
}

state_machine! {
    url_equality_machine {
        fields {
            // Immutable inputs
            pub a: ServoUrlView,
            pub b: ServoUrlView,
            pub exclude_fragments: bool,
            // temporary var used by Steps 1–3
            pub serialized_a: Seq<char>,
            // temporary var used by Steps 2–3
            pub serialized_b: Seq<char>,
            pub state: UrlEqualityState,
            pub result: bool,
        }
        init! {
            initialize(a: ServoUrlView, b: ServoUrlView, exclude_fragments: bool) {
                init a = a;
                init b = b;
                init exclude_fragments = exclude_fragments;
                init serialized_a = Seq::empty();
                init serialized_b = Seq::empty();
                init state = UrlEqualityState::Step1;
                init result = false;
            }
        }
        // 1. Let serializedA be the result of serializing A, with exclude fragment set to exclude fragments.
        transition! {
            step1() {
                require(pre.state == UrlEqualityState::Step1);
                update serialized_a = serialize_url(pre.a, pre.exclude_fragments);
                update state = UrlEqualityState::Step2;
            }
        }
        // 2. Let serializedB be the result of serializing B, with exclude fragment set to exclude fragments.
        transition! {
            step2() {
                require(pre.state == UrlEqualityState::Step2);
                update serialized_b = serialize_url(pre.b, pre.exclude_fragments);
                update state = UrlEqualityState::Step3;
            }
        }
        // 3. Return true if serializedA is serializedB; otherwise false.
        transition! {
            step3() {
                require(pre.state == UrlEqualityState::Step3);
                if pre.serialized_a == pre.serialized_b {
                    update state = UrlEqualityState::Final;
                    update result = true;
                } else {
                    update state = UrlEqualityState::Final;
                    update result = false;
                }
            }
        }
    }
}

verus! {

pub(crate) open spec fn url_equality_run(a: ServoUrlView, b: ServoUrlView, exclude_fragments: bool) -> url_equality_machine::State {
    let initial = choose|s| url_equality_machine::State::initialize(s, a, b, exclude_fragments);
    let s1 = choose|s| url_equality_machine::State::step1(initial, s);
    let s2 = choose|s| url_equality_machine::State::step2(s1, s);
    choose|s| url_equality_machine::State::step3(s2, s)
}

pub(crate) open spec fn url_equals(a: ServoUrlView, b: ServoUrlView, exclude_fragments: bool) -> bool {
    url_equality_run(a, b, exclude_fragments).result
}

/// URL equality's optional exclude-fragments argument defaults to false.
pub(crate) open spec fn url_equal(a: ServoUrlView, b: ServoUrlView) -> bool { url_equals(a, b, false) }

pub(crate) broadcast proof fn url_equality_result(a: ServoUrlView, b: ServoUrlView, exclude_fragments: bool)
    ensures
        (#[trigger] url_equality_run(a, b, exclude_fragments)).state == UrlEqualityState::Final,
        url_equality_run(a, b, exclude_fragments).a == a,
        url_equality_run(a, b, exclude_fragments).b == b,
        url_equality_run(a, b, exclude_fragments).result ==
            (serialize_url(a, exclude_fragments) == serialize_url(b, exclude_fragments)),
{
    let initial = url_equality_machine::take_step::initialize(a, b, exclude_fragments);
    assert(initial == (choose|s| url_equality_machine::State::initialize(s, a, b, exclude_fragments)));
    let s1 = url_equality_machine::take_step::step1(initial);
    assert(url_equality_machine::State::step1(initial, s1));
    assert(s1 == (choose|s| url_equality_machine::State::step1(initial, s)));
    let s2 = url_equality_machine::take_step::step2(s1);
    assert(url_equality_machine::State::step2(s1, s2));
    assert(s2 == (choose|s| url_equality_machine::State::step2(s1, s)));
    let s3 = url_equality_machine::take_step::step3(s2);
    assert(url_equality_machine::State::step3(s2, s3));
    assert(s3 == (choose|s| url_equality_machine::State::step3(s2, s)));
}

/// Structural record equality would incorrectly include the associated blob entry.
pub(crate) proof fn equality_ignores_blob_entry(a: ServoUrlView, entry: Option<BlobUrlEntryView>)
    ensures url_equal(a, ServoUrlView { blob_url_entry: entry, ..a }),
{
    url_equality_result(a, ServoUrlView { blob_url_entry: entry, ..a }, false);
}

pub(crate) proof fn excluding_fragment_ignores_fragment(a: ServoUrlView, fragment: Option<Seq<char>>)
    ensures url_equals(a, ServoUrlView { fragment, ..a }, true),
{
    url_equality_result(a, ServoUrlView { fragment, ..a }, true);
}

} // verus!
