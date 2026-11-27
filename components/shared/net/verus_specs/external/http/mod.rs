//! Type specifications for http 1.5.0.

use http::{HeaderMap, HeaderValue, Method};
use vstd::prelude::*;

verus! {

// TRUSTED(external-lib): opaque http field types and bytes response buffer.
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExHttpMethod(Method);
#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExHeaderValue(HeaderValue);
#[verifier::external_type_specification] #[verifier::external_body]
#[verifier::reject_recursive_types(V)]
pub struct ExHeaderMap<V>(HeaderMap<V>);

}
