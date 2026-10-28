/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// assume_specification generates unsafe scaffolding in this verifier-only module.
#![allow(unsafe_code)]

//! Temporary Verus support for Servo types and dependency APIs used by link processing.
//!
//! Destination, ReferrerPolicy, and the owned link-header records are transparent
//! so verified code can use their variants and public fields. Other external
//! types are opaque. The DOM accessor and source-set contracts
//! expose callability and Rust return types/lifetimes only; no document/window
//! mapping, identifier relation, media-query result, source-set contents,
//! request-building, or fetch behavior is specified. Borrowing and GC discipline
//! remain outside these temporary contracts.
//! Opaque preload-signature types from servo_base, script_bindings, and the DOM
//! are also declared here until their owning modules provide Verus support.
//! The DOM trait declarations preserve the bounds used by these signatures;
//! they provide no contracts for casting, reflection, or tracing operations.
//!
//! These declarations assume the behavior of external methods; they do not
//! verify their implementations. A re-export through net_traits does not change
//! which dependency owns the implementation.

use ::net_traits::ReferrerPolicy;
use ::net_traits::policy_container::PolicyContainer;
use ::net_traits::request::{CorsSettings, Destination, Referrer, RequestClient};
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

verus! {

// ----------------------------------------
// Destination
// ----------------------------------------

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
pub assume_specification[Destination::is_script_like](
    destination: Destination,
) -> (result: bool)
    ensures
        result == destination_is_script_like(destination),
;

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

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExCorsSettings(CorsSettings);

// ----------------------------------------
// ReferrerPolicy
// ----------------------------------------

// Transparent so process_link_headers can construct EmptyString.
#[verifier::external_type_specification]
pub struct ExReferrerPolicy(ReferrerPolicy);

// ----------------------------------------
// LinkParamOwned (nom-rfc8288 0.4.0)
// ----------------------------------------

// Public fields are imported from the pinned dependency; header lookup and
// parsing behavior are not specified by these type declarations.
#[verifier::external_type_specification]
pub struct ExLinkParamOwned(LinkParamOwned);

// ----------------------------------------
// LinkDataOwned (nom-rfc8288 0.4.0)
// ----------------------------------------

#[verifier::external_type_specification]
pub struct ExLinkDataOwned(LinkDataOwned);

// ----------------------------------------
// PolicyContainer
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPolicyContainer(PolicyContainer);

// ----------------------------------------
// SourceSet
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExSourceSet(SourceSet);

// TODO: Verify source-set construction in its owning module. This contract
// exposes only the signature, not parsing or density-normalization guarantees.
pub assume_specification[SourceSet::create_source_set](
    default_source: &str,
    srcset: &str,
    sizes: &str,
    document: &Document,
) -> SourceSet;

// ----------------------------------------
// ServoUrl
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExServoUrl(ServoUrl);

// ----------------------------------------
// url::ParseError
// ----------------------------------------

// Required by ServoUrl's existing FromStr implementation.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExUrlParseError(url::ParseError);

// ----------------------------------------
// ImmutableOrigin
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExImmutableOrigin(ImmutableOrigin);

// ----------------------------------------
// ToOwned for Clone types
// ----------------------------------------

// TODO: Move this delegation contract to vstd when available. Rust 1.99.1's
// alloc/src/borrow.rs implements this blanket to_owned by calling self.clone().
// Inherit Clone's contract rather than assuming that arbitrary clones are equal.
pub assume_specification<T: Clone>[<T as std::borrow::ToOwned>::to_owned](
    value: &T,
) -> (result: T)
    requires
        call_requires(<T as Clone>::clone, (value,)),
    ensures
        call_ensures(<T as Clone>::clone, (value,), result),
;

// ----------------------------------------
// MutableOrigin
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExMutableOrigin(MutableOrigin);

// TODO: Verify the accessor in servo-url. No origin relation is assumed here.
pub assume_specification<'a>[MutableOrigin::immutable](origin: &'a MutableOrigin) -> &'a ImmutableOrigin;

// ----------------------------------------
// Referrer
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExReferrer(Referrer);

// ----------------------------------------
// RequestClient
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRequestClient(RequestClient);

// TODO: Replace these opaque declarations with specifications from their owners.
// They expose type identities for preload's signature, not DOM, GC, reference
// counting, or fetch behavior. The preload body remains trusted.

// ----------------------------------------
// WebViewId
// ----------------------------------------

// Defined by servo_base::id.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExWebViewId(WebViewId);

// ----------------------------------------
// Trusted<T>
// ----------------------------------------

// Defined by script_bindings::refcounted; preserve its actual DomObject bound.
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExTrusted<T: DomObject>(Trusted<T>);

// ----------------------------------------
// Dom<T>
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExDom<T>(Dom<T>);

