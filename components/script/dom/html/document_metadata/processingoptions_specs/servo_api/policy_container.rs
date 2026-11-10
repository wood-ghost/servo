/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Policy-container correspondence, inspected 2026-10-30.
//! Native fields are projected directly. Missing CSP self-origin, embedder
//! reporting/report-only data and both integrity policies are explicit context.
//! No parser, enforcement, Arc allocation or reference-counting proof is implied.

use std::sync::Arc;
use content_security_policy::{CspList, Policy, Directive, PolicySource, PolicyDisposition};
use net_traits::policy_container::{PolicyContainer, EmbedderPolicy, EmbedderPolicyValue};
use vstd::prelude::*;
use super::super::processing::types::*;

verus! {

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExCspList(CspList);
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicy(Policy);
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExDirective(Directive);
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicySource(PolicySource);
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicyDisposition(PolicyDisposition);
// TRUSTED(scope): not verified yet; remove when net_traits::policy_container::EmbedderPolicy is verified
#[verifier::external_type_specification]
pub struct ExEmbedderPolicy(EmbedderPolicy);
// TRUSTED(scope): not verified yet; remove when net_traits::policy_container::EmbedderPolicyValue is verified
#[verifier::external_type_specification]
pub struct ExEmbedderPolicyValue(EmbedderPolicyValue);

pub(crate) struct PolicyContainerSupplement {
    pub csp_self_origin: OriginView,
    pub reporting_endpoint: Seq<char>,
    pub report_only_value: EmbedderPolicyValueView,
    pub report_only_reporting_endpoint: Seq<char>,
    pub integrity_policy: IntegrityPolicyView,
    pub report_only_integrity_policy: IntegrityPolicyView,
}

pub(crate) open spec fn directive_view(directive: Directive) -> CspDirectiveView {
    CspDirectiveView { name: directive.name@, value: directive.value@.map_values(|s: String| s@) }
}

pub(crate) open spec fn policy_view(policy: Policy) -> CspPolicyView {
    CspPolicyView {
        directive_set: policy.directive_set@.map_values(|d: Directive| directive_view(d)),
        disposition: match policy.disposition {
            PolicyDisposition::Enforce => CspDispositionView::Enforce,
            PolicyDisposition::Report => CspDispositionView::Report,
        },
        source: match policy.source {
            PolicySource::Header => CspSourceView::Header,
            PolicySource::Meta => CspSourceView::Meta,
        },
    }
}

pub(crate) open spec fn contents(native: &PolicyContainer, context: PolicyContainerSupplement) -> PolicyContainerView {
    PolicyContainerView {
        csp_list: CspListView {
            policies: match native.csp_list {
                None => Seq::empty(),
                Some(list) => list.0@.map_values(|p: Policy| policy_view(p)),
            },
            self_origin: context.csp_self_origin,
        },
        embedder_policy: EmbedderPolicyView {
            value: match native.embedder_policy.value {
                EmbedderPolicyValue::UnsafeNone => EmbedderPolicyValueView::UnsafeNone,
                EmbedderPolicyValue::RequireCorp => EmbedderPolicyValueView::RequireCorp,
            },
            reporting_endpoint: context.reporting_endpoint,
            report_only_value: context.report_only_value,
            report_only_reporting_endpoint: context.report_only_reporting_endpoint,
        },
        referrer_policy: ReferrerPolicyView::from_native(native.referrer_policy),
        integrity_policy: context.integrity_policy,
        report_only_integrity_policy: context.report_only_integrity_policy,
    }
}

/// A checkable relation on shared handles; it neither allocates an Arc in spec
/// mode nor assumes that arbitrary native CSP vectors are valid ordered sets.
pub(crate) open spec fn shared_contents_correspond(
    native: &Arc<PolicyContainer>, context: PolicyContainerSupplement, view: &Arc<PolicyContainerView>,
) -> bool {
    **view == contents(&**native, context)
}

} // verus!
