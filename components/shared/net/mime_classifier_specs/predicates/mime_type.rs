use mime::Mime;
use vstd::prelude::*;
use vstd::assert_seqs_equal;

use crate::mime_classifier_specs::mime_api::*;

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
                    // requires view(mt).suffix is None, // for servo behavior
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
                        let literal = (concat!($type_, "/", $subtype))@;
                        assert forall |i: int| 0 <= i < literal.len()
                            && #[trigger] literal[i] == '/'
                            implies i == ($type_)@.len()
                        by {}
                        assert(type_.len() == ($type_)@.len());
                        assert(subtype.len() == ($subtype)@.len());

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
    view(mt).suffix == Some(xml_name())
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
pub open spec fn is_font(mt: &Mime) -> bool { 
    ||| view(mt).type_ == font_name()
    ||| essence_str(mt) == "application/font-cff"@
    // https://github.com/servo/servo/issues/47903
    ||| essence_str(mt) == "application/font-off"@ //TODO: new version is font-otf
    ||| essence_str(mt) == "application/font-sfnt"@
    ||| essence_str(mt) == "application/font-ttf"@
    ||| essence_str(mt) == "application/font-woff"@
    ||| essence_str(mt) == "application/vnd.ms-fontobject"@
    ||| essence_str(mt) == "application/vnd.ms-opentype"@
}

/// <https://mimesniff.spec.whatwg.org/#json-mime-type>
pub open spec fn is_json(mt: &Mime) -> bool { 
    ||| view(mt).suffix == Some(json_name())@ 
    ||| essence_str(mt) == "application/json"@
    ||| essence_str(mt) == "text/json"@
}

pub open spec fn is_text(mt: &Mime) -> bool { 
    view(mt) == text_plain_identity()
    || essence_str(mt) == "text/vtt"@
}

pub open spec fn is_css(mt: &Mime) -> bool {
    essence_str(mt) == "text/css"@
}

pub open spec fn is_explicit_unknown(mt: &Mime) -> bool {
    ||| essence_str(mt) == "application/unknown"@
    ||| essence_str(mt) == "unknown/unknown"@
    ||| essence_str(mt) == "*/*"@
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
