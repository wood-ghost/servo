use mime::Mime;
use vstd::prelude::*;
use vstd::assert_seqs_equal;

use crate::mime_classifier::ByteMatcher;
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

pub open spec fn is_javascript(mt: &Mime) -> bool {
    essence_str(mt) == "application/javascript"@
}

pub open spec fn is_font(mt: &Mime) -> bool { //TODO:
    essence_str(mt) == "font/woff"@
        || essence_str(mt) == "font/woff2"@
        || essence_str(mt) == "application/font-woff"@
        || essence_str(mt) == "application/font-woff2"@
}

pub open spec fn is_json(mt: &Mime) -> bool { //TODO:
    essence_str(mt) == "application/json"@
}

pub open spec fn is_text(mt: &Mime) -> bool { //TODO:
    view(mt).type_ == text_name()
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
pub(crate) open spec fn is_video_webm(bm: &ByteMatcher) -> bool {
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "video/webm"@
    // If length is less than 4, return false. 
    &&& (bm.pattern@.len() < 4) ==> false
    // If the four bytes from sequence[0] to sequence[3], are not equal to 0x1A 0x45 0xDF 0xA3, return false. 
    &&& (bm.pattern@.len() == 4) ==> (
        (bm.pattern@ == b"\x1A\x45\xDF\xA3"@) 
        && bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    )
    // ...
}

// https://mimesniff.spec.whatwg.org/#matching-an-audio-or-video-type-pattern
//
// | Byte Pattern                              | Pattern Mask                             | Leading Bytes Ignored | Audio or Video MIME Type |
// |-------------------------------------------|------------------------------------------|-----------------------|--------------------------|

// https://whatpr.org/mimesniff/36/74065de...3f70580.html#matching-an-audio-or-video-type-pattern
// FIXME: deprecated
// | 2E 73 6E 64                               | FF FF FF FF                              | None                  | audio/basic              |
pub(crate) open spec fn is_audio_basic(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x2E\x73\x6E\x64"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "audio/basic"@
}

// | 46 4F 52 4D 00 00 00 00 41 49 46 46       | FF FF FF FF 00 00 00 00 FF FF FF FF      | None                  | audio/aiff               |
pub(crate) open spec fn is_audio_aiff(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x46\x4F\x52\x4D\x00\x00\x00\x00\x41\x49\x46\x46"@
    &&& bm.mask@ == b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "audio/aiff"@
}

// | 49 44 33                                  | FF FF FF                                 | None                  | audio/mpeg               |
pub(crate) open spec fn is_audio_mpeg(bm: &ByteMatcher) -> bool {
    &&& bm.pattern@ == b"\x49\x44\x33"@
    &&& bm.mask@ == b"\xFF\xFF\xFF"@
    &&& bm.leading_ignore@ == &[]@
    &&& essence_str(&bm.content_type) == "audio/mpeg"@
}

// | 4F 67 67 53 00                            | FF FF FF FF FF                           | None                  | application/ogg          |
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

// https://mimesniff.spec.whatwg.org/#identifying-a-resource-with-an-unknown-mime-type
//
// | Byte Pattern            | Pattern Mask            | Leading Bytes Ignored | Computed MIME Type |
// |------------------------------------------------|------------------------------------------------|-----------------------|--------------------|
// | 3C 21 44 4F 43 54 59 50 45 20 48 54 4D 4C TT | FF FF DF DF DF DF DF DF DF FF DF DF DF DF FF | Whitespace bytes      | `text/html`        |
// | 3C 48 54 4D 4C TT       | FF DF DF DF DF FF       | Whitespace bytes      | `text/html`        |
// | 3C 48 45 41 44 TT       | FF DF DF DF DF FF       | Whitespace bytes      | `text/html`        |
// | 3C 53 43 52 49 50 54 TT | FF DF DF DF DF DF DF FF | Whitespace bytes      | `text/html`        |
// | 3C 49 46 52 41 4D 45 TT | FF DF DF DF DF DF DF FF | Whitespace bytes      | `text/html`        |
// | 3C 48 31 TT             | FF DF FF FF             | Whitespace bytes      | `text/html`        |
// | 3C 44 49 56 TT          | FF DF DF DF FF          | Whitespace bytes      | `text/html`        |
// | 3C 46 4F 4E 54 TT       | FF DF DF DF DF FF       | Whitespace bytes      | `text/html`        |
// | 3C 54 41 42 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | `text/html`        |
// | 3C 41 TT                | FF DF FF                | Whitespace bytes      | `text/html`        |
// | 3C 53 54 59 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | `text/html`        |
// | 3C 54 49 54 4C 45 TT    | FF DF DF DF DF DF FF    | Whitespace bytes      | `text/html`        |
// | 3C 42 TT                | FF DF FF                | Whitespace bytes      | `text/html`        |
// | 3C 42 4F 44 59 TT       | FF DF DF DF DF FF       | Whitespace bytes      | `text/html`        |
// | 3C 42 52 TT             | FF DF DF FF             | Whitespace bytes      | `text/html`        |
// | 3C 50 TT                | FF DF FF                | Whitespace bytes      | `text/html`        |
// | 3C 21 2D 2D TT          | FF FF FF FF FF          | Whitespace bytes      | `text/html`        |
// | 3C 3F 78 6D 6C          | FF FF FF FF FF          | Whitespace bytes      | `text/xml`         |
// | 25 50 44 46 2D          | FF FF FF FF FF          | None                  | `application/pdf`  |


define_mime_essence_lemmas! {
    mime_essence_str_lemmas {
        lemma_image_bmp_essence_str => ("image", "bmp"),

        lemma_image_png_essence_str => ("image", "png"),

        lemma_image_gif_essence_str => ("image", "gif"),

        lemma_image_jpeg_essence_str => ("image", "jpeg"),

        lemma_text_html_essence_str => ("text", "html"),

        lemma_text_plain_essence_str => ("text", "plain"),

        // lemma_text_xml_essence_str => ("text", "xml"),

        lemma_application_xml_essence_str => ("application", "xml"),

        lemma_application_pdf_essence_str => ("application", "pdf"),

        lemma_application_octet_stream_essence_str => ("application", "octet-stream"),

        lemma_unknown_unknown_essence_str => ("unknown", "unknown"),

        lemma_application_unknown_essence_str => ("application", "unknown"),

        lemma_star_star_essence_str => ("*", "*"),
    }
}

} // verus!