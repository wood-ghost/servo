/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! External type and API support for checking the native preload body.
//!
//! Inspected 2026-10-30. These contracts trust the native implementations; they
//! do not prove image selection, UUID freshness, blob-claim lifetime, GC, or Fetch
//! effects. Signature-only boundaries deliberately leave those behaviors open.

#![allow(unsafe_code)]

use bytes::BytesMut;
use http::{HeaderMap, HeaderValue, Method};
use net_traits::blob_url_store::UrlWithBlobClaim;
use net_traits::policy_container::RequestPolicyContainer;
use net_traits::request::{
    CacheMode, CredentialsMode, Initiator, InternalRequest, Origin, ParserMetadata,
    PreloadId, PreloadKey, RedirectMode, RequestBody, RequestBuilder, RequestId,
    RequestMode, ResponseTainting, ServiceWorkersMode,
};
use script_bindings::str::USVString;
use servo_base::id::{NamespaceIndex, PipelineIndex};
use servo_url::ServoUrl;
use vstd::prelude::*;

use crate::dom::document::Document;
use crate::dom::html::document_metadata::processingoptions::{LinkFetchContext, LinkFetchContextType};
use crate::dom::srcset::SourceSet;
use crate::fetch::network_listener::FetchResponseListener;
use super::views::{StandardModel, source_set_view_in_context};
use super::super::processing::context_specs::link_headers_document_state;
use super::super::processing::model_trait::LinkProcessingModel;
use super::super::processing::types::ImageSourceDescriptorView;

