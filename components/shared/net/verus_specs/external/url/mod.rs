//! Parse error type specification for url 2.5.8.

use vstd::prelude::*;

verus! {

// Required by ServoUrl's existing FromStr implementation.
// TRUSTED(external-lib): url 2.5.8; validated by not yet
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExUrlParseError(url::ParseError);

}
