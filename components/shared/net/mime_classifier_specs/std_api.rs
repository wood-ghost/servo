use vstd::prelude::*;

verus! {

#[verifier::allow(undeclared_external_trait)]
pub assume_specification<T, F>
    [Option::<T>::or_else]
    (option: Option<T>, op: F) -> (result: Option<T>)
where
    F: FnOnce() -> Option<T> + std::marker::Destruct,
    T: std::marker::Destruct,
    requires
        option.is_some() || call_requires(op, ()),
    ensures
        match option {
            Some(value) => result == Some(value),
            None => call_ensures(op, (), result),
        },
;

pub assume_specification<T>
    [<[T]>::contains]
    (slice: &[T], value: &T) -> bool
where
    T: std::cmp::PartialEq,
;

pub assume_specification<'a, T, P>
    [<std::slice::Iter<'a, T> as std::iter::Iterator>::position]
    (
        iter: &mut std::slice::Iter<'a, T>,
        predicate: P,
    ) -> Option<usize>
where
    P: FnMut(
        <std::slice::Iter<'a, T> as std::iter::Iterator>::Item,
    ) -> bool,
    std::slice::Iter<'a, T>: Sized,
;

#[verifier::reject_recursive_types(A)]
#[verifier::reject_recursive_types(B)]
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExZip<A, B>(core::iter::Zip<A, B>);

#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExChunks<'a, T: 'a>(std::slice::Chunks<'a, T>);

pub assume_specification<'a, T>
    [<[T]>::chunks]
    (
        slice: &'a [T],
        chunk_size: usize,
    ) -> (result: std::slice::Chunks<'a, T>)
    requires
        chunk_size > 0,
;

pub assume_specification<T>
    [<[T]>::starts_with]
    (
        slice: &[T],
        prefix: &[T],
    ) -> (result: bool)
where
    T: std::cmp::PartialEq,
;

#[verifier::allow(undeclared_external_trait)]
pub assume_specification<T, F>
    [Option::<T>::is_some_and]
    (
        option: Option<T>,
        predicate: F,
    ) -> (result: bool)
where
    F: FnOnce(T) -> bool + std::marker::Destruct,
    requires
        match option {
            Some(value) => call_requires(predicate, (value,)),
            None => true,
        },
    ensures
        match option {
            Some(value) => call_ensures(predicate, (value,), result),
            None => !result,
        },
;

pub assume_specification<'a, T, P>
    [<std::slice::Iter<'a, T> as std::iter::Iterator>::position]
    (
        iter: &mut std::slice::Iter<'a, T>,
        predicate: P,
    ) -> Option<usize>
where
    P: FnMut(
        <std::slice::Iter<'a, T> as std::iter::Iterator>::Item,
    ) -> bool,
    std::slice::Iter<'a, T>: Sized,
;

pub assume_specification<'a, 'b, T>
    [<core::slice::Iter<'a, T> as core::iter::Iterator>::zip]
    (
        iter: core::slice::Iter<'a, T>, 
        other: core::slice::Iter<'b, T>,
    ) -> ( 
        result: core::iter::Zip<core::slice::Iter<'a, T>, core::slice::Iter<'b, T>>
    )
;


} // verus!