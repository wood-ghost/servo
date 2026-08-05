use mime::Mime;
use vstd::prelude::*;
verus! {

pub open spec fn validate_ok(pattern: Seq<u8>, mask:Seq<u8>) -> bool {
    &&& pattern.len() != 0
    &&& pattern.len() == mask.len() 
    &&& forall |i: int| #![trigger pattern[i]] 0 <= i < pattern.len() ==> (pattern[i] & mask[i]) == pattern[i] 
}

// https://mimesniff.spec.whatwg.org/#matching-a-mime-type-pattern
pub open spec fn pattern_matching_at(input: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, ignored: Set<u8>, s: int) -> bool {
    &&& !(input =~= pattern)
    // let s: int = get_first_not_ignored_idx(input, ignored);
    // Assert: pattern’s length is equal to mask’s length. 
    &&& pattern.len() == mask.len()
    // If input’s length is less than pattern’s length, return false.
    &&& input.len() >= pattern.len()
    &&& is_first_not_ignored_idx(input, ignored, s)
    // Let p be 0.
    // While p < pattern’s length:
    //     Let maskedData be the result of applying the bitwise AND operator to input[s] and mask[p].
    //     If maskedData is not equal to pattern[p], return false.
    //     Set s to s + 1.
    //     Set p to p + 1.
    // Return true
    &&& s + pattern.len() <= input.len()
    &&& forall |p: int| #![trigger input[s + p]] 0 <= p < pattern.len() ==> ((input[s+p] & mask[p]) == pattern[p])
}

pub open spec fn pattern_matching_success(input: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, ignored: Set<u8>) -> bool {
   exists |s: int| pattern_matching_at(input, pattern, mask, ignored, s) 
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

pub open spec fn match_result(input: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, ignored: Set<u8>, r: int) -> bool {
    exists |s: int| {
        &&& pattern_matching_at(input, pattern, mask, ignored, s)
        &&& r == s + pattern.len()
    }
}


// proofs
pub proof fn match_return_some(data: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, ignored: Set<u8>, start: int)
    requires
        // pattern.len() > 0,
        pattern.len() == mask.len(),
        // pattern.len() <= data.len(),
        !(data =~= pattern),
        start >= 0,
        start + pattern.len() <= data.len(),

        !ignored.contains(data[start]),

        forall |j: int| #![trigger data[j]] 0 <= j < start  ==> ignored.contains(data[j]),

        forall |p: int| #![trigger pattern.as_ref()[p]] 0 <= p < pattern.len() ==>
            (*data.subrange(start, data.len() as int).as_ref()[p] & *mask.as_ref()[p]) == *pattern.as_ref()[p],
    ensures
        match_result(data, pattern, mask, ignored, start + pattern.len())
{
    assert forall |p: int| #![trigger data[start + p]]
        0 <= p < pattern.len()
        implies
        (data[start + p] & mask[p]) == pattern[p]
    by {
        assert(*pattern.as_ref()[p] == pattern[p]);
    }
    assert(pattern_matching_at(data, pattern, mask, ignored, start));
}

pub proof fn match_return_none(data: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, ignored: Set<u8>, start: int)
    requires
        exists |p: int| #![trigger pattern[p]]
            0 <= p < pattern.len()
            && (data[start + p] & mask[p]) != pattern[p],
    ensures
        !pattern_matching_at(data, pattern, mask, ignored, start)
{}

} // verus!