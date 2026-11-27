//! Policy type specifications for content-security-policy 0.9.0.

use content_security_policy::{PolicyDisposition, PolicySource};
use vstd::prelude::*;

verus! {

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicyDisposition(PolicyDisposition);

// TRUSTED(external-lib): content-security-policy 0.9.0; validated by not yet
#[verifier::external_type_specification]
pub struct ExPolicySource(PolicySource);

}
