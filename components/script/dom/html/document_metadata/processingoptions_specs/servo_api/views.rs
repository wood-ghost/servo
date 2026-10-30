/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Runtime/specification views, following the Mime / MimeView convention.
//! Opaque identities and external primitive implementations remain model boundaries.

use std::sync::Arc;
use mime::Mime;
use net_traits::ReferrerPolicy as NativeReferrerPolicy;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{CorsSettings, Destination as NativeDestination, RequestBuilder, RequestClient};
use servo_url::{ImmutableOrigin, ServoUrl};
use vstd::prelude::*;

use crate::dom::html::document_metadata::processingoptions::LinkProcessingOptions;
use crate::dom::srcset::SourceSet;
use super::super::processing::model::*;
use super::super::processing::model_trait::LinkProcessingModel;
use super::super::processing::types::*;
use net_traits::mime_classifier_specs::{mime_api as SpecMime, predicates as SpecMimePredicates};

verus! {

// TRUSTED(scope): not verified yet; remove when ServoUrl's view correspondence is verified
pub(crate) uninterp spec fn native_url_record(url: &ServoUrl) -> ServoUrlView;

/// Context-free ServoUrl projection. Blob-store observations must be supplied
/// explicitly through url_view_with_blob_entry; ServoUrl itself stores no entry.
pub(crate) open spec fn url_view(url: &ServoUrl) -> ServoUrlView {
    url_view_with_blob_entry(url, None)
}

pub(crate) open spec fn url_view_with_blob_entry(url: &ServoUrl, entry: Option<BlobUrlEntryView>) -> ServoUrlView {
    ServoUrlView { blob_url_entry: entry, ..native_url_record(url) }
}
// TRUSTED(scope): not verified yet; remove when OpaqueOrigin's identity correspondence is verified
pub(crate) uninterp spec fn opaque_origin_identity(origin: &servo_url::OpaqueOrigin) -> int;

// TRUSTED(external-lib): url 2.5.8; validated by not yet
pub(crate) uninterp spec fn native_host_view(host: &servo_url::Host) -> HostView;

/// Native representation adapter. ImmutableOrigin stores an effective u16 port;
/// the standard tuple uses null for a scheme's default port. document.domain
/// state is supplied separately through OriginView::with_domain.
pub(crate) open spec fn origin_view(origin: &ImmutableOrigin) -> OriginView {
    match origin {
        ImmutableOrigin::Opaque(value) => OriginView::Opaque { identity: opaque_origin_identity(value) },
        ImmutableOrigin::Tuple(scheme, host, port) => OriginView::Tuple {
            scheme: scheme@, host: native_host_view(host),
            port: if Some(*port) == super::url::default_port(scheme@) { None } else { Some(*port) },
            domain: None,
        },
    }
}
// TRUSTED(scope): not verified yet; remove when PolicyContainer's view correspondence is verified
pub(crate) uninterp spec fn policy_container_view(container: &Arc<PolicyContainer>) -> Arc<PolicyContainerView>;
// TRUSTED(scope): not verified yet; remove when RequestClient's environment correspondence is verified
pub(crate) uninterp spec fn environment_view(client: &RequestClient) -> EnvironmentView;
// TRUSTED(scope): not verified yet; remove when SourceSet's contextual view correspondence is verified
pub(crate) uninterp spec fn source_set_view(source_set: &SourceSet) -> SourceSetView;
// TRUSTED(scope): not verified yet; remove when RequestBuilder's view correspondence is verified
pub(crate) uninterp spec fn request_view(request: &RequestBuilder) -> RequestView;

pub(crate) open spec fn option_request_view(request: &Option<RequestBuilder>) -> Option<RequestView> {
    match request {
        Some(request) => Some(request_view(request)),
        None => None,
    }
}

pub(crate) open spec fn cors_view(cors: Option<CorsSettings>) -> Option<CorsSettingsView> {
    match cors {
        None => None,
        Some(settings) => Some(CorsSettingsView::from_native(settings)),
    }
}

/// Wrap Servo's existing destination value in the local invariant-bearing type.
pub(crate) open spec fn destination_view(destination: NativeDestination) -> DestinationView {
    DestinationView::from_native(destination)
}

/// Reverse conversion used by contracts returning a native Destination.
pub(crate) open spec fn destination_from_view(destination: DestinationView) -> NativeDestination {
    destination.as_native()
}

/// String semantics of the existing external Destination enum, declared by
/// ExDestination in servo_api. Its closed set of variants enforces membership
/// by construction; the abstraction's lemma connects every variant to the standard's set.
/// Destination::None denotes the empty string, not the potential destination "fetch".
pub open spec fn destination_name(destination: NativeDestination) -> Seq<char> {
    match destination {
        NativeDestination::None => ""@,
        NativeDestination::Audio => "audio"@,
        NativeDestination::AudioWorklet => "audioworklet"@,
        NativeDestination::Document => "document"@,
        NativeDestination::Embed => "embed"@,
        NativeDestination::Font => "font"@,
        NativeDestination::Frame => "frame"@,
        NativeDestination::IFrame => "iframe"@,
        NativeDestination::Image => "image"@,
        NativeDestination::Json => "json"@,
        NativeDestination::Manifest => "manifest"@,
        NativeDestination::Object => "object"@,
        NativeDestination::PaintWorklet => "paintworklet"@,
        NativeDestination::Report => "report"@,
        NativeDestination::Script => "script"@,
        NativeDestination::ServiceWorker => "serviceworker"@,
        NativeDestination::SharedWorker => "sharedworker"@,
        NativeDestination::Style => "style"@,
        NativeDestination::Text => "text"@,
        NativeDestination::Track => "track"@,
        NativeDestination::Video => "video"@,
        NativeDestination::WebIdentity => "webidentity"@,
        NativeDestination::Worker => "worker"@,
        NativeDestination::Xslt => "xslt"@,
    }
}

pub(crate) open spec fn referrer_policy_view(policy: NativeReferrerPolicy) -> ReferrerPolicyView {
    ReferrerPolicyView::from_native(policy)
}

/// The runtime record carries no document/callback/priority fields. Its standalone
/// view uses their defaults; a caller-specific document context is supplied below.
/// A high/low fetch-priority update cannot be represented until Servo adds it.
pub(crate) open spec fn view(options: &LinkProcessingOptions) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView {
        href: options.href@,
        initiator: "link"@,
        integrity: options.integrity@,
        type_: options.link_type@,
        cryptographic_nonce_metadata: options.cryptographic_nonce_metadata@,
        destination: destination_view(options.destination),
        crossorigin: cors_view(options.cross_origin),
        referrer_policy: referrer_policy_view(options.referrer_policy),
        source_set: match options.source_set {
            Some(source_set) => Some(source_set_view(&source_set)),
            None => None,
        },
        base_url: url_view(&options.base_url),
        origin: origin_view(&options.origin),
        environment: environment_view(&options.request_client),
        policy_container: policy_container_view(&options.policy_container),
        document: None,
        on_document_ready: None,
        fetch_priority: LinkFetchPriorityView::Auto,
    }
}

