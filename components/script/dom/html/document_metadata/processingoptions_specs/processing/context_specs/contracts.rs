/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Executable-facing contracts and representation/effect boundaries.
//!
//! Thin wrappers connect runtime values to numbered reference algorithms. Native
//! storage/conversion behavior is identified explicitly where it differs from the
//! standard. Parser, UA-support, representation, and effect-observation agreement
//! remain separate obligations; execution hooks do not establish conformance.

use std::cell::Ref;
use std::sync::Arc;

use js::context::NoGC;
use mime::Mime;
use net_traits::ReferrerPolicy;
use net_traits::mime_classifier::MediaType;
use net_traits::mime_classifier_specs::{classifier as SpecClassifier, mime_api as SpecMime};
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{CorsSettings, Destination, Referrer, RequestBuilder, RequestClient};
use nom_rfc8288::complete::LinkParamOwned;
use script_bindings::conversions::DerivedFrom;
use script_bindings::refcounted::Trusted;
use script_bindings::reflector::DomObject;
use script_bindings::root::DomRoot;
use servo_base::id::WebViewId;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use vstd::prelude::*;

use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::Node;
use crate::dom::srcset::SourceSet;
use crate::dom::types::HTMLLinkElement;
use crate::dom::window::Window;

use crate::dom::html::document_metadata::processingoptions::{
    LinkHeader, LinkProcessingOptions, LinkProcessingPhase,
};
use super::super::model as SpecModel;
use super::super::algorithms as SpecAlgorithms;
use super::super::super::servo_api::destination_is_script_like;
use super::super::super::servo_api::views as SpecViews;
use super::super::super::servo_api::link_header as SpecLinkHeader;
use super::super::super::predicates::ascii_lowercase;

