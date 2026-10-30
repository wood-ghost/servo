use mime::{FromStrError, Mime};
use net_traits::mime_classifier_specs::{mime_api::FromStrSpec, requires as MimeRequires};
use net_traits::request::Destination;
use vstd::prelude::*;

use super::super::processingoptions::LinkProcessingOptions;
use super::super::processingoptions::{LinkHeader, LinkProcessingPhase};
use super::processing::model_trait::LinkProcessingModel;
use super::servo_api::views::{StandardModel, destination_name};
use super::predicates::is_preload_destination;
use net_traits::mime_classifier::MediaType;
use net_traits::mime_classifier_specs::{classifier as SpecClassifier, mime_api as SpecMime};

verus! {

pub(crate) open spec fn apply_link_options_requires(options: &LinkProcessingOptions, link_object: &LinkHeader, rel: Seq<char>) -> bool {
    if !super::processing::context_specs::apply_link_options_from_parsed_header_result(link_object, rel) {
        true
    } else {
        let parameters = link_object.params@;
        // IMPLICIT: native invalid crossorigin resets to No CORS instead of preserving the old state; witness: old UseCredentials and crossorigin="bad".
        &&& match super::servo_api::link_header::first_value(parameters, "crossorigin"@) {
            Some(value) => super::processing::algorithms::cors::determine_cors_settings_for_token(value) is None
                ==> options.cross_origin is None,
            None => true,
        }
        // IMPLICIT: native invalid referrerpolicy resets to empty instead of preserving the old state; witness: old Origin and referrerpolicy="bad".
        &&& match super::servo_api::link_header::first_value(parameters, "referrerpolicy"@) {
            Some(value) => super::processing::types::ReferrerPolicyView::from_name(super::predicates::ascii_lowercase(value)) is None
                ==> options.referrer_policy == net_traits::ReferrerPolicy::EmptyString,
            None => true,
        }
        // IMPLICIT: native options omit fetch priority; witness: fetchpriority="high" or "low".
        &&& match super::servo_api::link_header::first_value(parameters, "fetchpriority"@) {
            Some(value) => match super::processing::algorithms::options::fetch_priority_keyword_state(value) {
                Some(priority) => priority == super::processing::types::LinkFetchPriorityView::Auto,
                None => true,
            },
            None => true,
        }
    }
}

/// Step 1 of create_link_request asserts that href is nonempty.
pub(crate) open spec fn create_link_request_requires(options: &LinkProcessingOptions) -> bool {
    options.href@.len() > 0
}

/// The source-selection result, rather than the original href, controls the
/// request assertion on image paths. This is evaluated at preload entry.
pub(crate) open spec fn preload_selection_requires(options: &LinkProcessingOptions, document: &crate::dom::document::Document) -> bool {
    if options.destination == Destination::Image {
        match options.source_set {
            Some(source_set) => {
                // INHERITED from SourceSet::select_image_source_from_source_set.
                &&& super::servo_api::preload::source_selection_precondition(&source_set, document)
                // INHERITED from create_link_request after selected href replaces options.href.
                &&& super::servo_api::preload::selected_source_href(&source_set, document).len() > 0
            },
            None => create_link_request_requires(options),
        }
    } else {
        // INHERITED from create_link_request; no source selection changes href.
        create_link_request_requires(options)
    }
}

/// Inherit requirements only along the calls made by type_matches_destination.
/// Empty types and fetch destinations skip parsing. A parse failure imposes no
/// classifier requirement; every allowed successful result must satisfy the
/// existing classifier requirement, without copying its suffix restrictions.
pub(crate) open spec fn type_matches_destination_requires(
    link_type: Seq<char>,
    destination: Destination,
) -> bool {
    if link_type.len() == 0 || destination == Destination::None {
        true
    } else {
        &&& <Mime as FromStrSpec>::from_str_requires(link_type)
        &&& forall|mt: Mime|
            #[trigger] <Mime as FromStrSpec>::from_str_ensures(link_type, Ok(mt))
                ==> MimeRequires::get_media_type_requires(&mt)
    }
}

/// Per-input implementation/standard correspondence, separate from the inherited
/// parser/classifier safety requirements. These are conditional theorem premises,
/// not a proof of the actual parser or user-agent decoder configuration.
pub(crate) open spec fn type_matches_standard_requires(link_type: Seq<char>, destination: Destination) -> bool {
    if link_type.len() == 0 || destination == Destination::None {
        true
    } else {
        // IMPLICIT: Servo accepts non-preload destination categories; witness: Audio with type "audio/mpeg".
        &&& is_preload_destination(destination_name(destination))
        // IMPLICIT: the native MIME parser needs correspondence to the standard parser; none found (model agreement premise, not a demonstrated runtime defect).
        &&& (forall|parsed: Result<Mime, FromStrError>|
            #[trigger] <Mime as FromStrSpec>::from_str_ensures(link_type, parsed) ==>
                StandardModel.parse_mime_type(link_type) == match parsed {
                    Ok(mime) => Some(mime), Err(_) => None,
                })
        &&& (forall|mime: Mime|
            #[trigger] <Mime as FromStrSpec>::from_str_ensures(link_type, Ok(mime)) ==> {
                &&& (forall|classified: Option<MediaType>|
                    #[trigger] SpecClassifier::get_media_type_spec(&mime, classified) ==> {
                        // IMPLICIT: category recognition replaces actual UA support; witness candidate: "image/x-not-supported" (decoder configuration not validated).
                        &&& StandardModel.mime_supported(mime) == (classified is Some)
                        // IMPLICIT: native category classification needs agreement with the standard category; none found (is_xml explicitly excludes image MIME types, including SVG).
                        &&& (destination == Destination::Script ==>
                            StandardModel.is_javascript_mime(mime) == (classified == Some(MediaType::JavaScript)))
                        &&& (destination == Destination::Image ==>
                            StandardModel.is_image_mime(mime) == (classified == Some(MediaType::Image)))
                        // IMPLICIT: native font compatibility extensions require agreement with the standard font category; none found for this model interface.
                        &&& (destination == Destination::Font ==>
                            StandardModel.is_font_mime(mime) == (classified == Some(MediaType::Font)))
                    })
                // IMPLICIT: Servo compares the full MIME record for CSS instead of its essence; witness: Style with "text/css; charset=utf-8".
                &&& (destination == Destination::Style ==>
                    (SpecMime::view(&mime) == SpecMime::text_css_identity())
                        == (SpecMime::essence_str(&mime) == "text/css"@))
            })
    }
}

