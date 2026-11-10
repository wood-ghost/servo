/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Origin operations from HTML, snapshot and inspection 2026-10-30.

use vstd::prelude::*;
use verus_state_machines_macros::state_machine;
use super::super::types::{OriginView, HostView, host_equals};

verus! {

// https://html.spec.whatwg.org/multipage/browsers.html#same-origin
// Snapshot/inspection 2026-10-30.
// Form: row 3, direct spec fn; ordered returns without a carried temporary.
// Two origins, A and B, are said to be same origin if the following algorithm returns true:
pub(crate) open spec fn same_origin(a: OriginView, b: OriginView) -> bool {
    // 1. If A and B are the same opaque origin, then return true.
    if a is Opaque && b is Opaque && a->Opaque_identity == b->Opaque_identity {
        true
    // 2. If A and B are both tuple origins, their schemes and ports are identical, and their hosts are equal, then return true.
    } else if a is Tuple && b is Tuple
        && a->Tuple_scheme == b->Tuple_scheme
        && a->Tuple_port == b->Tuple_port
        && host_equals(a->Tuple_host, b->Tuple_host)
    {
        true
    } else {
        // 3. Return false.
        false
    }
}

// https://html.spec.whatwg.org/multipage/browsers.html#concept-origin
// An origin's equality, given origins A and B, returns true if A and B are same origin; otherwise false.
pub(crate) open spec fn origin_equals(a: OriginView, b: OriginView) -> bool {
    same_origin(a, b)
}

/// Domain mutation changes the record but must not change same-origin equality.
pub(crate) proof fn domain_does_not_change_same_origin(a: OriginView, domain: Option<super::super::types::HostView>)
    ensures same_origin(a, a.with_domain(domain)),
{}

pub(crate) proof fn same_origin_is_symmetric(a: OriginView, b: OriginView)
    ensures same_origin(a, b) == same_origin(b, a),
{}

// https://html.spec.whatwg.org/multipage/browsers.html#concept-origin-effective-domain
// Snapshot/inspection 2026-10-30.
// Form: row 3, direct spec fn; ordered returns with no temporary shared by steps.
// The effective domain of an origin origin is computed as follows:
pub(crate) open spec fn effective_domain(origin: OriginView) -> Option<HostView> {
    // 1. If origin is an opaque origin, then return null.
    if origin is Opaque {
        None
    // 2. If origin's domain is non-null, then return origin's domain.
    } else if origin->Tuple_domain is Some {
        origin->Tuple_domain
    } else {
        // 3. Return origin's host.
        Some(origin->Tuple_host)
    }
}

} // verus!

verus! {
// Closed projections keep the local view private while allowing generated
// public machine relations to use the standard's conditions.
pub closed spec fn same_opaque_origin(a: OriginView, b: OriginView) -> bool {
    a is Opaque && b is Opaque && a->Opaque_identity == b->Opaque_identity
}
pub closed spec fn both_tuple_origins(a: OriginView, b: OriginView) -> bool { a is Tuple && b is Tuple }
pub closed spec fn identical_schemes_and_nonnull_domains(a: OriginView, b: OriginView) -> bool {
    a->Tuple_scheme == b->Tuple_scheme && a->Tuple_domain == b->Tuple_domain && a->Tuple_domain is Some
}
pub closed spec fn same_origin_with_null_domains(a: OriginView, b: OriginView) -> bool {
    same_origin(a, b) && a->Tuple_domain is None && b->Tuple_domain is None
}
pub enum OriginDomainState { Step1, Step2, Step2_1, Step2_2, Step3, Final }
}

// https://html.spec.whatwg.org/multipage/browsers.html#same-origin-domain
// Snapshot/inspection 2026-10-30.
// Form: row 1, state_machine!; nested returns leave the enclosing Step 2 block.
// Keeping the parent and its substeps separate avoids merging their conditions.
state_machine! {
    same_origin_domain_machine {
        fields {
            // Immutable inputs
            pub a: OriginView,
            pub b: OriginView,
            pub state: OriginDomainState,
            pub result: bool,
        }
        init! {
            initialize(a: OriginView, b: OriginView) {
                init a = a;
                init b = b;
                init state = OriginDomainState::Step1;
                init result = false;
            }
        }
        // 1. If A and B are the same opaque origin, then return true.
        transition! {
            step1() {
                require(pre.state == OriginDomainState::Step1);
                if same_opaque_origin(pre.a, pre.b) {
                    update state = OriginDomainState::Final;
                    update result = true;
                } else {
                    update state = OriginDomainState::Step2;
                }
            }
        }
        // 2. If A and B are both tuple origins:
        transition! {
            step2() {
                require(pre.state == OriginDomainState::Step2);
                if both_tuple_origins(pre.a, pre.b) {
                    update state = OriginDomainState::Step2_1;
                } else {
                    update state = OriginDomainState::Step3;
                }
            }
        }
        // 2.1. If A and B's schemes are identical, and their domains are identical and non-null, then return true.
        transition! {
            step2_1() {
                require(pre.state == OriginDomainState::Step2_1);
                if identical_schemes_and_nonnull_domains(pre.a, pre.b) {
                    update state = OriginDomainState::Final;
                    update result = true;
                } else {
                    update state = OriginDomainState::Step2_2;
                }
            }
        }
        // 2.2. Otherwise, if A and B are same origin and their domains are both null, return true.
        transition! {
            step2_2() {
                require(pre.state == OriginDomainState::Step2_2);
                if same_origin_with_null_domains(pre.a, pre.b) {
                    update state = OriginDomainState::Final;
                    update result = true;
                } else {
                    update state = OriginDomainState::Step3;
                }
            }
        }
        // 3. Return false.
        transition! {
            step3() {
                require(pre.state == OriginDomainState::Step3);
                update state = OriginDomainState::Final;
                update result = false;
            }
        }
    }
}

