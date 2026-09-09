use mime::Mime;
use vstd::prelude::*;

use crate::mime_classifier::{
    NoSniffFlag,
    MimeClassifier,
};

// use crate::mime_classifier::MIMEChecker;
use crate::mime_classifier_specs::mime_api::*;
use crate::mime_classifier_specs::predicates::{
    is_xml,
};

verus! {

// ------------------------------------
// Plugin Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-plugin-context
// To determine the computed MIME type of a resource fetched in a plugin context, 
// execute the following rules for sniffing in a plugin context: 
pub open spec fn mime_classify_plugin_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    result == match supplied_type {
        // 1. If the supplied MIME type is undefined, the computed MIME type is "application/octet-stream". 
        Some(mt) => view(mt),
        // 2. The computed MIME type is the supplied MIME type. 
        None => application_octet_stream_identity(),
    }
}

// ------------------------------------
// Style Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-style-context
// To determine the computed MIME type of a resource fetched in a style context, execute the following 
// rules for sniffing in a style context: 
pub open spec fn mime_classify_style_result<'a>(
    classifier: &'a MimeClassifier,
    no_sniff_flag: NoSniffFlag,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    result == match supplied_type {
        // 1. If the supplied MIME type is undefined, …. (follow Servo behavior) 
        None => {
            if no_sniff_flag == NoSniffFlag::On {
                application_octet_stream_identity()
            } else {
                text_css_identity() 
            }
        } 
        // 2. The computed MIME type is the supplied MIME type. 
        Some(mt) => view(mt),
    }
}

// ------------------------------------
// Script Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-script-context
// To determine the computed MIME type of a resource fetched in a script context, 
// execute the following rules for sniffing in a script context: 
pub open spec fn mime_classify_script_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    data: Seq<u8>,
    result: MimeView,
) -> bool {
    result == match supplied_type {
        // 1. If the supplied MIME type is undefined, …. (follow Servo behavior) 
        None => {
           text_javascript_identity() 
        } 
        // 2. The computed MIME type is the supplied MIME type. 
        Some(mt) => view(mt),
    }
}

// ------------------------------------
// TextTrack Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-text-track-context
pub open spec fn mime_classify_text_track_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    result: MimeView,
) -> bool {
    if supplied_type is Some && is_xml(&supplied_type->Some_0) {
        view(&supplied_type->Some_0) == result
    } else {
        // The computed MIME type is "text/vtt". 
        essence_str_view(&result) == "text/vtt"@
    }
}

// ------------------------------------
// Cache Manifest Content Type Sniffing
// ------------------------------------
// https://mimesniff.spec.whatwg.org/#sniffing-in-a-cache-manifest-context
pub open spec fn mime_classify_cache_manifest_result<'a>(
    classifier: &'a MimeClassifier,
    supplied_type: &Option<Mime>,
    result: MimeView,
) -> bool {
    if supplied_type is Some && is_xml(&supplied_type->Some_0) {
        view(&supplied_type->Some_0) == result
    } else {
        // The computed MIME type is "text/cache-manifest". 
        essence_str_view(&result) == "text/cache-manifest"@
    }
}

} // verus!