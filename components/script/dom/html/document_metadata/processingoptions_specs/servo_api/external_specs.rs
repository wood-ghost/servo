/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// assume_specification generates unsafe scaffolding in this verifier-only module.
#![allow(unsafe_code)]

//! Temporary Verus support for Servo types and dependency APIs used by link processing.
//!
//! Destination, ReferrerPolicy, RequestBuilder, and the owned link-header records
//! are transparent so verified code can use their variants and public fields.
//! Other external types are opaque. The DOM accessor and source-set contracts expose Rust return
//! types/lifetimes and uninterpreted result relations. Document/window mappings,
//! identifiers, media-query evaluation, source-set contents, request-building,
//! and fetch behavior still need models. Borrowing and GC discipline remain
//! outside these temporary contracts.
//! Opaque preload-signature types from servo_base, script_bindings, and the DOM
//! are also declared here until their owning modules provide Verus support.
//! The DOM trait declarations preserve the bounds used by these signatures;
//! they provide no contracts for casting, reflection, or tracing operations.
//! Named postcondition boundaries live in `processing::context_specs`; they are
//! connected here so their behavior can be specified incrementally.
//!
//! These declarations assume the behavior of external methods; they do not
//! verify their implementations. A re-export through net_traits does not change
//! which dependency owns the implementation.

use ::net_traits::ReferrerPolicy;
use ::net_traits::policy_container::PolicyContainer;
use ::net_traits::request::{CorsSettings, Destination, Referrer, RequestBuilder, RequestClient};
use js::context::NoGC;
use js::gc::Traceable;
use nom_rfc8288::complete::{LinkDataOwned, LinkParamOwned};
use script_bindings::conversions::{DerivedFrom, IDLInterface};
use script_bindings::inheritance::Castable;
use script_bindings::refcounted::Trusted;
use script_bindings::reflector::DomObject;
use script_bindings::root::{Dom, DomRoot, Root, StableTraceObject};
use servo_base::id::WebViewId;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use std::cell::Ref;
use std::sync::Arc;
use vstd::prelude::*;

use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::medialist::MediaList;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::srcset::SourceSet;
use crate::dom::types::HTMLLinkElement;
use crate::dom::window::Window;

use super::views::destination_name;
use super::super::processing::context_specs as Spec;

