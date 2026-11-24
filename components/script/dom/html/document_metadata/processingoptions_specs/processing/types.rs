/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Link-processing views, native projections, and request and preload records.
//! The standard excerpts retain the backup's inspection dates.

#![verifier::auto_reveal_strlit]

use std::sync::Arc;
use net_traits::ReferrerPolicy;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{CredentialsMode, Destination, RequestClient};
use nom_rfc8288::complete::LinkParamOwned;
use servo_base::id::WebViewId;
use servo_url::{ImmutableOrigin, ServoUrl};
use vstd::prelude::*;

use crate::dom::srcset::SourceSet;
use super::super::super::processingoptions::{LinkHeader, LinkProcessingOptions};
use super::super::csp_api::CspListView;
use super::model::LinkProcessingOptionsView;

verus! {

// =========================
//      LinkHeader
// =========================

/// Preserve parser order and duplicates: lookup stops at the first matching key.
// Required by the public, open <LinkHeader as LinkHeaderViewSpec>::view, which constructs this record.
pub struct LinkHeaderView {
    pub url: Seq<char>,
    pub params: Seq<(Seq<char>, Option<Seq<char>>)>,
}

// A local extension trait supplies `header@`: Rust's orphan rule prevents
// implementing the external vstd::View trait for the external LinkHeader type.
pub trait LinkHeaderViewSpec {
    spec fn view(&self) -> LinkHeaderView;
}

impl LinkHeaderViewSpec for LinkHeader {
    open spec fn view(&self) -> LinkHeaderView {
        LinkHeaderView {
            url: self.url@,
            params: self.params@.map_values(|param: LinkParamOwned| (
                param.key@,
                match param.val {
                    Some(value) => Some(value@),
                    None => None,
                },
            )),
        }
    }
}

pub open spec fn link_header_has_key(h: LinkHeaderView, key: Seq<char>) -> bool {
    exists|i: int| 0 <= i < h.params.len() && (#[trigger] h.params[i]).0 == key
}

pub open spec fn link_header_value_for_key(h: LinkHeaderView, key: Seq<char>) -> Option<Seq<char>> {
    let keys = h.params.map_values(|param: (Seq<char>, Option<Seq<char>>)| param.0);
    match keys.index_of_first(key) {
        // A missing value at the first matching key does not fall through to later keys.
        Some(index) => h.params[index].1,
        None => None,
    }
}

pub broadcast proof fn link_header_lookup_properties(h: LinkHeaderView, key: Seq<char>)
    ensures
        #![trigger link_header_has_key(h, key)]
        #![trigger link_header_value_for_key(h, key)]
        {
            let keys = h.params.map_values(|param: (Seq<char>, Option<Seq<char>>)| param.0);
            match keys.index_of_first(key) {
                Some(index) => {
                    &&& link_header_has_key(h, key)
                    &&& 0 <= index < h.params.len()
                    &&& h.params[index].0 == key
                    &&& forall|i: int| 0 <= i < index ==> (#[trigger] h.params[i]).0 != key
                    &&& link_header_value_for_key(h, key) == h.params[index].1
                },
                None => {
                    &&& !link_header_has_key(h, key)
                    &&& link_header_value_for_key(h, key) is None
                },
            }
        },
{
    let keys = h.params.map_values(|param: (Seq<char>, Option<Seq<char>>)| param.0);
    keys.index_of_first_ensures(key);
    if let Some(index) = keys.index_of_first(key) {
        assert forall|i: int| 0 <= i < index implies (#[trigger] h.params[i]).0 != key by {
            assert(keys[i] != key);
        }
    } else {
        assert forall|i: int| 0 <= i < h.params.len() implies (#[trigger] h.params[i]).0 != key by {
            assert(keys[i] != key);
        }
    }
}

pub broadcast proof fn link_header_param_view(header: LinkHeader, i: int)
    requires 0 <= i < header.params@.len(),
    ensures
        #![trigger header@.params, header.params@[i]]
        header@.params[i].0 == header.params@[i].key@,
        header@.params[i].1 == match header.params@[i].val {
            Some(value) => Some(value@),
            None => None,
        },
{}

pub broadcast proof fn link_header_first_value(h: LinkHeaderView, key: Seq<char>, index: int)
    requires
        0 <= index < h.params.len(),
        h.params[index].0 == key,
        forall|i: int| 0 <= i < index ==> (#[trigger] h.params[i]).0 != key,
    ensures
        #![trigger link_header_value_for_key(h, key), h.params[index]]
        link_header_value_for_key(h, key) == h.params[index].1,
{
    link_header_lookup_properties(h, key);
    let keys = h.params.map_values(|param: (Seq<char>, Option<Seq<char>>)| param.0);
    assert(link_header_has_key(h, key));
    let first = keys.index_of_first(key)->0;
    if first < index {
        assert(h.params[first].0 != key);
    } else if index < first {
        assert(h.params[index].0 != key);
    }
}

pub proof fn link_header_has_key_properties(header: LinkHeader, key: Seq<char>)
    ensures
        link_header_has_key(header@, key) == (exists|i: int| 0 <= i < header.params@.len()
            && (#[trigger] header.params@[i]).key@ == key),
{
    if link_header_has_key(header@, key) {
        let i = choose|i: int| 0 <= i < header@.params.len()
            && (#[trigger] header@.params[i]).0 == key;
        assert(header.params@[i].key@ == key);
    }
    if exists|i: int| 0 <= i < header.params@.len() && (#[trigger] header.params@[i]).key@ == key {
        let i = choose|i: int| 0 <= i < header.params@.len()
            && (#[trigger] header.params@[i]).key@ == key;
        assert(header@.params[i].0 == key);
    }
}

pub broadcast group link_header_lemmas {
    link_header_lookup_properties,
    link_header_param_view,
    link_header_first_value,
}

// =========================
//      LinkFetchPriority
// =========================

/// HTML's fetch-priority states. Servo's link options currently lack this field.
/// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#fetch-priority-attribute
/// A fetch priority attribute is an enumerated attribute with the following keywords and states:
// Public because process_link_headers_automaton::State::create_new_option constructs Auto.
pub enum LinkFetchPriority {
    Auto,
    High,
    Low,
}

// =========================
//      SourceSet
// =========================

/// Placeholder identity for the optional image element in source-set creation.
pub struct ElementView { pub identity: int }

// https://html.spec.whatwg.org/multipage/images.html#source-set
// Snapshot/inspection 2026-10-30.
// A source set is an ordered set of zero or more image sources and a source size.
pub(crate) struct SourceSetView {
    pub image_sources: OrderedImageSourcesView,
    pub source_size: SourceSizeView,
}

// https://infra.spec.whatwg.org/#ordered-set
// An ordered set is a list that does not contain the same item twice.
// Source identities implement Infra object identity: equal URLs/densities do
// not collapse independently created source objects. Selection separately
// removes duplicate pixel densities.
pub(crate) struct OrderedImageSourcesView {
    pub(crate) entries: Seq<ImageSourceView>,
}

// https://html.spec.whatwg.org/multipage/images.html#image-source
// An image source is a URL, and optionally either a pixel density descriptor, or a width descriptor.
pub(crate) struct ImageSourceView {
    // Infra object identity, not the URL string or pixel density.
    pub identity: int,
    pub url: ServoUrlView,
    pub descriptor: Option<ImageSourceDescriptor>,
}

// https://html.spec.whatwg.org/multipage/images.html#width-descriptor
// https://html.spec.whatwg.org/multipage/images.html#pixel-density-descriptor
pub(crate) enum ImageSourceDescriptor {
    Width { width: nat },
    PixelDensity { density: PixelDensity },
}

// Normalizing width by a zero source size can produce positive infinity.
// Native f64 conversion/rounding is an explicit correspondence boundary.
pub(crate) enum PixelDensity { Finite { value: real }, PositiveInfinity }

// https://html.spec.whatwg.org/multipage/images.html#source-size-2
// A source size is a <source-size-value>. When a source size has a unit relative to the viewport, it must be interpreted relative to the img element's node document's viewport. Other units must be interpreted the same as in Media Queries.
// CSS syntax is retained separately from its contextual used value. Neither is
// equated to Servo's unevaluated SourceSizeList by an unconditional assumption.
pub(crate) struct SourceSizeView {
    pub css_value: Seq<char>,
    pub used_css_pixels: real,
}

// =========================
//      ServoUrl
// =========================

// https://url.spec.whatwg.org/#concept-url
// Snapshot 2026-10-19, inspected 2026-10-30.
// A URL is a struct that represents a universal identifier. To disambiguate from a valid URL string it can also be referred to as a URL record.
pub(crate) struct ServoUrlView {
    // A URL’s scheme is an ASCII string that identifies the type of URL and can be used to dispatch a URL for further processing after parsing. It is initially the empty string.
    pub scheme: Seq<char>,
    // A URL’s username is an ASCII string identifying a username. It is initially the empty string.
    pub username: Seq<char>,
    // A URL’s password is an ASCII string identifying a password. It is initially the empty string.
    pub password: Seq<char>,
    // A URL’s host is null or a host. It is initially null.
    pub host: Option<Host>,
    // A URL’s port is either null or a 16-bit unsigned integer that identifies a networking port. It is initially null.
    pub port: Option<u16>,
    // A URL’s path is a URL path, usually identifying a location. It is initially « ».
    pub path: UrlPath,
    // A URL’s query is either null or an ASCII string. It is initially null.
    pub query: Option<Seq<char>>,
    // A URL’s fragment is either null or an ASCII string that can be used for further processing on the resource the URL’s other components identify. It is initially null.
    pub fragment: Option<Seq<char>>,
    // A URL also has an associated blob URL entry that is either null or a blob URL entry. It is initially null.
    pub blob_url_entry: Option<BlobUrlEntryView>,
}

// https://url.spec.whatwg.org/#concept-host
// A host is a domain, an IP address, an opaque host, or an empty host.
// Host parsing and domain/opaque-host syntax are separate linked algorithms.
pub(crate) enum Host {
    Domain { value: Seq<char> },
    Ipv4 { address: u32 },
    Ipv6 { pieces: [u16; 8] },
    Opaque { value: Seq<char> },
    Empty,
}

// https://url.spec.whatwg.org/#url-path
// A URL path is either a URL path segment or a list of zero or more URL path segments.
pub(crate) enum UrlPath {
    Opaque { segment: Seq<char> },
    List { segments: Seq<Seq<char>> },
}

// https://w3c.github.io/FileAPI/#blob-url-entry
// A blob URL entry consists of an object (of type Blob or MediaSource), and an environment.
// The environment handle avoids recursively embedding snapshots of URL records.
pub(crate) struct BlobUrlEntryView {
    pub object: BlobUrlObject,
    pub environment: EnvironmentIdentityView,
}

pub(crate) enum BlobUrlObject { Blob { identity: int }, MediaSource { identity: int } }

// https://html.spec.whatwg.org/multipage/webappapis.html#environment
// Identity of the environment object, not a substitute for EnvironmentView's contents.
pub(crate) struct EnvironmentIdentityView { pub id: Seq<char> }

// =========================
//      Origin
// =========================

// https://html.spec.whatwg.org/multipage/browsers.html#concept-origin
// Snapshot and inspection 2026-10-30.
// An origin is one of the following:
pub(crate) enum Origin {
    // An opaque origin
    // An internal value, with no serialization it can be recreated from (it is serialized as "null" per serialization of an origin), for which the only meaningful operation is testing for equality.
    Opaque { identity: int },
    // A tuple origin
    // A tuple consisting of:
    Tuple {
        // A scheme (an ASCII string).
        scheme: Seq<char>,
        // A host (a host).
        host: Host,
        // A port (null or a 16-bit unsigned integer).
        port: Option<u16>,
        // A domain (null or a host). Null unless stated otherwise.
        domain: Option<Host>,
    },
}

// =========================
//      Environment
// =========================

// https://html.spec.whatwg.org/multipage/webappapis.html#environment
// Snapshot/inspection 2026-10-30.
// An environment is an object that identifies the settings of a current or potential execution environment (i.e., a navigation params's reserved environment or a request's reserved client). An environment has the following fields:
pub(crate) struct EnvironmentView {
    // An id
    // An opaque string that uniquely identifies this environment.
    pub id: Seq<char>,
    // A creation URL
    // A URL that represents the location of the resource with which this environment is associated.
    pub creation_url: ServoUrlView,
    // A top-level creation URL
    // Null or a URL that represents the creation URL of the "top-level" environment. It is null for workers and worklets.
    pub top_level_creation_url: Option<ServoUrlView>,
    // A top-level origin
    // A for now implementation-defined value, null, or an origin. For a "top-level" potential execution environment it is null (i.e., when there is no response yet); otherwise it is the "top-level" environment's origin. For a dedicated worker or worklet it is the top-level origin of its creator. For a shared or service worker it is an implementation-defined value.
    pub top_level_origin: TopLevelOrigin,
    // A target browsing context
    // Null or a target browsing context for a navigation request.
    pub target_browsing_context: Option<BrowsingContextView>,
    // An active service worker
    // Null or a service worker that controls the environment.
    pub active_service_worker: Option<ServiceWorkerView>,
    // An execution ready flag
    // A flag that indicates whether the environment setup is done. It is initially unset.
    pub execution_ready_flag: bool,
}

pub(crate) enum TopLevelOrigin { Null, Origin { origin: Origin }, ImplementationDefined { value: int } }
// https://html.spec.whatwg.org/multipage/document-sequences.html#browsing-context
// Identity of the navigation target; navigation state is outside this unit.
pub(crate) struct BrowsingContextView { pub identity: int }
// https://w3c.github.io/ServiceWorker/#dfn-service-worker
// Identity of the controlling worker; worker lifecycle is outside this unit.
pub(crate) struct ServiceWorkerView { pub identity: int }

// =========================
//      PolicyContainer
// =========================

/// Abstract contents of the user's Arc<PolicyContainerView> representation.
/// https://html.spec.whatwg.org/multipage/browsers.html#policy-container
/// A policy container is a struct containing policies that apply to a Document, a WorkerGlobalScope, or a WorkletGlobalScope. It has the following items:
// HTML snapshot/inspection 2026-10-30.
pub(crate) struct PolicyContainerView {
    // A CSP list, which is a CSP list. It is initially empty.
    pub csp_list: CspListView,
    // An embedder policy, which is an embedder policy. It is initially a new embedder policy.
    pub embedder_policy: EmbedderPolicyView,
    // A referrer policy, which is a referrer policy. It is initially the default referrer policy.
    pub referrer_policy: ReferrerPolicy,
    // An integrity policy, which is an integrity policy, initially a new integrity policy.
    pub integrity_policy: IntegrityPolicyView,
    // A report-only integrity policy, which is an integrity policy, initially a new integrity policy.
    pub report_only_integrity_policy: IntegrityPolicyView,
}

// https://html.spec.whatwg.org/multipage/browsers.html#embedder-policy
// An embedder policy consists of:
pub(crate) struct EmbedderPolicyView {
    // A value, which is an embedder policy value, initially "unsafe-none".
    pub value: EmbedderPolicyValue,
    // A reporting endpoint string, initially the empty string.
    pub reporting_endpoint: Seq<char>,
    // A report-only value, which is an embedder policy value, initially "unsafe-none".
    pub report_only_value: EmbedderPolicyValue,
    // A report-only reporting endpoint string, initially the empty string.
    pub report_only_reporting_endpoint: Seq<char>,
}

// https://html.spec.whatwg.org/multipage/browsers.html#embedder-policy-value
pub(crate) enum EmbedderPolicyValue { UnsafeNone, RequireCorp, Credentialless }

// https://w3c.github.io/webappsec-subresource-integrity/#integrity-policy
// Editor's Draft 2026-03-20; inspected 2026-10-30.
// An integrity policy, is a struct that contains the following:
pub(crate) struct IntegrityPolicyView {
    // sources, a list of sources, initially empty.
    pub sources: Seq<IntegritySource>,
    // blocked destinations, a list of destinations, initially empty.
    pub blocked_destinations: Seq<IntegrityDestination>,
    // endpoints, a list of strings, initially empty.
    pub endpoints: Seq<Seq<char>>,
}

// A source is a string. The only possible value for it is "inline".
pub(crate) enum IntegritySource { Inline }
// A destination is a destination type. The possible values for it are "script" and "style".
pub(crate) enum IntegrityDestination { Script, Style }

// =========================
//      Window
// =========================

/// Stable window metadata needed by link processing.
/// The identifier is Servo's existing WebViewId, as returned by Window::webview_id.
pub(crate) struct WindowView {
    pub webview_id: WebViewId,
}

// =========================
//      Document
// =========================

// https://html.spec.whatwg.org/multipage/dom.html#document
// Snapshot of the document properties read by link processing.
// Public because process_link_headers_automaton::State::create_new_option reads these fields.
pub struct DocumentView {
    pub id: int,
    // Device snapshot used by CSS Media Queries; the native Document view is still trusted.
    pub media_device: super::media_query::MediaDeviceView,
    pub window: WindowView,
    pub base_url: ServoUrlView,
    pub origin: Origin,
    pub environment: EnvironmentView,
    pub policy_container: Arc<PolicyContainerView>,
}

// =========================
//      DocsAlgo
// =========================

// on document ready (default null)
// Null or an algorithm accepting a Document
/// Identity of an algorithm accepting a Document, as used by on-document-ready.
/// TODO: I'm not really sure how it works, so leave it empty
pub(crate) struct DocsAlgoView {
    pub id: int,
}

// Structural projection of the native link processor's options argument.

// TRUSTED(scope): correspondence of the opaque native URL to a URL record is unverified.
pub(crate) uninterp spec fn url_view(url: &ServoUrl) -> ServoUrlView;

// TRUSTED(scope): correspondence of native immutable origins to origin records is unverified.
pub(crate) uninterp spec fn origin_view(origin: &ImmutableOrigin) -> Origin;

// TRUSTED(scope): the native policy container does not implement all WHATWG policy fields.
pub(crate) uninterp spec fn policy_container_view(container: &Arc<PolicyContainer>) -> Arc<PolicyContainerView>;

// TRUSTED(scope): RequestClient only partially represents the relevant settings object.
pub(crate) uninterp spec fn environment_view(client: &RequestClient) -> EnvironmentView;

impl View for SourceSet {
    type V = SourceSetView;

    // TRUSTED(scope): structural correspondence of the native candidates and source size is unverified.
    // No equality with the document-dependent source_set_view is assumed.
    uninterp spec fn view(&self) -> SourceSetView;
}

// The fields of the native options record are visible within this crate.
pub open(crate) spec fn options_view_of(o: LinkProcessingOptions) -> LinkProcessingOptionsView {
    // Not represented: referrer and request_client as native fields. In particular, the
    // client's preloaded_resources, is_nested_browsing_context, insecure_requests_policy,
    // and has_trustworthy_ancestor_origin are not separate link-processing options.
    LinkProcessingOptionsView {
        href: o.href@, // href: native string contents
        initiator: "link"@, // [default] Servo has no initiator field
        integrity: o.integrity@, // integrity: native string contents
        type_: o.link_type@, // type: native link_type string contents
        cryptographic_nonce_metadata: o.cryptographic_nonce_metadata@, // nonce: native string contents
        destination: o.destination, // destination: same enum
        crossorigin: o.cross_origin, // crossorigin: same optional enum
        referrer_policy: o.referrer_policy, // referrer policy: same enum
        source_set: match o.source_set { // source set: preserve presence and project its value
            Some(s) => Some(s@),
            None => None,
        },
        base_url: url_view(&o.base_url), // base URL: project the stored URL
        origin: origin_view(&o.origin), // origin: project the stored immutable origin
        environment: environment_view(&o.request_client), // [default] derive from the native request client
        policy_container: policy_container_view(&o.policy_container), // policy container: project the stored container
        document: None, // [default] Servo does not implement options' document
        on_document_ready: None, // [default] Servo has no on-document-ready callback field
        fetch_priority: LinkFetchPriority::Auto, // [default] Servo has no fetch-priority field
    }
}

/// The three Fetch modes allowed in an HTML preload key.
/// <https://html.spec.whatwg.org/multipage/links.html#preload-mode>
pub(crate) enum PreloadRequestMode { SameOrigin, Cors, NoCors }

/// Expose Servo's three Fetch credentials modes (omit, same-origin, include) to Verus.
/// <https://fetch.spec.whatwg.org/#concept-request-credentials-mode>
#[verifier::external_type_specification]
pub struct ExCredentialsMode(CredentialsMode);

/// Projection of a Fetch request as created for a link and passed to fetch by preload.
/// <https://fetch.spec.whatwg.org/#concept-request>
/// <https://html.spec.whatwg.org/multipage/semantics.html#create-a-link-request>
/// <https://html.spec.whatwg.org/multipage/urls-and-fetching.html#create-a-potential-cors-request>
/// <https://html.spec.whatwg.org/multipage/links.html#preload>
///
/// Omitted Fetch request fields (neither set by these algorithms nor read for the key):
/// - HTTP/cache/CORS: method, header list, body, local-URLs-only flag, unsafe-request flag,
///   use-CORS-preflight flag, cache mode, prevent no-cache cache-control header modification flag.
/// - Context/policy: reserved client, replaces client id, traversable for user prompts,
///   origin, top-level navigation initiator origin, referrer, initiator, parser metadata,
///   service-workers mode. CORS settings are an input used to derive mode and credentials mode.
/// - Navigation/scheduling: keepalive, internal priority, reload-navigation flag,
///   history-navigation flag, user-activation, WebDriver navigation id, render-blocking.
/// - Transport/Fetch bookkeeping: WebTransport-hash list, redirect mode, URL list, current URL,
///   redirect count, response tainting, done flag, timing allow failed flag, navigation timing
///   allow values list, WebDriver id. The key uses the request URL, not the current redirect URL.
pub(crate) struct RequestView {
    pub url: ServoUrlView,
    pub destination: Destination,
    pub mode: PreloadRequestMode,
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,
    pub integrity_metadata: Seq<char>,
    pub cryptographic_nonce_metadata: Seq<char>,
    pub referrer_policy: ReferrerPolicy,
    // Both are assigned from the options before create-a-link-request returns.
    pub policy_container: Arc<PolicyContainerView>,
    pub client: EnvironmentView,
    pub priority: LinkFetchPriority,
    // Fetch's nullable initiator type (preload can set it for early hints).
    pub initiator_type: Option<Seq<char>>,
}

/// HTML preload keys use itemwise equality on these four fields.
/// <https://html.spec.whatwg.org/multipage/links.html#preload-key>
pub(crate) struct PreloadKeyView {
    pub url: ServoUrlView,
    pub destination: Destination,
    pub mode: PreloadRequestMode,
    pub credentials_mode: CredentialsMode,
}

/// <https://html.spec.whatwg.org/multipage/links.html#create-a-preload-key>
pub(crate) open spec fn preload_key_of(r: RequestView) -> PreloadKeyView {
    PreloadKeyView {
        url: r.url,
        destination: r.destination,
        mode: r.mode,
        credentials_mode: r.credentials_mode,
    }
}

/// Response identity only; response contents and processing are not modelled.
/// <https://fetch.spec.whatwg.org/#concept-response>
pub(crate) struct ResponseView { pub id: int }

/// Identity of an algorithm accepting a response or null; its behavior is not modelled.
/// <https://html.spec.whatwg.org/multipage/links.html#preload-on-response-available>
pub(crate) struct OnResponseAvailableView { pub id: int }

/// <https://html.spec.whatwg.org/multipage/links.html#preload-entry>
pub(crate) struct PreloadEntryView {
    pub integrity_metadata: Seq<char>,
    pub response: Option<ResponseView>,
    pub on_response_available: Option<OnResponseAvailableView>,
}

/// Snapshot for reasoning across link-header calls on one document.
/// <https://html.spec.whatwg.org/multipage/links.html#map-of-preloaded-resources>
pub(crate) ghost struct DocumentLinkState {
    pub document: DocumentView,
    pub preloaded: Map<PreloadKeyView, PreloadEntryView>,
    // Cumulative fetch observations for this document, in dispatch order.
    pub fetches: Seq<RequestView>,
}

/// Observations for one handler invocation: the resulting map and just this call's fetches.
/// <https://html.spec.whatwg.org/multipage/links.html#map-of-preloaded-resources>
/// <https://html.spec.whatwg.org/multipage/links.html#preload>
pub(crate) struct ProcessLinkHeaderOutput {
    pub preloaded: Map<PreloadKeyView, PreloadEntryView>,
    pub fetches: Seq<RequestView>,
}

} // verus!
