use mime::Mime;
use vstd::prelude::*;
use vstd::assert_seqs_equal;

use super::views::*;

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
                        // view(mt).suffix is None, // for servo behavior
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

} // verus!
