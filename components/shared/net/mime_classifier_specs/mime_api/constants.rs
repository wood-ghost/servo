use vstd::prelude::*;

use super::views::MimeView;

verus! {

// ----------------
// Constant Identity
// -----------------
// The Rust Mime type https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#43
// pub struct Mime {
//     source: Source,
//     slash: usize,
//     plus: Option<usize>,
//     params: ParamSource,
// }

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#750
// TEXT_PLAIN, "text/plain", 4;
#[verifier::auto_reveal_literals(strlit)]
pub open spec fn text_plain_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "plain"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#751
// TEXT_PLAIN_UTF_8, "text/plain; charset=utf-8", 4, None, 10;
#[verifier::auto_reveal_literals(strlit)]
pub open spec fn text_plain_utf_8_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "plain"@,
        suffix: None,
        params: Map::empty().insert("charset"@, "utf-8"@),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#752
// TEXT_HTML, "text/html", 4;
pub open spec fn text_html_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "html"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#754
// TEXT_CSS, "text/css", 4;
pub open spec fn text_css_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "css"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#756
// TEXT_JAVASCRIPT, "text/javascript", 4;
pub open spec fn text_javascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#757
// TEXT_XML, "text/xml", 4;
pub open spec fn text_xml_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "xml"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#766
// IMAGE_JPEG, "image/jpeg", 5;
pub open spec fn image_jpeg_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "jpeg"@,
        suffix: None,
        params: Map::empty(),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#767
// IMAGE_GIF, "image/gif", 5;
pub open spec fn image_gif_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "gif"@,
        suffix: None,
        params: Map::empty(),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#768
// IMAGE_PNG, "image/png", 5;
pub open spec fn image_png_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "png"@,
        suffix: None,
        params: Map::empty(),
    }
}
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#769
// IMAGE_BMP, "image/bmp", 5;
pub open spec fn image_bmp_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "bmp"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#779
// APPLICATION_OCTET_STREAM, "application/octet-stream", 11;
#[verifier::auto_reveal_literals(strlit)]
pub open spec fn application_octet_stream_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "octet-stream"@,
        suffix: None,
        params: Map::empty(),
    }
}

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#781
// APPLICATION_PDF, "application/pdf", 11;
pub open spec fn application_pdf_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "pdf"@,
        suffix: None,
        params: Map::empty(),
    }
}

// parse from string
// "image/x-icon"
pub open spec fn image_x_icon_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "x-icon"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "image/webp"
pub open spec fn image_webp_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "webp"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "video/webm"
pub open spec fn video_webm_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "webm"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/basic"
pub open spec fn audio_basic_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "basic"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/aiff"
pub open spec fn audio_aiff_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "aiff"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/mpeg"
pub open spec fn audio_mpeg_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "mpeg"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/ogg"
pub open spec fn application_ogg_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "ogg"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/midi"
pub open spec fn audio_midi_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "midi"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "video/avi"
pub open spec fn video_avi_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "avi"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "audio/wave"
pub open spec fn audio_wave_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "wave"@,
        suffix: None,
        params: Map::empty(),
    }
}

// "application/postscript"
pub open spec fn application_postscript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "postscript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/x-gzip"
pub open spec fn application_x_gzip_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-gzip"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/zip"
pub open spec fn application_zip_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "zip"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/x-rar-compressed"
pub open spec fn application_x_rar_compressed_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-rar-compressed"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/font-woff"
pub open spec fn application_font_woff_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "font-woff"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/font-woff2"
pub open spec fn application_font_woff2_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "font-woff2"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/font-sfnt"
pub open spec fn application_font_sfnt_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "font-sfnt"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/vnd.ms-fontobject"
pub open spec fn application_vnd_ms_fontobject_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "vnd.ms-fontobject"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "video/mp4"
pub open spec fn video_mp4_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "mp4"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/vtt"
pub open spec fn text_vtt_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "vtt"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/cache-manifest"
pub open spec fn text_cache_manifest_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "cache-manifest"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/ecmascript"
pub open spec fn application_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "ecmascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/javascript"
pub open spec fn application_javascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "javascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/x-ecmascript"
pub open spec fn application_x_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-ecmascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "application/x-javascript"
pub open spec fn application_x_javascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-javascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/ecmascript"
pub open spec fn text_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "ecmascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.0"
pub open spec fn text_javascript1_0_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.0"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.1"
pub open spec fn text_javascript1_1_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.1"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.2"
pub open spec fn text_javascript1_2_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.2"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.3"
pub open spec fn text_javascript1_3_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.3"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.4"
pub open spec fn text_javascript1_4_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.4"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/javascript1.5"
pub open spec fn text_javascript1_5_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.5"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/jscript"
pub open spec fn text_jscript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "jscript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/livescript"
pub open spec fn text_livescript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "livescript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/x-ecmascript"
pub open spec fn text_x_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "x-ecmascript"@,
        suffix: None,
        params: Map::empty(),
    }
}
// "text/x-javascript"
pub open spec fn text_x_javascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "x-javascript"@,
        suffix: None,
        params: Map::empty(),
    }
}

} // verus!
