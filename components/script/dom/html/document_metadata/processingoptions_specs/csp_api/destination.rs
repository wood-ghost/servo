/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// assume_specification generates unsafe scaffolding in this verifier-only module.
#![allow(unsafe_code)]
#![verifier::auto_reveal_strlit]

use content_security_policy::Destination;
use vstd::prelude::*;

verus! {

// Destination is transparent so the type-matching body can name its variants.
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExDestination(Destination);

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

/// <https://fetch.spec.whatwg.org/#concept-potential-destination-translate>
pub(crate) open spec fn translate_a_potential_destination(value: Seq<char>) -> Seq<char>
    // 2. Assert: potentialDestination is a destination.
    recommends value == "fetch"@ || (value != ""@ && destination_types().contains(value)),
{
    if value == "fetch"@ {
        // 1. If potentialDestination is "fetch", then return the empty string.
        Seq::empty()
    } else {
        // 3. Return potentialDestination.
        value
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

// https://fetch.spec.whatwg.org/#request-destination-script-like
// A request’s destination is script-like if it is "audioworklet", "paintworklet", "script", "serviceworker", "sharedworker", or "worker".
// Algorithms that use script-like should also consider "xslt" as that too can cause script execution. It is not included in the list as it is not always relevant and might require different behavior.
// https://docs.rs/content-security-policy/latest/src/content_security_policy/lib.rs.html#1025-1031
pub open spec fn destination_is_script_like(destination: Destination) -> bool {
    matches!(
        destination,
        Destination::AudioWorklet
            | Destination::PaintWorklet
            | Destination::Script
            | Destination::ServiceWorker
            | Destination::SharedWorker
            | Destination::Worker
            | Destination::Xslt
    )
}

// TODO: Remove this local assumption when a verified or shared contract for
// Destination::is_script_like is supplied by its owning dependency or net_traits.
// Verifying net_traits alone does not verify this content-security-policy method.
// Matches the implementation in content-security-policy 0.9.0, src/lib.rs:1025-1031.
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::when_used_as_spec(destination_is_script_like)]
pub assume_specification[Destination::is_script_like](
    destination: Destination,
) -> (result: bool)
    ensures
        result == destination_is_script_like(destination),
;

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
pub assume_specification[ <Destination as PartialEq<Destination>>::eq ](
    left: &Destination,
    right: &Destination,
) -> (result: bool)
    ensures
        result == (*left == *right),
;

} // verus!
