use mime::Mime;
use vstd::prelude::*;

use crate::mime_classifier::ApacheBugFlag;
use crate::mime_classifier_specs::{
    is_text_plain,
    is_text_plain_utf8
};

verus! {
pub open spec fn from_content_type(mime_type: Option<&Mime>) -> ApacheBugFlag {
    match mime_type {
        Some(mt) => {
            if is_text_plain(mt)
                || is_text_plain_utf8(mt)
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