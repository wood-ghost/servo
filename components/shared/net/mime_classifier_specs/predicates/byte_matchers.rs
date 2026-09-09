use vstd::prelude::*;

use crate::mime_classifier::{
    ByteMatcher,
    TagTerminatedByteMatcher,
};
use crate::mime_classifier_specs::mime_api::*;

verus! {

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
    &&& view(&bm.content_type) == image_x_icon_identity()
    // &&& essence_str(&bm.content_type) == "image/x-icon"@
}

// | 00 00 02 00   | FF FF FF FF    | None                  | image/x-icon    |
pub(crate) open spec fn is_image_x_icon_cursor(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x00\x00\x02\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == image_x_icon_identity()
}

// | 42 4D         | FF FF          | None                  | image/bmp       |
pub(crate) open spec fn is_image_bmp(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x42\x4D"@
    &&& bm.mask@ == b"\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == image_bmp_identity()
}

// | 47 49 46 38 37 61   | FF FF FF FF FF FF  | None        | image/gif       |
pub(crate) open spec fn is_image_gif87a(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x47\x49\x46\x38\x37\x61"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == image_gif_identity()
}

// | 47 49 46 38 39 61   | FF FF FF FF FF FF   | None        | image/gif       |
pub(crate) open spec fn is_image_gif89a(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x47\x49\x46\x38\x39\x61"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == image_gif_identity()
}

// | 52 49 46 46 00 00 00 00 57 45 42 50 56 50 | FF FF FF FF 00 00 00 00 FF FF FF FF FF FF | None  | image/webp |
pub(crate) open spec fn is_image_webp(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x52\x49\x46\x46\x00\x00\x00\x00\x57\x45\x42\x50\x56\x50"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == image_webp_identity()
}

// | 89 50 4E 47 0D 0A 1A 0A   | FF FF FF FF FF FF FF FF | None | image/png       |
pub(crate) open spec fn is_image_png(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == image_png_identity()
}

// | FF D8 FF         | FF FF FF      | None                  | image/jpeg      |
pub(crate) open spec fn is_image_jpeg(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\xFF\xD8\xFF"@
    &&& bm.mask@ == b"\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == image_jpeg_identity()
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
    &&& view(&bm.content_type) == audio_basic_identity()
}

// | 46 4F 52 4D 00 00 00 00 41 49 46 46 | FF FF FF FF 00 00 00 00 FF FF FF FF | None | audio/aiff |
pub(crate) open spec fn is_audio_aiff(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x46\x4F\x52\x4D\x00\x00\x00\x00\x41\x49\x46\x46"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == audio_aiff_identity()
}

// | 49 44 33        | FF FF FF      | None                  | audio/mpeg               |
pub(crate) open spec fn is_audio_mpeg(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x49\x44\x33"@
    &&& bm.mask@ == b"\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == audio_mpeg_identity()
}

// | 4F 67 67 53 00  | FF FF FF FF FF | None                  | application/ogg          |
pub(crate) open spec fn is_application_ogg(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x4F\x67\x67\x53\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_ogg_identity()
}

// | 4D 54 68 64 00 00 00 06                   | FF FF FF FF FF FF FF FF                  | None                  | audio/midi               |
pub(crate) open spec fn is_audio_midi(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x4D\x54\x68\x64\x00\x00\x00\x06"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == audio_midi_identity()
}

// | 52 49 46 46 00 00 00 00 41 56 49 20       | FF FF FF FF 00 00 00 00 FF FF FF FF      | None                  | video/avi                |
pub(crate) open spec fn is_video_avi(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x52\x49\x46\x46\x00\x00\x00\x00\x41\x56\x49\x20"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == video_avi_identity()
}
// | 52 49 46 46 00 00 00 00 57 41 56 45       | FF FF FF FF 00 00 00 00 FF FF FF FF      | None                  | audio/wave               |
pub(crate) open spec fn is_audio_wave(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x52\x49\x46\x46\x00\x00\x00\x00\x57\x41\x56\x45"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == audio_wave_identity()
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
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}

