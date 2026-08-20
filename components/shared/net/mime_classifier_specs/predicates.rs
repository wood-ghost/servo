use mime::Mime;
use vstd::prelude::*;
use vstd::assert_seqs_equal;

use crate::mime_classifier::{
    ByteMatcher,
    TagTerminatedByteMatcher,
};
use super::mime_api::*;

macro_rules! define_mime_essence_lemmas {
    (
        $group_name:ident {
            $(
                $lemma_name:ident => ($type_:literal, $subtype:literal)
            ),* $(,)?
        }
    ) => {
        verus! {
            $(
                pub(crate) broadcast proof fn $lemma_name(mt: &Mime)
                    requires
                        view(mt).type_ =~= ($type_)@,
                        view(mt).subtype =~= ($subtype)@,
                    ensures
                        #[trigger] essence_str(mt) =~= (concat!($type_, "/", $subtype))@,
                {
                    reveal_strlit($type_);
                    reveal_strlit("/");
                    reveal_strlit($subtype);
                    reveal_strlit(concat!($type_, "/", $subtype));

                    assert(
                        (view(mt).type_ + "/"@ + view(mt).subtype) == (concat!($type_, "/", $subtype))@
                    );
                }
            )*

            pub(crate) broadcast group $group_name {
                $(
                    $lemma_name,
                )*
            }
        }
    };
}

