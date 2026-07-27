use vstd::prelude::*;

verus! {
pub struct MyZip2<V, U> {
    left: V,
    right: U,
}

pub trait Iterator: Sized {
    type Item;

    fn zip<U>(self, other: U) -> MyZip2<Self, U>;
}

impl<'a, T> Iterator for core::slice::Iter<'a, T> {
    type Item = &'a T;

    fn zip<U>(self, other: U) -> MyZip2<Self, U> {
        MyZip2 {
            left: self,
            right: other,
        }
    }
}








} // verus!