verus! {

// ----------------------------------------
// USVString and image-source selection
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when script_bindings::str::USVString is verified
#[verifier::external_type_specification]
pub struct ExUSVString(USVString);

// USVString derives Default for its sole String field.
// TRUSTED(scope): not verified yet; remove when USVString::default is verified
pub assume_specification[<USVString as Default>::default]() -> (result: USVString)
    ensures result.0@ == Seq::<char>::empty(),
;

// script_bindings/str.rs: From<USVString> for String returns value.0.
// TRUSTED(scope): not verified yet; remove when String::from(USVString) is verified
pub assume_specification[<String as From<USVString>>::from](value: USVString) -> (result: String)
    ensures result@ == value.0@,
;

/// Native selection unwraps densities and removes its chosen vector index.
/// The context relation is still trusted; this predicate is not a proof that
/// arbitrary srcset input produces a nonempty normalized set.
pub closed spec fn source_selection_precondition(source_set: &SourceSet, document: &Document) -> bool {
    let view = source_set_view_in_context(source_set, link_headers_document_state(document).document);
    &&& view.well_formed()
    &&& view.image_sources.entries.len() > 0
    &&& (forall|i: int| 0 <= i < view.image_sources.entries.len() ==>
        (#[trigger] view.image_sources.entries[i]).descriptor matches Some(ImageSourceDescriptorView::PixelDensity { .. }))
}

/// Chosen href component at the same document snapshot used by native selection.
pub closed spec fn selected_source_href(source_set: &SourceSet, document: &Document) -> Seq<char> {
    StandardModel.select_image_source(source_set_view_in_context(source_set, link_headers_document_state(document).document))
}

// This contract assumes the contextual UA choice agrees with the native choice;
// it does not verify CSS evaluation, float conversion or URL/string correspondence.
// TRUSTED(scope): not verified yet; remove when SourceSet::select_image_source_from_source_set is verified
pub assume_specification[SourceSet::select_image_source_from_source_set](
    source_set: &SourceSet,
    document: &Document,
) -> (result: Option<(USVString, f64)>)
    requires
        // IMPLICIT: native selection unwraps densities/removes a vector element; witnesses: empty source set or missing density.
        source_selection_precondition(source_set, document),
    ensures
        result is Some,
        result->Some_0.0.0@ == selected_source_href(source_set, document),
;

// ----------------------------------------
// Preload identity and key
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when net_traits::request::PreloadId is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPreloadId(PreloadId);

// Default creates a UUID. No uniqueness or freshness property is assumed here.
// TRUSTED(scope): not verified yet; remove when PreloadId::default is verified
pub assume_specification[<PreloadId as Default>::default]() -> PreloadId;

// PreloadId's derived Clone copies its UUID value.
// TRUSTED(scope): not verified yet; remove when PreloadId::clone is verified
pub assume_specification[<PreloadId as Clone>::clone](value: &PreloadId) -> (result: PreloadId)
    ensures result == *value,
;

// TRUSTED(scope): not verified yet; remove when net_traits::request::PreloadKey is verified
#[verifier::external_type_specification]
pub struct ExPreloadKey(PreloadKey);

// request.rs: PreloadKey::new reads exactly these four request fields.
// TRUSTED(scope): not verified yet; remove when PreloadKey::new is verified
pub assume_specification[PreloadKey::new](request: &RequestBuilder) -> (result: PreloadKey)
    ensures
        result.url == url_with_blob_claim_url(&request.url),
        result.destination == request.destination,
        result.mode == request.mode,
        result.credentials_mode == request.credentials_mode,
;

// ----------------------------------------
// URL and request builder
// ----------------------------------------

// TRUSTED(scope): not verified yet; remove when net_traits::blob_url_store is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExUrlWithBlobClaim(UrlWithBlobClaim);

/// Observation of the private URL field; blob-token ownership remains unmodeled.
// TRUSTED(scope): not verified yet; remove when UrlWithBlobClaim's view correspondence is verified
pub uninterp spec fn url_with_blob_claim_url(value: &UrlWithBlobClaim) -> ServoUrl;

// blob_url_store.rs: url() clones the stored ServoUrl.
// TRUSTED(scope): not verified yet; remove when UrlWithBlobClaim::url is verified
pub assume_specification[UrlWithBlobClaim::url](value: &UrlWithBlobClaim) -> (result: ServoUrl)
    ensures result == url_with_blob_claim_url(value),
;

// request.rs: this consuming setter updates only preload_id.
// TRUSTED(scope): not verified yet; remove when RequestBuilder::preload_id is verified
pub assume_specification[RequestBuilder::preload_id](
    request: RequestBuilder,
    preload_id: PreloadId,
) -> (result: RequestBuilder)
    ensures result == (RequestBuilder { preload_id: Some(preload_id), ..request }),
;

// RequestBuilder is transparent so its URL field can be read. Its other field
// types need identities, but their internals and operations are not exposed.

// TRUSTED(scope): not verified yet; remove when net_traits::request::RequestId is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRequestId(RequestId);

// TRUSTED(external-lib): http 1.5.0; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExHttpMethod(Method);

// TRUSTED(external-lib): http 1.5.0; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExHeaderValue(HeaderValue);

// TRUSTED(external-lib): http 1.5.0; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(V)]
pub struct ExHeaderMap<V>(HeaderMap<V>);

// TRUSTED(scope): not verified yet; remove when net_traits::request::RequestBody is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRequestBody(RequestBody);

// TRUSTED(scope): not verified yet; remove when net_traits::request::ServiceWorkersMode is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExServiceWorkersMode(ServiceWorkersMode);

// TRUSTED(scope): not verified yet; remove when net_traits::request::RequestMode is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRequestMode(RequestMode);

// TRUSTED(scope): not verified yet; remove when net_traits::request::CacheMode is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExCacheMode(CacheMode);

// TRUSTED(scope): not verified yet; remove when net_traits::request::CredentialsMode is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExCredentialsMode(CredentialsMode);

// TRUSTED(scope): not verified yet; remove when net_traits::request::Origin is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRequestOrigin(Origin);

// TRUSTED(scope): not verified yet; remove when net_traits::policy_container::RequestPolicyContainer is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRequestPolicyContainer(RequestPolicyContainer);

// TRUSTED(scope): not verified yet; remove when servo_base::id::NamespaceIndex is verified
#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExNamespaceIndex<T>(NamespaceIndex<T>);

// PipelineId is the existing alias NamespaceIndex<PipelineIndex>.
// TRUSTED(scope): not verified yet; remove when servo_base::id::PipelineIndex is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPipelineIndex(PipelineIndex);

// TRUSTED(scope): not verified yet; remove when net_traits::request::RedirectMode is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRedirectMode(RedirectMode);

// TRUSTED(scope): not verified yet; remove when net_traits::request::ParserMetadata is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExParserMetadata(ParserMetadata);

// TRUSTED(scope): not verified yet; remove when net_traits::request::Initiator is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExInitiator(Initiator);

// TRUSTED(scope): not verified yet; remove when net_traits::request::ResponseTainting is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExResponseTainting(ResponseTainting);

// TRUSTED(scope): not verified yet; remove when net_traits::request::InternalRequest is verified
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExInternalRequest(InternalRequest);

// ----------------------------------------
// Fetch context construction and dispatch
// ----------------------------------------

// TRUSTED(external-lib): bytes 1.12.1; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExBytesMut(BytesMut);

// TRUSTED(external-lib): bytes 1.12.1; validated by not yet
pub uninterp spec fn bytes_mut_contents(value: &BytesMut) -> Seq<u8>;

// TRUSTED(external-lib): bytes 1.12.1; validated by not yet
pub assume_specification[BytesMut::new]() -> (result: BytesMut)
    ensures bytes_mut_contents(&result) == Seq::<u8>::empty(),
;

// Transparent declarations permit the existing context literal and enum variant.
// The listener implementation itself remains an external boundary.
// TRUSTED(scope): not verified yet; remove when LinkFetchContextType is verified
#[verifier::external_type_specification]
pub(crate) struct ExLinkFetchContextType(LinkFetchContextType);

// TRUSTED(scope): not verified yet; remove when LinkFetchContext is verified
#[verifier::external_type_specification]
pub(crate) struct ExLinkFetchContext(LinkFetchContext);

// Only the native supertrait bounds are needed by fetch_background's signature.
// No listener callback or networking behavior is specified by this declaration.
// TRUSTED(scope): not verified yet; remove when FetchResponseListener is verified
#[verifier::external_trait_specification]
pub(crate) trait ExFetchResponseListener: Send + 'static {
    type ExternalTraitSpecificationFor: FetchResponseListener;
}

// Signature-only effect boundaries: map mutation and asynchronous Fetch effects
// still need a state/trace model. These do not assume preload_execution_ensures.
// TRUSTED(scope): not verified yet; remove when Document::insert_preloaded_resource is verified
pub(crate) assume_specification[Document::insert_preloaded_resource](
    document: &Document,
    key: PreloadKey,
    preload_id: PreloadId,
);

// TRUSTED(scope): not verified yet; remove when Document::fetch_background is verified
pub(crate) assume_specification<Listener: FetchResponseListener>[
    Document::fetch_background::<Listener>
](document: &Document, request_builder: RequestBuilder, listener: Listener);

} // verus!
