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

verus! {

// https://mimesniff.spec.whatwg.org/#matches-the-signature-for-mp4
// To determine whether a byte sequence matches the signature for MP4, use the following steps:
pub open spec fn matches_mp4_signature(seq: Seq<u8>) -> bool {
    // 1. Let sequence be the byte sequence to be matched, where sequence[s] is byte s 
    //   in sequence and sequence[0] is the first byte in sequence.
    // 2. Let length be the number of bytes in sequence.
    // 3. If length is less than 12, return false.
    if seq.len() < 12 {
        false
    } else {
        // 4. Let box-size be the four bytes from sequence[0] to sequence[3], 
        //   interpreted as a 32-bit unsigned big-endian integer.
        // let box_size: int = u32_from_bytes(seq.subrange(0, 4)) as int;
        let box_size: int = (seq[0] as int) * 0x1000000
            + (seq[1] as int) * 0x10000
            + (seq[2] as int) * 0x100
            + seq[3] as int;
        // 5. If length is less than box-size or if box-size modulo 4 is not equal to 0, 
        //   return false. 
        if seq.len() < box_size || (box_size % 4) != 0 {
            false
        }
        // 6. If the four bytes from sequence[4] to sequence[7] are not equal to 
        //   0x66 0x74 0x79 0x70 ("ftyp"), return false. 
        else if seq[4] != 0x66 || seq[5] != 0x74 || seq[6] != 0x79 || seq[7] != 0x70 {
            false
        }
        // 7. If the three bytes from sequence[8] to sequence[10] are equal to 
        //   0x6D 0x70 0x34 ("mp4"), return true. 
        else if seq[8] == 0x6D && seq[9] == 0x70 && seq[10] == 0x34 {
            true
        }
        // 8. Let bytes-read be 16. 
        // 9. While bytes-read is less than box-size, continuously 
        //   loop through these steps: 
        // 9.1 If the three bytes from sequence[bytes-read] to sequence[bytes-read + 2] 
        //     are equal to 0x6D 0x70 0x34 ("mp4"), return true. 
        // 9.2 Increment bytes-read by 4. 
        else {
            exists |bytes_read: int| 16 <= bytes_read < box_size
                && bytes_read % 4 == 0
                && #[trigger] has_mp4_at(seq, bytes_read)
        }
    }
}

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

pub open spec fn has_mp4_at(data: Seq<u8>, idx: int) -> bool {
    data.len() >= idx + 3
    && data[idx] == 0x6D
    && data[idx + 1] == 0x70
    && data[idx + 2] == 0x34
}

pub open spec fn has_mp4_prefix(chunk: Seq<u8>) -> bool {
    has_mp4_at(chunk, 0)
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