/// Only preload link headers call the type checker. Inherit its input domain;
/// the successful-match condition is established by the executable guard.
pub(crate) open spec fn process_link_header_requires(
    rel: Seq<char>,
    link_type: Seq<char>,
    destination: Destination,
) -> bool {
    rel == "preload"@ ==> {
        // INHERITED from type_matches_destination and preload.
        &&& type_matches_destination_requires(link_type, destination)
        // INHERITED from type_matches_destination and preload; witnesses are at the definition.
        &&& type_matches_standard_requires(link_type, destination)
    }
}

/// Sufficient native attribute domain when the loop has just initialized the
/// CORS/referrer fields. This is not a restriction on unrecognized keywords.
pub(crate) open spec fn header_priority_requires(link_object: &LinkHeader) -> bool {
    // INHERITED from apply_link_options_requires; native fetch priority is absent.
    match super::servo_api::link_header::first_value(link_object.params@, "fetchpriority"@) {
        Some(value) => match super::processing::algorithms::options::fetch_priority_keyword_state(value) {
            Some(priority) => priority == super::processing::types::LinkFetchPriorityView::Auto,
            None => true,
        },
        None => true,
    }
}

pub(crate) proof fn header_defaults_discharge_attribute_resets(options: &LinkProcessingOptions, link_object: &LinkHeader, rel: Seq<char>)
    requires
        // These are the fields set by the caller before apply_link_options.
        options.cross_origin is None,
        options.referrer_policy == net_traits::ReferrerPolicy::EmptyString,
        // INHERITED from apply_link_options_requires at the external header boundary.
        header_priority_requires(link_object),
    ensures apply_link_options_requires(options, link_object, rel),
{}