verus! {

// ----------------------------------------
// Destination
// ----------------------------------------

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExDestination(Destination);

// Trusted correspondence with content-security-policy 0.9.0's exhaustive as_str
// match (src/lib.rs:1033). The string semantics agree with the typed abstraction's view.
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
pub assume_specification[Destination::as_str](destination: &Destination) -> (result: &'static str)
    ensures
        result@ == destination_name(*destination),
;

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

// ----------------------------------------
// CorsSettings
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when net_traits::request::CorsSettings is verified
#[verifier::external_type_specification]
pub struct ExCorsSettings(CorsSettings);

// ----------------------------------------
// ReferrerPolicy
// ----------------------------------------

// Transparent so process_link_headers can construct EmptyString.
// TRUSTED(scope): not verified yet; remove when net_traits::ReferrerPolicy is verified
#[verifier::external_type_specification]
pub struct ExReferrerPolicy(ReferrerPolicy);

// Native conversion: invalid tokens become EmptyString. HTML's apply-attributes
// algorithm instead leaves the existing policy unchanged for invalid keywords.
// Preserve the impl's lifetime binder as well as the argument's elided binder.
// TRUSTED(scope): not verified yet; remove when ReferrerPolicy::from is verified
pub assume_specification<'a>[<ReferrerPolicy as From<&str>>::from](
    value: &str,
) -> (result: ReferrerPolicy)
    ensures result == Spec::native_referrer_policy_result(value@),
;

// ----------------------------------------
// LinkParamOwned (nom-rfc8288 0.4.0)
// ----------------------------------------

// Public fields are imported from the pinned dependency; header lookup and
// parsing behavior are not specified by these type declarations.
// TRUSTED(external-lib): nom-rfc8288 0.4.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExLinkParamOwned(LinkParamOwned);

// ----------------------------------------
// LinkDataOwned (nom-rfc8288 0.4.0)
// ----------------------------------------

// TRUSTED(external-lib): nom-rfc8288 0.4.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExLinkDataOwned(LinkDataOwned);

// ----------------------------------------
// PolicyContainer
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when net_traits::policy_container is verified
#[verifier::external_type_specification]
pub struct ExPolicyContainer(PolicyContainer);

// ----------------------------------------
// SourceSet
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when dom::srcset is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExSourceSet(SourceSet);

// TODO: Verify source-set construction in its owning module.
// The postcondition is an uninterpreted scaffold for its parsing/normalization model.
// TRUSTED(scope): not verified yet; remove when SourceSet::create_source_set is verified
pub assume_specification[SourceSet::create_source_set](
    default_source: &str,
    srcset: &str,
    sizes: &str,
    document: &Document,
) -> (result: SourceSet)
    ensures
        Spec::source_set_create_source_set_ensures(
            default_source@, srcset@, sizes@, document, &result,
        ),
;

// ----------------------------------------
// ServoUrl
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when servo_url::ServoUrl is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExServoUrl(ServoUrl);

// ----------------------------------------
// url::ParseError
// ----------------------------------------

// Required by ServoUrl's existing FromStr implementation.
// TRUSTED(external-lib): url 2.5.8; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExUrlParseError(url::ParseError);

// ----------------------------------------
// ImmutableOrigin
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when servo_url::ImmutableOrigin is verified
#[verifier::external_type_specification]
pub struct ExImmutableOrigin(ImmutableOrigin);

// TRUSTED(scope): not verified yet; remove when servo_url::OpaqueOrigin is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExOpaqueOrigin(servo_url::OpaqueOrigin);

// TRUSTED(external-lib): url 2.5.8; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(S)]
pub struct ExHost<S>(servo_url::Host<S>);

// ----------------------------------------
// ToOwned for Clone types
// ----------------------------------------

// TODO: Move this delegation contract to vstd when available. Rust 1.99.1's
// alloc/src/borrow.rs implements this blanket to_owned by calling self.clone().
// Inherit Clone's contract rather than assuming that arbitrary clones are equal.
// TRUSTED(external-lib): Rust alloc 1.99.1; validated by not yet
pub assume_specification<T: Clone>[<T as std::borrow::ToOwned>::to_owned](
    value: &T,
) -> (result: T)
    requires
        // INHERITED from T::clone.
        call_requires(<T as Clone>::clone, (value,)),
    ensures
        call_ensures(<T as Clone>::clone, (value,), result),
;

// ----------------------------------------
// MutableOrigin
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when servo_url::MutableOrigin is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExMutableOrigin(MutableOrigin);

// TODO: Verify the accessor in servo-url and define its origin relation.
// TRUSTED(scope): not verified yet; remove when MutableOrigin::immutable is verified
pub assume_specification<'a>[MutableOrigin::immutable](origin: &'a MutableOrigin) -> (result: &'a ImmutableOrigin)
    ensures
        Spec::mutable_origin_immutable_ensures(origin, result),
;

// ----------------------------------------
// Referrer
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when net_traits::request::Referrer is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExReferrer(Referrer);

// ----------------------------------------
// RequestClient
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when net_traits::request::RequestClient is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRequestClient(RequestClient);

// ----------------------------------------
// RequestBuilder
// ----------------------------------------

// Transparent for preload's direct request.url read. Supporting field-type
// declarations and request/preload method contracts live in servo_api::preload.
// TRUSTED(scope): not verified yet; remove when net_traits::request::RequestBuilder is verified
#[verifier::external_type_specification]
pub struct ExRequestBuilder(RequestBuilder);

// TODO: Replace these opaque declarations with specifications from their owners.
// They expose type identities for preload's signature, not DOM, GC, reference
// counting, or fetch behavior. Method specifications are also needed to check its body.

// ----------------------------------------
// WebViewId
// ----------------------------------------

// Defined by servo_base::id.
// TRUSTED(scope): not verified yet; remove when servo_base::id::WebViewId is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExWebViewId(WebViewId);

// ----------------------------------------
// Trusted<T>
// ----------------------------------------

// Defined by script_bindings::refcounted; preserve its actual DomObject bound.
// TRUSTED(scope): not verified yet; remove when script_bindings::refcounted is verified
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExTrusted<T: DomObject>(Trusted<T>);

// ----------------------------------------
// Dom<T>
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when script_bindings::root::Dom is verified
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExDom<T>(Dom<T>);

// ----------------------------------------
// Root<T> / DomRoot<T>
// ----------------------------------------

// DomRoot<T> is the existing alias Root<Dom<T>>, not a separate datatype.
// TRUSTED(scope): not verified yet; remove when script_bindings::root::Root is verified
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExRoot<T: StableTraceObject>(Root<T>);

// ----------------------------------------
// Ref<'a, T>
// ----------------------------------------

// Used by Document's origin and policy-container accessors. Borrow-state and
// dereference behavior are not described by this opaque type declaration.
// TRUSTED(external-lib): Rust core 1.99.1; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExRef<'a, T: ?Sized>(Ref<'a, T>);

// ----------------------------------------
// NoGC
// ----------------------------------------

// TRUSTED(external-lib): mozjs 0.26.9; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExNoGC(NoGC);

// ----------------------------------------
// Traceable
// ----------------------------------------

// Bound-only declaration of the unsafe trait re-exported by js::gc.
// Its unsafe trace method and GC-safety obligations remain unmodeled.
// TRUSTED(external-lib): mozjs 0.26.9; validated by not yet
#[verifier::external_trait_specification]
pub unsafe trait ExTraceable {
    type ExternalTraitSpecificationFor: Traceable;
}

// ----------------------------------------
// DomObject
// ----------------------------------------

// Only the trait's supertrait/lifetime bounds are needed by the current support
// signatures. ReflectorType and reflector() have no specification here.
// TRUSTED(scope): not verified yet; remove when script_bindings::reflector is verified
#[verifier::external_trait_specification]
pub trait ExDomObject: Traceable + 'static {
    type ExternalTraitSpecificationFor: DomObject;
}