// ----------------------------------------
// Root<T> / DomRoot<T>
// ----------------------------------------

// DomRoot<T> is the existing alias Root<Dom<T>>, not a separate datatype.
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExRoot<T: StableTraceObject>(Root<T>);

// ----------------------------------------
// Ref<'a, T>
// ----------------------------------------

// Used by Document's origin and policy-container accessors. Borrow-state and
// dereference behavior are not described by this opaque type declaration.
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExRef<'a, T: ?Sized>(Ref<'a, T>);

// ----------------------------------------
// NoGC
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExNoGC(NoGC);

// ----------------------------------------
// Traceable
// ----------------------------------------

// Bound-only declaration of the unsafe trait re-exported by js::gc.
// Its unsafe trace method and GC-safety obligations remain unmodeled.
#[verifier::external_trait_specification]
pub unsafe trait ExTraceable {
    type ExternalTraitSpecificationFor: Traceable;
}

// ----------------------------------------
// DomObject
// ----------------------------------------

// Only the trait's supertrait/lifetime bounds are needed by the current support
// signatures. ReflectorType and reflector() have no specification here.
#[verifier::external_trait_specification]
pub trait ExDomObject: Traceable + 'static {
    type ExternalTraitSpecificationFor: DomObject;
}

// ----------------------------------------
// IDLInterface
// ----------------------------------------

// Bound-only declaration; interface identifiers and derives() remain unmodeled.
#[verifier::external_trait_specification]
pub trait ExIDLInterface {
    type ExternalTraitSpecificationFor: IDLInterface;
}

// ----------------------------------------
// Castable
// ----------------------------------------

#[verifier::external_trait_specification]
pub trait ExCastable: IDLInterface + DomObject + Sized {
    type ExternalTraitSpecificationFor: Castable;
}

// ----------------------------------------
// DerivedFrom<T>
// ----------------------------------------

// Mirror both T: Castable and Self: Castable from the original marker trait.
#[verifier::external_trait_specification]
pub trait ExDerivedFrom<T: Castable>: Castable {
    type ExternalTraitSpecificationFor: DerivedFrom<T>;
}

// ----------------------------------------
// Node / NodeTraits
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExNode(Node);

// TODO: Verify the existing blanket implementation in node.rs. This preserves
// its actual bounds and exposes only the rooted return type, not ownership/GC facts.
pub(crate) assume_specification<T: DerivedFrom<Node> + DomObject>[
    <T as NodeTraits>::owner_global
](node: &T) -> DomRoot<GlobalScope>;

// ----------------------------------------
// GlobalScope
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExGlobalScope(GlobalScope);

// TODO: Verify these methods in globalscope.rs. Returned values are otherwise
// unconstrained; this does not establish referrer or request-client correctness.
pub(crate) assume_specification[GlobalScope::request_client](
    global: &GlobalScope,
    no_gc: Option<&NoGC>,
) -> RequestClient;

pub(crate) assume_specification[GlobalScope::get_referrer](global: &GlobalScope) -> Referrer;

// ----------------------------------------
// MediaList
// ----------------------------------------

// TODO: Verify the media-query evaluator. Either result is permitted by this
// callability-only contract; no environment-matching property is assumed.
pub(crate) assume_specification[MediaList::matches_environment](
    document: &Document,
    media_query: &str,
) -> bool;

// ----------------------------------------
// HTMLLinkElement
// ----------------------------------------

// Defined by script's DOM modules.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExHTMLLinkElement(HTMLLinkElement);

// ----------------------------------------
// Window
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExWindow(Window);

// TODO: Verify this accessor in the DOM module. The caller only needs a typed ID,
// so this temporary contract asserts no relation between the Window and its ID.
// Implementation: components/script/dom/window/window.rs, Window::webview_id.
pub(crate) assume_specification[Window::webview_id](window: &Window) -> WebViewId;

// ----------------------------------------
// Document
// ----------------------------------------

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExDocument(Document);

// TODO: Verify this accessor in the DOM module. Preserve its Rust borrow lifetime
// without assuming any additional document/window relationship.
// Implementation: components/script/dom/document/document.rs, Document::window.
pub(crate) assume_specification<'a>[Document::window](document: &'a Document) -> &'a Window;

// TODO: Verify these accessors in document.rs. Preserve their return types and
// borrow lifetimes without asserting values or a dynamic-borrow invariant.
pub(crate) assume_specification<'a>[Document::origin](document: &'a Document) -> Ref<'a, MutableOrigin>;

pub(crate) assume_specification<'a>[Document::policy_container](
    document: &'a Document,
) -> Ref<'a, Arc<PolicyContainer>>;

pub(crate) assume_specification[Document::base_url](document: &Document) -> ServoUrl;

} // verus!
