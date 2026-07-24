use mime::Mime;
use vstd::prelude::*;
verus! {

pub open spec fn validate_ok(pattern: Seq<u8>, mask:Seq<u8>) -> bool {
    &&& pattern.len() != 0
    &&& pattern.len() == mask.len() 
    &&& forall |i: int| #![trigger pattern[i]] 0 <= i < pattern.len() ==> (pattern[i] & mask[i]) == pattern[i] 
}

// https://mimesniff.spec.whatwg.org/#matching-a-mime-type-pattern
// pub open spec fn pattern_matching_algo(input: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, ignored: Set<u8>) -> bool {
//     let s: int = get_first_not_ignored_idx(input, ignored);
//     // Assert: pattern’s length is equal to mask’s length. 
//     &&& pattern.len() == mask.len()
//     // If input’s length is less than pattern’s length, return false.
//     &&& input.len() >= pattern.len()
//     &&& exists|i: int| is_first_not_ignored_idx(input, ignored, i)
//     // Let p be 0.
//     // While p < pattern’s length:
//     //     Let maskedData be the result of applying the bitwise AND operator to input[s] and mask[p].
//     //     If maskedData is not equal to pattern[p], return false.
//     //     Set s to s + 1.
//     //     Set p to p + 1.
//     // Return true
//     &&& s + pattern.len() <= input.len()
//     &&& forall |p: int| #![trigger input[s + p]] 0 <= p < pattern.len() ==> ((input[s+p] & mask[p]) == pattern[p])
// }
pub open spec fn pattern_matching_algo(
    input: Seq<u8>,
    pattern: Seq<u8>,
    mask: Seq<u8>,
    ignored: Set<u8>,
) -> bool {
    &&& pattern.len() == mask.len()
    &&& input.len() >= pattern.len()

    &&& exists|s: int|
        &&& is_first_not_ignored_idx(
            input,
            ignored,
            s,
        )

        &&& s + pattern.len() <= input.len()

        &&& forall|p: int|
            #![trigger input[s + p]]
            0 <= p < pattern.len()
            ==> (
                input[s + p] & mask[p]
            ) == pattern[p]
}


// Let s be 0. 
// While s < input’s length:
//     If ignored does not contain input[s], break.
//     Set s to s + 1.
pub open spec fn is_first_not_ignored_idx(input: Seq<u8>, ignored: Set<u8>, s: int) -> bool {
    &&& 0 <= s <= input.len()
    &&& forall |i: int| #![trigger input[i]] 0 <= i < s ==> ignored.contains(input[i])
    &&& s < input.len() ==> !ignored.contains(input[s])
}
pub open spec fn get_first_not_ignored_idx(input: Seq<u8>, ignored: Set<u8>) -> int {
    choose |s: int| is_first_not_ignored_idx(input, ignored, s)
}


// pub open spec fn function(input: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, s: int) -> bool {
//     &&& forall |p: int| 0 <= p < pattern.len() ==> (input[s+p] & mask[p] == pattern[p])
// }



} // verus!