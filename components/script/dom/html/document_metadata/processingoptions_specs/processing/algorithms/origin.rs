/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Origin operations from HTML, snapshot and inspection 2026-10-30.

use vstd::prelude::*;
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