// ----------------------------------------
// IDLInterface
// ----------------------------------------

// Bound-only declaration; interface identifiers and derives() remain unmodeled.
// TRUSTED(scope): not verified yet; remove when script_bindings::conversions::IDLInterface is verified
#[verifier::external_trait_specification]
pub trait ExIDLInterface {
    type ExternalTraitSpecificationFor: IDLInterface;
}

// ----------------------------------------
// Castable
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when script_bindings::inheritance::Castable is verified
#[verifier::external_trait_specification]
pub trait ExCastable: IDLInterface + DomObject + Sized {
    type ExternalTraitSpecificationFor: Castable;
}

// ----------------------------------------
// DerivedFrom<T>
// ----------------------------------------

// Mirror both T: Castable and Self: Castable from the original marker trait.
// TRUSTED(scope): not verified yet; remove when script_bindings::conversions::DerivedFrom is verified
#[verifier::external_trait_specification]
pub trait ExDerivedFrom<T: Castable>: Castable {
    type ExternalTraitSpecificationFor: DerivedFrom<T>;
}

// ----------------------------------------
// Node / NodeTraits
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when dom::node is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExNode(Node);

// TODO: Verify the existing blanket implementation in node.rs. This preserves
// its actual bounds; the uninterpreted postcondition awaits an ownership/GC model.
// TRUSTED(scope): not verified yet; remove when NodeTraits::owner_global is verified
pub(crate) assume_specification<T: DerivedFrom<Node> + DomObject>[
    <T as NodeTraits>::owner_global
](node: &T) -> (result: DomRoot<GlobalScope>)
    ensures
        Spec::node_owner_global_ensures(node, &result),
