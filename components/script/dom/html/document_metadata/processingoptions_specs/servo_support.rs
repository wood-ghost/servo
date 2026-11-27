/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// assume_specification generates unsafe scaffolding in this verifier-only module.
#![allow(unsafe_code)]

//! Temporary type declarations needed to inspect LinkProcessingOptions.
//! Replace these declarations as their owning modules gain Verus support.
//! Method contracts assume dependency behavior rather than verifying its implementation.

use std::cell::Ref;
use std::sync::Arc;

use js::gc::Traceable;
use js::context::NoGC;
use nom_rfc8288::complete::LinkParamOwned;
use net_traits::ReferrerPolicy;
use net_traits::blob_url_store::UrlWithBlobClaim;
use net_traits::verus_specs::external::mime::ascii_lowercase;
use net_traits::policy_container::{PolicyContainer, RequestPolicyContainer};
use net_traits::request::{
    CacheMode, CorsSettings, Initiator, InternalRequest, Origin, ParserMetadata, PreloadId,
    PreloadKey, RedirectMode, Referrer, RequestBody, RequestBuilder, RequestClient, RequestId,
    RequestMode, ResponseTainting, ServiceWorkersMode,
};
use script_bindings::conversions::{DerivedFrom, IDLInterface};
use script_bindings::inheritance::Castable;
use script_bindings::refcounted::Trusted;
use script_bindings::reflector::DomObject;
use script_bindings::root::{Dom, DomRoot, Root, StableTraceObject};
use script_bindings::str::USVString;
use servo_base::id::{NamespaceIndex, PipelineIndex, WebViewId};
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use vstd::prelude::*;

use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::medialist::MediaList;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::srcset::SourceSet;
use crate::dom::types::HTMLLinkElement;
use crate::dom::window::Window;
use crate::fetch::network_listener::FetchResponseListener;

use super::super::processingoptions::LinkHeader;
use super::processing::types::{DocumentView, SourceSetView, WindowView};

verus! {

// TRUSTED(external-lib): nom-rfc8288 0.4.0; expose LinkDataOwned's actual public fields.
#[verifier::external_type_specification]
pub struct ExLinkHeader(LinkHeader);

// TRUSTED(external-lib): nom-rfc8288 0.4.0; expose LinkParamOwned's key and optional value.
#[verifier::external_type_specification]
pub struct ExLinkParamOwned(LinkParamOwned);

// TRUSTED(scope): not verified yet; remove when net_traits::request::CorsSettings is verified
#[verifier::external_type_specification]
pub struct ExCorsSettings(CorsSettings);

// TRUSTED(scope): not verified yet; remove when net_traits::ReferrerPolicy is verified
#[verifier::external_type_specification]
pub struct ExReferrerPolicy(ReferrerPolicy);

// https://w3c.github.io/webappsec-referrer-policy/#referrer-policy
// A referrer policy is the empty string, "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", or "unsafe-url".
pub(crate) open spec fn referrer_policy_for_token(value: Seq<char>) -> Option<ReferrerPolicy> {
    let keyword = ascii_lowercase(value);
    if keyword == ""@ { Some(ReferrerPolicy::EmptyString) }
    else if keyword == "no-referrer"@ { Some(ReferrerPolicy::NoReferrer) }
    else if keyword == "no-referrer-when-downgrade"@ { Some(ReferrerPolicy::NoReferrerWhenDowngrade) }
    else if keyword == "same-origin"@ { Some(ReferrerPolicy::SameOrigin) }
    else if keyword == "origin"@ { Some(ReferrerPolicy::Origin) }
    else if keyword == "strict-origin"@ { Some(ReferrerPolicy::StrictOrigin) }
    else if keyword == "origin-when-cross-origin"@ { Some(ReferrerPolicy::OriginWhenCrossOrigin) }
    else if keyword == "strict-origin-when-cross-origin"@ { Some(ReferrerPolicy::StrictOriginWhenCrossOrigin) }
    else if keyword == "unsafe-url"@ { Some(ReferrerPolicy::UnsafeUrl) }
    else { None }
}

// A public trait-method contract cannot directly name the crate-private token parser.
pub open(crate) spec fn referrer_policy_from_str(value: Seq<char>) -> ReferrerPolicy {
    referrer_policy_for_token(value).unwrap_or(ReferrerPolicy::EmptyString)
}

// TRUSTED(scope): net_traits ReferrerPolicy::from (components/shared/net/lib.rs):
// ASCII case-insensitive match against the referrer-policy keywords; any other value
// gives EmptyString.
pub assume_specification<'a>[<ReferrerPolicy as From<&'a str>>::from](value: &str) -> (r: ReferrerPolicy)
    ensures r == referrer_policy_from_str(value@),
;

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
pub assume_specification[SourceSet::create_source_set](
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

// TRUSTED(scope): not verified yet; remove when servo_url::ImmutableOrigin is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExImmutableOrigin(ImmutableOrigin);

// TRUSTED(scope): expose the native types returned or borrowed while building link options.
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExMutableOrigin(MutableOrigin);

// TRUSTED(external-lib): mozjs token in the request_client signature.
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExNoGC(NoGC);

// TRUSTED(scope): TODO: relate media evaluation to the current document snapshot.
pub(crate) assume_specification[MediaList::matches_environment](document: &Document, media_query: &str) -> bool;

// TRUSTED(scope): TODO: relate each document/global accessor to the native snapshot.
pub(crate) assume_specification<'a>[Document::policy_container](document: &'a Document) -> Ref<'a, Arc<PolicyContainer>>;
pub(crate) assume_specification<'a>[Document::origin](document: &'a Document) -> Ref<'a, MutableOrigin>;
pub assume_specification<'a>[MutableOrigin::immutable](origin: &'a MutableOrigin) -> &'a ImmutableOrigin;
pub(crate) assume_specification[Document::base_url](document: &Document) -> ServoUrl;
pub(crate) assume_specification[GlobalScope::request_client](global: &GlobalScope, no_gc: Option<&NoGC>) -> RequestClient;
pub(crate) assume_specification[GlobalScope::get_referrer](global: &GlobalScope) -> Referrer;

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

