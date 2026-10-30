/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Abstraction-owned types for the standard's constrained value domains.
//!
//! Type invariants apply to exec/tracked values. The underlying closed enums also
//! enforce the domains for spec-mode values, where Verus does not assume type invariants.
//! Servo representations are converted explicitly in servo_api::views.

use net_traits::ReferrerPolicy as NativeReferrerPolicy;
use net_traits::request::{CorsSettings as NativeCorsSettings, Destination as NativeDestination};
use vstd::prelude::*;
use std::sync::Arc;
use super::super::predicates::{ascii_case_insensitive_match, is_referrer_policy};
use super::super::servo_api::views::destination_name as native_destination_name;

verus! {

// =========================
//      Destination
// =========================

/// Local invariant-bearing wrapper around Servo's existing Fetch destination.
/// NativeDestination::None denotes the empty destination; "fetch" is a potential
/// destination translated by the preload algorithm, not a value of the enum.
/// This struct is used for capturing the servo Destination enum
pub(crate) struct DestinationView {
    // Crate visibility lets the open specifications expose wrapping/unwrapping;
    // exec/tracked construction and mutation must satisfy the local type invariant.
    pub(crate) value: NativeDestination,
}

impl DestinationView {
    pub(crate) open spec fn from_native(value: NativeDestination) -> Self {
        Self { value }
    }

    pub(crate) open spec fn as_native(self) -> NativeDestination {
        self.value
    }

    pub(crate) open spec fn as_str(self) -> Seq<char> {
        native_destination_name(self.as_native())
    }

    #[verifier::type_invariant]
    pub(crate) open spec fn well_formed(&self) -> bool {
        destination_types().contains(self.as_str())
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
// pub(crate) proof fn lemma_destination_name_valid(destination: DestinationView)
//     ensures
//         destination.well_formed(),
// {}

// =========================
//      EnumeratedAttribute
// =========================

// https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#enumerated-attribute
// Inspected 2026-10-30, Living Standard last updated 2026-10-30.
// Some attributes, called enumerated attributes, take on a finite set of states.
pub(crate) struct EnumerateAttribute<State> {
    pub states: Set<State>,
    // The state for such an attribute is derived by combining the attribute's
    // value, a set of keyword/state mappings, and three possible special states
    // that can also be given in the specification of the attribute.
    // The value (including whether the attribute is specified) is supplied to
    // determine_the_state_of_an_attribute in algorithms/enumerated_attributes.rs.
    pub keyword_states: Map<Seq<char>, State>,
    // These special states are the invalid value default, the missing value
    // default, and the empty value default.
    pub invalid_value_default: Option<State>,
    pub missing_value_default: Option<State>,
    pub empty_value_default: Option<State>,
}

impl<State> EnumerateAttribute<State> {
    /// The mapped/default states are members of the attribute's state set.
    /// Multiple keywords can map to the same state: the map need not be injective.
    pub(crate) open spec fn well_formed(self) -> bool {
        &&& (forall|keyword: Seq<char>| #[trigger] self.keyword_states.dom().contains(keyword)
            ==> self.states.contains(self.keyword_states[keyword]))
        &&& match self.invalid_value_default {
            Some(state) => self.states.contains(state),
            None => true,
        }
        &&& match self.missing_value_default {
            Some(state) => self.states.contains(state),
            None => true,
        }
        &&& match self.empty_value_default {
            Some(state) => self.states.contains(state),
            None => true,
        }
        // A case-insensitive keyword match must identify one state. This is
        // coherence of the keyword/state mapping, not a restriction on input values.
        &&& (forall|left: Seq<char>, right: Seq<char>|
            self.keyword_states.dom().contains(left)
            && self.keyword_states.dom().contains(right)
            && #[trigger] ascii_case_insensitive_match(left, right)
            ==> self.keyword_states[left] == self.keyword_states[right])
    }
}

/// The standard can return "no state". This is distinct from every State value,
/// including the valid CORS No CORS state, represented by None in this model.
pub(crate) enum EnumeratedAttributeState<State> {
    NoState,
    State { value: State },
}

// =========================
//      CorsSettings
// =========================

/// The two CORS settings carried by Servo. An Option<CorsSettingsView> uses
/// None for the HTML No CORS state.
pub(crate) struct CorsSettingsView {
    pub(crate) value: NativeCorsSettings,
}

impl CorsSettingsView {
    pub(crate) open spec fn from_native(value: NativeCorsSettings) -> Self {
        Self { value }
    }

    pub(crate) open spec fn as_native(self) -> NativeCorsSettings {
        self.value
    }

    #[verifier::type_invariant]
    pub(crate) open spec fn well_formed(&self) -> bool {
        cors_settings_attribute_definition().states.contains(Some(*self))
    }
}

// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-settings-attribute
// Inspected 2026-10-30, Living Standard last updated 2026-10-30.
// A CORS settings attribute is an enumerated attribute with the following keywords and states:
pub(crate) open spec fn cors_settings_attribute_definition()
    -> EnumerateAttribute<Option<CorsSettingsView>>
{
    let no_cors = None;
    let anonymous = Some(CorsSettingsView::from_native(NativeCorsSettings::Anonymous));
    let use_credentials = Some(CorsSettingsView::from_native(NativeCorsSettings::UseCredentials));
    EnumerateAttribute {
        states: set![no_cors, anonymous, use_credentials],
        // Keyword              State
        keyword_states: map![
            "anonymous"@       => anonymous,
            "use-credentials"@ => use_credentials,
        ],
        // The attribute's missing value default is the No CORS state,
        missing_value_default: Some(no_cors),
        // and its invalid value default and empty value default are both the Anonymous state.
        invalid_value_default: Some(anonymous),
        empty_value_default: Some(anonymous),
    }
}

// pub(crate) proof fn lemma_cors_settings_valid(settings: CorsSettingsView)
//     ensures settings.well_formed(),
// {}

// =========================
//      ReferrerPolicy
// =========================

/// Local wrapper for Servo's referrer-policy enum.
/// <https://w3c.github.io/webappsec-referrer-policy/#referrer-policy>
pub(crate) struct ReferrerPolicyView {
    pub(crate) value: NativeReferrerPolicy,
}

impl ReferrerPolicyView {
    pub(crate) open spec fn from_native(value: NativeReferrerPolicy) -> Self {
        Self { value }
    }

    pub(crate) open spec fn as_native(self) -> NativeReferrerPolicy {
        self.value
    }

    // A referrer policy is the empty string, "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", or "unsafe-url".
    pub(crate) open spec fn as_str(self) -> Seq<char> {
        match self.value {
            NativeReferrerPolicy::EmptyString => ""@,
            NativeReferrerPolicy::NoReferrer => "no-referrer"@,
            NativeReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade"@,
            NativeReferrerPolicy::Origin => "origin"@,
            NativeReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin"@,
            NativeReferrerPolicy::SameOrigin => "same-origin"@,
            NativeReferrerPolicy::StrictOrigin => "strict-origin"@,
            NativeReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin"@,
            NativeReferrerPolicy::UnsafeUrl => "unsafe-url"@,
        }
    }
    #[verifier::type_invariant]
    pub(crate) open spec fn well_formed(&self) -> bool {
        is_referrer_policy(self.as_str())
    }

    /// Accept canonical policy names; callers perform any required ASCII folding.
    pub(crate) open spec fn from_name(value: Seq<char>) -> Option<Self> {
        let native = if value == ""@ { Some(NativeReferrerPolicy::EmptyString) }
            else if value == "no-referrer"@ { Some(NativeReferrerPolicy::NoReferrer) }
            else if value == "no-referrer-when-downgrade"@ { Some(NativeReferrerPolicy::NoReferrerWhenDowngrade) }
            else if value == "origin"@ { Some(NativeReferrerPolicy::Origin) }
            else if value == "origin-when-cross-origin"@ { Some(NativeReferrerPolicy::OriginWhenCrossOrigin) }
            else if value == "same-origin"@ { Some(NativeReferrerPolicy::SameOrigin) }
            else if value == "strict-origin"@ { Some(NativeReferrerPolicy::StrictOrigin) }
            else if value == "strict-origin-when-cross-origin"@ { Some(NativeReferrerPolicy::StrictOriginWhenCrossOrigin) }
            else if value == "unsafe-url"@ { Some(NativeReferrerPolicy::UnsafeUrl) }
            else { None };
        match native {
            Some(value) => Some(Self::from_native(value)),
            None => None,
        }
    }
}

pub(crate) proof fn lemma_referrer_policy_valid(policy: ReferrerPolicyView)
    ensures
        policy.well_formed(),
        ReferrerPolicyView::from_name(policy.as_str()) == Some(policy),
{}

pub(crate) proof fn lemma_referrer_policy_from_name(value: Seq<char>)
    ensures
        match ReferrerPolicyView::from_name(value) {
            Some(policy) => is_referrer_policy(value) && policy.as_str() == value,
            None => !is_referrer_policy(value),
        },
{}

// =========================
//      LinkFetchPriority
// =========================

/// HTML's fetch-priority states. Servo's link options currently lack this field.
/// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#fetch-priority-attribute
/// A fetch priority attribute is an enumerated attribute with the following keywords and states:
pub(crate) enum LinkFetchPriorityView {
    Auto,
    High,
    Low,
}

// Inspected 2026-10-30, Living Standard last updated 2026-10-30.
pub(crate) open spec fn fetch_priority_attribute_definition()
    -> EnumerateAttribute<LinkFetchPriorityView>
{
    EnumerateAttribute {
        states: set![LinkFetchPriorityView::High, LinkFetchPriorityView::Low, LinkFetchPriorityView::Auto],
        // Keyword       State
        keyword_states: map![
            "high"@ => LinkFetchPriorityView::High,
            "low"@  => LinkFetchPriorityView::Low,
            "auto"@ => LinkFetchPriorityView::Auto,
        ],
        // The attribute's missing value default and invalid value default are both the Auto state.
        missing_value_default: Some(LinkFetchPriorityView::Auto),
        invalid_value_default: Some(LinkFetchPriorityView::Auto),
        // This attribute's definition specifies no empty value default.
        empty_value_default: None,
    }
}

// Standard-facing records and the remaining explicitly abstract object identities.

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

impl SourceSetView {
    pub(crate) open spec fn well_formed(self) -> bool {
        &&& self.image_sources.well_formed()
        &&& (forall|i: int| 0 <= i < self.image_sources.entries.len() ==>
            (#[trigger] self.image_sources.entries[i]).well_formed())
    }
}

// https://infra.spec.whatwg.org/#ordered-set
// An ordered set is a list that does not contain the same item twice.
// Source identities implement Infra object identity: equal URLs/densities do
// not collapse independently created source objects. Selection separately
// removes duplicate pixel densities.
pub(crate) struct OrderedImageSourcesView {
    pub(crate) entries: Seq<ImageSourceView>,
}

impl OrderedImageSourcesView {
    #[verifier::type_invariant]
    pub(crate) open spec fn well_formed(&self) -> bool {
        forall|i: int, j: int| 0 <= i < j < self.entries.len() ==>
            (#[trigger] self.entries[i]).identity != (#[trigger] self.entries[j]).identity
    }
}

// https://html.spec.whatwg.org/multipage/images.html#image-source
// An image source is a URL, and optionally either a pixel density descriptor, or a width descriptor.
pub(crate) struct ImageSourceView {
    // Infra object identity, not the URL string or pixel density.
    pub identity: int,
    pub url: ServoUrlView,
    pub descriptor: Option<ImageSourceDescriptorView>,
}

impl ImageSourceView {
    pub(crate) open spec fn well_formed(self) -> bool {
        match self.descriptor {
            None => true,
            Some(ImageSourceDescriptorView::Width { width }) => width > 0,
            Some(ImageSourceDescriptorView::PixelDensity { density }) => match density {
                PixelDensityView::Finite { value } => value >= 0real,
                PixelDensityView::PositiveInfinity => true,
            },
        }
    }
}

// https://html.spec.whatwg.org/multipage/images.html#width-descriptor
// https://html.spec.whatwg.org/multipage/images.html#pixel-density-descriptor
pub(crate) enum ImageSourceDescriptorView {
    Width { width: nat },
    PixelDensity { density: PixelDensityView },
}

// Normalizing width by a zero source size can produce positive infinity.
// Native f64 conversion/rounding is an explicit correspondence boundary.
pub(crate) enum PixelDensityView { Finite { value: real }, PositiveInfinity }

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
    pub host: Option<HostView>,
    // A URL’s port is either null or a 16-bit unsigned integer that identifies a networking port. It is initially null.
    pub port: Option<u16>,
    // A URL’s path is a URL path, usually identifying a location. It is initially « ».
    pub path: UrlPathView,
    // A URL’s query is either null or an ASCII string. It is initially null.
    pub query: Option<Seq<char>>,
    // A URL’s fragment is either null or an ASCII string that can be used for further processing on the resource the URL’s other components identify. It is initially null.
    pub fragment: Option<Seq<char>>,
    // A URL also has an associated blob URL entry that is either null or a blob URL entry. It is initially null.
    pub blob_url_entry: Option<BlobUrlEntryView>,
}

impl ServoUrlView {
    /// Definition defaults, not a claim that parsing the empty string succeeds.
    pub(crate) open spec fn initial() -> Self {
        Self {
            scheme: Seq::empty(), username: Seq::empty(), password: Seq::empty(),
            host: None, port: None, path: UrlPathView::List { segments: Seq::empty() },
            query: None, fragment: None, blob_url_entry: None,
        }
    }

    pub(crate) open spec fn component_domains(self) -> bool {
        &&& ascii_string(self.scheme)
        &&& ascii_string(self.username)
        &&& ascii_string(self.password)
        &&& (self.query is Some ==> ascii_string(self.query->Some_0))
        &&& (self.fragment is Some ==> ascii_string(self.fragment->Some_0))
        &&& self.path.component_domains()
    }
}

// https://infra.spec.whatwg.org/#ascii-string
// An ASCII string is a string whose code points are all ASCII code points.
// Form: row 2, forall; independent code-point membership checks.
pub(crate) open spec fn ascii_string(value: Seq<char>) -> bool {
    forall|i: int| 0 <= i < value.len() ==> (#[trigger] value[i]) as u32 <= 0x7f
}

// https://url.spec.whatwg.org/#concept-host
// A host is a domain, an IP address, an opaque host, or an empty host.
// Host parsing and domain/opaque-host syntax are separate linked algorithms.
pub(crate) enum HostView {
    Domain { value: Seq<char> },
    Ipv4 { address: u32 },
    Ipv6 { pieces: [u16; 8] },
    Opaque { value: Seq<char> },
    Empty,
}

// https://url.spec.whatwg.org/#concept-host-equals
// To determine whether a host A equals host B, return true if A is B, and false otherwise.
pub(crate) open spec fn host_equals(a: HostView, b: HostView) -> bool { a == b }

// https://url.spec.whatwg.org/#url-path
// A URL path is either a URL path segment or a list of zero or more URL path segments.
pub(crate) enum UrlPathView {
    Opaque { segment: Seq<char> },
    List { segments: Seq<Seq<char>> },
}

impl UrlPathView {
    pub(crate) open spec fn component_domains(self) -> bool {
        match self {
            Self::Opaque { segment } => ascii_string(segment),
            Self::List { segments } => forall|i: int| 0 <= i < segments.len() ==>
                ascii_string(#[trigger] segments[i]) && !segments[i].contains('/'),
        }
    }
}

// https://w3c.github.io/FileAPI/#blob-url-entry
// A blob URL entry consists of an object (of type Blob or MediaSource), and an environment.
// The environment handle avoids recursively embedding snapshots of URL records.
pub(crate) struct BlobUrlEntryView {
    pub object: BlobUrlObjectView,
    pub environment: EnvironmentIdentityView,
}

pub(crate) enum BlobUrlObjectView { Blob { identity: int }, MediaSource { identity: int } }

// https://html.spec.whatwg.org/multipage/webappapis.html#environment
// Identity of the environment object, not a substitute for EnvironmentView's contents.
pub(crate) struct EnvironmentIdentityView { pub id: Seq<char> }

// =========================
//      Origin
// =========================

// https://html.spec.whatwg.org/multipage/browsers.html#concept-origin
// Snapshot and inspection 2026-10-30.
// An origin is one of the following:
pub(crate) enum OriginView {
    // An opaque origin
    // An internal value, with no serialization it can be recreated from (it is serialized as "null" per serialization of an origin), for which the only meaningful operation is testing for equality.
    Opaque { identity: int },
    // A tuple origin
    // A tuple consisting of:
    Tuple {
        // A scheme (an ASCII string).
        scheme: Seq<char>,
        // A host (a host).
        host: HostView,
        // A port (null or a 16-bit unsigned integer).
        port: Option<u16>,
        // A domain (null or a host). Null unless stated otherwise.
        domain: Option<HostView>,
    },
}

impl OriginView {
    /// Representation adapter for domain state owned by MutableOrigin/OriginSnapshot,
    /// not by ImmutableOrigin. Opaque origins have no mutable domain component.
    pub(crate) open spec fn with_domain(self, domain: Option<HostView>) -> Self {
        match self {
            Self::Opaque { .. } => self,
            Self::Tuple { scheme, host, port, .. } => Self::Tuple { scheme, host, port, domain },
        }
    }
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
    pub top_level_origin: TopLevelOriginView,
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

impl EnvironmentView {
    pub(crate) open spec fn identity(self) -> EnvironmentIdentityView {
        EnvironmentIdentityView { id: self.id }
    }
}

pub(crate) enum TopLevelOriginView { Null, Origin { origin: OriginView }, ImplementationDefined { value: int } }
// https://html.spec.whatwg.org/multipage/document-sequences.html#browsing-context
// Identity of the navigation target; navigation state is outside this unit.
pub(crate) struct BrowsingContextView { pub identity: int }
// https://w3c.github.io/ServiceWorker/#dfn-service-worker
// Identity of the controlling worker; worker lifecycle is outside this unit.
pub(crate) struct ServiceWorkerView { pub identity: int }

// https://html.spec.whatwg.org/multipage/webappapis.html#environment-settings-object
// Snapshot/inspection 2026-10-30.
// An environment settings object is an environment that additionally specifies algorithms for:
// These fields are observations of the listed algorithms at a snapshot, not a
// claim that RequestClient stores the entire settings object.
pub(crate) struct EnvironmentSettingsView {
    pub environment: EnvironmentView,
    // A realm execution context
    // A JavaScript execution context shared by all scripts that use this settings object, i.e., all scripts in a given realm. When we run a classic script or run a module script, this execution context becomes the top of the JavaScript execution context stack, on top of which another execution context specific to the script in question is pushed. (This setup ensures Source Text Module Record's Evaluate knows which realm to use.)
    pub realm_execution_context: int,
    // A module map
    // A module map that is used when importing JavaScript modules.
    pub module_map: ModuleMapView,
    // An API base URL
    // A URL used by APIs called by scripts that use this environment settings object to parse URLs.
    pub api_base_url: ServoUrlView,
    // An origin
    // An origin used in security checks.
    pub origin: OriginView,
    // A has cross-site ancestor
    // A boolean used in security checks.
    pub has_cross_site_ancestor: bool,
    // A policy container
    // A policy container containing policies used for security checks.
    pub policy_container: Arc<PolicyContainerView>,
    // A cross-origin isolated capability
    // A boolean representing whether scripts that use this environment settings object are allowed to use APIs that require cross-origin isolation.
    pub cross_origin_isolated_capability: bool,
    // A time origin
    // A number used as the baseline for performance-related timestamps.
    pub time_origin: real,
}

// https://html.spec.whatwg.org/multipage/webappapis.html#module-map
// Keys pair a URL and module type; entries represent fetching, failure or a module script.
// The URL key is its canonical serialization so blob-entry identity is not part
// of key equality. Producing this key requires the linked URL serializer.
pub(crate) struct ModuleMapView {
    pub entries: Map<(SerializedUrlKeyView, Seq<char>), ModuleMapEntryView>,
}

pub(crate) enum ModuleMapEntryView { Fetching, Failed, ModuleScript { identity: int } }

// https://url.spec.whatwg.org/#concept-url-equals
pub(crate) struct SerializedUrlKeyView { pub serialization: Seq<char> }

// =========================
//      PolicyContainer
// =========================

/// Abstract contents of the user's Arc<PolicyContainerView> representation.
/// https://html.spec.whatwg.org/multipage/browsers.html#policy-container
/// A policy container is a struct containing policies that apply to a Document, a WorkerGlobalScope, or a WorkletGlobalScope. It has the following items:
/// TODO: the PolicyContainer should follow previous structs as well, especially the connection with NaitivePolicyContainer in Servo
pub(crate) struct PolicyContainerView {
    pub id: int,
}

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
