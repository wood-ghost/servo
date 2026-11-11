/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Nested type declarations for the standard's link-processing-options record.
//! The standard excerpts retain the backup's inspection dates.

#![verifier::auto_reveal_strlit]

use content_security_policy::{PolicyDisposition, PolicySource};
use net_traits::ReferrerPolicy;
use net_traits::request::Destination;
use vstd::prelude::*;

verus! {

// =========================
//      Destination
// =========================

/// Specification-level conversions for Servo's existing Fetch destination.
/// https://fetch.spec.whatwg.org/#destination-type
/// Destination::None denotes the empty destination; "fetch" is a potential
/// destination translated by the preload algorithm, not a value of the enum.
pub(crate) trait DestinationSpec: Sized {
    spec fn from_str(value: Seq<char>) -> Option<Self>;
    spec fn to_str(&self) -> Seq<char>;
}

impl DestinationSpec for Destination {
    open spec fn to_str(&self) -> Seq<char> {
        match *self {
            Destination::None => ""@,
            Destination::Audio => "audio"@,
            Destination::AudioWorklet => "audioworklet"@,
            Destination::Document => "document"@,
            Destination::Embed => "embed"@,
            Destination::Font => "font"@,
            Destination::Frame => "frame"@,
            Destination::IFrame => "iframe"@,
            Destination::Image => "image"@,
            Destination::Json => "json"@,
            Destination::Manifest => "manifest"@,
            Destination::Object => "object"@,
            Destination::PaintWorklet => "paintworklet"@,
            Destination::Report => "report"@,
            Destination::Script => "script"@,
            Destination::ServiceWorker => "serviceworker"@,
            Destination::SharedWorker => "sharedworker"@,
            Destination::Style => "style"@,
            Destination::Text => "text"@,
            Destination::Track => "track"@,
            Destination::Video => "video"@,
            Destination::WebIdentity => "webidentity"@,
            Destination::Worker => "worker"@,
            Destination::Xslt => "xslt"@,
        }
    }

    /// Representation lookup, not a second definition of Fetch translation.
    /// Every standard destination name maps to the existing native enum variant.
    open spec fn from_str(value: Seq<char>) -> Option<Self> {
        if value == ""@ { Some(Destination::None) }
            else if value == "audio"@ { Some(Destination::Audio) }
            else if value == "audioworklet"@ { Some(Destination::AudioWorklet) }
            else if value == "document"@ { Some(Destination::Document) }
            else if value == "embed"@ { Some(Destination::Embed) }
            else if value == "font"@ { Some(Destination::Font) }
            else if value == "frame"@ { Some(Destination::Frame) }
            else if value == "iframe"@ { Some(Destination::IFrame) }
            else if value == "image"@ { Some(Destination::Image) }
            else if value == "json"@ { Some(Destination::Json) }
            else if value == "manifest"@ { Some(Destination::Manifest) }
            else if value == "object"@ { Some(Destination::Object) }
            else if value == "paintworklet"@ { Some(Destination::PaintWorklet) }
            else if value == "report"@ { Some(Destination::Report) }
            else if value == "script"@ { Some(Destination::Script) }
            else if value == "serviceworker"@ { Some(Destination::ServiceWorker) }
            else if value == "sharedworker"@ { Some(Destination::SharedWorker) }
            else if value == "style"@ { Some(Destination::Style) }
            else if value == "text"@ { Some(Destination::Text) }
            else if value == "track"@ { Some(Destination::Track) }
            else if value == "video"@ { Some(Destination::Video) }
            else if value == "webidentity"@ { Some(Destination::WebIdentity) }
            else if value == "worker"@ { Some(Destination::Worker) }
            else if value == "xslt"@ { Some(Destination::Xslt) }
            else { None }
    }
}

// https://fetch.spec.whatwg.org/#destination-type
// A destination type is one of: the empty string, "audio", "audioworklet", "document", "embed", "font", "frame", "iframe", "image", "json", "manifest", "object", "paintworklet", "report", "script", "serviceworker", "sharedworker", "style", "text", "track", "video", "webidentity", "worker", or "xslt". 
pub open spec fn destination_types() -> Set<Seq<char>> {
    set![
        ""@,
        "audio"@,
        "audioworklet"@,
        "document"@,
        "embed"@,
        "font"@,
        "frame"@,
        "iframe"@,
        "image"@,
        "json"@,
        "manifest"@,
        "object"@,
        "paintworklet"@,
        "report"@,
        "script"@,
        "serviceworker"@,
        "sharedworker"@,
        "style"@,
        "text"@,
        "track"@,
        "video"@,
        "webidentity"@,
        "worker"@,
        "xslt"@,
    ]
}

/// Also establishes the domain for arbitrary spec-mode values.
/// TODO: Useful?
pub(crate) broadcast proof fn lemma_destination_name_valid(d: Destination)
    ensures
        destination_types().contains(#[trigger] d.to_str()),
{}

pub(crate) proof fn destination_from_name_roundtrip(destination: Destination)
    ensures Destination::from_str(destination.to_str()) == Some(destination),
{}

// =========================
//      LinkFetchPriority
// =========================

/// HTML's fetch-priority states. Servo's link options currently lack this field.
/// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#fetch-priority-attribute
/// A fetch priority attribute is an enumerated attribute with the following keywords and states:
pub(crate) enum LinkFetchPriority {
    Auto,
    High,
    Low,
}

// =========================
//      SourceSet
// =========================

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

// https://w3c.github.io/webappsec-csp/#csp-list
// Editor's Draft 2026-10-27; inspected 2026-10-30.
// A CSP list is a struct consisting of policies (a list of policies) and a self-origin (an origin which is used when matching the 'self' keyword).
pub(crate) struct CspListView {
    pub policies: Seq<CspPolicyView>,
    pub self_origin: Origin,
}

// https://w3c.github.io/webappsec-csp/#content-security-policy-object
pub(crate) struct CspPolicyView {
    // Each policy has an associated directive set, which is an ordered set of directives that define the policy’s implications when applied.
    pub directive_set: Seq<CspDirectiveView>,
    // Each policy has an associated disposition, which is either "enforce" or "report".
    pub disposition: PolicyDisposition,
    // Each policy has an associated source, which is either "header" or "meta".
    pub source: PolicySource,
}

// https://w3c.github.io/webappsec-csp/#directives
// Each directive is a name / value pair. The name is a non-empty string, and the value is a set of non-empty strings. The value MAY be empty.
pub(crate) struct CspDirectiveView { pub name: Seq<char>, pub value: Seq<Seq<char>> }

// =========================
//      Document
// =========================

// https://html.spec.whatwg.org/multipage/dom.html#document
// as it is not included in servo implementation, leave the related parts as uninterp
pub(crate) struct DocumentView {
    pub id: int,
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

} // verus!
