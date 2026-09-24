//! Proved sequence bridges for the iterator contracts available in servo-stable.
use vstd::prelude::*;

verus! {

pub broadcast proof fn lemma_as_ref_index<A>(s: Seq<A>, i: int)
    requires 0 <= i < s.len(),
    ensures
        #![trigger s.as_ref()[i]]
        #![trigger s.as_ref(), s[i]]
        *s.as_ref()[i] == s[i],
{}

pub broadcast proof fn lemma_as_ref_len<A>(s: Seq<A>)
    ensures (#[trigger] s.as_ref().len()) == s.len(),
{}

pub broadcast proof fn lemma_zip_component_index<A, B>(left: Seq<A>, right: Seq<B>, i: int)
    requires 0 <= i < left.len(), i < right.len(),
    ensures
        #![trigger left.zip_truncate(right), left[i]]
        left.zip_truncate(right)[i] == (left[i], right[i]),
{}

pub broadcast group iterator_bridges {
    lemma_as_ref_index,
    lemma_as_ref_len,
    lemma_zip_component_index,
}

} // verus!