pub(crate) open spec fn view_in_document(
    options: &LinkProcessingOptions,
    document: DocumentView,
) -> LinkProcessingOptionsView {
    LinkProcessingOptionsView { document: Some(document), ..view(options) }
}

/// Standard-facing primitive interface. Parsing, support decisions, URL/Fetch
/// operations, and clocks need independent correspondence proofs. In particular,
/// classifier recognition is not assumed to imply UA format support.
pub(crate) struct StandardModel;

impl LinkProcessingModel for StandardModel {
    type Mime = Mime;

    uninterp spec fn get_decode_and_split_link_headers(&self, headers: HeaderListView) -> Seq<Seq<char>>;
    uninterp spec fn parse_link_header(&self, value: Seq<char>) -> Seq<Option<ParsedLinkHeader>>;
    uninterp spec fn parse_mime_type(&self, value: Seq<char>) -> Option<Mime>;
    uninterp spec fn mime_supported(&self, mime: Mime) -> bool;
    open spec fn is_javascript_mime(&self, mime: Mime) -> bool { SpecMimePredicates::is_javascript(&mime) }
    open spec fn is_image_mime(&self, mime: Mime) -> bool { SpecMimePredicates::is_image(&mime) }
    // Keep the standard's font-category contract separate from current classifier
    // compatibility extensions until their agreement is established.
    uninterp spec fn is_font_mime(&self, mime: Mime) -> bool;
    open spec fn mime_essence(&self, mime: Mime) -> Seq<char> { SpecMime::essence_str(&mime) }
    uninterp spec fn encoding_parse_url(&self, href: Seq<char>, base_url: ServoUrlView) -> Option<ServoUrlView>;
    uninterp spec fn new_request(&self) -> RequestView;
    uninterp spec fn select_image_source(&self, source_set: SourceSetView) -> Seq<char>;
    open spec fn url_is_http_or_https(&self, url: ServoUrlView) -> bool {
        url.scheme == "http"@ || url.scheme == "https"@
    }
    uninterp spec fn url_origin(&self, url: ServoUrlView) -> OriginView;
    open spec fn same_origin(&self, left: OriginView, right: OriginView) -> bool {
        super::super::processing::algorithms::origin::same_origin(left, right)
    }
    uninterp spec fn network_partition_key(&self, environment: EnvironmentView) -> int;
    uninterp spec fn network_error(&self) -> ResponseView;
    uninterp spec fn unsafe_shared_current_time(&self) -> int;
    uninterp spec fn document_relevant_global(&self, document: DocumentView) -> int;
}

} // verus!
