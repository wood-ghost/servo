/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
// use verus_state_machines_macros::tokenized_state_machine;
// use std::sync::Arc;
// use vstd::atomic_ghost::*;
// use vstd::modes::*;
use vstd::prelude::*;
// use vstd::thread::*;
// use vstd::{pervasive::*, prelude::*, *};


use mime::{self, Mime, Name};

use crate::LoadContext;

use crate::mime_classifier_specs::{
    model as Model,
    predicates as Spec,
    apache_bug_flag as SpecApacheBugFlag,
    classifier as SpecClassifier,
};

verus! {

pub struct MimeClassifier {
    image_classifier: GroupedClassifier,
    audio_video_classifier: GroupedClassifier,
    scriptable_classifier: GroupedClassifier,
    plaintext_classifier: GroupedClassifier,
    archive_classifier: GroupedClassifier,
    binary_or_plaintext: BinaryOrPlaintextClassifier,
    font_classifier: GroupedClassifier,
}

#[derive(PartialEq)]
pub enum MediaType {
    Xml,
    Html,
    AudioVideo,
    Image,
    JavaScript,
    Json,
    Font,
    Text,
    Css,
}

#[derive(PartialEq)]
pub enum ApacheBugFlag {
    On,
    Off,
}

impl ApacheBugFlag {
    #[verifier::external_body]
    fn mime_equal(a: &Mime, b: Mime) -> (result: bool)
        ensures
            result == (Model::mime_identity(a) == Model::mime_identity(&b)),
    {
        *a == b
    }
    /// <https://mimesniff.spec.whatwg.org/#supplied-mime-type-detection-algorithm>
    pub fn from_content_type(mime_type: Option<&Mime>) -> (result: ApacheBugFlag)
        ensures
            result == SpecApacheBugFlag::from_content_type(mime_type),
    {
        // TODO(36801): also handle charset ISO-8859-1
        if mime_type.is_some_and(|mime_type| -> (r: bool)
            ensures 
                r == (Spec::is_text_plain(mime_type) || Spec::is_text_plain_utf8(mime_type))
            {
            // *mime_type == mime::TEXT_PLAIN || *mime_type == mime::TEXT_PLAIN_UTF_8
            Self::mime_equal(mime_type, mime::TEXT_PLAIN) || Self::mime_equal(mime_type, mime::TEXT_PLAIN_UTF_8) 
        }) {
            ApacheBugFlag::On
        } else {
            ApacheBugFlag::Off
        }
    }
}

#[derive(PartialEq)]
pub enum NoSniffFlag {
    On,
    Off,
}

impl From<bool> for NoSniffFlag {
    #[verifier::external_body] //TODO:
    fn from(boolean: bool) -> Self {
        if boolean {
            NoSniffFlag::On
        } else {
            NoSniffFlag::Off
        }
    }
}

impl Default for MimeClassifier {
    fn default() -> Self {
        Self {
            image_classifier: GroupedClassifier::image_classifer(),
            audio_video_classifier: GroupedClassifier::audio_video_classifier(),
            scriptable_classifier: GroupedClassifier::scriptable_classifier(),
            plaintext_classifier: GroupedClassifier::plaintext_classifier(),
            archive_classifier: GroupedClassifier::archive_classifier(),
            binary_or_plaintext: BinaryOrPlaintextClassifier,
            font_classifier: GroupedClassifier::font_classifier(),
        }
    }
}


impl MimeClassifier {
    /// <https://mimesniff.spec.whatwg.org/#mime-type-sniffing-algorithm>
    #[verifier::external_body]
    pub fn classify<'a>(
        &'a self,
        context: LoadContext,
        no_sniff_flag: NoSniffFlag,
        apache_bug_flag: ApacheBugFlag,
        supplied_type: &Option<Mime>,
        data: &'a [u8],
    ) -> (result: Mime)
        requires
            SpecClassifier::classify_input_is_valid(context, no_sniff_flag, apache_bug_flag, supplied_type, data),
        ensures
            result == SpecClassifier::classify(context, no_sniff_flag, apache_bug_flag, supplied_type, data),
    {
        let supplied_type_or_octet_stream = supplied_type
            .clone()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);
        // Step 1. If the supplied MIME type is an XML MIME type or HTML MIME type,
        // the computed MIME type is the supplied MIME type.
        if Self::is_xml(&supplied_type_or_octet_stream) ||
            Self::is_html(&supplied_type_or_octet_stream)
        {
            return supplied_type_or_octet_stream;
        }
        match context {
            LoadContext::Browsing => match *supplied_type {
                // Step 2. If the supplied MIME type is undefined or if the supplied MIME type’s essence is "unknown/unknown",
                // "application/unknown", or "*/*", execute the rules for identifying
                // an unknown MIME type with the sniff-scriptable flag equal to the inverse of the no-sniff flag and abort these steps.
                None => self.sniff_unknown_type(no_sniff_flag, data),
                Some(ref supplied_type) => {
                    if MimeClassifier::is_explicit_unknown(supplied_type) {
                        return self.sniff_unknown_type(no_sniff_flag, data);
                    }
                    // Step 3. If the no-sniff flag is set, the computed MIME type is the supplied MIME type.
                    // Abort these steps.
                    if no_sniff_flag == NoSniffFlag::On {
                        return supplied_type.clone();
                    }
                    // Step 4. If the check-for-apache-bug flag is set,
                    // execute the rules for distinguishing if a resource is text or binary and abort these steps.
                    if apache_bug_flag == ApacheBugFlag::On {
                        return self.sniff_text_or_data(data);
                    }
                    assert(MimeClassifier::get_media_type(supplied_type) != Some(MediaType::Html));
                    assert(MimeClassifier::get_media_type(supplied_type) != Some(MediaType::Xml));
                    match MimeClassifier::get_media_type(supplied_type) {
                        // Step 5. If the supplied MIME type is an image MIME type supported by the user agent,
                        // let matched-type be the result of executing the image type pattern matching algorithm with
                        // the resource header as the byte sequence to be matched.
                        Some(MediaType::Image) => {
                            // Step 6. If matched-type is not undefined, the computed MIME type is matched-type.
                            self.image_classifier.classify(data)
                        },
                        // Step 7. If the supplied MIME type is an audio or video MIME type supported by the user agent,
                        // let matched-type be the result of executing the audio or video type pattern matching algorithm
                        // with the resource header as the byte sequence to be matched.
                        Some(MediaType::AudioVideo) => {
                            // Step 8. If matched-type is not undefined, the computed MIME type is matched-type.
                            self.audio_video_classifier.classify(data)
                        },
                        // Some(MediaType::Html) | Some(MediaType::Xml) => unreachable!(),
                        _ => None,
                    }
                    // Step 9. The computed MIME type is the supplied MIME type.
                    .unwrap_or(supplied_type.clone())
                },
            },
            LoadContext::Image => {
                // Section 8.2 Sniffing an image context
                match MimeClassifier::maybe_get_media_type(supplied_type) {
                    Some(MediaType::Xml) => None,
                    _ => self.image_classifier.classify(data),
                }
                .unwrap_or(supplied_type_or_octet_stream)
            },
            LoadContext::AudioVideo => {
                // Section 8.3 Sniffing an image context
                match MimeClassifier::maybe_get_media_type(supplied_type) {
                    Some(MediaType::Xml) => None,
                    _ => self.audio_video_classifier.classify(data),
                }
                .unwrap_or(supplied_type_or_octet_stream)
            },
            LoadContext::Plugin => {
                // 8.4 Sniffing in a plugin context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                match *supplied_type {
                    None => mime::APPLICATION_OCTET_STREAM,
                    _ => supplied_type_or_octet_stream,
                }
            },
            LoadContext::Style => {
                // 8.5 Sniffing in a style context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                supplied_type.clone().unwrap_or_else(|| {
                    if no_sniff_flag == NoSniffFlag::On {
                        mime::APPLICATION_OCTET_STREAM
                    } else {
                        mime::TEXT_CSS
                    }
                })
            },
            LoadContext::Script => {
                // 8.6 Sniffing in a script context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                match *supplied_type {
                    None => mime::TEXT_JAVASCRIPT,
                    _ => supplied_type_or_octet_stream,
                }
            },
            LoadContext::Font => {
                // 8.7 Sniffing in a font context
                match MimeClassifier::maybe_get_media_type(supplied_type) {
                    Some(MediaType::Xml) => None,
                    _ => self.font_classifier.classify(data),
                }
                .unwrap_or(supplied_type_or_octet_stream)
            },
            LoadContext::TextTrack => {
                // 8.8 Sniffing in a text track context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                "text/vtt".parse().unwrap()
            },
            LoadContext::CacheManifest => {
                // 8.9 Sniffing in a cache manifest context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                "text/cache-manifest".parse().unwrap()
            },
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.image_classifier.validate()?;
        self.audio_video_classifier.validate()?;
        self.scriptable_classifier.validate()?;
        self.plaintext_classifier.validate()?;
        self.archive_classifier.validate()?;
        self.binary_or_plaintext.validate()?;
        self.font_classifier.validate()?;
        Ok(())
    }

    // some sort of iterator over the classifiers might be better?
    #[verifier::external_body] //TODO:
    fn sniff_unknown_type(&self, no_sniff_flag: NoSniffFlag, data: &[u8]) -> Mime {
        let should_sniff_scriptable = no_sniff_flag == NoSniffFlag::Off;
        let sniffed = if should_sniff_scriptable {
            self.scriptable_classifier.classify(data)
        } else {
            None
        };

        sniffed
            .or_else(|| self.plaintext_classifier.classify(data))
            .or_else(|| self.image_classifier.classify(data))
            .or_else(|| self.audio_video_classifier.classify(data))
            .or_else(|| self.archive_classifier.classify(data))
            .or_else(|| self.binary_or_plaintext.classify(data))
            .expect("BinaryOrPlaintextClassifier always succeeds")
    }

    #[verifier::external_body] //TODO:
    fn sniff_text_or_data<'a>(&'a self, data: &'a [u8]) -> Mime {
        self.binary_or_plaintext
            .classify(data)
            .expect("BinaryOrPlaintextClassifier always succeeds")
    }
    
    #[verifier::external_body]
    fn str_equal(a: &str, b: &str) -> (result: bool)
        ensures
            result == (a@ =~= b@),
    {
        a == b
    }

    #[verifier::external_body] // For type_
    fn name_equal<'a, 'b>(a: Name<'a>, b: Name<'b>) -> (result: bool)
        ensures
            result == (Model::name_text(a) =~= Model::name_text(b)),
    {
        a == b
    }

    #[verifier::external_body]
    fn suffix_equal<'a, 'b>(a: Option<Name<'a>>, b: Option<Name<'b>>) -> (result: bool)
        ensures
            result == match (a, b) {
                    (Some(a_name), Some(b_name)) =>
                        Model::name_text(a_name) =~= Model::name_text(b_name),
                    (None, None) => true,
                    _ => false,
                },
    {
        a == b
    }

    /// <https://mimesniff.spec.whatwg.org/#xml-mime-type>
    /// SVG is worth distinguishing from other XML MIME types:
    /// <https://mimesniff.spec.whatwg.org/#mime-type-miscellaneous>
    fn is_xml(mt: &Mime) -> (result: bool) 
        ensures
            result == Spec::is_xml(mt),
    {
        !Self::is_image(mt) &&
            // (mt.suffix() == Some(mime::XML) ||
            (Self::suffix_equal(mt.suffix(), Some(mime::XML)) ||
                // mt.essence_str() == "text/xml" ||
                Self::str_equal(mt.essence_str(), "text/xml") ||
                // mt.essence_str() == "application/xml")
                Self::str_equal(mt.essence_str(), "application/xml"))
    }

    /// <https://mimesniff.spec.whatwg.org/#html-mime-type>
    fn is_html(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_html(mt),
    {
        // mt.essence_str() == "text/html"
        Self::str_equal(mt.essence_str(), "text/html")
    }

    /// <https://mimesniff.spec.whatwg.org/#image-mime-type>
    fn is_image(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_image(mt),
    {
        // mt.type_() == mime::IMAGE
        Self::name_equal(mt.type_(), mime::IMAGE)
    }

    /// <https://mimesniff.spec.whatwg.org/#audio-or-video-mime-type>
    fn is_audio_video(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_audio_video(mt),
    {
        // mt.type_() == mime::AUDIO ||
        Self::name_equal(mt.type_(), mime::AUDIO) ||
            // mt.type_() == mime::VIDEO ||
            Self::name_equal(mt.type_(), mime::VIDEO) ||
            // mt.essence_str() == "application/ogg"
            Self::str_equal(mt.essence_str(), "application/ogg")
    }

    fn is_explicit_unknown(mt: &Mime) -> bool {
        mt.type_().as_str() == "unknown" && mt.subtype().as_str() == "unknown" ||
            mt.type_() == mime::APPLICATION && mt.subtype().as_str() == "unknown" ||
            mt.type_() == mime::STAR && mt.subtype() == mime::STAR
    }

    /// <https://mimesniff.spec.whatwg.org/#javascript-mime-type>
    pub fn is_javascript(mt: &Mime) -> bool {
        (mt.type_() == mime::APPLICATION &&
            (["ecmascript", "javascript", "x-ecmascript", "x-javascript"]
                .contains(&mt.subtype().as_str()))) ||
            (mt.type_() == mime::TEXT &&
                ([
                    "ecmascript",
                    "javascript",
                    "javascript1.0",
                    "javascript1.1",
                    "javascript1.2",
                    "javascript1.3",
                    "javascript1.4",
                    "javascript1.5",
                    "jscript",
                    "livescript",
                    "x-ecmascript",
                    "x-javascript",
                ]
                .contains(&mt.subtype().as_str())))
    }

    /// <https://mimesniff.spec.whatwg.org/#json-mime-type>
    pub fn is_json(mt: &Mime) -> bool {
        mt.suffix() == Some(mime::JSON) ||
            mt.essence_str() == "application/json" ||
            mt.essence_str() == "text/json"
    }

    /// <https://mimesniff.spec.whatwg.org/#font-mime-type>
    fn is_font(mt: &Mime) -> bool {
        mt.type_() == mime::FONT ||
            (mt.type_() == mime::APPLICATION &&
                ([
                    "font-cff",
                    "font-off",
                    "font-sfnt",
                    "font-ttf",
                    "font-woff",
                    "vnd.ms-fontobject",
                    "vnd.ms-opentype",
                ]
                .contains(&mt.subtype().as_str())))
    }

    fn is_text(mt: &Mime) -> bool {
        *mt == mime::TEXT_PLAIN || mt.essence_str() == "text/vtt"
    }

    pub fn is_css(mt: &Mime) -> bool {
        mt.essence_str() == "text/css"
    }

    pub fn get_media_type(mime: &Mime) -> Option<MediaType> {
        if MimeClassifier::is_xml(mime) {
            Some(MediaType::Xml)
        } else if MimeClassifier::is_html(mime) {
            Some(MediaType::Html)
        } else if MimeClassifier::is_image(mime) {
            Some(MediaType::Image)
        } else if MimeClassifier::is_audio_video(mime) {
            Some(MediaType::AudioVideo)
        } else if MimeClassifier::is_javascript(mime) {
            Some(MediaType::JavaScript)
        } else if MimeClassifier::is_font(mime) {
            Some(MediaType::Font)
        } else if MimeClassifier::is_json(mime) {
            Some(MediaType::Json)
        } else if MimeClassifier::is_text(mime) {
            Some(MediaType::Text)
        } else if MimeClassifier::is_css(mime) {
            Some(MediaType::Css)
        } else {
            None
        }
    }

    fn maybe_get_media_type(supplied_type: &Option<Mime>) -> Option<MediaType> {
        supplied_type
            .as_ref()
            .and_then(MimeClassifier::get_media_type)
    }
}

// Interface used for composite types
trait MIMEChecker {
    fn classify(&self, data: &[u8]) -> Option<Mime>;
    /// Validate the MIME checker configuration
    fn validate(&self) -> Result<(), String>;
}

struct ByteMatcher {
    pattern: &'static [u8],
    mask: &'static [u8],
    leading_ignore: &'static [u8],
    content_type: Mime,
}

impl ByteMatcher {
    fn matches(&self, data: &[u8]) -> Option<usize> 
        requires
            self.pattern.len() > 0,
            self.pattern.len() == self.mask.len(),
    {
        if data.len() < self.pattern.len() {
            return None;
        } else if data == self.pattern {
            return Some(self.pattern.len());
        } else {
            let max_start = data.len() - self.pattern.len();
            let mut start: usize = 0;

            while start <= max_start
                invariant
                    start <= max_start + 1,
                    self.pattern.len() <= data.len(),
                    self.pattern.len() == self.mask.len(),
                    max_start == data.len() - self.pattern.len(), 
                    self.pattern.len() > 0,
                decreases
                    max_start + 1 - start,
            {
                if !self.leading_ignore.contains(&data[start]) {
                    assert(start + self.pattern.len() <= data.len());
                    let mut i: usize = 0;
                    while i < self.pattern.len()
                        invariant
                            i <= self.pattern.len(),
                            self.pattern.len() == self.mask.len(),
                            start + self.pattern.len() <= data.len(),
                        decreases
                            self.pattern.len() - i,
                    {
                        if (data[start + i] & self.mask[i]) != self.pattern[i] {
                            return None;
                        }
                        i += 1;
                    }
                    return Some(start + self.pattern.len());
                }
                start += 1;
            }
            return None;
        }
        // } else {
        //     data[..data.len() - self.pattern.len() + 1]
        //         .iter()
        //         .position(|x| !self.leading_ignore.contains(x))
        
        //         .and_then(|start| {
        //             if data[start..]
        //                 .iter()
        //                 .zip(self.pattern.iter())
        //                 .zip(self.mask.iter())
        //                 // .all(|((&data, &pattern), &mask)| (data & mask) == pattern)
        //                 .all(|x| {
        //                     let ((data_p, pattern_p), mask_p) = x;
        //                     (*data_p & *mask_p) == *pattern_p
        //                 })
        //             {
        //                 Some(start + self.pattern.len())
        //             } else {
        //                 None
        //             }
        //         })
        //     }
    }
    
}

#[verifier::external_body]
fn MIMEChecker_validate_empty_pattern_error(mt: &Mime) -> (result: String) {
    format!("Zero length pattern for {:?}", mt)
}
#[verifier::external_body]
fn MIMEChecker_validate_unequal_pattern_error(mt: &Mime) -> (result: String) {
    format!("Unequal pattern and mask length for {:?}", mt)
}
#[verifier::external_body]
fn MIMEChecker_validate_premasked_error(mt: &Mime) -> (result: String) {
    format!("Pattern not pre-masked for {:?}", mt)
}

impl MIMEChecker for ByteMatcher {
    #[verifier::external_body] //TODO:
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        // self.matches(data).map(|_| self.content_type.clone())
        self.matches(data).map(|x| self.content_type.clone())
    }

    fn validate(&self) -> Result<(), String> {
        if self.pattern.is_empty() {
            // return Err(format!("Zero length pattern for {:?}", self.content_type));
            return Err(MIMEChecker_validate_empty_pattern_error(&self.content_type));
        }
        if self.pattern.len() != self.mask.len() {
            // return Err(format!(
            //     "Unequal pattern and mask length for {:?}",
            //     self.content_type
            // ));
            return Err(MIMEChecker_validate_unequal_pattern_error(&self.content_type));
        }

        let mut i: usize = 0;

        while i < self.pattern.len()
            invariant
                i <= self.pattern.len(),
                self.pattern.len() == self.mask.len(),
            decreases
                self.pattern.len() - i,
        {
            if self.pattern[i] & self.mask[i] != self.pattern[i] {
                return Err(MIMEChecker_validate_premasked_error(
                    &self.content_type,
                ));
            }

            i += 1;
        }

        // if self
        //     .pattern
        //     .iter()
        //     .zip(self.mask.iter())
        //     .any(|(&pattern, &mask)| pattern & mask != pattern)
        // {
        //     // return Err(format!(
        //     //     "Pattern not pre-masked for {:?}",
        //     //     self.content_type
        //     // ));
        //     return Err(MIMEChecker_validate_premasked_error(&self.content_type));
        // }
        Ok(())
    }
}

struct TagTerminatedByteMatcher {
    matcher: ByteMatcher,
}

#[verifier::external_body]
fn equal_b_space_or_g (d: u8) -> (result: bool) 
    ensures
        result == Spec::equal_b_space_or_g(d),
{
    d == b' ' || d == b'>'
}

impl MIMEChecker for TagTerminatedByteMatcher {
    #[verifier::external_body] //TODO:
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        self.matcher.matches(data).and_then(|j| {
            // if j < data.len() && (data[j] == b' ' || data[j] == b'>') {
            if j < data.len() && (equal_b_space_or_g(data[j])) {
                Some(self.matcher.content_type.clone())
            } else {
                None
            }
        })
    }

    fn validate(&self) -> Result<(), String> {
        self.matcher.validate()
    }
}

