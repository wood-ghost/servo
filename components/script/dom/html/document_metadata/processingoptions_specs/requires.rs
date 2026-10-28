use mime::{FromStrError, Mime};
use net_traits::mime_classifier_specs::{mime_api::FromStrSpec, requires as MimeRequires};
use net_traits::request::Destination;
use vstd::prelude::*;

verus! {

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

/// Only preload link headers call the type checker. Inherit its input domain;
/// the successful-match condition is established by the executable guard.
pub(crate) open spec fn process_link_header_requires(
    rel: Seq<char>,
    link_type: Seq<char>,
    destination: Destination,
) -> bool {
    rel == "preload"@ ==> type_matches_destination_requires(link_type, destination)
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