// | 3C 48 54 4D 4C TT       | FF DF DF DF DF FF       | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_page(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x48\x54\x4D\x4C"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 48 45 41 44 TT       | FF DF DF DF DF FF       | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_head(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x48\x45\x41\x44"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 53 43 52 49 50 54 TT | FF DF DF DF DF DF DF FF | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_script(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x53\x43\x52\x49\x50\x54"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 49 46 52 41 4D 45 TT | FF DF DF DF DF DF DF FF | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_iframe(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x49\x46\x52\x41\x4D\x45"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 48 31 TT             | FF DF FF FF             | Whitespace bytes      | text/html        |
pub (crate) open spec fn is_text_html_h1(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x48\x31"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xFF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 44 49 56 TT          | FF DF DF DF FF          | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_div(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x44\x49\x56"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 46 4F 4E 54 TT       | FF DF DF DF DF FF       | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_font(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x46\x4F\x4E\x54"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 54 41 42 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_table(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x54\x41\x42\x4C\x45"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 41 TT                | FF DF FF                | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_a(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x41"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 53 54 59 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_style(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x53\x54\x59\x4C\x45"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 54 49 54 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_title(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x54\x49\x54\x4C\x45"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 42 TT                | FF DF FF                | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_b(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x42"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 42 4F 44 59 TT       | FF DF DF DF DF FF       | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_body(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x42\x4F\x44\x59"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 42 52 TT             | FF DF DF FF             | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_br(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x42\x52"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 50 TT                | FF DF FF                | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_p(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x50"@
    &&& ttbm.matcher.mask@ == b"\xFF\xDF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 21 2D 2D TT          | FF FF FF FF FF          | Whitespace bytes      | text/html        |
pub(crate) open spec fn is_text_html_comment(ttbm: &TagTerminatedByteMatcher) -> bool {
    &&& ttbm.matcher.pattern@ == b"\x3C\x21\x2D\x2D"@
    &&& ttbm.matcher.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& contains_all_whitespace(ttbm.matcher.leading_ignore@)
    &&& view(&ttbm.matcher.content_type) == text_html_identity()
}
// | 3C 3F 78 6D 6C          | FF FF FF FF FF          | Whitespace bytes      | text/xml         |
pub(crate) open spec fn is_text_xml(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x3C\x3F\x78\x6D\x6C"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == b"\t\n\x0C\r "@
    &&& view(&bm.content_type) == text_xml_identity()
}
// | 25 50 44 46 2D          | FF FF FF FF FF          | None                  | application/pdf  |
pub(crate) open spec fn is_application_pdf(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x25\x50\x44\x46\x2D"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_pdf_identity()
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
    &&& view(&bm.content_type) == application_vnd_ms_fontobject_identity()
    // &&& essence_str(&bm.content_type) == "application/vnd.ms-fontobject"@
}
// | 00 01 00 00                | FF FF FF FF                | None                  | font/ttf                      |
pub(crate) open spec fn is_true_type(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x00\x01\x00\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_font_sfnt_identity()// FIXME: font/ttf
}
// | 4F 54 54 4F                | FF FF FF FF                | None                  | font/otf                      |
pub(crate) open spec fn is_open_type(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x4F\x54\x54\x4F"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_font_sfnt_identity()// FIXME: font/otf
}
// | 74 74 63 66                | FF FF FF FF                | None                  | font/collection               |
pub(crate) open spec fn is_true_type_collection(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x74\x74\x63\x66"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_font_sfnt_identity()// FIXME: font/collection
}
// | 77 4F 46 46                | FF FF FF FF                | None                  | font/woff                     |
pub(crate) open spec fn is_application_font_woff(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x77\x4F\x46\x46"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_font_woff_identity()
}
// | 77 4F 46 32                | FF FF FF FF                | None                  | font/woff2                    |
// unused 
pub(crate) open spec fn is_application_font_woff2(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x77\x4F\x46\x32"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_font_woff2_identity()// FIXME: font/woff2
}

// https://mimesniff.spec.whatwg.org/#matching-an-archive-type-pattern
//
// | Byte Pattern          | Pattern Mask          | Leading Bytes Ignored | Archive MIME Type               |
// |-----------------------|-----------------------|-----------------------|---------------------------------|
// | 1F 8B 08              | FF FF FF              | None                  | application/x-gzip            | 
pub(crate) open spec fn is_application_x_gzip(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x1F\x8B\x08"@
    &&& bm.mask@ == b"\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_x_gzip_identity()
}
// | 50 4B 03 04           | FF FF FF FF           | None                  | application/zip               | 
pub(crate) open spec fn is_application_zip(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x50\x4B\x03\x04"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_zip_identity()
}
// | 52 61 72 21 1A 07 00  | FF FF FF FF FF FF FF  | None                  | application/x-rar-compressed  |
pub(crate) open spec fn is_application_x_rar_compressed(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x52\x61\x72\x20\x1A\x07\x00"@ //TODO: Wrong
    // &&& bm.pattern@ == b"\x52\x61\x72\x21\x1A\x07\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == application_x_rar_compressed_identity()
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
    &&& view(&bm.content_type) == application_postscript_identity() 
}
// | FE FF 00 00        | FF FF 00 00        | None                  | text/plain             |
pub(crate) open spec fn is_text_plain_utf_16be_bom(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\xFE\xFF\x00\x00"@
    &&& bm.mask@ == b"\xFF\xFF\x00\x00"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == text_plain_identity()
}
// | FF FE 00 00        | FF FF 00 00        | None                  | text/plain             |
pub(crate) open spec fn is_text_plain_utf_16le_bom(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\xFF\xFE\x00\x00"@
    &&& bm.mask@ == b"\xFF\xFF\x00\x00"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == text_plain_identity()
}
// | EF BB BF 00        | FF FF FF 00        | None                  | text/plain             |
pub(crate) open spec fn is_text_plain_utf_8_bom(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\xEF\xBB\xBF\x00"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\x00"@
    &&& bm.leading_ignore@ == &[]@
    &&& view(&bm.content_type) == text_plain_identity()
}
} // verus!
