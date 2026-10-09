//! Named model values and trusted bindings for `mime` library constants.
//!
//! Each library constant's external contract accompanies its model value.
//! The final section contains identities for parsing literals, which are not
//! all public constants of the Rust library.
//! The library bindings cover all 38 `Name` constants and 33 `Mime` constants
//! (including the deprecated alias) in mime 0.3.17. These are trusted value
//! contracts, checked against ordinary-Rust observations of that pinned crate.
//! Marked blocks are for library completeness rather than classifier/proof use.

use ::mime::{self, Mime, Name};
use vstd::prelude::*;

use super::mime::{MimeView, view};
use super::name::name_identity;

verus! {

// ----------------
// Constant Identity
// -----------------

// NAME
pub open spec fn image_name() -> Seq<char> { "image"@ }
pub assume_specification [mime::IMAGE] -> (result: Name<'static>)
    ensures
        name_identity(&result) == image_name(),
;

pub open spec fn audio_name() -> Seq<char> { "audio"@ }
pub assume_specification [mime::AUDIO] -> (result: Name<'static>)
    ensures
        name_identity(&result) == audio_name(),
;

pub open spec fn video_name() -> Seq<char> { "video"@ }
pub assume_specification [mime::VIDEO] -> (result: Name<'static>)
    ensures
        name_identity(&result) == video_name(),
;

pub open spec fn xml_name() -> Seq<char> { "xml"@ }
pub assume_specification [mime::XML] -> (result: Name<'static>)
    ensures
        name_identity(&result) == xml_name(),
;

pub open spec fn application_name() -> Seq<char> { "application"@ }
pub assume_specification [mime::APPLICATION] -> (result: Name<'static>)
    ensures
        name_identity(&result) == application_name(),
;

pub open spec fn star_name() -> Seq<char> { "*"@ }
pub assume_specification [mime::STAR] -> (result: Name<'static>)
    ensures
        name_identity(&result) == star_name(),
;

pub open spec fn text_name() -> Seq<char> { "text"@ }
pub assume_specification [mime::TEXT] -> (result: Name<'static>)
    ensures
        name_identity(&result) == text_name(),
;

pub open spec fn json_name() -> Seq<char> { "json"@ }
pub assume_specification [mime::JSON] -> (result: Name<'static>)
    ensures
        name_identity(&result) == json_name(),
;

pub open spec fn font_name() -> Seq<char> { "font"@ }
pub assume_specification [mime::FONT] -> (result: Name<'static>)
    ensures
        name_identity(&result) == font_name(),
;

// =====================> Not used in mime_classifier
// Remaining names from https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#596-650
pub open spec fn multipart_name() -> Seq<char> { "multipart"@ }
pub assume_specification [mime::MULTIPART] -> (result: Name<'static>)
    ensures
        name_identity(&result) == multipart_name(),
;

pub open spec fn message_name() -> Seq<char> { "message"@ }
pub assume_specification [mime::MESSAGE] -> (result: Name<'static>)
    ensures
        name_identity(&result) == message_name(),
;

pub open spec fn model_name() -> Seq<char> { "model"@ }
pub assume_specification [mime::MODEL] -> (result: Name<'static>)
    ensures
        name_identity(&result) == model_name(),
;

pub open spec fn plain_name() -> Seq<char> { "plain"@ }
pub assume_specification [mime::PLAIN] -> (result: Name<'static>)
    ensures
        name_identity(&result) == plain_name(),
;

pub open spec fn html_name() -> Seq<char> { "html"@ }
pub assume_specification [mime::HTML] -> (result: Name<'static>)
    ensures
        name_identity(&result) == html_name(),
;

pub open spec fn javascript_name() -> Seq<char> { "javascript"@ }
pub assume_specification [mime::JAVASCRIPT] -> (result: Name<'static>)
    ensures
        name_identity(&result) == javascript_name(),
;

pub open spec fn css_name() -> Seq<char> { "css"@ }
pub assume_specification [mime::CSS] -> (result: Name<'static>)
    ensures
        name_identity(&result) == css_name(),
;

pub open spec fn csv_name() -> Seq<char> { "csv"@ }
pub assume_specification [mime::CSV] -> (result: Name<'static>)
    ensures
        name_identity(&result) == csv_name(),
;

pub open spec fn event_stream_name() -> Seq<char> { "event-stream"@ }
pub assume_specification [mime::EVENT_STREAM] -> (result: Name<'static>)
    ensures
        name_identity(&result) == event_stream_name(),
;

pub open spec fn vcard_name() -> Seq<char> { "vcard"@ }
pub assume_specification [mime::VCARD] -> (result: Name<'static>)
    ensures
        name_identity(&result) == vcard_name(),
;

pub open spec fn www_form_urlencoded_name() -> Seq<char> { "x-www-form-urlencoded"@ }
pub assume_specification [mime::WWW_FORM_URLENCODED] -> (result: Name<'static>)
    ensures
        name_identity(&result) == www_form_urlencoded_name(),
;

pub open spec fn msgpack_name() -> Seq<char> { "msgpack"@ }
pub assume_specification [mime::MSGPACK] -> (result: Name<'static>)
    ensures
        name_identity(&result) == msgpack_name(),
;

pub open spec fn octet_stream_name() -> Seq<char> { "octet-stream"@ }
pub assume_specification [mime::OCTET_STREAM] -> (result: Name<'static>)
    ensures
        name_identity(&result) == octet_stream_name(),
;

pub open spec fn pdf_name() -> Seq<char> { "pdf"@ }
pub assume_specification [mime::PDF] -> (result: Name<'static>)
    ensures
        name_identity(&result) == pdf_name(),
;

pub open spec fn woff_name() -> Seq<char> { "woff"@ }
pub assume_specification [mime::WOFF] -> (result: Name<'static>)
    ensures
        name_identity(&result) == woff_name(),
;

pub open spec fn woff2_name() -> Seq<char> { "woff2"@ }
pub assume_specification [mime::WOFF2] -> (result: Name<'static>)
    ensures
        name_identity(&result) == woff2_name(),
;

pub open spec fn form_data_name() -> Seq<char> { "form-data"@ }
pub assume_specification [mime::FORM_DATA] -> (result: Name<'static>)
    ensures
        name_identity(&result) == form_data_name(),
;

pub open spec fn bmp_name() -> Seq<char> { "bmp"@ }
pub assume_specification [mime::BMP] -> (result: Name<'static>)
    ensures
        name_identity(&result) == bmp_name(),
;

pub open spec fn gif_name() -> Seq<char> { "gif"@ }
pub assume_specification [mime::GIF] -> (result: Name<'static>)
    ensures
        name_identity(&result) == gif_name(),
;

pub open spec fn jpeg_name() -> Seq<char> { "jpeg"@ }
pub assume_specification [mime::JPEG] -> (result: Name<'static>)
    ensures
        name_identity(&result) == jpeg_name(),
;

pub open spec fn png_name() -> Seq<char> { "png"@ }
pub assume_specification [mime::PNG] -> (result: Name<'static>)
    ensures
        name_identity(&result) == png_name(),
;

pub open spec fn svg_name() -> Seq<char> { "svg"@ }
pub assume_specification [mime::SVG] -> (result: Name<'static>)
    ensures
        name_identity(&result) == svg_name(),
;

pub open spec fn basic_name() -> Seq<char> { "basic"@ }
pub assume_specification [mime::BASIC] -> (result: Name<'static>)
    ensures
        name_identity(&result) == basic_name(),
;

pub open spec fn mpeg_name() -> Seq<char> { "mpeg"@ }
pub assume_specification [mime::MPEG] -> (result: Name<'static>)
    ensures
        name_identity(&result) == mpeg_name(),
;

pub open spec fn mp4_name() -> Seq<char> { "mp4"@ }
pub assume_specification [mime::MP4] -> (result: Name<'static>)
    ensures
        name_identity(&result) == mp4_name(),
;

pub open spec fn ogg_name() -> Seq<char> { "ogg"@ }
pub assume_specification [mime::OGG] -> (result: Name<'static>)
    ensures
        name_identity(&result) == ogg_name(),
;

pub open spec fn charset_name() -> Seq<char> { "charset"@ }
pub assume_specification [mime::CHARSET] -> (result: Name<'static>)
    ensures
        name_identity(&result) == charset_name(),
;

pub open spec fn boundary_name() -> Seq<char> { "boundary"@ }
pub assume_specification [mime::BOUNDARY] -> (result: Name<'static>)
    ensures
        name_identity(&result) == boundary_name(),
;

pub open spec fn utf_8_name() -> Seq<char> { "utf-8"@ }
pub assume_specification [mime::UTF_8] -> (result: Name<'static>)
    ensures
        name_identity(&result) == utf_8_name(),
;
// <===================== Not used in mime_classifier

// ----------------
// Constant
// -----------------

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#750
// TEXT_PLAIN, "text/plain", 4;
pub open spec fn text_plain_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "plain"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_PLAIN] -> (result: Mime)
    ensures
        view(&result) == text_plain_identity(),
;

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#751
// TEXT_PLAIN_UTF_8, "text/plain; charset=utf-8", 4, None, 10;
pub open spec fn text_plain_utf_8_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "plain"@,
        suffix: None,
        params: seq![("charset"@, "utf-8"@)],
    }
}

pub(crate) assume_specification [mime::TEXT_PLAIN_UTF_8] -> (result: Mime)
    ensures
        view(&result) == text_plain_utf_8_identity(),
;

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#752
// TEXT_HTML, "text/html", 4;
pub open spec fn text_html_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "html"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_HTML] -> (result: Mime)
    ensures
        view(&result) == text_html_identity(),
;

// =====================> Not used in mime_classifier
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#753
// TEXT_HTML_UTF_8, "text/html; charset=utf-8", 4, None, 9;
pub open spec fn text_html_utf_8_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "html"@,
        suffix: None,
        params: seq![("charset"@, "utf-8"@)],
    }
}

pub(crate) assume_specification [mime::TEXT_HTML_UTF_8] -> (result: Mime)
    ensures
        view(&result) == text_html_utf_8_identity(),
;
// <===================== Not used in mime_classifier

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#754
// TEXT_CSS, "text/css", 4;
pub open spec fn text_css_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "css"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_CSS] -> (result: Mime)
    ensures
        view(&result) == text_css_identity(),
;

// =====================> Not used in mime_classifier
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#755
// TEXT_CSS_UTF_8, "text/css; charset=utf-8", 4, None, 8;
pub open spec fn text_css_utf_8_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "css"@,
        suffix: None,
        params: seq![("charset"@, "utf-8"@)],
    }
}

pub(crate) assume_specification [mime::TEXT_CSS_UTF_8] -> (result: Mime)
    ensures
        view(&result) == text_css_utf_8_identity(),
;
// <===================== Not used in mime_classifier

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#756
// TEXT_JAVASCRIPT, "text/javascript", 4;
pub open spec fn text_javascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_JAVASCRIPT] -> (result: Mime)
    ensures
        view(&result) == text_javascript_identity(),
;

// =====================> Not used in mime_classifier
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#786-788
// Deprecated compatibility spelling: TEXT_JAVSCRIPT = TEXT_JAVASCRIPT.
#[allow(deprecated)]
pub(crate) assume_specification [mime::TEXT_JAVSCRIPT] -> (result: Mime)
    ensures
        view(&result) == text_javascript_identity(),
;
// <===================== Not used in mime_classifier

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#757
// TEXT_XML, "text/xml", 4;
pub open spec fn text_xml_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "xml"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub assume_specification [mime::TEXT_XML] -> (result: Mime)
    ensures
        view(&result) == text_xml_identity(),
;

// =====================> Not used in mime_classifier
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#758-763
pub open spec fn text_event_stream_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "event-stream"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_EVENT_STREAM] -> (result: Mime)
    ensures
        view(&result) == text_event_stream_identity(),
;

pub open spec fn text_csv_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "csv"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_CSV] -> (result: Mime)
    ensures
        view(&result) == text_csv_identity(),
;

pub open spec fn text_csv_utf_8_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "csv"@,
        suffix: None,
        params: seq![("charset"@, "utf-8"@)],
    }
}

pub(crate) assume_specification [mime::TEXT_CSV_UTF_8] -> (result: Mime)
    ensures
        view(&result) == text_csv_utf_8_identity(),
;

pub open spec fn text_tab_separated_values_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "tab-separated-values"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_TAB_SEPARATED_VALUES] -> (result: Mime)
    ensures
        view(&result) == text_tab_separated_values_identity(),
;

pub open spec fn text_tab_separated_values_utf_8_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "tab-separated-values"@,
        suffix: None,
        params: seq![("charset"@, "utf-8"@)],
    }
}

pub(crate) assume_specification [mime::TEXT_TAB_SEPARATED_VALUES_UTF_8] -> (result: Mime)
    ensures
        view(&result) == text_tab_separated_values_utf_8_identity(),
;

pub open spec fn text_vcard_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "vcard"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_VCARD] -> (result: Mime)
    ensures
        view(&result) == text_vcard_identity(),
;
// <===================== Not used in mime_classifier

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#766
// IMAGE_JPEG, "image/jpeg", 5;
pub open spec fn image_jpeg_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "jpeg"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::IMAGE_JPEG] -> (result: Mime)
    ensures
        view(&result) == image_jpeg_identity(),
;

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#767
// IMAGE_GIF, "image/gif", 5;
pub open spec fn image_gif_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "gif"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::IMAGE_GIF] -> (result: Mime)
    ensures
        view(&result) == image_gif_identity(),
;

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#768
// IMAGE_PNG, "image/png", 5;
pub open spec fn image_png_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "png"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::IMAGE_PNG] -> (result: Mime)
    ensures
        view(&result) == image_png_identity(),
;

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#769
// IMAGE_BMP, "image/bmp", 5;
pub open spec fn image_bmp_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "bmp"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::IMAGE_BMP] -> (result: Mime)
    ensures
        view(&result) == image_bmp_identity(),
;

// =====================> Not used in mime_classifier
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#770
// IMAGE_SVG, "image/svg+xml", 5, Some(9);
pub open spec fn image_svg_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "svg"@,
        suffix: Some("xml"@),
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::IMAGE_SVG] -> (result: Mime)
    ensures
        view(&result) == image_svg_identity(),
;

// Wildcards: https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#746-765
pub open spec fn star_star_identity() -> MimeView {
    MimeView {
        type_: "*"@,
        subtype: "*"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::STAR_STAR] -> (result: Mime)
    ensures
        view(&result) == star_star_identity(),
;

pub open spec fn text_star_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "*"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::TEXT_STAR] -> (result: Mime)
    ensures
        view(&result) == text_star_identity(),
;

pub open spec fn image_star_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "*"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::IMAGE_STAR] -> (result: Mime)
    ensures
        view(&result) == image_star_identity(),
;

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#772-773
pub open spec fn font_woff_identity() -> MimeView {
    MimeView {
        type_: "font"@,
        subtype: "woff"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::FONT_WOFF] -> (result: Mime)
    ensures
        view(&result) == font_woff_identity(),
;

pub open spec fn font_woff2_identity() -> MimeView {
    MimeView {
        type_: "font"@,
        subtype: "woff2"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::FONT_WOFF2] -> (result: Mime)
    ensures
        view(&result) == font_woff2_identity(),
;

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#775-778
pub open spec fn application_json_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "json"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::APPLICATION_JSON] -> (result: Mime)
    ensures
        view(&result) == application_json_identity(),
;
// <===================== Not used in mime_classifier

// "application/javascript"
// Also used by the classifier's parsing contract in parser.rs.
pub open spec fn application_javascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "javascript"@,
        suffix: None,
        params: Seq::empty(),
    }
}

// =====================> Not used in mime_classifier
pub(crate) assume_specification [mime::APPLICATION_JAVASCRIPT] -> (result: Mime)
    ensures
        view(&result) == application_javascript_identity(),
;

pub open spec fn application_javascript_utf_8_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "javascript"@,
        suffix: None,
        params: seq![("charset"@, "utf-8"@)],
    }
}

pub(crate) assume_specification [mime::APPLICATION_JAVASCRIPT_UTF_8] -> (result: Mime)
    ensures
        view(&result) == application_javascript_utf_8_identity(),
;

pub open spec fn application_www_form_urlencoded_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-www-form-urlencoded"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::APPLICATION_WWW_FORM_URLENCODED] -> (result: Mime)
    ensures
        view(&result) == application_www_form_urlencoded_identity(),
;
// <===================== Not used in mime_classifier

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#779
// APPLICATION_OCTET_STREAM, "application/octet-stream", 11;
pub open spec fn application_octet_stream_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "octet-stream"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::APPLICATION_OCTET_STREAM] -> (result: Mime)
    ensures
        view(&result) == application_octet_stream_identity(),
;

// =====================> Not used in mime_classifier
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#780
pub open spec fn application_msgpack_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "msgpack"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::APPLICATION_MSGPACK] -> (result: Mime)
    ensures
        view(&result) == application_msgpack_identity(),
;
// <===================== Not used in mime_classifier

// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#781
// APPLICATION_PDF, "application/pdf", 11;
pub open spec fn application_pdf_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "pdf"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::APPLICATION_PDF] -> (result: Mime)
    ensures
        view(&result) == application_pdf_identity(),
;

// =====================> Not used in mime_classifier
// https://docs.rs/mime/0.3.17/src/mime/lib.rs.html#783
pub open spec fn multipart_form_data_identity() -> MimeView {
    MimeView {
        type_: "multipart"@,
        subtype: "form-data"@,
        suffix: None,
        params: Seq::empty(),
    }
}

pub(crate) assume_specification [mime::MULTIPART_FORM_DATA] -> (result: Mime)
    ensures
        view(&result) == multipart_form_data_identity(),
;
// <===================== Not used in mime_classifier

// parse from string
// Canonical values for supported parsing literals, not additional library constants.
// "image/x-icon"
pub open spec fn image_x_icon_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "x-icon"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "image/webp"
pub open spec fn image_webp_identity() -> MimeView {
    MimeView {
        type_: "image"@,
        subtype: "webp"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "video/webm"
pub open spec fn video_webm_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "webm"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "audio/basic"
pub open spec fn audio_basic_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "basic"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "audio/aiff"
pub open spec fn audio_aiff_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "aiff"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "audio/mpeg"
pub open spec fn audio_mpeg_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "mpeg"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/ogg"
pub open spec fn application_ogg_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "ogg"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "audio/midi"
pub open spec fn audio_midi_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "midi"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "video/avi"
pub open spec fn video_avi_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "avi"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "audio/wave"
pub open spec fn audio_wave_identity() -> MimeView {
    MimeView {
        type_: "audio"@,
        subtype: "wave"@,
        suffix: None,
        params: Seq::empty(),
    }
}

// "application/postscript"
pub open spec fn application_postscript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "postscript"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/x-gzip"
pub open spec fn application_x_gzip_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-gzip"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/zip"
pub open spec fn application_zip_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "zip"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/x-rar-compressed"
pub open spec fn application_x_rar_compressed_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-rar-compressed"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/font-woff"
pub open spec fn application_font_woff_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "font-woff"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/font-woff2"
pub open spec fn application_font_woff2_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "font-woff2"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/font-sfnt"
pub open spec fn application_font_sfnt_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "font-sfnt"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/vnd.ms-fontobject"
pub open spec fn application_vnd_ms_fontobject_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "vnd.ms-fontobject"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "video/mp4"
pub open spec fn video_mp4_identity() -> MimeView {
    MimeView {
        type_: "video"@,
        subtype: "mp4"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/vtt"
pub open spec fn text_vtt_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "vtt"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/cache-manifest"
pub open spec fn text_cache_manifest_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "cache-manifest"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/ecmascript"
pub open spec fn application_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "ecmascript"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/x-ecmascript"
pub open spec fn application_x_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-ecmascript"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "application/x-javascript"
pub open spec fn application_x_javascript_identity() -> MimeView {
    MimeView {
        type_: "application"@,
        subtype: "x-javascript"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/ecmascript"
pub open spec fn text_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "ecmascript"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/javascript1.0"
pub open spec fn text_javascript1_0_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.0"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/javascript1.1"
pub open spec fn text_javascript1_1_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.1"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/javascript1.2"
pub open spec fn text_javascript1_2_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.2"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/javascript1.3"
pub open spec fn text_javascript1_3_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.3"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/javascript1.4"
pub open spec fn text_javascript1_4_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.4"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/javascript1.5"
pub open spec fn text_javascript1_5_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "javascript1.5"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/jscript"
pub open spec fn text_jscript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "jscript"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/livescript"
pub open spec fn text_livescript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "livescript"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/x-ecmascript"
pub open spec fn text_x_ecmascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "x-ecmascript"@,
        suffix: None,
        params: Seq::empty(),
    }
}
// "text/x-javascript"
pub open spec fn text_x_javascript_identity() -> MimeView {
    MimeView {
        type_: "text"@,
        subtype: "x-javascript"@,
        suffix: None,
        params: Seq::empty(),
    }
}

} // verus!
