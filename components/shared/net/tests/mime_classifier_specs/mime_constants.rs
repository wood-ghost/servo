/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Every public constant in mime 0.3.17, including its deprecated alias.
//! Expectations are concrete values checked against ordinary Rust execution.

#[macro_use]
#[allow(dead_code)]
mod common;

// These tables expand to the same Verus test format used by mime_api.rs.
macro_rules! name_constant {
    ($test:ident, $constant:ident, $expected:literal) => {
        test_verify_one_file! {
            #[test] $test verus_code! {
                use vstd::prelude::*;

                #[verifier::auto_reveal_strlit]
                fn test() {
                    let name = mime::$constant;
                    let text = name.as_str();
                    assert(text@ == $expected@);
                }
            } => Ok(())
        }
    };
}

macro_rules! mime_constant {
    ($test:ident, $constant:ident, $type_:literal, $subtype:literal,
     $suffix:ident $(($suffix_text:literal))?, $params:expr, $essence:literal) => {
        test_verify_one_file! {
            #[test] $test verus_code! {
                use vstd::prelude::*;

                #[allow(deprecated)]
                #[verifier::auto_reveal_strlit]
                fn test() {
                    let mime = mime::$constant;
                    let type_ = mime.type_().as_str();
                    let subtype = mime.subtype().as_str();
                    let suffix = mime.suffix();
                    let essence = mime.essence_str();
                    assert(type_@ == $type_@);
                    assert(subtype@ == $subtype@);
                    let ghost expected_suffix: Option<Seq<char>> = $suffix $(($suffix_text@))?;
                    match suffix {
                        Some(name) => {
                            let text = name.as_str();
                            assert(expected_suffix == Some(text@));
                        },
                        None => { assert(expected_suffix.is_none()); },
                    }
                    // Use the intrinsic form of =~= because nested Rust macro
                    // stringification separates Verus-only punctuation tokens.
                    assert(ext_equal(essence@, $essence@));
                    let ghost view = crate::mime_api::view(&mime);
                    assert(ext_equal(view.params, $params));
                }
            } => Ok(())
        }
    };
}

// Name constants, in the pinned crate's declaration order.
name_constant!(name_star, STAR, "*");
name_constant!(name_text, TEXT, "text");
name_constant!(name_image, IMAGE, "image");
name_constant!(name_audio, AUDIO, "audio");
name_constant!(name_video, VIDEO, "video");
name_constant!(name_application, APPLICATION, "application");
name_constant!(name_multipart, MULTIPART, "multipart");
name_constant!(name_message, MESSAGE, "message");
name_constant!(name_model, MODEL, "model");
name_constant!(name_font, FONT, "font");
name_constant!(name_plain, PLAIN, "plain");
name_constant!(name_html, HTML, "html");
name_constant!(name_xml, XML, "xml");
name_constant!(name_javascript, JAVASCRIPT, "javascript");
name_constant!(name_css, CSS, "css");
name_constant!(name_csv, CSV, "csv");
name_constant!(name_event_stream, EVENT_STREAM, "event-stream");
name_constant!(name_vcard, VCARD, "vcard");
name_constant!(name_json, JSON, "json");
name_constant!(
    name_www_form_urlencoded,
    WWW_FORM_URLENCODED,
    "x-www-form-urlencoded"
);
name_constant!(name_msgpack, MSGPACK, "msgpack");
name_constant!(name_octet_stream, OCTET_STREAM, "octet-stream");
name_constant!(name_pdf, PDF, "pdf");
name_constant!(name_woff, WOFF, "woff");
name_constant!(name_woff2, WOFF2, "woff2");
name_constant!(name_form_data, FORM_DATA, "form-data");
name_constant!(name_bmp, BMP, "bmp");
name_constant!(name_gif, GIF, "gif");
name_constant!(name_jpeg, JPEG, "jpeg");
name_constant!(name_png, PNG, "png");
name_constant!(name_svg, SVG, "svg");
name_constant!(name_basic, BASIC, "basic");
name_constant!(name_mpeg, MPEG, "mpeg");
name_constant!(name_mp4, MP4, "mp4");
name_constant!(name_ogg, OGG, "ogg");
name_constant!(name_charset, CHARSET, "charset");
name_constant!(name_boundary, BOUNDARY, "boundary");
name_constant!(name_utf_8, UTF_8, "utf-8");

