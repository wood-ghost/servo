use vstd::prelude::*;

use crate::mime_classifier::ByteMatcher;
use crate::mime_classifier_specs::mime_api::*;

verus! {

// https://mimesniff.spec.whatwg.org/#signature-for-webm
// FIXME: https://github.com/whatwg/mimesniff/issues/93
pub open spec fn webm_scan(sequence: Seq<u8>, iter: nat) -> bool {
    webm_scan_fuel(sequence, iter, 38)
}

pub open spec fn webm_scan_fuel(sequence: Seq<u8>, iter: nat, fuel: nat,) -> bool
    decreases fuel
{
    // 6. While iter is less than length and iter is less than 38, continuously 
    //    loop through these steps: 
    if fuel == 0 || iter >= sequence.len() || iter >= 38 {
        // 7. Return false. 
        false
    } else if
        // 6.1 If the two bytes from sequence[iter] to sequence[iter + 1] are equal to 0x42 0x82
        iter + 1 < sequence.len()
        && sequence[iter as int] == 0x42u8
        && sequence[(iter + 1) as int] == 0x82u8
    {
        // 6.1.1 Increment iter by 2. 
        let iter1 = iter + 2;
        // 6.1.2 If iter is greater or equal than length, abort these steps. 
        if iter1 >= sequence.len() {
            // 6.2 Increment iter by 1. 
            webm_scan_fuel(sequence, iter1 + 1, (fuel - 1) as nat)
        } else {
            // 6.1.3 Let number size be the result of parsing a vint starting at sequence[iter]. 
            let number_size = parse_vint_number_size(sequence, iter1 as int);
            // 6.1.4 Increment iter by number size. 
            let iter2 = iter1 + number_size;
            // 6.1.5 If iter is greater than or equal to length - 4, abort these steps. 
            if iter2 >= sequence.len() - 4 {
                // 6.2 Increment iter by 1. 
                webm_scan_fuel(sequence, iter2 + 1, (fuel - 1) as nat)
            // 6.1.6 Let matched be the result of matching a padded sequence 
            //       0x77 0x65 0x62 0x6D ("webm") on sequence at offset iter. 
            } else if padded_webm_match(sequence, iter2 as int, sequence.len() as int - 1) {
                // 6.1.7 If matched is true, abort these steps and return true.  
                true
            } else {
                // Step 6.2
                webm_scan_fuel(sequence, iter2 + 1, (fuel - 1) as nat)
            }
        }
    } else {
        // 6.2 Increment iter by 1. 
        webm_scan_fuel(sequence, iter + 1, (fuel - 1) as nat)
    }
}

// https://mimesniff.spec.whatwg.org/#matching-a-padded-sequence
pub open spec fn padded_webm_match(
    sequence: Seq<u8>,
    offset: int,
    end: int,
) -> bool {
    // Matching a padded sequence pattern on a sequence sequence at starting at byte offset 
    // and ending at by end means returning true 
    &&& 0 <= offset <= end
    // if sequence has a length greater than end, 
    &&& end < sequence.len()

    // and contains exactly, in the range [offset, end], the bytes in pattern, in the same order, 
    &&& exists |start: int| #![trigger sequence[start]]
        offset <= start <= end
        // eventually preceded by bytes with a value of 0x00, false otherwise. 
        && (forall |i: int|
            offset <= i < start ==>
                #[trigger] sequence[i] == 0x00u8)
        // "webm"
        && sequence[start]     == 0x77u8
        && sequence[start + 1] == 0x65u8
        && sequence[start + 2] == 0x62u8
        && sequence[start + 3] == 0x6Du8
}

// https://mimesniff.spec.whatwg.org/#parse-a-vint
// pub(crate) uninterp spec fn parse_vint(sequence: Seq<u8>, iter: int) -> Option<(nat, nat)>; 
pub uninterp spec fn parse_vint_number_size(sequence: Seq<u8>, iter: int) -> nat; // incorrect output type

pub(crate) open spec fn is_video_webm(bm: &ByteMatcher) -> bool {
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "video/webm"@
    // // 3. If length is less than 4, return false. 
    // &&& bm.pattern@.len() >= 4 
    // // 4. If the four bytes from sequence[0] to sequence[3], are not equal to 
    // //    0x1A 0x45 0xDF 0xA3, return false. 
    // &&& bm.pattern@[0] == 0x1Au8
    // &&& bm.pattern@[1] == 0x45u8
    // &&& bm.pattern@[2] == 0xDFu8
    // &&& bm.pattern@[3] == 0xA3u8
    // // 5. Let iter be 4. webm_scan
    // &&& (bm.pattern@.len() > 4) ==> webm_scan(bm.pattern@, 4)
    &&& matches_webm_signature(bm.pattern@)
    // only for current servo implementation
    &&& (bm.pattern@.len() == 4) ==> bm.mask@ == b"\xFF\xFF\xFF\xFF"@
}

pub open spec fn matches_webm_signature(data: Seq<u8>) -> bool {
    // 3. If length is less than 4, return false. 
    // 4. If the four bytes from sequence[0] to sequence[3], are not equal to 
    //    0x1A 0x45 0xDF 0xA3, return false. 
    &&& data[0] == 0x1Au8
    &&& data[1] == 0x45u8
    &&& data[2] == 0xDFu8
    &&& data[3] == 0xA3u8
    // 5. Let iter be 4. webm_scan
    &&& (data.len() > 4) ==> webm_scan(data, 4)
}

// https://mimesniff.spec.whatwg.org/#signature-for-mp3-without-id3
pub(crate) open spec fn matches_mp3_without_id3_signature(data: Seq<u8>) -> bool {
    // 4. If the result of the operation match mp3 header is false, return false. 
    let s = 0;
    if !match_mp3_header(data, s) {
        false
    } else {
        // 5. Parse an mp3 frame on sequence at offset s 
        let (version, bitrate, samplerate, pad) = parse_mp3_frame(data, s);
        // 6. Let skipped-bytes the return value of the execution of mp3 framesize computation
        let skipped_bytes = mp3_framesize_computation(version, bitrate, samplerate, pad);
        // 7. If skipped-bytes is less than 4, or skipped-bytes is greater than s - length, return false. 
        if skipped_bytes < 4 || skipped_bytes > s - data.len() {
            false
        } else {
            // 8. Increment s by skipped-bytes. 
            // 9. If the result of the operation match mp3 header operation is false, return false, else, return true. 
            if !match_mp3_header(data, s + skipped_bytes) {
                false
            } else {
                true
            }
        }
    }
}

// https://mimesniff.spec.whatwg.org/#match-an-mp3-header
pub(crate) open spec fn match_mp3_header(data: Seq<u8>, s: nat) -> bool {
    // 1. If length is less than 4, return false. 
    if data.len() < 4 {
        false
    // 2. If sequence[s] is not equal to 0xff and sequence[s + 1] & 0xe0 is not equal to 0xe0, return false. 
    } else if data[s as int] != 0xFFu8 && (data[s as int + 1] & 0xE0u8) != 0xE0u8 {
        false
    } else {
        // 3. Let layer be the result of sequence[s + 1] & 0x06 >> 1. 
        let layer = (data[s as int + 1] & 0x06) >> 1;
        // 4. If layer is 0, return false. 
        if layer == 0 {
            false
        } else {
            // 5. Let bit-rate be sequence[s + 2] & 0xf0 >> 4. 
            let bit_rate = (data[s as int + 2] & 0xF0) >> 4;
            // 6. If bit-rate is 15, return false. 
            if bit_rate == 15 {
                false
            } else {
                // 7. Let sample-rate be sequence[s + 2] & 0x0c >> 2. 
                let sample_rate = (data[s as int + 2] & 0x0C) >> 2;
                // 8. If sample-rate is 3, return false. 
                if sample_rate == 3 {
                    false
                } else {
                    // 9. Let freq be the value given by sample-rate in the table sample-rate. 
                    let freq: nat = sample_rate_table(sample_rate); 
                    // 10. Let final-layer be the result of 4 - ((sequence[s + 1] & 0x06) >> 1). 
                    let final_layer = 4 - ((data[s as int + 1] & 0x06) >> 1);
                    // 11. If final-layer is not 3, return false. 
                    if final_layer != 3 {
                        false
                    } else {
                        // 12. Return true. 
                        true
                    }
                }
            }
        }
    }
}

// https://mimesniff.spec.whatwg.org/#parse-an-mp3-frame
pub(crate) open spec fn parse_mp3_frame(data: Seq<u8>, s: nat) -> (u8, nat, nat, u8)
    recommends
        s + 3 <= data.len(),
        ((data[s as int + 2] & 0xF0u8) >> 4) < 15,
        ((data[s as int + 2] & 0x0Cu8) >> 2) < 3,
{
    // 1. Let version be sequence[s + 1] & 0x18 >> 3.
    let version = (data[s as int + 1] & 0x18) >> 3;
    // 2. Let bitrate-index be sequence[s + 2] & 0xf0 >> 4.
    let bitrate_index = (data[s as int + 2] & 0xF0) >> 4;
    // 3. If the version & 0x01 is non-zero, let bitrate be the value given by bitrate-index in the table mp2.5-rates
    let bitrate = if (version & 0x01) != 0 {
        mp2_5_rates(bitrate_index)
    } else {
        // 4. If version & 0x01 is zero, let bitrate be the value given by bitrate-index in the table mp3-rates
        mp3_rates(bitrate_index) 
    };
    // 5. Let samplerate-index be sequence[s + 2] & 0x0c >> 2.
    let samplerate_index = (data[s as int + 2] & 0x0C) >> 2;
    // 6. Let samplerate be the value given by samplerate-index in the sample-rate table.
    let samplerate = sample_rate_table(samplerate_index); //TODO:

    // 7. If version is 2, let samplerate be the result of samplerate / 2.
    let samplerate = if version == 2 {
        samplerate / 2
    } else {
        samplerate
    };
    // 8. If version is 0, let samplerate be the result of samplerate / 4.
    let samplerate = if version == 0 {
        samplerate / 4
    } else {
        samplerate
    };
    // 9. Let pad be sequence[s + 2] & 0x02 >> 1.
    let pad = (data[s as int + 2] & 0x02) >> 1;
    (version, bitrate, samplerate, pad)
}

// https://mimesniff.spec.whatwg.org/#compute-an-mp3-frame-size
pub(crate) open spec fn mp3_framesize_computation(version: u8, bitrate: nat, samplerate: nat, pad: u8) -> nat
    recommends
        samplerate > 0,
{
    // 1. If version is 1, let scale be 72, else, let scale be 144.
    let scale: nat = if version == 1 { 72 } else { 144 };
    // 2. Let size be bitrate * scale / freq.
    let size = bitrate * scale / samplerate;
    // 3. If pad is not zero, increment size by 1.
    if pad != 0 {
        size + 1
    } else {
        // 4. Return size. 
        size
    }
}

// mp2.5-rates table
// | index | mp2.5-rates |
// |-------|-------------|
// | 0     | 0           |
// | 1     | 8000        |
// | 2     | 16000       |
// | 3     | 24000       |
// | 4     | 32000       |
// | 5     | 40000       |
// | 6     | 48000       |
// | 7     | 56000       |
// | 8     | 64000       |
// | 9     | 80000       |
// | 10    | 96000       |
// | 11    | 112000      |
// | 12    | 128000      |
// | 13    | 144000      |
// | 14    | 160000      |
pub(crate) open spec fn mp2_5_rates(index: u8) -> nat
    recommends
        index < 15,
{
    seq![
        0nat, 8000, 16000, 24000, 32000, 40000, 48000, 56000,
        64000, 80000, 96000, 112000, 128000, 144000, 160000,
    ][index as int]
}

// mp3-rates table
// | index | mp3-rates |
// |-------|-----------|
// | 0     | 0         |
// | 1     | 32000     |
// | 2     | 40000     |
// | 3     | 48000     |
// | 4     | 56000     |
// | 5     | 64000     |
// | 6     | 80000     |
// | 7     | 96000     |
// | 8     | 112000    |
// | 9     | 128000    |
// | 10    | 160000    |
// | 11    | 192000    |
// | 12    | 224000    |
// | 13    | 256000    |
// | 14    | 320000    |
pub(crate) open spec fn mp3_rates(index: u8) -> nat
    recommends
        index < 15,
{
    seq![
        0nat, 32000, 40000, 48000, 56000, 64000, 80000, 96000,
        112000, 128000, 160000, 192000, 224000, 256000, 320000,
    ][index as int]
}

// | index | samplerate |
// |-------|------------|
// | 0     | 44100      |
// | 1     | 48000      |
// | 2     | 32000      |
pub(crate) open spec fn sample_rate_table(index: u8) -> nat
    recommends
        index < 3,
{
    seq![44100nat, 48000, 32000][index as int]
}

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

pub open spec fn has_mp4_at(data: Seq<u8>, idx: int) -> bool {
    data.len() >= idx + 3
    && data[idx] == 0x6D
    && data[idx + 1] == 0x70
    && data[idx + 2] == 0x34
}

pub open spec fn has_mp4_prefix(chunk: Seq<u8>) -> bool {
    has_mp4_at(chunk, 0)
}

} // verus!
