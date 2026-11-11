/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The WHATWG link-processing-options record restored from the backup.

use std::sync::Arc;
use net_traits::ReferrerPolicy;
use net_traits::request::{CorsSettings, Destination};
use vstd::prelude::*;
use super::types::*;

verus! {

// LinkProcessingOptions does not yet represent: initiator, document,
// https://html.spec.whatwg.org/multipage/semantics.html#link-processing-options
// A link processing options is a struct. It has the following items:
pub(crate) struct LinkProcessingOptionsView {
    // href (default the empty string)
    pub href: Seq<char>,
    // initiator (default "link")
    pub initiator: Seq<char>,
    // integrity (default the empty string)
    pub integrity: Seq<char>,
    // type (default the empty string)
    pub type_: Seq<char>,
    // cryptographic nonce metadata (default the empty string)
    // A string
    pub cryptographic_nonce_metadata: Seq<char>,
    // destination (default the empty string)
    // A destination type.
    pub destination: Destination,
    // crossorigin (default No CORS)
    // A CORS settings attribute state
    // None represents the HTML No CORS state.
    // https://html.spec.whatwg.org/multipage/urls-and-fetching.html#cors-settings-attribute
    pub crossorigin: Option<CorsSettings>,
    // referrer policy (default the empty string)
    // A referrer policy
    // <https://w3c.github.io/webappsec-referrer-policy/#referrer-policy>
    pub referrer_policy: ReferrerPolicy,
    // source set (default null)
    // Null or a source set
    pub source_set: Option<SourceSetView>,
    // base URL
    // A URL
    pub base_url: ServoUrlView,
    // origin
    // An origin
    pub origin: Origin,
    // environment
    // An environment
    pub environment: EnvironmentView,
    // policy container
    // A policy container
    pub policy_container: Arc<PolicyContainerView>,
    // document (default null)
    // Null or a Document
    pub document: Option<DocumentView>, // Not implemented in Servo
    // on document ready (default null)
    // Null or an algorithm accepting a Document
    pub on_document_ready: Option<DocsAlgoView>, // Not implemented in Servo
    // fetch priority (default Auto)
    // A fetch priority attribute state
    pub fetch_priority: LinkFetchPriority, // Not implemented in Servo
}

} // verus!
