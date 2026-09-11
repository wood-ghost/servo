use mime::Mime;
use vstd::prelude::*;

use super::views::*;

verus! {

pub(crate) broadcast proof fn lemma_essence_str(mt: &Mime, expected: &str)
    requires
        // view(mt).suffix is None, // for servo behavior
        expected@.len() == view(mt).type_.len() + 1 + view(mt).subtype.len(),
        forall|i: int| 0 <= i < expected@.len() ==> #[trigger] expected@[i] ==
            if i < view(mt).type_.len() {
                view(mt).type_[i]
            } else if i == view(mt).type_.len() {
                '/'
            } else {
                view(mt).subtype[i - view(mt).type_.len() - 1]
            },
    ensures
        #![trigger essence_str(mt), expected@]
        essence_str(mt) =~= expected@,
{
    assert(essence_str(mt) =~= expected@);
}

pub(crate) broadcast group mime_essence_str_lemmas {
    lemma_essence_str,
}

} // verus!
