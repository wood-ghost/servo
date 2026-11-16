/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use content_security_policy::{PolicyDisposition, PolicySource};
use vstd::prelude::*;

use super::super::processing::types::Origin;

verus! {

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicyDisposition(PolicyDisposition);

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicySource(PolicySource);

// https://w3c.github.io/webappsec-csp/#csp-list
// Editor's Draft 2026-10-27; inspected 2026-10-30.
// A CSP list is a struct consisting of policies (a list of policies) and a self-origin (an origin which is used when matching the 'self' keyword).
pub(crate) struct CspListView {
    pub policies: Seq<CspPolicyView>,
    pub self_origin: Origin,
}

// https://w3c.github.io/webappsec-csp/#content-security-policy-object
pub(crate) struct CspPolicyView {
    // Each policy has an associated directive set, which is an ordered set of directives that define the policy’s implications when applied.
    pub directive_set: Seq<CspDirectiveView>,
    // Each policy has an associated disposition, which is either "enforce" or "report".
    pub disposition: PolicyDisposition,
    // Each policy has an associated source, which is either "header" or "meta".
    pub source: PolicySource,
}

// https://w3c.github.io/webappsec-csp/#directives
// Each directive is a name / value pair. The name is a non-empty string, and the value is a set of non-empty strings. The value MAY be empty.
pub(crate) struct CspDirectiveView { pub name: Seq<char>, pub value: Seq<Seq<char>> }

} // verus!
