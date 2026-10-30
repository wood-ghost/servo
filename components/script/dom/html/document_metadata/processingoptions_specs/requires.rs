use mime::{FromStrError, Mime};
use net_traits::mime_classifier_specs::{mime_api::FromStrSpec, requires as MimeRequires};
use net_traits::request::Destination;
use vstd::prelude::*;

use super::super::processingoptions::LinkProcessingOptions;
use super::processing::model_trait::LinkProcessingModel;
use super::servo_api::views::{StandardModel, destination_name};
use super::predicates::is_preload_destination;
use net_traits::mime_classifier::MediaType;
use net_traits::mime_classifier_specs::{classifier as SpecClassifier, mime_api as SpecMime};

verus! {

/// Step 1 of create_link_request asserts that href is nonempty.
pub(crate) open spec fn create_link_request_requires(options: &LinkProcessingOptions) -> bool {
    options.href@.len() > 0
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
                        // IMPLICIT: classifier precedence may disagree with MIME categories; witness: Image with "image/svg+xml" is classified as XML first.
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