pub struct Mp4Matcher;

impl Mp4Matcher {
    /// <https://mimesniff.spec.whatwg.org/#matches-the-signature-for-mp4>
    #[verifier::external_body] //TODO:
    pub fn matches(&self, data: &[u8]) -> bool {
        // Step 1. Let sequence be the byte sequence to be matched,
        // where sequence[s] is byte s in sequence and sequence[0] is the first byte in sequence.
        // Step 2. Let length be the number of bytes in sequence.
        // Step 3. If length is less than 12, return false.
        if data.len() < 12 {
            return false;
        }

        // Step 4. Let box-size be the four bytes from sequence[0] to sequence[3],
        // interpreted as a 32-bit unsigned big-endian integer.
        let box_size = (((data[0] as u32) << 24) |
            ((data[1] as u32) << 16) |
            ((data[2] as u32) << 8) |
            (data[3] as u32)) as usize;
        // Step 5. If length is less than box-size or if box-size modulo 4 is not equal to 0, return false.
        if (data.len() < box_size) || !box_size.is_multiple_of(4) {
            return false;
        }

        // Step 6. If the four bytes from sequence[4] to sequence[7] are not equal to 0x66 0x74 0x79 0x70 ("ftyp"), return false.
        let ftyp = [0x66, 0x74, 0x79, 0x70];
        if !data[4..].starts_with(&ftyp) {
            return false;
        }

        // Step 7. If the three bytes from sequence[8] to sequence[10] are equal to 0x6D 0x70 0x34 ("mp4"), return true.
        let mp4 = [0x6D, 0x70, 0x34];
        data[8..].starts_with(&mp4) ||
        // Step 8. Let bytes-read be 16.
        // Step 9. While bytes-read is less than box-size, continuously loop through these steps:
            data[16..box_size]
            // Step 11. Increment bytes-read by 4.
                .chunks(4)
                // Step 10. If the three bytes from sequence[bytes-read] to sequence[bytes-read + 2]
                // are equal to 0x6D 0x70 0x34 ("mp4"), return true.
                .any(|chunk| chunk.starts_with(&mp4))
        // Step 12. Return false.
    }
}
impl MIMEChecker for Mp4Matcher {
    #[verifier::external_body]
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        if self.matches(data) {
            Some("video/mp4".parse().unwrap())
        } else {
            None
        }
    }

    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

