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
    (slice: &[T], value: &T) -> (result: bool)
where
    T: std::cmp::PartialEq,
    ensures
        result == slice@.to_set().contains(*value),
;

// pub assume_specification<'a, T, P>
//     [<std::slice::Iter<'a, T> as std::iter::Iterator>::position]
//     (
//         iter: &mut std::slice::Iter<'a, T>,
//         predicate: P,
//     ) -> Option<usize>
// where
//     P: FnMut(
//         <std::slice::Iter<'a, T> as std::iter::Iterator>::Item,
//     ) -> bool,
//     std::slice::Iter<'a, T>: Sized,
// ;



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

#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExChunks<'a, T: 'a>(std::slice::Chunks<'a, T>);

// #[verifier::external_body]
// pub fn u8_slice_to_vec(
//     slice: &[u8],
// ) -> (result: Vec<u8>)
//     ensures
//         result@ == slice@,
// {
//     slice.to_vec()
// }

// pub assume_specification<T> [std::str::parse::<T>] (s: &str) -> (result: T)
// where
//     T: std::str::FromStr,
//     ensures
//         result@ == s@,
// ;

} // verus!