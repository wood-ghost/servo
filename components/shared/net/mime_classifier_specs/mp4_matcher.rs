use mime::Mime;
use vstd::prelude::*;
use crate::mime_classifier::{
    Mp4Matcher,
};
use crate::mime_classifier_specs::classifier::MIMECheckerSpec;
use crate::mime_classifier_specs::mime_api::{
    video_mp4_identity,
    MimeView,
};

pub use crate::mime_classifier_specs::predicates::signature::{
    matches_mp4_signature,
    has_mp4_at,
    has_mp4_prefix,
};

verus! {

impl MIMECheckerSpec for Mp4Matcher {
    open spec fn classify_spec(&self, data: Seq<u8>) -> Option<MimeView> {
        if matches_mp4_signature(data) {
            Some(video_mp4_identity())
        } else {
            None
        }
    }

    open spec fn validate_spec(&self) -> bool {
        true
    }
}

pub(crate) proof fn lemma_step4_u32_eq_int(box_size: u32, data: Seq<u8>) 
    requires
        data.len() >= 4,
        box_size == ((data[0] as u32) << 24)
            | ((data[1] as u32) << 16)
            | ((data[2] as u32) << 8)
            | (data[3] as u32),
    ensures
        box_size as int == (data[0] as int) * 0x1000000
            + (data[1] as int) * 0x10000
            + (data[2] as int) * 0x100
            + data[3] as int,
{
    let b0 = data[0];
    let b1 = data[1];
    let b2 = data[2];
    let b3 = data[3];

    assert(
        (((b0 as u32) << 24)
            | ((b1 as u32) << 16)
            | ((b2 as u32) << 8)
            | (b3 as u32)) as int
        == (b0 as int) * 0x1000000
            + (b1 as int) * 0x10000
            + (b2 as int) * 0x100
            + b3 as int
    ) by (bit_vector);
}

pub broadcast proof fn lemma_chunks_mp4_equivalence(
    data: Seq<u8>,
    box_size: int,
    chunks: Seq<&[u8]>,
)
    requires
        16 <= box_size <= data.len(),
        box_size % 4 == 0,
        chunks.len() == (data.subrange(16, box_size).len() + 3) / 4,

        forall |i: int|
            #![trigger chunks[i]]
            0 <= i < chunks.len() ==> {
                let source = data.subrange(16, box_size);
                let start = 4 * i;
                let end =
                    if start + 4 <= source.len() {
                        start + 4
                    } else {
                        source.len() as int
                    };

                chunks[i]@ == source.subrange(start, end)
            },
    ensures
        #![trigger chunks.len(), data.subrange(16, box_size)] 
        (exists |i: int|
            0 <= i < chunks.len()
            && #[trigger] has_mp4_prefix(chunks[i]@))
        ==
        (exists |bytes_read: int|
            16 <= bytes_read < box_size
            && bytes_read % 4 == 0
            && #[trigger] has_mp4_at(data, bytes_read)),
{
    let source = data.subrange(16, box_size);

    // chunck idx matches bytes_read
    assert(
        (exists |i: int| 0 <= i < chunks.len()
            && #[trigger] has_mp4_prefix(chunks[i]@))
        ==
        (exists |i: int| 0 <= i && 16 + 4 * i < box_size
            && #[trigger] has_mp4_at(data, 16 + 4 * i))
    ) by {
        if exists |i: int| 0 <= i < chunks.len()
            && #[trigger] has_mp4_prefix(chunks[i]@)
        {
            let i = choose |i: int| 0 <= i < chunks.len()
                && #[trigger] has_mp4_prefix(chunks[i]@);

            assert(has_mp4_at(data, 16 + 4 * i));
        }

        if exists |i: int| 0 <= i && 16 + 4 * i < box_size
            && #[trigger] has_mp4_at(data, 16 + 4 * i)
        {
            let i = choose |i: int| 0 <= i && 16 + 4 * i < box_size
                && #[trigger] has_mp4_at(data, 16 + 4 * i);

            assert(has_mp4_prefix(chunks[i]@));
        }
    }

    // byte_read implies chunk idx
    if exists |bytes_read: int|
        16 <= bytes_read < box_size
            && bytes_read % 4 == 0
            && #[trigger] has_mp4_at(data, bytes_read)
    {
        let bytes_read = choose |bytes_read: int|
            16 <= bytes_read < box_size
                && bytes_read % 4 == 0
                && #[trigger] has_mp4_at(data, bytes_read);

        let chunk_index = (bytes_read - 16) / 4;

        assert(bytes_read == 16 + 4 * chunk_index);
    }

}

} // verus!