// Preload callees. The signatures below give callers no guarantees about the
// selected URL, generated identifiers, preload map, or network effects.

// TRUSTED(scope): DOM and GC identities used by NodeTraits::owner_global.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExGlobalScope(GlobalScope);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExNode(Node);

#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExDom<T>(Dom<T>);

#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExRoot<T: StableTraceObject>(Root<T>);

// TRUSTED(scope): bound-only specifications for the native node blanket implementation.
#[verifier::external_trait_specification]
pub trait ExIDLInterface {
    type ExternalTraitSpecificationFor: IDLInterface;
}

#[verifier::external_trait_specification]
pub trait ExCastable: IDLInterface + DomObject + Sized {
    type ExternalTraitSpecificationFor: Castable;
}

#[verifier::external_trait_specification]
pub trait ExDerivedFrom<T: Castable>: Castable {
    type ExternalTraitSpecificationFor: DerivedFrom<T>;
}

// TRUSTED(scope): NodeTraits::owner_global.
// TODO: ensures the result roots this node's owning global.
pub(crate) assume_specification<T: DerivedFrom<Node> + DomObject>[<T as NodeTraits>::owner_global](
    node: &T,
) -> DomRoot<GlobalScope>;

// TRUSTED(scope): script_bindings::str::USVString.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExUSVString(USVString);

// TRUSTED(scope): SourceSet::select_image_source_from_source_set.
// TODO: ensures the selected URL and density for this document snapshot.
pub assume_specification[SourceSet::select_image_source_from_source_set](
    source_set: &SourceSet, document: &Document,
) -> Option<(USVString, f64)>;

// TRUSTED(scope): USVString's derived Default implementation.
// TODO: ensures the default string is empty.
pub assume_specification[<USVString as Default>::default]() -> USVString;

// TRUSTED(scope): USVString's conversion into String.
// TODO: ensures the returned String has the input's contents.
pub assume_specification[<String as From<USVString>>::from](value: USVString) -> String;

// TRUSTED(scope): opaque preload identifiers and keys.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPreloadId(PreloadId);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPreloadKey(PreloadKey);

// TRUSTED(scope): PreloadId::default.
// TODO: ensures the returned preload identifier is fresh.
pub assume_specification[<PreloadId as Default>::default]() -> PreloadId;

// TRUSTED(scope): PreloadId::clone.
// TODO: ensures the returned identifier denotes the same preload.
pub assume_specification[<PreloadId as Clone>::clone](id: &PreloadId) -> PreloadId;

// TRUSTED(scope): RequestBuilder::preload_id.
// TODO: ensures only the builder's preload ID is replaced.
pub assume_specification[RequestBuilder::preload_id](request: RequestBuilder, preload_id: PreloadId) -> RequestBuilder;

// TRUSTED(scope): PreloadKey::new.
// TODO: ensures the key projects URL, destination, mode and credentials mode.
pub assume_specification[PreloadKey::new](request: &RequestBuilder) -> PreloadKey;

// TRUSTED(scope): UrlWithBlobClaim::url.
// TODO: ensures the result is the wrapped URL.
pub assume_specification[UrlWithBlobClaim::url](url: &UrlWithBlobClaim) -> ServoUrl;

// TRUSTED(scope): RequestBuilder must be transparent for preload's direct request.url read.
#[verifier::external_type_specification]
pub struct ExRequestBuilder(RequestBuilder);

// TRUSTED(scope): type identities of RequestBuilder's project-owned fields.
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExRequestId(RequestId);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExUrlWithBlobClaim(UrlWithBlobClaim);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExRequestBody(RequestBody);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExServiceWorkersMode(ServiceWorkersMode);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExRequestMode(RequestMode);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExCacheMode(CacheMode);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExRequestOrigin(Origin);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExRequestPolicyContainer(RequestPolicyContainer);
#[verifier::external_type_specification] #[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExNamespaceIndex<T>(NamespaceIndex<T>);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExPipelineIndex(PipelineIndex);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExRedirectMode(RedirectMode);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExParserMetadata(ParserMetadata);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExInitiator(Initiator);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExResponseTainting(ResponseTainting);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExInternalRequest(InternalRequest);

// TRUSTED(scope): only the listener bound is exposed; callbacks are not specified.
#[verifier::external_trait_specification]
pub(crate) trait ExFetchResponseListener: Send + 'static {
    type ExternalTraitSpecificationFor: FetchResponseListener;
}

// TRUSTED(scope): Document::insert_preloaded_resource.
// TODO: ensures the document maps the supplied key to the supplied preload ID.
pub(crate) assume_specification[Document::insert_preloaded_resource](
    document: &Document, key: PreloadKey, preload_id: PreloadId,
);

// TRUSTED(scope): Document::fetch_background.
// TODO: ensures dispatch of the request with the supplied listener.
pub(crate) assume_specification<Listener: FetchResponseListener>[Document::fetch_background::<Listener>](
    document: &Document, request_builder: RequestBuilder, listener: Listener,
);

/// Verification client for composing the two trusted accessor contracts.
fn document_window_webview_id_contract(document: &Document) -> (result: WebViewId)
    ensures
        result == document@.window.webview_id,
{
    document.window().webview_id()
}

} // verus!
