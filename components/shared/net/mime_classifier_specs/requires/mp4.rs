use vstd::prelude::*;

verus! {

/// Upstream's MP4 matcher indexes `data[16..box_size]` after its early exits.
/// Exclude precisely the inputs that reach that expression with a reversed range.
/// This is an input restriction, not a property of arbitrary resource headers.
pub open spec fn mp4_matches_requires(data: Seq<u8>) -> bool {
    if data.len() < 12 {
        true
    } else {
        let box_size = (data[0] as int) * 0x1000000
            + (data[1] as int) * 0x10000
            + (data[2] as int) * 0x100
            + data[3] as int;
        (box_size <= data.len()
            && box_size % 4 == 0
            && data[4] == 0x66 && data[5] == 0x74 && data[6] == 0x79 && data[7] == 0x70
            && !(data[8] == 0x6D && data[9] == 0x70 && data[10] == 0x34))
            ==> 16 <= box_size
    }
}

} // verus!
