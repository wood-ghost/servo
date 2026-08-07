use mime::Mime;
use vstd::prelude::*;
use vstd::assert_seqs_equal;

use super::model::*;
pub(crate) use super::model::essence_str;

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
                        #[trigger] essence_str(mt) =~=
                            (concat!($type_, "/", $subtype))@,
                {
                    reveal_strlit($type_);
                    reveal_strlit("/");
                    reveal_strlit($subtype);
                    reveal_strlit(concat!($type_, "/", $subtype));

                    assert_seqs_equal!(
                        (
                            view(mt).type_
                                + "/"@
                                + view(mt).subtype
                        ) == (concat!($type_, "/", $subtype))@
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

pub(crate) open spec fn essence_is_text_xml(mt: &Mime) -> bool {
    // view(mt).essence =~= "text/xml"@
    essence_str(mt) =~= "text/xml"@
}

pub(crate) open spec fn essence_is_application_ogg(mt: &Mime) -> bool {
    // view(mt).essence =~= "application/ogg"@
    essence_str(mt) == "application/ogg"@
}

pub(crate) uninterp spec fn has_html_suffix(mt: &Mime) -> bool;
pub open spec fn is_text_plain(mt: &Mime) -> bool {
    view(mt) == text_plain_identity()
}
pub open spec fn is_text_plain_utf8(mt: &Mime) -> bool {
    view(mt) == text_plain_utf8_identity()
}
pub open spec fn is_image(mt: &Mime) -> bool {
    view(mt).type_ =~= image_name()
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

pub(crate) open spec fn is_xml(mt: &Mime) -> bool {
    !is_image(mt) && (
        has_xml_suffix(mt)
            || essence_is_text_xml(mt)
            || (essence_str(mt) == "application/xml"@)
    )
} 

pub(crate) open spec fn is_html(mt: &Mime) -> bool {
    essence_str(mt) == "text/html"@
}

pub(crate) open spec fn is_audio_video(mt: &Mime) -> bool {
    is_audio(mt) || is_video(mt) || essence_is_application_ogg(mt)
}

// ----------------------------
// check hard coded byte matchers
// ----------------------------

// Source: WHATWG MIME Sniffing Standard
// https://mimesniff.spec.whatwg.org/#matching-an-image-type-pattern
//
// | Byte Pattern  | Pattern Mask  | Leading Bytes Ignored | Image MIME Type |
// |---------------|---------------|-----------------------|-----------------|
// | 42 4D                                      | FF FF                                      | None                  | image/bmp       |
// | 47 49 46 38 37 61                          | FF FF FF FF FF FF                          | None                  | image/gif       |
// | 47 49 46 38 39 61                          | FF FF FF FF FF FF                          | None                  | image/gif       |
// | 52 49 46 46 00 00 00 00 57 45 42 50 56 50 | FF FF FF FF 00 00 00 00 FF FF FF FF FF FF | None                  | image/webp      |
// | 89 50 4E 47 0D 0A 1A 0A                   | FF FF FF FF FF FF FF FF                    | None                  | image/png       |
// | FF D8 FF                                   | FF FF FF                                   | None                  | image/jpeg      |

// | 00 00 01 00   | FF FF FF FF   | None                  | image/x-icon    |
pub(crate) open spec fn is_image_x_icon(pattern: Seq<u8>, mask: Seq<u8>, content_type: &Mime, leading_ignore: Seq<u8>) -> bool {
    &&& pattern == b"\x00\x00\x01\x00"@
    &&& mask == b"\xFF\xFF\xFF\xFF"@
    &&& leading_ignore == &[]@
    &&& essence_str(content_type) == "image/x-icon"@
}

// | 00 00 02 00   | FF FF FF FF    | None                  | image/x-icon    |
pub(crate) open spec fn is_image_x_icon_cursor(pattern: Seq<u8>, mask: Seq<u8>, content_type: &Mime, leading_ignore: Seq<u8>) -> bool {
    &&& pattern == b"\x00\x00\x02\x00"@
    &&& mask == b"\xFF\xFF\xFF\xFF"@
    &&& leading_ignore == &[]@
    &&& essence_str(content_type) == "image/x-icon"@
}

// | 42 4D         | FF FF          | None                  | image/bmp       |
pub(crate) open spec fn is_image_bmp(pattern: Seq<u8>, mask: Seq<u8>, content_type: &Mime, leading_ignore: Seq<u8>) -> bool {
    &&& pattern == b"\x42\x4D"@
    &&& mask == b"\xFF\xFF"@
    &&& leading_ignore == &[]@
    &&& essence_str(content_type) =~= "image/bmp"@
}

define_mime_essence_lemmas! {
    mime_essence_str_lemmas {
        lemma_image_bmp_essence_str =>
            ("image", "bmp"),

        lemma_image_png_essence_str =>
            ("image", "png"),

        lemma_image_gif_essence_str =>
            ("image", "gif"),

        lemma_image_jpeg_essence_str =>
            ("image", "jpeg"),

        lemma_text_html_essence_str =>
            ("text", "html"),

        lemma_text_plain_essence_str =>
            ("text", "plain"),

        lemma_application_xml_essence_str =>
            ("application", "xml"),

        lemma_application_pdf_essence_str =>
            ("application", "pdf"),
    }
}

} // verus!