verus! {

// ----------------------------------------
// ValueForKeyInLinkHeader predicates
// ----------------------------------------

/// Compare returned string contents without prescribing reference identity.
pub(crate) open spec fn option_str_view(value: Option<&str>) -> Option<Seq<char>> {
    match value {
        Some(value) => Some(value@),
        None => None,
    }
}

/// Membership using Servo's exact, case-sensitive key comparison.
/// HTML's attribute-existence operations use RFC 8288 parsed parameters; parser
/// normalization and agreement with those standards remain separate obligations.
pub(crate) open spec fn has_key_in_link_header_result(
    params: Seq<LinkParamOwned>,
    key: Seq<char>,
) -> bool {
    SpecLinkHeader::has_key(params, key)
}

/// Return the optional value of the first matching key. A first match with
/// no value returns None even when a later duplicate has a value. This differs
/// from key membership and must preserve Some("") versus None.
pub(crate) open spec fn value_for_key_in_link_header_result(
    params: Seq<LinkParamOwned>,
    key: Seq<char>,
) -> Option<Seq<char>> {
    SpecLinkHeader::first_value(params, key)
}

// ----------------------------------------
// LinkProcessingOptions predicates
// ----------------------------------------

/// Boolean result of applying parsed header attributes to link processing options.
/// <https://html.spec.whatwg.org/multipage/#apply-link-options-from-parsed-header-attributes>
/// The Boolean result depends on the preload destination check in step 1.
pub(crate) open spec fn apply_link_options_from_parsed_header_result(
    link_object: &LinkHeader,
    rel: Seq<char>,
) -> bool {
    // 1. If rel is "preload":
    if rel == "preload"@ {
        // 1.1. If attribs["as"] does not exist, then return false.
        match value_for_key_in_link_header_result(link_object.params@, "as"@) {
            None => false,
            Some(as_) => {
                // 1.2–1.3. Translate the destination; null returns false.
                translate_a_preload_destination_result(as_) is Some
            },
        }
    } else {
        // 8. Return true after applying the other attributes.
        true
    }
}

/// Field updates, preserved fields, and early failure are defined by the numbered
/// algorithm in processing/algorithms/options.rs. This is a native proof goal.
/// The before/after arguments are snapshots around the mutable options borrow.
// https://html.spec.whatwg.org/multipage/semantics.html#apply-link-options-from-parsed-header-attributes
// To apply link options from parsed header attributes to a link processing options options given attribs and a string rel:
#[verifier::auto_reveal_strlit]
pub(crate) open spec fn apply_link_options_from_parsed_header_ensures(
    before: &LinkProcessingOptions,
    after: &LinkProcessingOptions,
    link_object: &LinkHeader,
    rel: Seq<char>,
    result: bool,
) -> bool {
    SpecAlgorithms::options::apply_link_options_from_parsed_header_ensures(
        SpecViews::view(before),
        SpecViews::view(after),
        SpecLinkHeader::valued_attributes(link_object.params@),
        rel,
        result,
    )
}

/// Per-type behavior is defined in algorithms/link_types. The observed plan is
/// an explicit execution boundary until it is obtained from ghost instrumentation.
// https://html.spec.whatwg.org/multipage/semantics.html#process-a-link-header
pub(crate) open spec fn process_link_header_ok(
    options: &LinkProcessingOptions,
    rel: Seq<char>,
    document: &Document,
) -> bool {
    // All link types that can be external resource links define a process a link header algorithm, which takes a link processing options. This algorithm defines whether and how they react to appearing in an HTTP `Link` response header.
    SpecAlgorithms::link_types::process_link_header_ensures(
        SpecViews::StandardModel,
        SpecViews::view_in_document(options, link_headers_document_state(document).document),
        rel,
        observed_process_link_header(options, rel, document),
    )
}

pub(crate) uninterp spec fn observed_process_link_header(
    options: &LinkProcessingOptions,
    rel: Seq<char>,
    document: &Document,
) -> SpecModel::LinkHeaderPlan;

/// <https://html.spec.whatwg.org/multipage/#translate-a-preload-destination>
pub(crate) open spec fn translate_a_preload_destination_result(
    potential_destination: Seq<char>,
) -> Option<Destination> {
    match SpecAlgorithms::link_types::preload::translate_a_preload_destination(potential_destination) {
        Some(destination) => Some(SpecViews::destination_from_view(destination)),
        None => None,
    }
}

/// <https://html.spec.whatwg.org/multipage/#create-a-link-request>
/// Reference postcondition through RequestView. The native request-view and URL
/// primitive correspondence must be established before attaching this to a trusted body.
pub(crate) open spec fn create_link_request_ensures(
    options: &LinkProcessingOptions,
    webview_id: WebViewId,
    global: &GlobalScope,
    result: &Option<RequestBuilder>,
) -> bool {
    SpecAlgorithms::request::create_link_request_ensures(
        SpecViews::StandardModel, SpecViews::view(options), SpecViews::option_request_view(result),
    )
}

/// Callable native boundary while the URL/RequestBuilder representation proof is pending.
pub(crate) uninterp spec fn create_link_request_execution_ensures(
    options: &LinkProcessingOptions,
    webview_id: WebViewId,
    global: &GlobalScope,
    result: &Option<RequestBuilder>,
) -> bool;

/// <https://html.spec.whatwg.org/multipage/#preload>
/// Standard preload postcondition over an explicit observed operation plan.
pub(crate) open spec fn preload_ensures(
    options: &LinkProcessingOptions,
    webview_id: WebViewId,
    link: &Option<Trusted<HTMLLinkElement>>,
    document: &Document,
    observed: SpecModel::PreloadPlan,
) -> bool {
    SpecAlgorithms::link_types::preload::preload_ensures(
        SpecViews::StandardModel,
        SpecViews::view_in_document(options, link_headers_document_state(document).document),
        link_response_callback(link),
        observed,
    )
}

pub(crate) uninterp spec fn element_response_callback(link: &Trusted<HTMLLinkElement>) -> int;

pub(crate) open spec fn link_response_callback(link: &Option<Trusted<HTMLLinkElement>>) -> Option<int> {
    match link {
        Some(link) => Some(element_response_callback(link)),
        None => None,
    }
}

/// Existing native effect boundary until an actual preload plan/trace is recorded.
pub(crate) uninterp spec fn preload_execution_ensures(
    options: &LinkProcessingOptions,
    webview_id: WebViewId,
    link: &Option<Trusted<HTMLLinkElement>>,
    document: &Document,
) -> bool;

/// Compatibility predicate for Servo's current classifier-side acceptance. This
/// is useful for an agreement proof, but is not the WHATWG preload-type algorithm.
pub(crate) open spec fn destination_matches_media_type(
    destination: Destination,
    mime: &Mime,
    media_type: MediaType,
) -> bool {
    ||| (destination_is_script_like(destination) && media_type == MediaType::JavaScript)
    ||| match destination {
        Destination::Audio | Destination::Video => media_type == MediaType::AudioVideo,
        Destination::Image => media_type == MediaType::Image,
        Destination::Font => media_type == MediaType::Font,
        Destination::Json => media_type == MediaType::Json,
        // Preserve Servo's full-record comparison, including parameters.
        Destination::Style => SpecMime::view(mime) == SpecMime::text_css_identity(),
        Destination::Track => SpecMime::essence_str(mime) == "text/vtt"@,
        _ => false,
    }
}

/// Standard algorithm: processing/algorithms/link_types/preload.rs, steps 1–7.
/// UA support and WHATWG/Rust MIME parser agreement remain primitive-model obligations.
pub(crate) open spec fn type_matches_destination_result(
    options: &LinkProcessingOptions,
) -> bool {
    SpecAlgorithms::link_types::preload::type_matches_destination(
        SpecViews::StandardModel, options.link_type@, SpecViews::destination_view(options.destination),
    )
}

// ----------------------------------------
// Link header processing predicates
// ----------------------------------------

/// Keyword recognition follows HTML's CORS keyword table. Full enumerated-
/// attribute defaulting is a separate algorithm in algorithms/cors.rs.
pub(crate) open spec fn determine_cors_settings_for_token_result(
    token: Seq<char>,
) -> Option<CorsSettings> {
    match SpecAlgorithms::cors::determine_cors_settings_for_token(token) {
        Some(settings) => Some(settings.as_native()),
        None => None,
    }
}

/// Exact behavior of the pinned native ReferrerPolicy conversion; invalid tokens
/// become EmptyString. The standard's apply-attributes algorithm has a validity
/// guard and preserves the old policy for invalid tokens.
pub open spec fn native_referrer_policy_result(token: Seq<char>) -> ReferrerPolicy {
    let token = ascii_lowercase(token);
    if token == "no-referrer"@ { ReferrerPolicy::NoReferrer }
    else if token == "no-referrer-when-downgrade"@ { ReferrerPolicy::NoReferrerWhenDowngrade }
    else if token == "origin"@ { ReferrerPolicy::Origin }
    else if token == "same-origin"@ { ReferrerPolicy::SameOrigin }
    else if token == "origin-when-cross-origin"@ { ReferrerPolicy::OriginWhenCrossOrigin }
    else if token == "unsafe-url"@ { ReferrerPolicy::UnsafeUrl }
    else if token == "strict-origin"@ { ReferrerPolicy::StrictOrigin }
    else if token == "strict-origin-when-cross-origin"@ { ReferrerPolicy::StrictOriginWhenCrossOrigin }
    else { ReferrerPolicy::EmptyString }
}

/// TODO: Establish the representation bridge to the standard's extracted links,
/// including relation splitting, parameter normalization, and attribute lookup.
/// No correspondence with nom-rfc8288 is assumed by this declaration.
pub(crate) uninterp spec fn parsed_link_headers_view(
    link_headers: Seq<LinkHeader>,
) -> Seq<SpecModel::ParsedLinkHeader>;

/// TODO: Connect a document-state snapshot to the abstract context/object identities.
pub(crate) uninterp spec fn link_headers_document_state(
    document: &Document,
) -> SpecModel::LinkHeaderDocumentState;

/// WHATWG process-link-headers relation over an explicitly supplied execution trace.
/// <https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers>
/// This checks the caller algorithm. Its referenced algorithms keep their separate
/// contracts; capturing an actual trace and proving the input views are follow-ups.
pub(crate) open spec fn process_link_headers_ensures(
    link_headers: &[LinkHeader],
    document: &Document,
    phase: LinkProcessingPhase,
    trace: SpecModel::ProcessLinkHeadersTrace,
) -> bool {
    SpecAlgorithms::headers::process_link_headers_ensures(
        parsed_link_headers_view(link_headers@),
        link_headers_document_state(document),
        match phase {
            LinkProcessingPhase::Media => SpecModel::LinkHeadersPhase::Media,
            LinkProcessingPhase::PreMedia => SpecModel::LinkHeadersPhase::PreMedia,
        },
        trace,
    )
}

/// Existing executable-facing obligation until ghost trace capture is connected.
/// TODO: Replace this hook with process_link_headers_ensures over the recorded trace.
/// Keeping it uninterpreted preserves the outstanding proof obligation.
pub(crate) uninterp spec fn process_link_headers_execution_ensures(
    link_headers: &[LinkHeader],
    document: &Document,
    phase: LinkProcessingPhase,
) -> bool;

// ----------------------------------------
// External Servo API predicates
// ----------------------------------------

// These uninterpreted relations are scaffolds for the assume_specification
// declarations in servo_api. Defining their behavior will strengthen trusted
// contracts and must be justified against the corresponding implementations.

/// TODO: Relate the returned global to the node's current owner document/window.
/// A result relation avoids asserting equality of separately allocated root wrappers.
pub(crate) uninterp spec fn node_owner_global_ensures<T: DerivedFrom<Node> + DomObject>(
    node: &T,
    result: &DomRoot<GlobalScope>,
) -> bool;

/// TODO: Relate the returned window to the document.
pub(crate) uninterp spec fn document_window_ensures<'a>(
    document: &'a Document,
    result: &'a Window,
) -> bool;