struct BinaryOrPlaintextClassifier;

impl BinaryOrPlaintextClassifier {
    /// <https://mimesniff.spec.whatwg.org/#rules-for-text-or-binary>
    fn classify_impl(&self, data: &[u8]) -> Mime {
        // Step 1. Let length be the number of bytes in the resource header.
        // Step 2. If length is greater than or equal to 2 and
        // the first 2 bytes of the resource header are equal to 0xFE 0xFF (UTF-16BE BOM)
        // or 0xFF 0xFE (UTF-16LE BOM), the computed MIME type is "text/plain".
        // Step 3. If length is greater than or equal to 3
        // and the first 3 bytes of the resource header are equal to
        // 0xEF 0xBB 0xBF (UTF-8 BOM), the computed MIME type is "text/plain".
        if data.starts_with(&[0xFFu8, 0xFEu8]) ||
            data.starts_with(&[0xFEu8, 0xFFu8]) ||
            data.starts_with(&[0xEFu8, 0xBBu8, 0xBFu8])
        {
            mime::TEXT_PLAIN
        // } else if data.iter().any(|&x| {
            // x <= 0x08u8 ||
                // x == 0x0Bu8 ||
                // (0x0Eu8..=0x1Au8).contains(&x) ||
                // (0x1Cu8..=0x1Fu8).contains(&x)
        } else if data.iter().any(|xp| {
            *xp <= 0x08u8 ||
                *xp == 0x0Bu8 ||
                (0x0Eu8..=0x1Au8).contains(xp) ||
                (0x1Cu8..=0x1Fu8).contains(xp)
        }) {
            // Step 5. The computed MIME type is "application/octet-stream".
            mime::APPLICATION_OCTET_STREAM
        } else {
            // Step 4. If the resource header contains no binary data bytes,
            // the computed MIME type is "text/plain".
            mime::TEXT_PLAIN
        }
    }
}
impl MIMEChecker for BinaryOrPlaintextClassifier {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        Some(self.classify_impl(data))
    }

    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
