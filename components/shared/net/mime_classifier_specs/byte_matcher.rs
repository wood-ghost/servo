use mime::Mime;
use vstd::prelude::*;
use crate::mime_classifier::{
    ByteMatcher,
    TagTerminatedByteMatcher,
};
use crate::mime_classifier_specs::classifier::MIMECheckerSpec;
use crate::mime_classifier_specs::mime_api::{
    view,
    MimeView,
};

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

pub open spec fn is_match_result(input: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, ignored: Set<u8>, r: int) -> bool {
    exists |s: int| {
        &&& pattern_matching_at(input, pattern, mask, ignored, s)
        &&& r == s + pattern.len()
    }
}

pub open spec fn position_found(input: Seq<&u8>, ignored: Set<u8>, start: int) -> bool {
    &&& 0 <= start < input.len()
    &&& !ignored.contains(*input[start])
    &&& forall |j: int| #![trigger input[j]] 0 <= j < start ==> ignored.contains(*input[j])
}

pub open spec fn position_not_found(input: Seq<&u8>, ignored: Set<u8>) -> bool {
    forall |j: int| #![trigger ignored.contains(*input[j])]
        0 <= j < input.len() ==> ignored.contains(*input[j])
}

pub broadcast proof fn position_not_found_implies_no_pattern_matching(
    input: Seq<u8>,
    pattern: Seq<u8>,
    mask: Seq<u8>,
    ignored: Set<u8>,
)
    requires
        0 < pattern.len(),
        pattern.len() <= input.len(),
    ensures
        #![trigger pattern_matching_success(input, pattern, mask, ignored)]
        position_not_found(
            input.subrange(0, input.len() - pattern.len() + 1).as_ref(),
            ignored,
        ) ==> !pattern_matching_success(input, pattern, mask, ignored),
{
    let prefix = input.subrange(0, input.len() - pattern.len() + 1);

    if position_not_found(prefix.as_ref(), ignored)
        && pattern_matching_success(input, pattern, mask, ignored)
    {
        let start = choose |start: int|
            pattern_matching_at(input, pattern, mask, ignored, start);
        assert(ignored.contains(*prefix.as_ref()[start]));

        assert(false);
    }
}


// proofs
pub proof fn match_return_some(data: Seq<u8>, pattern: Seq<u8>, mask: Seq<u8>, ignored: Set<u8>, start: int)
    requires
        pattern.len() == mask.len(),
        !(data =~= pattern),
        start >= 0,
        start + pattern.len() <= data.len(),

        !ignored.contains(data[start]),

        forall |j: int| #![trigger data[j]] 0 <= j < start  ==> ignored.contains(data[j]),

        forall |p: int| #![trigger pattern.as_ref()[p]] 0 <= p < pattern.len() ==>
            (*data.subrange(start, data.len() as int).as_ref()[p] & *mask.as_ref()[p]) == *pattern.as_ref()[p],
    ensures
        is_match_result(data, pattern, mask, ignored, start + pattern.len())
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
        start + pattern.len() <= data.len(),
        is_first_not_ignored_idx(data, ignored, start),
        exists |p: int| #![trigger pattern[p]]
            0 <= p < pattern.len()
            && (data[start + p] & mask[p]) != pattern[p],
    ensures
        !pattern_matching_at(data, pattern, mask, ignored, start)
{
    if pattern_matching_success(data, pattern, mask, ignored) {
        let p = choose |p: int| #![trigger pattern[p]]
            0 <= p < pattern.len()
            && (data[start + p] & mask[p]) != pattern[p];
        assert((data[start + p] & mask[p]) != pattern[p]);

        assert(false);
    }
}

// impl
impl MIMECheckerSpec for ByteMatcher {
    open spec fn classify_spec(&self, data: Seq<u8>) -> Option<MimeView> {
        if data == self.pattern@
            || pattern_matching_success(data,
                self.pattern@,
                self.mask@,
                self.leading_ignore@.to_set(),
            )
        {
            Some(view(&self.content_type))
        } else {
            None
        }
    }

    open spec fn validate_spec(&self) -> bool {
        validate_ok(self.pattern@, self.mask@)
    }
}

impl MIMECheckerSpec for TagTerminatedByteMatcher {
    open spec fn classify_spec(&self, data: Seq<u8>) -> Option<MimeView> {
        if exists |r: int| #![trigger data[r]]
            0 <= r < data.len()
                && (data[r] == b' ' || data[r] == b'>')
                && (
                    ((data == self.matcher.pattern@)
                        && r == self.matcher.pattern@.len())
                    || is_match_result(
                        data,
                        self.matcher.pattern@,
                        self.matcher.mask@,
                        self.matcher.leading_ignore@.to_set(),
                        r,
                    )
                )
        {
            Some(view(&self.matcher.content_type))
        } else {
            None
        }
    }

    open spec fn validate_spec(&self) -> bool {
        self.matcher.validate_spec()
    }
}

pub(crate) open spec fn ttbm_match_success(j: int, data: Seq<u8>) -> bool {
    &&& j < data.len()
    &&& (data[j] == b' ' || data[j] == b'>')
}

} // verus!