// Mime constants: type, base subtype, suffix, exact parameter sequence, and essence.
mime_constant!(mime_star_star, STAR_STAR, "*", "*", None, seq![], "*/*");
mime_constant!(
    mime_text_star,
    TEXT_STAR,
    "text",
    "*",
    None,
    seq![],
    "text/*"
);
mime_constant!(
    mime_text_plain,
    TEXT_PLAIN,
    "text",
    "plain",
    None,
    seq![],
    "text/plain"
);
mime_constant!(
    mime_text_plain_utf_8,
    TEXT_PLAIN_UTF_8,
    "text",
    "plain",
    None,
    seq![("charset"@, "utf-8"@)],
    "text/plain"
);
mime_constant!(
    mime_text_html,
    TEXT_HTML,
    "text",
    "html",
    None,
    seq![],
    "text/html"
);
mime_constant!(
    mime_text_html_utf_8,
    TEXT_HTML_UTF_8,
    "text",
    "html",
    None,
    seq![("charset"@, "utf-8"@)],
    "text/html"
);
mime_constant!(
    mime_text_css,
    TEXT_CSS,
    "text",
    "css",
    None,
    seq![],
    "text/css"
);
mime_constant!(
    mime_text_css_utf_8,
    TEXT_CSS_UTF_8,
    "text",
    "css",
    None,
    seq![("charset"@, "utf-8"@)],
    "text/css"
);
mime_constant!(
    mime_text_javascript,
    TEXT_JAVASCRIPT,
    "text",
    "javascript",
    None,
    seq![],
    "text/javascript"
);
mime_constant!(
    mime_text_xml,
    TEXT_XML,
    "text",
    "xml",
    None,
    seq![],
    "text/xml"
);
mime_constant!(
    mime_text_event_stream,
    TEXT_EVENT_STREAM,
    "text",
    "event-stream",
    None,
    seq![],
    "text/event-stream"
);
mime_constant!(
    mime_text_csv,
    TEXT_CSV,
    "text",
    "csv",
    None,
    seq![],
    "text/csv"
);
mime_constant!(
    mime_text_csv_utf_8,
    TEXT_CSV_UTF_8,
    "text",
    "csv",
    None,
    seq![("charset"@, "utf-8"@)],
    "text/csv"
);
mime_constant!(
    mime_text_tab_separated_values,
    TEXT_TAB_SEPARATED_VALUES,
    "text",
    "tab-separated-values",
    None,
    seq![],
    "text/tab-separated-values"
);
mime_constant!(
    mime_text_tab_separated_values_utf_8,
    TEXT_TAB_SEPARATED_VALUES_UTF_8,
    "text",
    "tab-separated-values",
    None,
    seq![("charset"@, "utf-8"@)],
    "text/tab-separated-values"
);
mime_constant!(
    mime_text_vcard,
    TEXT_VCARD,
    "text",
    "vcard",
    None,
    seq![],
    "text/vcard"
);
mime_constant!(
    mime_image_star,
    IMAGE_STAR,
    "image",
    "*",
    None,
    seq![],
    "image/*"
);
mime_constant!(
    mime_image_jpeg,
    IMAGE_JPEG,
    "image",
    "jpeg",
    None,
    seq![],
    "image/jpeg"
);
mime_constant!(
    mime_image_gif,
    IMAGE_GIF,
    "image",
    "gif",
    None,
    seq![],
    "image/gif"
);
mime_constant!(
    mime_image_png,
    IMAGE_PNG,
    "image",
    "png",
    None,
    seq![],
    "image/png"
);
mime_constant!(
    mime_image_bmp,
    IMAGE_BMP,
    "image",
    "bmp",
    None,
    seq![],
    "image/bmp"
);
mime_constant!(
    mime_image_svg,
    IMAGE_SVG,
    "image",
    "svg",
    Some("xml"),
    seq![],
    "image/svg+xml"
);
mime_constant!(
    mime_font_woff,
    FONT_WOFF,
    "font",
    "woff",
    None,
    seq![],
    "font/woff"
);
mime_constant!(
    mime_font_woff2,
    FONT_WOFF2,
    "font",
    "woff2",
    None,
    seq![],
    "font/woff2"
);
mime_constant!(
    mime_application_json,
    APPLICATION_JSON,
    "application",
    "json",
    None,
    seq![],
    "application/json"
);
mime_constant!(
    mime_application_javascript,
    APPLICATION_JAVASCRIPT,
    "application",
    "javascript",
    None,
    seq![],
    "application/javascript"
);
mime_constant!(
    mime_application_javascript_utf_8,
    APPLICATION_JAVASCRIPT_UTF_8,
    "application",
    "javascript",
    None,
    seq![("charset"@, "utf-8"@)],
    "application/javascript"
);
mime_constant!(
    mime_application_www_form_urlencoded,
    APPLICATION_WWW_FORM_URLENCODED,
    "application",
    "x-www-form-urlencoded",
    None,
    seq![],
    "application/x-www-form-urlencoded"
);
mime_constant!(
    mime_application_octet_stream,
    APPLICATION_OCTET_STREAM,
    "application",
    "octet-stream",
    None,
    seq![],
    "application/octet-stream"
);
mime_constant!(
    mime_application_msgpack,
    APPLICATION_MSGPACK,
    "application",
    "msgpack",
    None,
    seq![],
    "application/msgpack"
);
mime_constant!(
    mime_application_pdf,
    APPLICATION_PDF,
    "application",
    "pdf",
    None,
    seq![],
    "application/pdf"
);
mime_constant!(
    mime_multipart_form_data,
    MULTIPART_FORM_DATA,
    "multipart",
    "form-data",
    None,
    seq![],
    "multipart/form-data"
);
mime_constant!(
    mime_deprecated_text_javscript,
    TEXT_JAVSCRIPT,
    "text",
    "javascript",
    None,
    seq![],
    "text/javascript"
);