#[cfg(not(verus_only))]
struct GroupedClassifier {
    byte_matchers: Vec<Box<dyn MIMEChecker + Send + Sync>>,
}

#[cfg(verus_only)]
trait ThreadSafeMIMEChecker: MIMEChecker + Send + Sync {}

#[cfg(verus_only)]
impl<T> ThreadSafeMIMEChecker for T
where
    T: MIMEChecker + Send + Sync,
{}

#[cfg(verus_only)]
struct GroupedClassifier {
    byte_matchers: Vec<Box<dyn ThreadSafeMIMEChecker>>,
}


impl GroupedClassifier {
    fn image_classifer() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                // Keep this in sync with 'is_supported_mime_type' from
                // components/style/servo/media_queries.rs
                Box::new(ByteMatcher::image_x_icon()),
                Box::new(ByteMatcher::image_x_icon_cursor()),
                Box::new(ByteMatcher::image_bmp()),
                Box::new(ByteMatcher::image_gif89a()),
                Box::new(ByteMatcher::image_gif87a()),
                Box::new(ByteMatcher::image_webp()),
                Box::new(ByteMatcher::image_png()),
                Box::new(ByteMatcher::image_jpeg()),
            ],
        }
    }
    fn audio_video_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::video_webm()),
                Box::new(ByteMatcher::audio_basic()),
                Box::new(ByteMatcher::audio_aiff()),
                Box::new(ByteMatcher::audio_mpeg()),
                Box::new(ByteMatcher::application_ogg()),
                Box::new(ByteMatcher::audio_midi()),
                Box::new(ByteMatcher::video_avi()),
                Box::new(ByteMatcher::audio_wave()),
                Box::new(Mp4Matcher),
            ],
        }
    }
    fn scriptable_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::text_html_doctype()),
                Box::new(ByteMatcher::text_html_page()),
                Box::new(ByteMatcher::text_html_head()),
                Box::new(ByteMatcher::text_html_script()),
                Box::new(ByteMatcher::text_html_iframe()),
                Box::new(ByteMatcher::text_html_h1()),
                Box::new(ByteMatcher::text_html_div()),
                Box::new(ByteMatcher::text_html_font()),
                Box::new(ByteMatcher::text_html_table()),
                Box::new(ByteMatcher::text_html_a()),
                Box::new(ByteMatcher::text_html_style()),
                Box::new(ByteMatcher::text_html_title()),
                Box::new(ByteMatcher::text_html_b()),
                Box::new(ByteMatcher::text_html_body()),
                Box::new(ByteMatcher::text_html_br()),
                Box::new(ByteMatcher::text_html_p()),
                Box::new(ByteMatcher::text_html_comment()),
                Box::new(ByteMatcher::text_xml()),
                Box::new(ByteMatcher::application_pdf()),
            ],
        }
    }
    fn plaintext_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::text_plain_utf_8_bom()),
                Box::new(ByteMatcher::text_plain_utf_16le_bom()),
                Box::new(ByteMatcher::text_plain_utf_16be_bom()),
                Box::new(ByteMatcher::application_postscript()),
            ],
        }
    }
    fn archive_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::application_x_gzip()),
                Box::new(ByteMatcher::application_zip()),
                Box::new(ByteMatcher::application_x_rar_compressed()),
            ],
        }
    }

    fn font_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::application_font_woff()),
                Box::new(ByteMatcher::true_type_collection()),
                Box::new(ByteMatcher::open_type()),
                Box::new(ByteMatcher::true_type()),
                Box::new(ByteMatcher::application_vnd_ms_font_object()),
            ],
        }
    }
}
impl MIMEChecker for GroupedClassifier {
    #[verifier::external_body]
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        self.byte_matchers
            .iter()
            .find_map(|matcher| matcher.classify(data))
    }

    fn validate(&self) -> Result<(), String> {
        for byte_matcher in &self.byte_matchers {
            byte_matcher.validate()?
        }
        Ok(())
    }
}