/// TODO: Relate the returned identifier to the window's webview.
pub(crate) uninterp spec fn window_webview_id_ensures(
    window: &Window,
    result: WebViewId,
) -> bool;

/// TODO: Model the document's origin and the Ref borrow guard.
pub(crate) uninterp spec fn document_origin_ensures<'a>(
    document: &'a Document,
    result: &Ref<'a, MutableOrigin>,
) -> bool;

/// TODO: Relate the mutable-origin wrapper to its immutable component.
pub uninterp spec fn mutable_origin_immutable_ensures<'a>(
    origin: &'a MutableOrigin,
    result: &'a ImmutableOrigin,
) -> bool;

/// TODO: Model the document's policy container and the Ref borrow guard.
pub(crate) uninterp spec fn document_policy_container_ensures<'a>(
    document: &'a Document,
    result: &Ref<'a, Arc<PolicyContainer>>,
) -> bool;

/// TODO: Define the document base-URL selection algorithm.
pub(crate) uninterp spec fn document_base_url_ensures(
    document: &Document,
    result: &ServoUrl,
) -> bool;

/// TODO: Model the request-client fields for the relevant global environment.
pub(crate) uninterp spec fn global_scope_request_client_ensures(
    global: &GlobalScope,
    no_gc: Option<&NoGC>,
    result: &RequestClient,
) -> bool;

/// TODO: Model the referrer selected from the global environment.
pub(crate) uninterp spec fn global_scope_get_referrer_ensures(
    global: &GlobalScope,
    result: &Referrer,
) -> bool;

/// TODO: Relate the Boolean result to parsing and evaluating the media query.
pub(crate) uninterp spec fn media_list_matches_environment_ensures(
    document: &Document,
    media_query: Seq<char>,
    result: bool,
) -> bool;

/// TODO: Model source-set parsing, normalization, and the document environment.
pub uninterp spec fn source_set_create_source_set_ensures(
    default_source: Seq<char>,
    srcset: Seq<char>,
    sizes: Seq<char>,
    document: &Document,
    result: &SourceSet,
) -> bool;

} // verus!
