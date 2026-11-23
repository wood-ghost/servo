/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// assume_specification generates unsafe scaffolding in this verifier-only module.
#![allow(unsafe_code)]

//! Temporary type declarations needed to inspect LinkProcessingOptions.
//! Replace these declarations as their owning modules gain Verus support.
//! Method contracts assume dependency behavior rather than verifying its implementation.

use js::gc::Traceable;
use nom_rfc8288::complete::LinkParamOwned;
use net_traits::ReferrerPolicy;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{CorsSettings, Referrer, RequestClient};
use script_bindings::refcounted::Trusted;
use script_bindings::reflector::DomObject;
use servo_base::id::WebViewId;
use servo_url::{ImmutableOrigin, ServoUrl};
use vstd::prelude::*;

use crate::dom::document::Document;
use crate::dom::srcset::SourceSet;
use crate::dom::types::HTMLLinkElement;
use crate::dom::window::Window;

use super::super::processingoptions::LinkHeader;
use super::processing::types::{DocumentView, SourceSetView, WindowView};

verus! {

// TRUSTED(external-lib): nom-rfc8288 0.4.0; expose LinkDataOwned's actual public fields.
#[verifier::external_type_specification]
pub struct ExLinkHeader(LinkHeader);

// TRUSTED(external-lib): nom-rfc8288 0.4.0; expose LinkParamOwned's key and optional value.
#[verifier::external_type_specification]
pub struct ExLinkParamOwned(LinkParamOwned);

// TRUSTED(std): Rust 1.99.1 alloc::string::impl_eq! compares the two string slices.
// vstd specifies String == String, but not this String == &str implementation.
// The impl lifetime and the method's elided input lifetimes must match std's signature.
pub assume_specification<'a>[<String as PartialEq<&'a str>>::eq](
    value: &String,
    key: &&str,
) -> (r: bool)
    ensures r == (value@ == (*key)@),
;

// TRUSTED(std): Rust 1.99.1 Option::as_deref is self.as_ref().map(Deref::deref).
// vstd has no as_deref contract. Preserve presence and the wrapped Deref contract;
// vstd's existing String::deref specification then supplies the string contents.
pub assume_specification<T: std::ops::Deref>[Option::<T>::as_deref](
    value: &Option<T>,
) -> (r: Option<&T::Target>)
    ensures
        r is Some <==> value is Some,
        r is Some ==> call_ensures(T::deref, (&value->0,), r->0),
;

// TRUSTED(scope): not verified yet; remove when net_traits::request::CorsSettings is verified
#[verifier::external_type_specification]
pub struct ExCorsSettings(CorsSettings);

// TRUSTED(scope): not verified yet; remove when net_traits::ReferrerPolicy is verified
#[verifier::external_type_specification]
pub struct ExReferrerPolicy(ReferrerPolicy);

// TRUSTED(scope): not verified yet; remove when net_traits::policy_container is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPolicyContainer(PolicyContainer);

// TRUSTED(scope): not verified yet; remove when dom::srcset is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExSourceSet(SourceSet);

// TODO: Define how parsing, default-source insertion, and density normalization
// determine the returned source set.
pub uninterp spec fn create_source_set(
    default_source: Seq<char>,
    srcset: Seq<char>,
    sizes: Seq<char>,
    document: DocumentView,
) -> SourceSet;

// TODO: Relate the native candidates and document-dependent source size to their WHATWG view.
pub uninterp spec fn source_set_view(source_set: &SourceSet, document: DocumentView) -> SourceSetView;

// TRUSTED(scope): remove when the native SourceSet::create_source_set body is verified.
pub(crate) assume_specification[SourceSet::create_source_set](
    default_source: &str,
    srcset: &str,
    sizes: &str,
    document: &Document,
) -> (source_set: SourceSet)
    ensures source_set == create_source_set(default_source@, srcset@, sizes@, document@),
;

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

// TRUSTED(scope): not verified yet; remove when dom::window is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExWindow(Window);

// DocumentView includes the fields needed for a link-processing snapshot.
// Their correspondence to native document contents remains unverified.
// WindowView exposes stable window identity metadata.
impl View for Document {
    type V = DocumentView;

    // TRUSTED(scope): not verified yet; remove when Document's view correspondence is verified
    uninterp spec fn view(&self) -> DocumentView;
}

impl View for Window {
    type V = WindowView;

    // TRUSTED(scope): not verified yet; remove when Window's view correspondence is verified
    uninterp spec fn view(&self) -> WindowView;
}

// Document::window returns &self.window; preserve its native borrow lifetime.
// TRUSTED(scope): not verified yet; remove when Document::window is verified
pub(crate) assume_specification<'a>[Document::window](
    document: &'a Document,
) -> (result: &'a Window)
    ensures
        result@ == document@.window,
;

// Window::webview_id reads the immutable id in its shared WebViewState.
// TRUSTED(scope): not verified yet; remove when Window::webview_id is verified
pub(crate) assume_specification[Window::webview_id](
    window: &Window,
) -> (result: WebViewId)
    ensures
        result == window@.webview_id,
;

/// Verification client for composing the two trusted accessor contracts.
fn document_window_webview_id_contract(document: &Document) -> (result: WebViewId)
    ensures
        result == document@.window.webview_id,
{
    document.window().webview_id()
}

} // verus!
