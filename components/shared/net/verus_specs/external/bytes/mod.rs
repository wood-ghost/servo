//! Buffer specifications for bytes 1.12.1.

use bytes::BytesMut;
use vstd::prelude::*;

verus! {

#[verifier::external_type_specification] #[verifier::external_body]
pub struct ExBytesMut(BytesMut);

// TRUSTED(external-lib): bytes::BytesMut::new.
// TODO: ensures the buffer is initially empty.
pub assume_specification[BytesMut::new]() -> BytesMut;

}
