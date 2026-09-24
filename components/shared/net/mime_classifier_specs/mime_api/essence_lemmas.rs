use mime::Mime;
use vstd::prelude::*;

use super::views::*;

verus! {

/// Reconstruct the full essence from the type, base subtype, and optional suffix.
pub(crate) broadcast proof fn lemma_essence_str(mt: &Mime, expected: &str)
    requires
        expected@.len() == view(mt).type_.len() + 1 + view(mt).subtype.len()
            + match view(mt).suffix {
                Some(suffix) => 1 + suffix.len(),
                None => 0,
            },
        forall|i: int| 0 <= i < expected@.len() ==> #[trigger] expected@[i] ==
            if i < view(mt).type_.len() {
                view(mt).type_[i]
            } else if i == view(mt).type_.len() {
                '/'
            } else if i < view(mt).type_.len() + 1 + view(mt).subtype.len() {
                view(mt).subtype[i - view(mt).type_.len() - 1]
            } else if i == view(mt).type_.len() + 1 + view(mt).subtype.len() {
                '+'
            } else {
                view(mt).suffix->Some_0[i - view(mt).type_.len() - 2 - view(mt).subtype.len()]
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