verus! {

pub(crate) open spec fn origin_domain_advance(s: same_origin_domain_machine::State) -> same_origin_domain_machine::State {
    match s.state {
        OriginDomainState::Step1 => choose|post| same_origin_domain_machine::State::step1(s, post),
        OriginDomainState::Step2 => choose|post| same_origin_domain_machine::State::step2(s, post),
        OriginDomainState::Step2_1 => choose|post| same_origin_domain_machine::State::step2_1(s, post),
        OriginDomainState::Step2_2 => choose|post| same_origin_domain_machine::State::step2_2(s, post),
        OriginDomainState::Step3 => choose|post| same_origin_domain_machine::State::step3(s, post),
        OriginDomainState::Final => s,
    }
}

pub(crate) open spec fn origin_domain_run(a: OriginView, b: OriginView) -> same_origin_domain_machine::State {
    let s = choose|s| same_origin_domain_machine::State::initialize(s, a, b);
    origin_domain_advance(origin_domain_advance(origin_domain_advance(origin_domain_advance(origin_domain_advance(s)))))
}

pub(crate) open spec fn same_origin_domain(a: OriginView, b: OriginView) -> bool {
    origin_domain_run(a, b).result
}

proof fn origin_domain_advance_correct(s: same_origin_domain_machine::State)
    ensures
        s.state == OriginDomainState::Step1 ==> same_origin_domain_machine::State::step1(s, origin_domain_advance(s)),
        s.state == OriginDomainState::Step2 ==> same_origin_domain_machine::State::step2(s, origin_domain_advance(s)),
        s.state == OriginDomainState::Step2_1 ==> same_origin_domain_machine::State::step2_1(s, origin_domain_advance(s)),
        s.state == OriginDomainState::Step2_2 ==> same_origin_domain_machine::State::step2_2(s, origin_domain_advance(s)),
        s.state == OriginDomainState::Step3 ==> same_origin_domain_machine::State::step3(s, origin_domain_advance(s)),
        s.state == OriginDomainState::Final ==> origin_domain_advance(s) == s,
{
    match s.state {
        OriginDomainState::Step1 => {
            let post = same_origin_domain_machine::take_step::step1(s);
            assert(same_origin_domain_machine::State::step1(s, post));
        },
        OriginDomainState::Step2 => {
            let post = same_origin_domain_machine::take_step::step2(s);
            assert(same_origin_domain_machine::State::step2(s, post));
        },
        OriginDomainState::Step2_1 => {
            let post = same_origin_domain_machine::take_step::step2_1(s);
            assert(same_origin_domain_machine::State::step2_1(s, post));
        },
        OriginDomainState::Step2_2 => {
            let post = same_origin_domain_machine::take_step::step2_2(s);
            assert(same_origin_domain_machine::State::step2_2(s, post));
        },
        OriginDomainState::Step3 => {
            let post = same_origin_domain_machine::take_step::step3(s);
            assert(same_origin_domain_machine::State::step3(s, post));
        },
        OriginDomainState::Final => {},
    }
}

pub(crate) broadcast proof fn origin_domain_result(a: OriginView, b: OriginView)
    ensures
        (#[trigger] origin_domain_run(a, b)).state == OriginDomainState::Final,
        origin_domain_run(a, b).a == a,
        origin_domain_run(a, b).b == b,
        origin_domain_run(a, b).result == (same_opaque_origin(a, b) ||
            (both_tuple_origins(a, b) && (identical_schemes_and_nonnull_domains(a, b)
                || same_origin_with_null_domains(a, b)))),
{
    let s = same_origin_domain_machine::take_step::initialize(a, b);
    assert(s == (choose|s| same_origin_domain_machine::State::initialize(s, a, b)));
    origin_domain_advance_correct(s);
    let s = origin_domain_advance(s);
    origin_domain_advance_correct(s);
    let s = origin_domain_advance(s);
    origin_domain_advance_correct(s);
    let s = origin_domain_advance(s);
    origin_domain_advance_correct(s);
    let s = origin_domain_advance(s);
    origin_domain_advance_correct(s);
}

/// The HTML table's port/domain distinction: different ports prevent same-origin
/// equality, but equal non-null domains can still establish same origin-domain.
proof fn relaxed_domain_can_ignore_port() {
    let host = HostView::Domain { value: "example.org"@ };
    let a = OriginView::Tuple { scheme: "https"@, host, port: Some(314u16), domain: Some(host) };
    let b = OriginView::Tuple { scheme: "https"@, host, port: Some(420u16), domain: Some(host) };
    origin_domain_result(a, b);
    assert(!same_origin(a, b));
    assert(same_origin_domain(a, b));
}

} // verus!
