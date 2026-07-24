use mime::Mime;
use vstd::prelude::*;
verus! {

pub open spec fn validate_ok(pattern: Seq<u8>, mask:Seq<u8>) -> bool {
    &&& pattern.len() != 0
    &&& pattern.len() == mask.len() 
    &&& forall |i: int| #![trigger pattern[i]] 0 <= i < pattern.len() ==> (pattern[i] & mask[i]) == pattern[i] 
}


} // verus!