;

// ----------------------------------------
// GlobalScope
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when dom::globalscope is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExGlobalScope(GlobalScope);

// TODO: Verify these methods in globalscope.rs. Returned values are otherwise
// unconstrained; this does not establish referrer or request-client correctness.
// TRUSTED(scope): not verified yet; remove when GlobalScope::request_client is verified
pub(crate) assume_specification[GlobalScope::request_client](
    global: &GlobalScope,
    no_gc: Option<&NoGC>,
) -> (result: RequestClient)
    ensures
        Spec::global_scope_request_client_ensures(global, no_gc, &result),
;

// TRUSTED(scope): not verified yet; remove when GlobalScope::get_referrer is verified
pub(crate) assume_specification[GlobalScope::get_referrer](global: &GlobalScope) -> (result: Referrer)
    ensures
        Spec::global_scope_get_referrer_ensures(global, &result),
;

// ----------------------------------------
// MediaList
// ----------------------------------------

// TODO: Verify the media-query evaluator. The uninterpreted result relation
// still needs a model of matching against the document environment.
// TRUSTED(scope): not verified yet; remove when MediaList::matches_environment is verified
pub(crate) assume_specification[MediaList::matches_environment](
    document: &Document,
    media_query: &str,
) -> (result: bool)
    ensures
        Spec::media_list_matches_environment_ensures(document, media_query@, result),
;

// ----------------------------------------
// HTMLLinkElement
// ----------------------------------------

// Defined by script's DOM modules.
// TRUSTED(scope): not verified yet; remove when dom::html::document_metadata::htmllinkelement is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExHTMLLinkElement(HTMLLinkElement);

// ----------------------------------------
// Window
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when dom::window is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExWindow(Window);

// TODO: Verify this accessor in the DOM module and define the window/webview relation.
// Implementation: components/script/dom/window/window.rs, Window::webview_id.
// TRUSTED(scope): not verified yet; remove when Window::webview_id is verified
pub(crate) assume_specification[Window::webview_id](window: &Window) -> (result: WebViewId)
    ensures
        Spec::window_webview_id_ensures(window, result),
;

// ----------------------------------------
// Document
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when dom::document is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExDocument(Document);

// TODO: Verify this accessor in the DOM module. Preserve its Rust borrow lifetime;
// the document/window relation remains an uninterpreted scaffold.
// Implementation: components/script/dom/document/document.rs, Document::window.
// TRUSTED(scope): not verified yet; remove when Document::window is verified
pub(crate) assume_specification<'a>[Document::window](document: &'a Document) -> (result: &'a Window)
    ensures
        Spec::document_window_ensures(document, result),
;

// TODO: Verify these accessors in document.rs. Preserve their return types and
// borrow lifetimes without asserting values or a dynamic-borrow invariant.
// TRUSTED(scope): not verified yet; remove when Document::origin is verified
pub(crate) assume_specification<'a>[Document::origin](document: &'a Document) -> (result: Ref<'a, MutableOrigin>)
    ensures
        Spec::document_origin_ensures(document, &result),
;

// TRUSTED(scope): not verified yet; remove when Document::policy_container is verified
pub(crate) assume_specification<'a>[Document::policy_container](
    document: &'a Document,
) -> (result: Ref<'a, Arc<PolicyContainer>>)
    ensures
        Spec::document_policy_container_ensures(document, &result),
;

// TRUSTED(scope): not verified yet; remove when Document::base_url is verified
pub(crate) assume_specification[Document::base_url](document: &Document) -> (result: ServoUrl)
    ensures
        Spec::document_base_url_ensures(document, &result),
;

} // verus!