/// Conditions for one candidate header, using only input parameters and the
/// contracts of operations that precede dispatch. No later effect establishes
/// an earlier callee requirement.
pub(crate) open spec fn header_input_requires(link_object: &LinkHeader, document: &crate::dom::document::Document,
    phase: LinkProcessingPhase) -> bool {
    let parameters = link_object.params@;
    let media_phase = super::servo_api::link_header::has_key(parameters, "srcset"@)
        || super::servo_api::link_header::has_key(parameters, "imagesrcset"@)
        || super::servo_api::link_header::has_key(parameters, "media"@);
    let phase_matches = match phase { LinkProcessingPhase::Media => media_phase, LinkProcessingPhase::PreMedia => !media_phase };
    let may_match_media = match super::servo_api::link_header::first_value(parameters, "media"@) {
        None => true,
        Some(query) => super::processing::context_specs::media_list_matches_environment_ensures(document, query, true),
    };
    if !phase_matches || !may_match_media { true } else {
        match super::servo_api::link_header::first_value(parameters, "rel"@) {
            None => true,
            Some(rel) => if !super::processing::context_specs::apply_link_options_from_parsed_header_result(link_object, rel) {
                true
            } else {
                // INHERITED from apply_link_options_requires; default CORS/referrer fields discharge its other clauses.
                &&& header_priority_requires(link_object)
                &&& if rel == "preload"@ {
                    match super::servo_api::link_header::first_value(parameters, "as"@) {
                        None => true,
                        Some(as_) => match super::processing::context_specs::translate_a_preload_destination_result(as_) {
                            None => true,
                            Some(destination) => {
                                let type_ = super::servo_api::link_header::first_value(parameters, "type"@).unwrap_or(Seq::empty());
                                // INHERITED from type_matches_destination before the dispatch guard.
                                &&& type_matches_destination_requires(type_, destination)
                                // INHERITED from type_matches_destination; explicit model/configuration agreement.
                                &&& type_matches_standard_requires(type_, destination)
                                &&& (super::processing::algorithms::link_types::preload::type_matches_destination(
                                    StandardModel, type_, super::processing::types::DestinationView::from_native(destination)) ==> {
                                    if destination == Destination::Image {
                                        match (super::servo_api::link_header::first_value(parameters, "imagesrcset"@),
                                            super::servo_api::link_header::first_value(parameters, "imagesizes"@)) {
                                            (Some(srcset), Some(sizes)) => {
                                                // INHERITED from preload through the preceding SourceSet construction contract.
                                                forall|source_set: crate::dom::srcset::SourceSet|
                                                    #[trigger] super::processing::context_specs::source_set_create_source_set_ensures(
                                                        link_object.url@, srcset, sizes, document, &source_set) ==> {
                                                        &&& super::servo_api::preload::source_selection_precondition(&source_set, document)
                                                        &&& super::servo_api::preload::selected_source_href(&source_set, document).len() > 0
                                                    }
                                            },
                                            // INHERITED from create_link_request when href is not replaced.
                                            _ => link_object.url@.len() > 0,
                                        }
                                    } else {
                                        // INHERITED from create_link_request when href is not replaced.
                                        link_object.url@.len() > 0
                                    }
                                })
                            },
                        },
                    }
                } else { true }
            },
        }
    }
}

/// External HTTP-header/input-model assumption. It covers every phase-matching
/// candidate that may pass media checking, conservatively including headers
/// after a possible earlier return. It is not asserted for arbitrary network data.
pub(crate) open spec fn process_link_headers_requires(link_headers: &[LinkHeader], document: &crate::dom::document::Document,
    phase: LinkProcessingPhase) -> bool {
    // INHERITED from attribute application and preload dispatch for each candidate.
    forall|i: int| 0 <= i < link_headers@.len() ==> #[trigger] header_input_requires(&link_headers@[i], document, phase)
}

pub(crate) proof fn lemma_type_matches_destination_requires_short_circuits(
    link_type: Seq<char>,
    destination: Destination,
)
    ensures
        type_matches_destination_requires(Seq::empty(), destination),
        type_matches_destination_requires(link_type, Destination::None),
{}

/// Bridge a successful parse to the next executable call's requirement.
pub(crate) proof fn lemma_type_matches_destination_requires_for_parsed_mime(
    link_type: Seq<char>,
    destination: Destination,
    mt: Mime,
)
    requires
        type_matches_destination_requires(link_type, destination),
        link_type.len() > 0,
        destination != Destination::None,
        <Mime as FromStrSpec>::from_str_ensures(link_type, Ok(mt)),
    ensures
        <Mime as FromStrSpec>::from_str_requires(link_type),
        MimeRequires::get_media_type_requires(&mt),
{}

pub(crate) proof fn lemma_type_matches_destination_requires_for_parse_failure(
    link_type: Seq<char>,
    destination: Destination,
    error: FromStrError,
)
    requires
        <Mime as FromStrSpec>::from_str_requires(link_type),
        <Mime as FromStrSpec>::from_str_ensures(link_type, Err(error)),
    ensures
        type_matches_destination_requires(link_type, destination),
{}

/// If the inherited requirements become unconditional, this wrapper does too.
/// This is a conditional theorem, not an assumption that the current classifier
/// accepts every MIME input or that its body verifies without its restrictions.
pub(crate) proof fn lemma_type_matches_destination_requires_when_unrestricted(
    link_type: Seq<char>,
    destination: Destination,
)
    requires
        <Mime as FromStrSpec>::from_str_requires(link_type),
        forall|mt: Mime| #[trigger] MimeRequires::get_media_type_requires(&mt),
    ensures
        type_matches_destination_requires(link_type, destination),
{}

} // verus!
