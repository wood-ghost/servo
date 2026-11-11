/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// assume_specification generates unsafe scaffolding in this verifier-only module.
#![allow(unsafe_code)]

//! Temporary type declarations needed to inspect LinkProcessingOptions.
//! Replace these declarations as their owning modules gain Verus support.
//! Method contracts assume dependency behavior rather than verifying its implementation.

use content_security_policy::{PolicyDisposition, PolicySource};
use js::gc::Traceable;
use net_traits::ReferrerPolicy;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{CorsSettings, Destination, Referrer, RequestClient};
use script_bindings::refcounted::Trusted;
use script_bindings::reflector::DomObject;
use servo_base::id::WebViewId;
use servo_url::{ImmutableOrigin, ServoUrl};
use vstd::prelude::*;

use crate::dom::document::Document;
use crate::dom::srcset::SourceSet;
use crate::dom::types::HTMLLinkElement;

verus! {

// Destination is transparent so the type-matching body can name its variants.
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExDestination(Destination);

pub open spec fn destination_is_script_like(destination: Destination) -> bool {
    matches!(
        destination,
        Destination::AudioWorklet
            | Destination::PaintWorklet
            | Destination::Script
            | Destination::ServiceWorker
            | Destination::SharedWorker
            | Destination::Worker
            | Destination::Xslt
    )
}

// TODO: Remove this local assumption when a verified or shared contract for
// Destination::is_script_like is supplied by its owning dependency or net_traits.
// Verifying net_traits alone does not verify this content-security-policy method.
// Matches the implementation in content-security-policy 0.9.0, src/lib.rs:1025-1031.
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
pub assume_specification[Destination::is_script_like](
    destination: Destination,
) -> (result: bool)
    ensures
        result == destination_is_script_like(destination),
;

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
pub assume_specification[ <Destination as PartialEq<Destination>>::eq ](
    left: &Destination,
    right: &Destination,
) -> (result: bool)
    ensures
        result == (*left == *right),
;

// TRUSTED(scope): not verified yet; remove when net_traits::request::CorsSettings is verified
#[verifier::external_type_specification]
pub struct ExCorsSettings(CorsSettings);

// TRUSTED(scope): not verified yet; remove when net_traits::ReferrerPolicy is verified
#[verifier::external_type_specification]
pub struct ExReferrerPolicy(ReferrerPolicy);

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicyDisposition(PolicyDisposition);

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicySource(PolicySource);

// TRUSTED(scope): not verified yet; remove when net_traits::policy_container is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPolicyContainer(PolicyContainer);

// TRUSTED(scope): not verified yet; remove when dom::srcset is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExSourceSet(SourceSet);

// TRUSTED(scope): not verified yet; remove when servo_url::ServoUrl is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExServoUrl(ServoUrl);

// Required by ServoUrl's existing FromStr implementation.
// TRUSTED(external-lib): url 2.5.8; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExUrlParseError(url::ParseError);

// TRUSTED(scope): not verified yet; remove when servo_url::ImmutableOrigin is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExImmutableOrigin(ImmutableOrigin);

// TRUSTED(scope): not verified yet; remove when net_traits::request::Referrer is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExReferrer(Referrer);

// TRUSTED(scope): not verified yet; remove when net_traits::request::RequestClient is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRequestClient(RequestClient);

// Types needed by preload's callable signature. Their declarations expose type
// identities and bounds; document state, GC, and reference-counting remain unmodeled.

// TRUSTED(scope): not verified yet; remove when servo_base::id::WebViewId is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExWebViewId(WebViewId);

// Bound-only declaration; tracing operations and GC safety remain unmodeled.
// TRUSTED(external-lib): mozjs 0.26.9; validated by not yet
#[verifier::external_trait_specification]
pub unsafe trait ExTraceable {
    type ExternalTraitSpecificationFor: Traceable;
}

// Preserve the native supertrait/lifetime bounds required by Trusted<T>.
// TRUSTED(scope): not verified yet; remove when script_bindings::reflector is verified
#[verifier::external_trait_specification]
pub trait ExDomObject: Traceable + 'static {
    type ExternalTraitSpecificationFor: DomObject;
}

// TRUSTED(scope): not verified yet; remove when script_bindings::refcounted is verified
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExTrusted<T: DomObject>(Trusted<T>);

// TRUSTED(scope): not verified yet; remove when dom::html::document_metadata::htmllinkelement is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExHTMLLinkElement(HTMLLinkElement);

// TRUSTED(scope): not verified yet; remove when dom::document is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExDocument(Document);

} // verus!