macro_rules! define_mime_essence_parts_lemmas {
    (
        $group_name:ident {
            $(
                $lemma_name:ident => ($type_:literal, $subtype:literal, $subtype_start:literal)
            ),* $(,)?
        }
    ) => {
        verus! {
            $(
                pub(crate) broadcast proof fn $lemma_name(mt: &Mime)
                    ensures
                        (#[trigger] essence_str(mt) == (concat!($type_, "/", $subtype))@) ==
                        (view(mt).type_ == ($type_)@ && view(mt).subtype == ($subtype)@),
                {
                    reveal_strlit($type_);
                    reveal_strlit("/");
                    reveal_strlit($subtype);
                    reveal_strlit(concat!($type_, "/", $subtype));

                    let type_ = view(mt).type_;
                    let subtype = view(mt).subtype;

                    assert(($type_)@.len() as int + 1 == $subtype_start as int);

                    // Forward: components imply essence.
                    if type_ == ($type_)@ && subtype == ($subtype)@ {
                        assert(essence_str(mt) == (concat!($type_, "/", $subtype))@);
                    }

                    // Backward: essence implies components.
                    if essence_str(mt) == (concat!($type_, "/", $subtype))@ {
                        assert(essence_str(mt)[type_.len() as int] == '/');

                        assert_seqs_equal!(type_ == ($type_)@, i => {
                                assert(essence_str(mt)[i] == type_[i]);
                            }
                        );

                        assert_seqs_equal!(subtype == ($subtype)@, i => {
                                assert(essence_str(mt)[$subtype_start as int + i] == subtype[i]);
                            }
                        );
                    }
                }
            )*

            pub(crate) broadcast group $group_name {
                $(
                    $lemma_name,
                )*
            }
        }
    };
}

verus! {

pub open spec fn essence_is_text_xml(mt: &Mime) -> bool {
    essence_str(mt) == "text/xml"@
}

pub open spec fn essence_is_application_ogg(mt: &Mime) -> bool {
    // view(mt).essence =~= "application/ogg"@
    essence_str(mt) == "application/ogg"@
}

pub uninterp spec fn has_html_suffix(mt: &Mime) -> bool;
pub open spec fn is_text_plain(mt: &Mime) -> bool {
    essence_str(mt) == "text/plain"@
}
pub open spec fn is_image(mt: &Mime) -> bool {
    view(mt).type_ == image_name()
}
pub open spec fn is_audio(mt: &Mime) -> bool {
    view(mt).type_ == audio_name()
}
pub open spec fn is_video(mt: &Mime) -> bool {
    view(mt).type_ == video_name()
}
pub open spec fn has_xml_suffix(mt: &Mime) -> bool {
    suffix(mt) == Some(xml_name())
}

pub open spec fn is_xml(mt: &Mime) -> bool {
    !is_image(mt) && (
        has_xml_suffix(mt)
            || essence_is_text_xml(mt)
            || (essence_str(mt) == "application/xml"@)
    )
} 

pub open spec fn is_html(mt: &Mime) -> bool {
    essence_str(mt) == "text/html"@
}

pub open spec fn is_audio_video(mt: &Mime) -> bool {
    is_audio(mt) || is_video(mt) || essence_is_application_ogg(mt)
}

pub(crate) broadcast proof fn lemma_image_audio_video_disjoint(mt: &Mime)
    ensures
        #[trigger] is_image(mt) ==> !is_audio_video(mt),
        #[trigger] is_audio_video(mt) ==> !is_image(mt),
{
    reveal_strlit("image");
    reveal_strlit("audio");
    reveal_strlit("video");
    reveal_strlit("/");
    reveal_strlit("application/ogg");

    assert("image"@ != "audio"@) by {
        if "image"@ == "audio"@ {
            assert("image"@[0] == "audio"@[0]);
            assert(false);
        }
    }

    assert("image"@ != "video"@) by {
        if "image"@ == "video"@ {
            assert("image"@[0] == "video"@[0]);
            assert(false);
        }
    }

    if is_image(mt) {
        assert(!essence_is_application_ogg(mt)) by {
            if essence_is_application_ogg(mt) {
                assert(
                    (view(mt).type_ + "/"@ + view(mt).subtype)[0]
                        == "application/ogg"@[0]
                );

                assert(false);
            }
        }

        assert(!is_audio_video(mt));
    }

    if is_audio_video(mt) {
        if is_image(mt) {
            assert(false);
        }
    }
}

/// <https://mimesniff.spec.whatwg.org/#javascript-mime-type>
pub open spec fn is_javascript(mt: &Mime) -> bool {
    ||| essence_str(mt) == "application/ecmascript"@
    ||| essence_str(mt) == "application/javascript"@
    ||| essence_str(mt) == "application/x-ecmascript"@
    ||| essence_str(mt) == "application/x-javascript"@
    ||| essence_str(mt) == "text/ecmascript"@
    ||| essence_str(mt) == "text/javascript"@
    ||| essence_str(mt) == "text/javascript1.0"@
    ||| essence_str(mt) == "text/javascript1.1"@
    ||| essence_str(mt) == "text/javascript1.2"@
    ||| essence_str(mt) == "text/javascript1.3"@
    ||| essence_str(mt) == "text/javascript1.4"@
    ||| essence_str(mt) == "text/javascript1.5"@
    ||| essence_str(mt) == "text/jscript"@
    ||| essence_str(mt) == "text/livescript"@
    ||| essence_str(mt) == "text/x-ecmascript"@
    ||| essence_str(mt) == "text/x-javascript"@
}

/// <https://mimesniff.spec.whatwg.org/#font-mime-type>
pub open spec fn is_font(mt: &Mime) -> bool { //TODO:
    ||| view(mt).type_ == font_name()
    ||| essence_str(mt) == "application/font-cff"@
    ||| essence_str(mt) == "application/font-off"@ //TODO: new version is font-otf
    ||| essence_str(mt) == "application/font-sfnt"@
    ||| essence_str(mt) == "application/font-ttf"@
    ||| essence_str(mt) == "application/font-woff"@
    ||| essence_str(mt) == "application/vnd.ms-fontobject"@
    ||| essence_str(mt) == "application/vnd.ms-opentype"@
}

/// <https://mimesniff.spec.whatwg.org/#json-mime-type>
pub open spec fn is_json(mt: &Mime) -> bool { 
    ||| view(mt).suffix == Some(json_name())@ //TODO: suffix is +.* at the end of subtype
    ||| essence_str(mt) == "application/json"@
    ||| essence_str(mt) == "text/json"@
}

pub open spec fn is_text(mt: &Mime) -> bool { //TODO:
    view(mt) == text_plain_identity()
    || essence_str(mt) == "text/vtt"@
}

pub open spec fn is_css(mt: &Mime) -> bool { //TODO:
    essence_str(mt) == "text/css"@
}

pub open spec fn is_explicit_unknown(mt: &Mime) -> bool {
    ||| essence_str(mt) == "application/unknown"@
    ||| essence_str(mt) == "unknown/unknown"@
    ||| essence_str(mt) == "*/*"@
}

// ----------------------------
// check hard coded byte matchers
// ----------------------------

// Source: WHATWG MIME Sniffing Standard
// https://mimesniff.spec.whatwg.org/#matching-an-image-type-pattern
//
// | Byte Pattern  | Pattern Mask  | Leading Bytes Ignored | Image MIME Type |
// |---------------|---------------|-----------------------|-----------------|
// | 00 00 01 00   | FF FF FF FF   | None                  | image/x-icon    |
pub(crate) open spec fn is_image_x_icon(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x00\x00\x01\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "image/x-icon"@
}

// | 00 00 02 00   | FF FF FF FF    | None                  | image/x-icon    |
pub(crate) open spec fn is_image_x_icon_cursor(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x00\x00\x02\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "image/x-icon"@
}

// | 42 4D         | FF FF          | None                  | image/bmp       |
pub(crate) open spec fn is_image_bmp(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x42\x4D"@
    &&& bm.mask@ == b"\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "image/bmp"@
}

// | 47 49 46 38 37 61   | FF FF FF FF FF FF  | None        | image/gif       |
pub(crate) open spec fn is_image_gif87a(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x47\x49\x46\x38\x37\x61"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "image/gif"@
}

// | 47 49 46 38 39 61   | FF FF FF FF FF FF   | None        | image/gif       |
pub(crate) open spec fn is_image_gif89a(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x47\x49\x46\x38\x39\x61"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "image/gif"@
}

// | 52 49 46 46 00 00 00 00 57 45 42 50 56 50 | FF FF FF FF 00 00 00 00 FF FF FF FF FF FF | None  | image/webp |
pub(crate) open spec fn is_image_webp(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x52\x49\x46\x46\x00\x00\x00\x00\x57\x45\x42\x50\x56\x50"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "image/webp"@
}

// | 89 50 4E 47 0D 0A 1A 0A   | FF FF FF FF FF FF FF FF | None | image/png       |
pub(crate) open spec fn is_image_png(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "image/png"@
}

// | FF D8 FF         | FF FF FF      | None                  | image/jpeg      |
pub(crate) open spec fn is_image_jpeg(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\xFF\xD8\xFF"@
    &&& bm.mask@ == b"\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "image/jpeg"@
}

// https://mimesniff.spec.whatwg.org/#signature-for-webm
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
// FIXME: Iter and index are ambiguous in the specification.  
// pub(crate) uninterp spec fn parse_vint(sequence: Seq<u8>, iter: int) -> Option<(nat, nat)>; 
pub uninterp spec fn parse_vint_number_size(sequence: Seq<u8>, iter: int) -> nat;

pub(crate) open spec fn is_video_webm(bm: &ByteMatcher) -> bool {
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "video/webm"@
    // 3. If length is less than 4, return false. 
    &&& bm.pattern@.len() >= 4 
    // 4. If the four bytes from sequence[0] to sequence[3], are not equal to 
    //    0x1A 0x45 0xDF 0xA3, return false. 
    &&& bm.pattern@[0] == 0x1Au8
    &&& bm.pattern@[1] == 0x45u8
    &&& bm.pattern@[2] == 0xDFu8
    &&& bm.pattern@[3] == 0xA3u8
    // 5. Let iter be 4. webm_scan
    &&& (bm.pattern@.len() > 4) ==> webm_scan(bm.pattern@, 4)
    // only for current servo implementation
    &&& (bm.pattern@.len() == 4) ==> bm.mask@ == b"\xFF\xFF\xFF\xFF"@
}

// https://mimesniff.spec.whatwg.org/#matching-an-audio-or-video-type-pattern
//
// | Byte Pattern   | Pattern Mask    | Leading Bytes Ignored | Audio or Video MIME Type |
// |----------------|-----------------|-----------------------|--------------------------|

// https://whatpr.org/mimesniff/36/74065de...3f70580.html#matching-an-audio-or-video-type-pattern
// FIXME: deprecated
// | 2E 73 6E 64    | FF FF FF FF     | None                  | audio/basic              |
pub(crate) open spec fn is_audio_basic(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x2E\x73\x6E\x64"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "audio/basic"@
}

// | 46 4F 52 4D 00 00 00 00 41 49 46 46 | FF FF FF FF 00 00 00 00 FF FF FF FF | None | audio/aiff |
pub(crate) open spec fn is_audio_aiff(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x46\x4F\x52\x4D\x00\x00\x00\x00\x41\x49\x46\x46"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "audio/aiff"@
}

// | 49 44 33        | FF FF FF      | None                  | audio/mpeg               |
pub(crate) open spec fn is_audio_mpeg(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x49\x44\x33"@
    &&& bm.mask@ == b"\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "audio/mpeg"@
}

// | 4F 67 67 53 00  | FF FF FF FF FF | None                  | application/ogg          |
pub(crate) open spec fn is_application_ogg(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x4F\x67\x67\x53\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/ogg"@
}

// | 4D 54 68 64 00 00 00 06                   | FF FF FF FF FF FF FF FF                  | None                  | audio/midi               |
pub(crate) open spec fn is_audio_midi(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x4D\x54\x68\x64\x00\x00\x00\x06"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "audio/midi"@
}

// | 52 49 46 46 00 00 00 00 41 56 49 20       | FF FF FF FF 00 00 00 00 FF FF FF FF      | None                  | video/avi                |
pub(crate) open spec fn is_video_avi(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x52\x49\x46\x46\x00\x00\x00\x00\x41\x56\x49\x20"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "video/avi"@
}
// | 52 49 46 46 00 00 00 00 57 41 56 45       | FF FF FF FF 00 00 00 00 FF FF FF FF      | None                  | audio/wave               |
pub(crate) open spec fn is_audio_wave(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x52\x49\x46\x46\x00\x00\x00\x00\x57\x41\x56\x45"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "audio/wave"@
}

// https://mimesniff.spec.whatwg.org/#whitespace-byte
pub open spec fn contains_byte(bytes: Seq<u8>, byte: u8) -> bool {
    exists |i: int| 0 <= i < bytes.len() && #[trigger] bytes[i] == byte
}

// A whitespace byte (abbreviated 0xWS) is any one of the following bytes: 
// 0x09 (HT), 0x0A (LF), 0x0C (FF), 0x0D (CR), 0x20 (SP). 
pub open spec fn contains_all_whitespace(leading_ignore: Seq<u8>) -> bool {
    &&& leading_ignore.len() == 5
    &&& contains_byte(leading_ignore, 0x09u8) // HT  \t
    &&& contains_byte(leading_ignore, 0x0Au8) // LF  \n
    &&& contains_byte(leading_ignore, 0x0Cu8) // FF
    &&& contains_byte(leading_ignore, 0x0Du8) // CR  \r
    &&& contains_byte(leading_ignore, 0x20u8) // SP
}

pub(crate) broadcast proof fn lemma_contains_all_whitespace(bstr: Seq<u8>)
    requires
        bstr == b"\t\n\x0C\r "@,
    ensures
        #[trigger] contains_all_whitespace(bstr),
{
    reveal_byteslit(b"\t\n\x0C\r ");
}

pub(crate) broadcast group whitespace_lemmas {
    lemma_contains_all_whitespace,
}

// https://mimesniff.spec.whatwg.org/#identifying-a-resource-with-an-unknown-mime-type
// A tag-terminating byte (abbreviated 0xTT) is any one of the following bytes: 0x20 (SP), 0x3E (">"). 
// | Byte Pattern            | Pattern Mask            | Leading Bytes Ignored | Computed MIME Type |
// |------------------------------------------------|------------------------------------------------|-----------------------|--------------------|
// | 3C 21 44 4F 43 54 59 50 45 20 48 54 4D 4C TT | FF FF DF DF DF DF DF DF DF FF DF DF DF DF FF | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_doctype(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x21\x44\x4F\x43\x54\x59\x50\x45\x20\x48\x54\x4D\x4C"@
    &&& ttbm.matcher.mask@ == b"\xFF\xFF\xDF\xDF\xDF\xDF\xDF\xDF\xDF\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}

// | 3C 48 54 4D 4C TT       | FF DF DF DF DF FF       | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_page(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x48\x54\x4D\x4C"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 48 45 41 44 TT       | FF DF DF DF DF FF       | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_head(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x48\x45\x41\x44"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 53 43 52 49 50 54 TT | FF DF DF DF DF DF DF FF | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_script(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x53\x43\x52\x49\x50\x54"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 49 46 52 41 4D 45 TT | FF DF DF DF DF DF DF FF | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_iframe(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x49\x46\x52\x41\x4D\x45"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 48 31 TT             | FF DF FF FF             | Whitespace bytes      | text/html        |
pub (crate) open spec fn is_text_html_h1(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x48\x31"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xFF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 44 49 56 TT          | FF DF DF DF FF          | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_div(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x44\x49\x56"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 46 4F 4E 54 TT       | FF DF DF DF DF FF       | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_font(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x46\x4F\x4E\x54"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 54 41 42 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_table(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x54\x41\x42\x4C\x45"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 41 TT                | FF DF FF                | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_a(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x41"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 53 54 59 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_style(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x53\x54\x59\x4C\x45"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 54 49 54 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_title(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x54\x49\x54\x4C\x45"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 42 TT                | FF DF FF                | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_b(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x42"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 42 4F 44 59 TT       | FF DF DF DF DF FF       | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_body(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x42\x4F\x44\x59"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 42 52 TT             | FF DF DF FF             | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_br(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x42\x52"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 50 TT                | FF DF FF                | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_p(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x50"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 21 2D 2D TT          | FF FF FF FF FF          | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_comment(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x21\x2D\x2D"@
    &&& ttbm.matcher.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& essence_str(&ttbm.matcher.content_type) == "text/html"@
}
// | 3C 3F 78 6D 6C          | FF FF FF FF FF          | Whitespace bytes      | text/xml         |
pub(crate) open spec fn is_text_xml(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x3C\x3F\x78\x6D\x6C"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == b"\t\n\x0C\r "@
    &&& essence_str(&bm.content_type) == "text/xml"@
}
// | 25 50 44 46 2D          | FF FF FF FF FF          | None                  | application/pdf  |
pub(crate) open spec fn is_application_pdf(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x25\x50\x44\x46\x2D"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/pdf"@
}


// https://mimesniff.spec.whatwg.org/#matching-a-font-type-pattern
// https://mimesniff.spec.whatwg.org/#matching-a-font-type-pattern
//
// | Byte Pattern               | Pattern Mask               | Leading Bytes Ignored | Font MIME Type                |
// |----------------------------|----------------------------|-----------------------|-------------------------------|
// | 00 00 00 00 00 00 00 00 00 | 00 00 00 00 00 00 00 00 00 |                       |                               |
// | 00 00 00 00 00 00 00 00 00 | 00 00 00 00 00 00 00 00 00 | None                  | application/vnd.ms-fontobject |
// | 00 00 00 00 00 00 00 00 00 | 00 00 00 00 00 00 00 00 00 |                       |                               |
// | 00 00 00 00 00 00 00 4C 50 | 00 00 00 00 00 00 00 FF FF |                       |                               |
pub(crate) open spec fn is_application_vnd_ms_font_object(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                         \x00\x00\x00\x00\x00\x00\x00\x00\x00\
                         \x00\x00\x00\x00\x00\x00\x00\x00\x00\
                         \x00\x00\x00\x00\x00\x00\x00\x4C\x50"@
    &&& bm.mask@ == b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                      \x00\x00\x00\x00\x00\x00\x00\x00\x00\
                      \x00\x00\x00\x00\x00\x00\x00\x00\x00\
                      \x00\x00\x00\x00\x00\x00\x00\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/vnd.ms-fontobject"@
}
// | 00 01 00 00                | FF FF FF FF                | None                  | font/ttf                      |
pub(crate) open spec fn is_true_type(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x00\x01\x00\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/font-sfnt"@ // FIXME: font/ttf
}
// | 4F 54 54 4F                | FF FF FF FF                | None                  | font/otf                      |
pub(crate) open spec fn is_open_type(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x4F\x54\x54\x4F"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/font-sfnt"@ // FIXME: font/otf
}
// | 74 74 63 66                | FF FF FF FF                | None                  | font/collection               |
pub(crate) open spec fn is_true_type_collection(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x74\x74\x63\x66"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/font-sfnt"@ // FIXME: font/collection
}
// | 77 4F 46 46                | FF FF FF FF                | None                  | font/woff                     |
pub(crate) open spec fn is_application_font_woff(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x77\x4F\x46\x46"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/font-woff"@ // FIXME: font/woff
}
// | 77 4F 46 32                | FF FF FF FF                | None                  | font/woff2                    |
// TODO: missing


// https://mimesniff.spec.whatwg.org/#matching-an-archive-type-pattern
//
// | Byte Pattern          | Pattern Mask          | Leading Bytes Ignored | Archive MIME Type               |
// |-----------------------|-----------------------|-----------------------|---------------------------------|
// | 1F 8B 08              | FF FF FF              | None                  | application/x-gzip            | 
pub(crate) open spec fn is_application_x_gzip(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x1F\x8B\x08"@
    &&& bm.mask@ == b"\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/x-gzip"@
}
// | 50 4B 03 04           | FF FF FF FF           | None                  | application/zip               | 
pub(crate) open spec fn is_application_zip(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x50\x4B\x03\x04"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/zip"@
}
// | 52 61 72 21 1A 07 00  | FF FF FF FF FF FF FF  | None                  | application/x-rar-compressed  |
pub(crate) open spec fn is_application_x_rar_compressed(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x52\x61\x72\x20\x1A\x07\x00"@ //TODO: Wrong
    // &&& bm.pattern@ == b"\x52\x61\x72\x21\x1A\x07\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/x-rar-compressed"@
}


// https://mimesniff.spec.whatwg.org/#identifying-a-resource-with-an-unknown-mime-type
// | Byte Pattern       | Pattern Mask       | Leading Bytes Ignored | Computed MIME Type     |
// |--------------------|--------------------|-----------------------|------------------------|
// | 25 21 50 53 2D 41  | FF FF FF FF FF FF  | None                  |                        |
// | 64 6F 62 65 2D     | FF FF FF FF FF     | None                  | application/postscript |
pub(crate) open spec fn is_application_postscript(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x25\x21\x50\x53\x2D\x41\x64\x6F\x62\x65\x2D"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "application/postscript"@
}
// | FE FF 00 00        | FF FF 00 00        | None                  | text/plain             |
pub(crate) open spec fn is_text_plain_utf_16be_bom(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\xFE\xFF\x00\x00"@
    &&& bm.mask@ == b"\xFF\xFF\x00\x00"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "text/plain"@
}
// | FF FE 00 00        | FF FF 00 00        | None                  | text/plain             |
pub(crate) open spec fn is_text_plain_utf_16le_bom(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\xFF\xFE\x00\x00"@
    &&& bm.mask@ == b"\xFF\xFF\x00\x00"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "text/plain"@
}
// | EF BB BF 00        | FF FF FF 00        | None                  | text/plain             |
pub(crate) open spec fn is_text_plain_utf_8_bom(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\xEF\xBB\xBF\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\x00"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "text/plain"@
}

define_mime_essence_lemmas! {
    mime_essence_str_lemmas {
        lemma_image_bmp_essence_str => ("image", "bmp"),
        lemma_image_png_essence_str => ("image", "png"),
        lemma_image_gif_essence_str => ("image", "gif"),
        lemma_image_jpeg_essence_str => ("image", "jpeg"),

        lemma_text_html_essence_str => ("text", "html"),
        lemma_text_xml_essence_str => ("text", "xml"),
        lemma_text_plain_essence_str => ("text", "plain"),

        lemma_application_pdf_essence_str => ("application", "pdf"),
        lemma_application_octet_stream_essence_str => ("application", "octet-stream"),
    }
}

define_mime_essence_parts_lemmas! {
    mime_essence_parts_str_lemmas {
        lemma_unknown_unknown_essence_parts_str => ("unknown", "unknown", 8),
        lemma_application_unknown_essence_parts_str => ("application", "unknown", 12),
        lemma_star_star_essence_parts_str => ("*", "*", 2),

        lemma_application_ecmascript_essence_parts_str => ("application", "ecmascript", 12),
        lemma_application_javascript_essence_parts_str => ("application", "javascript", 12),
        lemma_application_x_ecmascript_essence_parts_str => ("application", "x-ecmascript", 12),
        lemma_application_x_javascript_essence_parts_str => ("application", "x-javascript", 12),
        lemma_text_ecmascript_essence_parts_str => ("text", "ecmascript", 5),
        lemma_text_javascript_essence_parts_str => ("text", "javascript", 5),
        lemma_text_javascript0_essence_parts_str => ("text", "javascript1.0", 5),
        lemma_text_javascript1_essence_parts_str => ("text", "javascript1.1", 5),
        lemma_text_javascript2_essence_parts_str => ("text", "javascript1.2", 5),
        lemma_text_javascript3_essence_parts_str => ("text", "javascript1.3", 5),
        lemma_text_javascript4_essence_parts_str => ("text", "javascript1.4", 5),
        lemma_text_javascript5_essence_parts_str => ("text", "javascript1.5", 5),
        lemma_text_jscript_essence_parts_str => ("text", "jscript", 5),
        lemma_text_livescript_essence_parts_str => ("text", "livescript", 5),
        lemma_text_x_ecmascript_essence_parts_str => ("text", "x-ecmascript", 5),
        lemma_text_x_javascript_essence_parts_str => ("text", "x-javascript", 5),

        lemma_application_font_cff_essence_parts_str => ("application", "font-cff", 12),
        lemma_application_font_off_essence_parts_str => ("application", "font-off", 12),
        lemma_application_font_sfnt_essence_parts_str => ("application", "font-sfnt", 12),
        lemma_application_font_ttf_essence_parts_str => ("application", "font-ttf", 12),
        lemma_application_font_woff_essence_parts_str => ("application", "font-woff", 12),
        lemma_application_font_vnd_ms_fontobject_essence_parts_str => ("application", "vnd.ms-fontobject", 12),
        lemma_application_font_vnd_ms_opentype_essence_parts_str => ("application", "vnd.ms-opentype", 12),
    }
}

} // verus!