// Contains hard coded byte matchers
// TODO: These should be configured and not hard coded
impl ByteMatcher {
    // A Windows Icon signature
    #[verifier::external_body]
    fn image_x_icon() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x00\x01\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "image/x-icon".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // A Windows Cursor signature.
    #[verifier::external_body]
    fn image_x_icon_cursor() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x00\x02\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "image/x-icon".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "BM", a BMP signature.
    #[verifier::external_body]
    fn image_bmp() -> ByteMatcher {
        ByteMatcher {
            pattern: b"BM",
            mask: b"\xFF\xFF",
            content_type: mime::IMAGE_BMP,
            leading_ignore: &[],
        }
    }
    // The string "GIF89a", a GIF signature.
    #[verifier::external_body]
    fn image_gif89a() -> ByteMatcher {
        ByteMatcher {
            pattern: b"GIF89a",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_GIF,
            leading_ignore: &[],
        }
    }
    // The string "GIF87a", a GIF signature.
    #[verifier::external_body]
    fn image_gif87a() -> ByteMatcher {
        ByteMatcher {
            pattern: b"GIF87a",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_GIF,
            leading_ignore: &[],
        }
    }
    // The string "RIFF" followed by four bytes followed by the string "WEBPVP".
    #[verifier::external_body]
    fn image_webp() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00WEBPVP",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "image/webp".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // An error-checking byte followed by the string "PNG" followed by CR LF SUB LF, the PNG
    // signature.
    #[verifier::external_body]
    fn image_png() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x89PNG\r\n\x1A\n",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_PNG,
            leading_ignore: &[],
        }
    }
    // The JPEG Start of Image marker followed by the indicator byte of another marker.
    #[verifier::external_body]
    fn image_jpeg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFF\xD8\xFF",
            mask: b"\xFF\xFF\xFF",
            content_type: mime::IMAGE_JPEG,
            leading_ignore: &[],
        }
    }
    // The WebM signature. [TODO: Use more bytes?]
    #[verifier::external_body]
    fn video_webm() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x1A\x45\xDF\xA3",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "video/webm".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string ".snd", the basic audio signature.
    #[verifier::external_body]
    fn audio_basic() -> ByteMatcher {
        ByteMatcher {
            pattern: b".snd",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "audio/basic".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "FORM" followed by four bytes followed by the string "AIFF", the AIFF signature.
    #[verifier::external_body]
    fn audio_aiff() -> ByteMatcher {
        ByteMatcher {
            pattern: b"FORM\x00\x00\x00\x00AIFF",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "audio/aiff".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "ID3", the ID3v2-tagged MP3 signature.
    #[verifier::external_body]
    fn audio_mpeg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"ID3",
            mask: b"\xFF\xFF\xFF",
            content_type: "audio/mpeg".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "OggS" followed by NUL, the Ogg container signature.
    #[verifier::external_body]
    fn application_ogg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"OggS\x00",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/ogg".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "MThd" followed by four bytes representing the number 6 in 32 bits (big-endian),
    // the MIDI signature.
    #[verifier::external_body]
    fn audio_midi() -> ByteMatcher {
        ByteMatcher {
            pattern: b"MThd\x00\x00\x00\x06",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "audio/midi".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "RIFF" followed by four bytes followed by the string "AVI ", the AVI signature.
    #[verifier::external_body]
    fn video_avi() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00AVI ",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "video/avi".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "RIFF" followed by four bytes followed by the string "WAVE", the WAVE signature.
    #[verifier::external_body]
    fn audio_wave() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00WAVE",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "audio/wave".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // doctype terminated with Tag terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_doctype() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<!DOCTYPE HTML",
                mask: b"\xFF\xFF\xDF\xDF\xDF\xDF\xDF\xDF\xDF\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // HTML terminated with Tag terminating (TT) Byte: 0x20 (SP)
    #[verifier::external_body]
    fn text_html_page() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<HTML",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // head terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_head() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<HEAD",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // script terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_script() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<SCRIPT",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // iframe terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_iframe() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<IFRAME",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // h1 terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_h1() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<H1",
                mask: b"\xFF\xDF\xFF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // div terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_div() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<DIV",
                mask: b"\xFF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // font terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_font() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<FONT",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // table terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_table() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<TABLE",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // a terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_a() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<A",
                mask: b"\xFF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // style terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_style() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<STYLE",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // title terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_title() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<TITLE",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // b terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_b() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<B",
                mask: b"\xFF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // body terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_body() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<BODY",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // br terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_br() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<BR",
                mask: b"\xFF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // p terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_p() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<P",
                mask: b"\xFF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // comment terminated with Tag Terminating (TT) Byte
    #[verifier::external_body]
    fn text_html_comment() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<!--",
                mask: b"\xFF\xFF\xFF\xFF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // The string "<?xml".
    #[verifier::external_body]
    fn text_xml() -> ByteMatcher {
        ByteMatcher {
            pattern: b"<?xml",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::TEXT_XML,
            leading_ignore: b"\t\n\x0C\r ",
        }
    }
    // The string "%PDF-", the PDF signature.
    #[verifier::external_body]
    fn application_pdf() -> ByteMatcher {
        ByteMatcher {
            pattern: b"%PDF-",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::APPLICATION_PDF,
            leading_ignore: &[],
        }
    }
    // 34 bytes followed by the string "LP", the Embedded OpenType signature.
    #[verifier::external_body]
    fn application_vnd_ms_font_object() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                       \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                       \x00\x00LP",
            mask: b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x00\x00\xFF\xFF",
            content_type: "application/vnd.ms-fontobject".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // 4 bytes representing the version number 1.0, a TrueType signature.
    #[verifier::external_body]
    fn true_type() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x01\x00\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "OTTO", the OpenType signature.
    #[verifier::external_body]
    fn open_type() -> ByteMatcher {
        ByteMatcher {
            pattern: b"OTTO",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "ttcf", the TrueType Collection signature.
    #[verifier::external_body]
    fn true_type_collection() -> ByteMatcher {
        ByteMatcher {
            pattern: b"ttcf",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "wOFF", the Web Open Font Format signature.
    #[verifier::external_body]
    fn application_font_woff() -> ByteMatcher {
        ByteMatcher {
            pattern: b"wOFF",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-woff".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The GZIP archive signature.
    #[verifier::external_body]
    fn application_x_gzip() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x1F\x8B\x08",
            mask: b"\xFF\xFF\xFF",
            content_type: "application/x-gzip".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "PK" followed by ETX EOT, the ZIP archive signature.
    #[verifier::external_body]
    fn application_zip() -> ByteMatcher {
        ByteMatcher {
            pattern: b"PK\x03\x04",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/zip".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "Rar " followed by SUB BEL NUL, the RAR archive signature.
    #[verifier::external_body]
    fn application_x_rar_compressed() -> ByteMatcher {
        ByteMatcher {
            pattern: b"Rar \x1A\x07\x00",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/x-rar-compressed".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "%!PS-Adobe-", the PostScript signature.
    #[verifier::external_body]
    fn application_postscript() -> ByteMatcher {
        ByteMatcher {
            pattern: b"%!PS-Adobe-",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/postscript".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // UTF-16BE BOM
    #[verifier::external_body]
    fn text_plain_utf_16be_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFE\xFF\x00\x00",
            mask: b"\xFF\xFF\x00\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
    // UTF-16LE BOM
    #[verifier::external_body]
    fn text_plain_utf_16le_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFF\xFE\x00\x00",
            mask: b"\xFF\xFF\x00\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
    // UTF-8 BOM
    #[verifier::external_body]
    fn text_plain_utf_8_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xEF\xBB\xBF\x00",
            mask: b"\xFF\xFF\xFF\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
}

} // verus!
