//! Destination specifications for content-security-policy 0.9.0.

use content_security_policy::Destination;
use vstd::prelude::*;

verus! {

// Destination is transparent so the type-matching body can name its variants.
// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExDestination(Destination);

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

}
