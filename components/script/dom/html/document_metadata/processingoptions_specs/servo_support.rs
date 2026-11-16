/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// assume_specification generates unsafe scaffolding in this verifier-only module.
#![allow(unsafe_code)]

//! Temporary type declarations needed to inspect LinkProcessingOptions.
//! Replace these declarations as their owning modules gain Verus support.
//! Method contracts assume dependency behavior rather than verifying its implementation.

use js::gc::Traceable;
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

use super::processing::types::{DocumentView, WindowView};

verus! {

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

// These projections expose stable identity metadata. Mutable document/window
// contents need separate invocation snapshots and are not included here.
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
