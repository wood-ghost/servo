use mime::Mime;
use vstd::prelude::*;
use vstd::std_specs::convert::FromSpecImpl;

use crate::mime_classifier::ApacheBugFlag;
use crate::mime_classifier::NoSniffFlag;
use crate::mime_classifier_specs::is_text_plain;

verus! {

pub open spec fn from_content_type(mime_type: Option<&Mime>) -> ApacheBugFlag {
    match mime_type {
        Some(mt) => {
            if is_text_plain(mt)
            {
                ApacheBugFlag::On
            } else {
                ApacheBugFlag::Off
            }
        },
        None => ApacheBugFlag::Off,
    }
}
pub open spec fn from(boolean: bool) -> NoSniffFlag {
    if boolean {
        NoSniffFlag::On
    } else {
        NoSniffFlag::Off
    }
}

impl FromSpecImpl<bool> for NoSniffFlag {
    open spec fn obeys_from_spec() -> bool {
        true
    }

    open spec fn from_spec(boolean: bool) -> NoSniffFlag {
        from(boolean)
    }
}

} // verus!