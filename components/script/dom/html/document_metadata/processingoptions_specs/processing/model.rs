/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specification views shared by the WHATWG link-processing algorithms.
//!
//! Source: <https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers>,
//! inspected 2026-10-29, Living Standard last updated 2026-10-26.
//! Step 1 supplies already-extracted, normalized link objects. Connecting these
//! objects to Servo's `LinkDataOwned` representation is a separate obligation.
//!
//! The trace records calls, their arguments/results, and document-state snapshots.
//! This model specifies the caller's ordering, branching, option construction, and
//! source-set update. Media evaluation, attribute application, source-set creation,
//! and each relation type's handler retain their separate algorithm contracts.
//! A valid hypothetical trace is not evidence that Servo executed it: connecting
//! the supplied trace to execution requires ghost tracking or a state model.

#![verifier::auto_reveal_strlit]

use std::sync::Arc;
use vstd::prelude::*;
use super::types::*;

verus! {

pub(crate) type LinkAttributes = Map<Seq<char>, Seq<char>>;

/// A link object after the standard's extraction/parsing step, including relation
/// splitting and parameter normalization. Attribute values may be empty strings.
pub(crate) struct ParsedLinkHeader {
    pub target_uri: Seq<char>,
    pub relation_type: Seq<char>,
    pub target_attributes: LinkAttributes,
}

pub(crate) enum LinkHeadersPhase {
    PreMedia,
    Media,
}

/// Document/context snapshots with a typed document identity, not Rust addresses. `effects`
/// identifies the remaining observable state, e.g. registered preloads. Context
/// values can change between dispatched links; the document identity stays fixed.
pub(crate) struct LinkHeaderDocumentState {
    pub document: DocumentView,
    pub base_url: ServoUrlView,
    pub origin: OriginView,
    pub environment: EnvironmentView,
    pub policy_container: Arc<PolicyContainerView>,
    pub effects: int,
}

/// Specification view of the standard's full options record, following MimeView's
/// naming convention. This includes fields that Servo's current
/// LinkProcessingOptions does not yet represent: initiator, document,
/// on-document-ready, and fetch priority. Nullable algorithms use typed identities;
/// source sets carry detailed records. None represents the standard's null value.
/// https://html.spec.whatwg.org/multipage/semantics.html#link-processing-options
// A link processing options is a struct. It has the following items:
pub(crate) struct LinkProcessingOptionsView {
    // href (default the empty string)
    // A string
    pub href: Seq<char>,
    // initiator (default "link")
    // A string
    pub initiator: Seq<char>,
    // integrity (default the empty string)
    // A string
    pub integrity: Seq<char>,
    // type (default the empty string)
    // A string
    pub type_: Seq<char>,
    // cryptographic nonce metadata (default the empty string)
    // A string
    pub cryptographic_nonce_metadata: Seq<char>,
    // destination (default the empty string)
    // A destination type.
    pub destination: DestinationView,
    // crossorigin (default No CORS)
    // A CORS settings attribute state
    pub crossorigin: Option<CorsSettingsView>,
    // referrer policy (default the empty string)
    // A referrer policy
    pub referrer_policy: ReferrerPolicyView,
    // source set (default null)
    // Null or a source set
    pub source_set: Option<SourceSetView>,
    // base URL
    // A URL
    pub base_url: ServoUrlView,
    // origin
    // An origin
    pub origin: OriginView,
    // environment
    // An environment
    pub environment: EnvironmentView,
    // policy container
    // A policy container
    pub policy_container: Arc<PolicyContainerView>,
    // document (default null)
    // Null or a Document
    pub document: Option<DocumentView>,
    // on document ready (default null)
    // Null or an algorithm accepting a Document
    pub on_document_ready: Option<DocsAlgoView>,
    // fetch priority (default Auto)
    // A fetch priority attribute state
    pub fetch_priority: LinkFetchPriorityView,
}

// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-settings-attribute
// A CORS settings attribute is an enumerated attribute with the following keywords and states:

// https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#enumerated-attribute
// Enumerated Attributes
// Some attributes, called enumerated attributes, take on a finite set of states. The state for such an attribute is derived by combining the attribute's value, a set of keyword/state mappings, and three possible special states that can also be given in the specification of the attribute. These special states are the invalid value default, the missing value default, and the empty value default.
// The generic standard definition is types::EnumerateAttribute. Servo's
// make_enumerated_getter! in dom/macros.rs is an executable macro, not this trait/model.

pub(crate) struct MediaQueryCall {
    pub document: DocumentView,
    pub query: Seq<char>,
    pub result: bool,
}

pub(crate) struct ApplyLinkOptionsCall {
    pub before: LinkProcessingOptionsView,
    pub attributes: LinkAttributes,
    pub relation: Seq<char>,
    pub after: LinkProcessingOptionsView,
    pub result: bool,
}

pub(crate) struct CreateSourceSetCall {
    pub default_source: Seq<char>,
    pub srcset: Seq<char>,
    pub sizes: Seq<char>,
    pub img: Option<int>,
    pub result: SourceSetView,
}

pub(crate) struct ProcessLinkHeaderCall {
    pub relation: Seq<char>,
    pub options: LinkProcessingOptionsView,
}

/// These variants record which calls occurred, rather than merely whether some
/// suitable options could exist. In particular, skipped/failed links have no
/// dispatch call, and a failed application has no source-set creation call.
/// Dispatch denotes the sequence apply -> optional source-set creation -> process.
/// A future execution bridge must establish that this is the actual call order.
pub(crate) enum LinkHeaderAction {
    SkipPhase,
    SkipMedia,
    ReturnAfterApply { apply: ApplyLinkOptionsCall },
    Dispatch {
        apply: ApplyLinkOptionsCall,
        source_set: Option<CreateSourceSetCall>,
        process: ProcessLinkHeaderCall,
    },
}

pub(crate) struct LinkHeaderStep {
    pub header_index: nat,
    pub media: Option<MediaQueryCall>,
    pub action: LinkHeaderAction,
}

/// One step per visited link, including skipped links. states[i] and states[i+1]
/// are the snapshots before and after steps[i]. The last state is the final one.
pub(crate) struct ProcessLinkHeadersTrace {
    pub steps: Seq<LinkHeaderStep>,
    pub states: Seq<LinkHeaderDocumentState>,
}

pub(crate) type HeaderListView = Seq<(Seq<char>, Seq<char>)>;

pub(crate) enum RequestMode { NoCors, Cors, SameOrigin }
pub(crate) enum CredentialsMode { Include, SameOrigin, Omit }

/// The fields used by the algorithms below. `other` carries the remaining Fetch
/// request fields. Their defaults belong to the external Fetch model interface.
/// https://fetch.spec.whatwg.org/#concept-request
/// Fetch snapshot 2026-10-19, inspected 2026-10-30.
pub(crate) struct RequestView {
    pub url: ServoUrlView,
    pub destination: DestinationView,
    pub mode: RequestMode,
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,
    // None represents Fetch's "client" sentinel, not a null policy container.
    // A request has an associated policy container, which is "client" or a policy container. Unless stated otherwise it is "client".
    pub policy_container: Option<Arc<PolicyContainerView>>,
    // A request has associated integrity metadata (a string). Unless stated otherwise, it is the empty string.
    pub integrity_metadata: Seq<char>,
    // A request has associated cryptographic nonce metadata (a string). Unless stated otherwise, it is the empty string.
    pub cryptographic_nonce_metadata: Seq<char>,
    pub referrer_policy: ReferrerPolicyView,
    pub client: Option<EnvironmentView>,
    pub priority: LinkFetchPriorityView,
    pub initiator_type: Option<Seq<char>>,
    pub other: int,
}

// https://html.spec.whatwg.org/multipage/links.html#preload-key
// Snapshot/inspection 2026-10-30.
// A preload key is a struct. It has the following items:
// FINDING: the destination item says preload destination, but the constructor
// copies request.destination (already translated, including the empty string).
// This view preserves that constructor's value, exposing the domain mismatch.
pub(crate) struct PreloadKeyView {
    // URL
    // A URL
    pub url: ServoUrlView,
    // destination
    // A preload destination
    pub destination: DestinationView,
    // mode
    // A request mode, either "same-origin", "cors", or "no-cors"
    pub mode: RequestMode,
    // credentials mode
    // A credentials mode
    pub credentials_mode: CredentialsMode,
}

pub(crate) struct ResponseView {
    pub network_error: bool,
    pub body: Option<Seq<u8>>,
    pub other: int,
}

// https://html.spec.whatwg.org/multipage/links.html#preload-entry
// Snapshot/inspection 2026-10-30.
// A preload entry is a struct. It has the following items:
pub(crate) struct PreloadEntryView {
    // integrity metadata
    // A string
    pub integrity_metadata: Seq<char>,
    // response
    // Null or a response
    pub response: Option<ResponseView>,
    // on response available
    // Null, or an algorithm accepting a response or null
    pub on_response_available: Option<int>,
}

pub(crate) enum PreloadCommit {
    Document { document: DocumentView },
    OnDocumentReady,
}

/// Definitions of the nested algorithms in preload. Their bodies are in
/// algorithms/callbacks.rs and take callback state at invocation time.
pub(crate) enum PreloadCallback {
    ReportTiming,
    ProcessResponseConsumeBody,
    Commit,
}

/// The Fetch invocation of preload step 11, including assignment of its returned
/// controller. A descriptor is an intended operation, not an executed fetch.
pub(crate) struct PreloadFetch {
    pub request: RequestView,
    pub process_response_consume_body: PreloadCallback,
    pub controller: int,
}

/// A plan records operations selected by the standard, not evidence of an
/// executable fetch. The callback bodies have their own numbered specifications.
pub(crate) enum PreloadPlan {
    TypeMismatch,
    RequestFailure,
    Fetch {
        options: LinkProcessingOptionsView,
        request: RequestView,
        initial_unsafe_end_time: int,
        entry: PreloadEntryView,
        key: PreloadKeyView,
        initial_controller: Option<int>,
        process_response: Option<int>,
        report_timing: PreloadCallback,
        fetch: PreloadFetch,
        commit_callback: PreloadCallback,
        commit_algorithm: DocsAlgoView,
        commit: PreloadCommit,
    },
}

pub(crate) enum PreconnectPlan {
    Noop,
    MayConnect { partition_key: int, origin: OriginView, use_credentials: bool },
}

pub(crate) enum LinkHeaderPlan {
    Noop,
    Preload { preload: PreloadPlan },
    Preconnect { preconnect: PreconnectPlan },
}

/// Supplied observations at the link-element caller boundary. The options are
/// observed after source-set update, when create-link-options runs. raw_as is a
/// native correspondence observation, not a field of HTML link processing options.
pub(crate) struct LinkElementPreloadObservations {
    pub element: int,
    pub options_after_source_update: LinkProcessingOptionsView,
    pub as_keyword: Option<Seq<char>>,
    pub raw_as: Option<Seq<char>>,
    pub response_callback: int,
}

pub(crate) struct LinkElementPreloadPlan {
    pub source_set_updated: bool,
    pub preload: Option<PreloadPlan>,
}

pub(crate) enum CallbackEffect {
    ReportTiming { controller: Option<int>, global: int },
    InvokeResponse { callback: int, response: ResponseView },
    StorePreload { document: DocumentView, key: PreloadKeyView, entry: PreloadEntryView },
    FireEvent { element: int, name: Seq<char> },
    SubmitTiming { global: int, timing: int, failed: bool },
    ReportCspViolations { global: int, violations: Seq<int> },
}

pub(crate) struct PreloadCallbackState {
    pub entry: PreloadEntryView,
    pub key: PreloadKeyView,
    pub controller: Option<int>,
    pub unsafe_end_time: int,
    pub document: Option<DocumentView>,
    pub process_response: Option<int>,
}

pub(crate) enum BodyResult {
    Bytes { bytes: Seq<u8> },
    Failure,
    Null,
}

pub(crate) struct CallbackResult {
    pub state: PreloadCallbackState,
    pub effects: Seq<CallbackEffect>,
}

pub(crate) enum LinkFetchContextTypeView { Prefetch, Preload }

pub(crate) struct LinkFetchContextView {
    pub kind: LinkFetchContextTypeView,
    pub url: ServoUrlView,
    pub link: Option<int>,
    pub response_body: Seq<u8>,
    pub capacity: nat,
}

} // verus!
