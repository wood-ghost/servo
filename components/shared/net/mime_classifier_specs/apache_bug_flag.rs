use mime::Mime;
use vstd::prelude::*;

use crate::mime_classifier::ApacheBugFlag;

verus! {
pub open spec fn from_content_type(mime_type: Option<&Mime>) -> ApacheBugFlag {
    match mime_type {
        Some(mt) => {
            if SpecMime::is_text_plain(mt)
                || SpecMime::is_text_plain_utf8(mt)
            {
                ApacheBugFlag::On
            } else {
                ApacheBugFlag::Off
            }
        },
        None => ApacheBugFlag::Off,
    }
}
} // verus!