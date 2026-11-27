//! Contracts for the Rust 1.100.0 standard library and iterator proof support.

pub mod iter;

use std::cell::Ref;
use vstd::prelude::*;

use crate::verus_specs::external::mime::ascii_lowercase;

verus! {

// TRUSTED(std): Rust 1.99.1 alloc::string::impl_eq! compares the two string slices.
// vstd specifies String == String, but not this String == &str implementation.
// The impl lifetime and the method's elided input lifetimes must match std's signature.
pub assume_specification<'a>[<String as PartialEq<&'a str>>::eq](
    value: &String,
    key: &&str,
) -> (r: bool)
    ensures r == (value@ == (*key)@),
;

// TRUSTED(std): str::eq_ignore_ascii_case compares with ASCII case folding on both sides.
// Non-ASCII characters are unchanged.
pub assume_specification[str::eq_ignore_ascii_case](left: &str, right: &str) -> (result: bool)
    ensures result == (ascii_lowercase(left@) == ascii_lowercase(right@)),
;

// TRUSTED(std): Rust 1.99.1 Option::as_deref is self.as_ref().map(Deref::deref).
// vstd has no as_deref contract. Preserve presence and the wrapped Deref contract;
// vstd's existing String::deref specification then supplies the string contents.
pub assume_specification<T: std::ops::Deref>[Option::<T>::as_deref](
    value: &Option<T>,
) -> (r: Option<&T::Target>)
    ensures
        r is Some <==> value is Some,
        r is Some ==> call_ensures(T::deref, (&value->0,), r->0),
;

// TRUSTED(external-lib): Rust's Ref guard; borrow tracking is not modelled.
#[verifier::external_type_specification] #[verifier::external_body]
#[verifier::reject_recursive_types(T)]
pub struct ExRef<'a, T: ?Sized>(Ref<'a, T>);

// TRUSTED(external-lib): signature of the blanket Clone-based ToOwned implementation.
// TODO: specify the result in terms of Clone's contract.
pub assume_specification<T: Clone>[<T as ToOwned>::to_owned](value: &T) -> T;

}
