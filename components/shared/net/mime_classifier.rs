/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use vstd::prelude::*;
use vstd::std_specs::iter::{IteratorSpec, zip_seq};
use vstd::assert_seqs_equal;


use mime::{self, Mime, Name};

use crate::LoadContext;

use crate::mime_classifier_specs::{
    predicates as Spec,
    flag as SpecFlag,
    classifier as SpecClassifier,
    byte_matcher as SpecByteMatcher,
    mime_api as SpecMime,
    mp4_matcher as SpecMP4Matcher,
};

use crate::mime_classifier_specs::classifier::MIMECheckerSpec;

macro_rules! prove_valid_byte_literals {
    ($pattern:literal, $mask:literal) => {
        ::vstd::prelude::proof! {
            reveal_byteslit($pattern);
            reveal_byteslit($mask);

            let pattern = ($pattern)@;
            let mask = ($mask)@;

            assert forall |i: int| #![trigger pattern[i]]
                0 <= i < pattern.len()
                implies
                (pattern[i] & mask[i]) == pattern[i]
            by {
                let p = pattern[i];
                let m = mask[i];
                assert((p & m) == p) by (bit_vector)
                    requires
                        m == 0xFFu8
                        || (m == 0x00u8 && p == 0x00u8)
                        || (m == 0xDFu8 && 0x40u8 <= p && p < 0x60u8),
                ;
            }
        }
    };
}

verus! {

broadcast use {
    Spec::mime_essence_str_lemmas,
    Spec::lemma_image_audio_video_disjoint,
    Spec::whitespace_lemmas,
    Spec::mime_essence_parts_str_lemmas,
    SpecMime::group_name_partial_eq_axioms,
    SpecFlag::group_flag_partial_eq_axioms,
};

pub struct MimeClassifier {
    pub(crate) image_classifier: GroupedClassifier,
    pub(crate) audio_video_classifier: GroupedClassifier,
    pub(crate) scriptable_classifier: GroupedClassifier,
    pub(crate) plaintext_classifier: GroupedClassifier,
    pub(crate) archive_classifier: GroupedClassifier,
    pub(crate) binary_or_plaintext: BinaryOrPlaintextClassifier,
    pub(crate) font_classifier: GroupedClassifier,
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
    /// <https://mimesniff.spec.whatwg.org/#supplied-mime-type-detection-algorithm>
    pub fn from_content_type(mime_type: Option<&Mime>) -> (result: ApacheBugFlag)
        ensures
            (result == ApacheBugFlag::On) ==> match mime_type {
                Some(mt) => Spec::is_text_plain(mt),
                None => false,
            },
    {
        // TODO(36801): also handle charset ISO-8859-1
        if mime_type.is_some_and(|mime_type| -> (r: bool)
            ensures 
                r ==> Spec::is_text_plain(mime_type)
            {
            *mime_type == mime::TEXT_PLAIN || *mime_type == mime::TEXT_PLAIN_UTF_8
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
    fn from(boolean: bool) -> Self {
        if boolean {
            NoSniffFlag::On
        } else {
            NoSniffFlag::Off
        }
    }
}

impl Default for MimeClassifier {
    fn default() -> (result: Self) 
        ensures SpecClassifier::mime_classifier_validate_spec(&result),
    {
        broadcast use SpecClassifier::lemma_valid_mime_classifier_from_valid_fields;
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
    pub fn classify<'a>(
        &'a self,
        context: LoadContext,
        no_sniff_flag: NoSniffFlag,
        apache_bug_flag: ApacheBugFlag,
        supplied_type: &Option<Mime>,
        data: &'a [u8],
    ) -> (result: Mime)
        requires
            SpecClassifier::mime_classifier_validate_spec(self),
        ensures
            match context {
                LoadContext::Browsing =>
                    SpecClassifier::mime_classify_browsing_result(
                        self,
                        no_sniff_flag,
                        apache_bug_flag,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    ),
                LoadContext::Image =>
                    SpecClassifier::mime_classify_image_result(
                        self,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    ),
                LoadContext::AudioVideo =>
                    SpecClassifier::mime_classify_audio_video_result(
                        self,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    ),
                LoadContext::Plugin =>
                    SpecClassifier::mime_classify_plugin_result(
                        self,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    ),
                LoadContext::Style =>
                    SpecClassifier::mime_classify_style_result(
                        self,
                        no_sniff_flag,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    ),
                LoadContext::Script =>
                    SpecClassifier::mime_classify_script_result(
                        self,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    ),
                LoadContext::Font =>
                    SpecClassifier::mime_classify_font_result(
                        self,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    ),
                LoadContext::TextTrack =>
                    SpecClassifier::mime_classify_text_track_result(
                        self,
                        supplied_type, // for Servo behavior
                        SpecMime::view(&result),
                    ),
                LoadContext::CacheManifest =>
                    SpecClassifier::mime_classify_cache_manifest_result(
                        self,
                        supplied_type, // for Servo behavior
                        SpecMime::view(&result),
                    ),
                _ => true,
            },
    {
        proof {
            SpecClassifier::lemma_mime_classifier_validate_spec(self);
        }
        let supplied_type_or_octet_stream = supplied_type
            .clone()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);
        // Step 1. If the supplied MIME type is an XML MIME type or HTML MIME type,
        // the computed MIME type is the supplied MIME type.
        if Self::is_xml(&supplied_type_or_octet_stream) ||
            Self::is_html(&supplied_type_or_octet_stream)
        {
            proof {
                reveal_strlit("application/octet-stream");
                reveal_strlit("text/xml");
                reveal_strlit("application/xml");
                reveal_strlit("text/html");

                SpecClassifier::lemma_mime_classify_browsing_step1_result(
                    self,
                    no_sniff_flag,
                    apache_bug_flag,
                    supplied_type,
                    data@,
                    SpecMime::view(&supplied_type_or_octet_stream),
                ); 
            }
            return supplied_type_or_octet_stream;
        }

        let ghost supplied_type_input = supplied_type;

        match context {
            LoadContext::Browsing => match *supplied_type {
                // Step 2. If the supplied MIME type is undefined or if the supplied MIME type’s essence is "unknown/unknown",
                // "application/unknown", or "*/*", execute the rules for identifying
                // an unknown MIME type with the sniff-scriptable flag equal to the inverse of the no-sniff flag and abort these steps.
                // None => self.sniff_unknown_type(no_sniff_flag, data),
                None => {
                    proof {
                        SpecClassifier::lemma_mime_classify_browsing_none_result(
                            self,
                            no_sniff_flag,
                            apache_bug_flag,
                            supplied_type_input,
                            data@,
                        );
                    }

                    self.sniff_unknown_type(no_sniff_flag, data)
                }
                Some(ref supplied_type) => {
                    if MimeClassifier::is_explicit_unknown(supplied_type) {

                        proof {
                            SpecClassifier::lemma_mime_classify_browsing_explicit_unknown_result(
                                self,
                                no_sniff_flag,
                                apache_bug_flag,
                                supplied_type_input,
                                supplied_type,
                                data@
                            );
                        }

                        return self.sniff_unknown_type(no_sniff_flag, data);
                    }
                    // Step 3. If the no-sniff flag is set, the computed MIME type is the supplied MIME type.
                    // Abort these steps.
                    if no_sniff_flag == NoSniffFlag::On {

                        proof {
                            SpecClassifier::lemma_mime_classify_browsing_no_sniff_result(
                                self,
                                no_sniff_flag,
                                apache_bug_flag,
                                supplied_type_input,
                                supplied_type,
                                data@,
                            );
                        }

                        return supplied_type.clone();
                    }
                    // Step 4. If the check-for-apache-bug flag is set,
                    // execute the rules for distinguishing if a resource is text or binary and abort these steps.
                    if apache_bug_flag == ApacheBugFlag::On {

                        proof {
                            SpecClassifier::lemma_mime_classify_browsing_apache_bug_result(
                                self,
                                no_sniff_flag,
                                apache_bug_flag,
                                supplied_type_input,
                                supplied_type,
                                data@,
                            );
                        }

                        return self.sniff_text_or_data(data);
                    }

                    proof {
                        SpecClassifier::lemma_mime_classify_browsing_after_step4_trace(
                            self,
                            // supplied_type_input,
                            supplied_type,
                            no_sniff_flag,
                            apache_bug_flag,
                            data@
                        );
                    }

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
                let result =  self.image_classifier.classify(data)
                    .unwrap_or(supplied_type_or_octet_stream);

                proof {
                    SpecClassifier::lemma_mime_classify_image_result(
                        self,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    );
                } 

                // result
                match MimeClassifier::maybe_get_media_type(supplied_type) {
                    Some(MediaType::Xml) => None,
                    _ => self.image_classifier.classify(data),
                }
                // self.image_classifier.classify(data)
                .unwrap_or(supplied_type_or_octet_stream)
            },
            LoadContext::AudioVideo => {
                let result = self.audio_video_classifier.classify(data)
                    .unwrap_or(supplied_type_or_octet_stream);
                proof {
                    SpecClassifier::lemma_mime_classify_audio_video_result(
                        self,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    );
                }

                // Section 8.3 Sniffing an image context
                match MimeClassifier::maybe_get_media_type(supplied_type) {
                    Some(MediaType::Xml) => {
                        assert(false);
                        None
                    },
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
                supplied_type.clone().unwrap_or_else(|| -> (r: Mime)
                    ensures
                        SpecMime::view(&r) == if no_sniff_flag == NoSniffFlag::On {
                            SpecMime::application_octet_stream_identity()
                        } else {
                            SpecMime::text_css_identity()
                        },
                {
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
                let result = self.font_classifier.classify(data)
                    .unwrap_or(supplied_type_or_octet_stream);
                proof {
                    SpecClassifier::lemma_mime_classify_font_result(
                        self,
                        supplied_type,
                        data@,
                        SpecMime::view(&result),
                    );
                }
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

    pub fn validate(&self) -> (r: Result<(), String>) 
        ensures
            r.is_ok() == SpecClassifier::mime_classifier_validate_spec(self)
    {
        proof {
            SpecClassifier::lemma_mime_classifier_validate_spec(self);
        }
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
    fn sniff_unknown_type(&self, no_sniff_flag: NoSniffFlag, data: &[u8]) -> (result: Mime)
        requires
            SpecClassifier::mime_classifier_validate_spec(self),
        ensures
            SpecMime::view(&result)
                == SpecClassifier::sniff_unknown_type_spec(
                    self,
                    no_sniff_flag,
                    data@
                ),
    {
        proof {
            SpecClassifier::lemma_mime_classifier_validate_spec(self);
        }
        let should_sniff_scriptable = no_sniff_flag == NoSniffFlag::Off;
        let sniffed = if should_sniff_scriptable {
            self.scriptable_classifier.classify(data)
        } else {
            None
        };

        proof {
            assert(
                should_sniff_scriptable
                    == (no_sniff_flag == NoSniffFlag::Off)
            );

            if should_sniff_scriptable {
                assert(no_sniff_flag == NoSniffFlag::Off);
                assert(
                    SpecMime::option_view(&sniffed)
                        == self.scriptable_classifier.classify_spec(data@)
                );
            } else {
                assert(no_sniff_flag != NoSniffFlag::Off);
                assert(sniffed.is_none());
                assert(SpecMime::option_view(&sniffed) == None);
            }

            assert(
                SpecMime::option_view(&sniffed)
                    == if no_sniff_flag == NoSniffFlag::Off {
                        self.scriptable_classifier.classify_spec(data@)
                    } else {
                        None
                    }
            );
        } 

        sniffed
            .or_else(
                || -> (r: Option<Mime>)
                    ensures
                        SpecMime::option_view(&r)
                            == self.plaintext_classifier.classify_spec(data@),
                { self.plaintext_classifier.classify(data) }
            )
            .or_else(
                || -> (r: Option<Mime>)
                    ensures
                        SpecMime::option_view(&r)
                            == self.image_classifier.classify_spec(data@),
                { self.image_classifier.classify(data) }
            )
            .or_else(
                || -> (r: Option<Mime>)
                    ensures
                        SpecMime::option_view(&r)
                            == self.audio_video_classifier.classify_spec(data@),
                { self.audio_video_classifier.classify(data) }
            )
            .or_else(
                || -> (r: Option<Mime>)
                    ensures
                        SpecMime::option_view(&r)
                            == self.archive_classifier.classify_spec(data@),
                { self.archive_classifier.classify(data) }
            )
            .or_else(
                || -> (r: Option<Mime>)
                    ensures 
                        r.is_some(),
                        SpecMime::option_view(&r) 
                            == Some(SpecClassifier::bin_or_plain_classify_spec(data@))
                            // == self.binary_or_plaintext.classify_spec(data@),
                { self.binary_or_plaintext.classify(data) }
            )
            .expect("BinaryOrPlaintextClassifier always succeeds")
    }

    fn sniff_text_or_data<'a>(&'a self, data: &'a [u8]) -> (result: Mime)
        requires
            SpecClassifier::mime_classifier_validate_spec(self),
        ensures
            SpecMime::view(&result) == SpecClassifier::sniff_text_or_data_spec(self, data@),
    {   
        proof {
            SpecClassifier::lemma_mime_classifier_validate_spec(self);
        }
        self.binary_or_plaintext
            .classify(data)
            .expect("BinaryOrPlaintextClassifier always succeeds")
    }
    
    /// <https://mimesniff.spec.whatwg.org/#xml-mime-type>
    /// SVG is worth distinguishing from other XML MIME types:
    /// <https://mimesniff.spec.whatwg.org/#mime-type-miscellaneous>
    fn is_xml(mt: &Mime) -> (result: bool) 
        ensures
            result == Spec::is_xml(mt),
    {
        // broadcast use SpecMime::group_name_partial_eq_axioms;
        
        !Self::is_image(mt) &&
            (mt.suffix() == Some(mime::XML) ||
                mt.essence_str() == "text/xml" ||
                mt.essence_str() == "application/xml")
    }

    /// <https://mimesniff.spec.whatwg.org/#html-mime-type>
    fn is_html(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_html(mt),
    {
        mt.essence_str() == "text/html"
    }

    /// <https://mimesniff.spec.whatwg.org/#image-mime-type>
    fn is_image(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_image(mt),
    {
        mt.type_() == mime::IMAGE
    }

    /// <https://mimesniff.spec.whatwg.org/#audio-or-video-mime-type>
    fn is_audio_video(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_audio_video(mt),
    {
        mt.type_() == mime::AUDIO ||
            mt.type_() == mime::VIDEO ||
            mt.essence_str() == "application/ogg"
    }

    fn is_explicit_unknown(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_explicit_unknown(mt),
    {
        mt.type_().as_str() == "unknown" && mt.subtype().as_str() == "unknown" ||
            mt.type_() == mime::APPLICATION && mt.subtype().as_str() == "unknown" ||
            mt.type_() == mime::STAR && mt.subtype() == mime::STAR
    }

    /// <https://mimesniff.spec.whatwg.org/#javascript-mime-type>
    pub fn is_javascript(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_javascript(mt),
    {
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
    pub fn is_json(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_json(mt),
    {
        mt.suffix() == Some(mime::JSON) ||
            mt.essence_str() == "application/json" ||
            mt.essence_str() == "text/json"
    }

    /// <https://mimesniff.spec.whatwg.org/#font-mime-type>
    fn is_font(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_font(mt),
    {
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

    fn is_text(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_text(mt),
    {
        *mt == mime::TEXT_PLAIN || mt.essence_str() == "text/vtt"
    }

    pub fn is_css(mt: &Mime) -> (result: bool)
        ensures
            result == Spec::is_css(mt),
    {
        mt.essence_str() == "text/css"
    }

    pub fn get_media_type(mime: &Mime) -> (result: Option<MediaType>) 
        ensures
            SpecClassifier::get_media_type_spec(mime, result),
    {
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

    fn maybe_get_media_type(supplied_type: &Option<Mime>) -> (result: Option<MediaType>) 
        ensures
            match supplied_type {
                Some(mt) => SpecClassifier::get_media_type_spec(mt, result),
                None => result == None,
            },
    {
        supplied_type
            .as_ref()
            .and_then(MimeClassifier::get_media_type)
    }
}

// Interface used for composite types
pub(crate) trait MIMEChecker: MIMECheckerSpec {
    fn classify(&self, data: &[u8]) -> (r: Option<Mime>)
        requires
            self.validate_spec(),
        ensures
            SpecMime::option_view(&r) == self.classify_spec(data@),
    ;
    /// Validate the MIME checker configuration
    fn validate(&self) -> (r: Result<(), String>)
        ensures
            r.is_ok() == self.validate_spec()
    ;
}

#[cfg(not(verus_only))]
struct ByteMatcher {
    pattern: &'static [u8],
    mask: &'static [u8],
    leading_ignore: &'static [u8],
    content_type: Mime,
}

#[cfg(verus_only)]
pub(crate) struct ByteMatcher {
    pub(crate) pattern: &'static [u8],
    pub(crate) mask: &'static [u8],
    pub(crate) content_type: Mime,
    pub(crate) leading_ignore: &'static [u8],
}

impl ByteMatcher {
    fn matches(&self, data: &[u8]) -> (result: Option<usize>)
        requires
            self.pattern.len() > 0,
            self.pattern.len() == self.mask.len(),
        ensures
            match result {
                Some(r) => {
                   ||| (data@ == self.pattern@) && (r as int == self.pattern@.len())
                   ||| SpecByteMatcher::is_match_result(data@, self.pattern@, self.mask@, self.leading_ignore@.to_set(), r as int)
                } 
                None => {
                    &&& !(data@ == self.pattern@) 
                    &&& !SpecByteMatcher::pattern_matching_success(data@, self.pattern@, self.mask@, self.leading_ignore@.to_set())
                }
            }
    {
        if data.len() < self.pattern.len() {
            return None;
        } else if data == self.pattern {
            return Some(self.pattern.len());
        } else {
            broadcast use SpecByteMatcher::position_not_found_implies_no_pattern_matching;
            data[..data.len() - self.pattern.len() + 1]
                .iter()
                .position(|x: &u8| -> (r: bool)
                    ensures 
                        r == !self.leading_ignore@.to_set().contains(*x),
                    { 
                        !self.leading_ignore.contains(&x) 
                    },
                )
                .and_then(|start: usize| -> (result: Option<usize>)
                    requires 
                        SpecByteMatcher::position_found(
                            data@.subrange(0, (data.len() - self.pattern.len() + 1) as int).as_ref(),
                            self.leading_ignore@.to_set(),
                            start as int,
                        ),
                    ensures
                        match result {
                            Some(r) => SpecByteMatcher::is_match_result(data@, self.pattern@, self.mask@, self.leading_ignore@.to_set(), r as int),
                            None => !SpecByteMatcher::pattern_matching_success(data@, self.pattern@, self.mask@, self.leading_ignore@.to_set()), 
                        },
                {
                    proof {
                        if SpecByteMatcher::position_found(
                            data@.subrange(0, 
                                (data.len() - self.pattern.len() + 1) as int
                            ).as_ref(),
                            self.leading_ignore@.to_set(),
                            start as int,
                        ) {
                            assert forall |j: int| #![trigger data@[j]]
                                0 <= j < start as int
                                implies
                                self.leading_ignore@.to_set().contains(data@[j])
                            by {
                                assert(
                                    self.leading_ignore@.to_set().contains(
                                        *data@.subrange(0, 
                                            (data.len() - self.pattern.len() + 1) as int
                                        ).as_ref()[j],
                                    )
                                );
                            }
                        }
                    } 
                    if data[start..]
                        .iter()
                        .zip(self.pattern.iter())
                        .zip(self.mask.iter())
                        // .all(|((&data, &pattern), &mask)| (data & mask) == pattern)
                        .all(|x: ((&u8, &u8), &u8)| -> (r: bool) 
                            ensures
                                r == ((*x.0.0 & *x.1) == *x.0.1),
                        {
                            let ((data_p, pattern_p), mask_p) = x;
                            (*data_p & *mask_p) == *pattern_p
                        })
                    {
                        proof {
                            if SpecByteMatcher::position_found(
                                data@.subrange(
                                    0,
                                    (data.len() - self.pattern.len() + 1) as int,
                                ).as_ref(),
                                self.leading_ignore@.to_set(),
                                start as int,
                            ) {
                                SpecByteMatcher::match_return_some(
                                    data@,
                                    self.pattern@,
                                    self.mask@,
                                    self.leading_ignore@.to_set(),
                                    start as int,
                                );
                            }
                        }
                    
                        Some(start + self.pattern.len())
                    } else {
                        proof {
                            if SpecByteMatcher::position_found(
                                data@.subrange(
                                    0,
                                    (data.len() - self.pattern.len() + 1) as int,
                                ).as_ref(),
                                self.leading_ignore@.to_set(),
                                start as int,
                            ) {
                                SpecByteMatcher::match_return_none(
                                    data@,
                                    self.pattern@,
                                    self.mask@,
                                    self.leading_ignore@.to_set(),
                                    start as int,
                                );
                            }
                        }
                        None
                    }
                })
            }
    }
    
}

impl MIMEChecker for ByteMatcher {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        // self.matches(data).map(|_| self.content_type.clone())
        self.matches(data).map(|x: usize| -> (r: Mime) 
            ensures
                SpecMime::view(&r)  == SpecMime::view(&self.content_type),
        {
            self.content_type.clone()
        })
    }

    fn validate(&self) -> Result<(), String> {
        broadcast use SpecMime::axiom_fmt_req_all_mime;

        if self.pattern.is_empty() {
            return Err(format!("Zero length pattern for {:?}", self.content_type));
        }
        if self.pattern.len() != self.mask.len() {
            return Err(format!(
                "Unequal pattern and mask length for {:?}",
                self.content_type
            ));
        }

        if self
            .pattern
            .iter()
            .zip(self.mask.iter())
            // .any(|(&pattern, &mask)| pattern & mask != pattern)
            .any(|x: (&u8, &u8)| -> (r: bool)
                ensures
                    r == (*x.0 & *x.1 != *x.0), 
            {
                let (pattern_p, mask_p) = x;
                *pattern_p & *mask_p != *pattern_p
            })
        {
            return Err(format!(
                "Pattern not pre-masked for {:?}",
                self.content_type
            ));
        }

        proof {
            assert forall |i: int| #![trigger self.pattern[i]] 0 <= i < self.pattern@.len()
                implies
                (self.pattern@[i] & self.mask@[i]) == self.pattern@[i]
            by {
                assert(*self.pattern@.as_ref()[i] == self.pattern@[i]);
                assert(*self.mask@.as_ref()[i] == self.mask@[i]);
            }
        }

        Ok(())
    }
}

#[cfg(not(verus_only))]
struct TagTerminatedByteMatcher {
    matcher: ByteMatcher,
}
#[cfg(verus_only)]
pub(crate) struct TagTerminatedByteMatcher {
    pub(crate) matcher: ByteMatcher,
}

impl MIMEChecker for TagTerminatedByteMatcher {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        proof {
            assert(self.matcher.pattern@.len() > 0);
            assert(
                self.matcher.pattern@.len() == self.matcher.mask@.len()
            );
        }

        self.matcher.matches(data).and_then(|j: usize| -> (result: Option<Mime>)
            ensures
                match result {
                    Some(mt) => {
                        &&& SpecByteMatcher::ttbm_match_success(j as int, data@) 
                        &&& SpecMime::view(&mt) == SpecMime::view(&self.matcher.content_type)
                    }
                    None => !SpecByteMatcher::ttbm_match_success(j as int, data@),
                }
        {
            if j < data.len() && (data[j] == b' ' || data[j] == b'>') {
                Some(self.matcher.content_type.clone())
            } else {
                None
            }
        })
    }

    fn validate(&self) -> (result: Result<(), String>) {
        self.matcher.validate()
    }
}

pub struct Mp4Matcher;

impl Mp4Matcher {
    /// <https://mimesniff.spec.whatwg.org/#matches-the-signature-for-mp4>
    pub fn matches(&self, data: &[u8]) -> (result: bool) 
        ensures
            result == SpecMP4Matcher::matches_mp4_signature(data@),
    {
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

        proof {
            SpecMP4Matcher::lemma_step4_u32_eq_int(box_size as u32, data@);
        }

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

        let ghost input = data@;
        broadcast use SpecMP4Matcher::lemma_chunks_mp4_equivalence;
            
        data[8..].starts_with(&mp4) ||
        // Step 8. Let bytes-read be 16.
        // Step 9. While bytes-read is less than box-size, continuously loop through these steps:
            data.get(16..box_size).map(|data| -> (r: bool)
                requires
                    box_size as int >= 16,
                    data@ == input.subrange(16, box_size as int),
                ensures
                    r == (exists |bytes_read: int| 16 <= bytes_read < (box_size as int)
                        && bytes_read % 4 == 0
                        && #[trigger] input[bytes_read] == 0x6D
                        && input[bytes_read + 1] == 0x70
                        && input[bytes_read + 2] == 0x34),
            // Step 11. Increment bytes-read by 4.
            {
                data.chunks(4).any(|chunk: &[u8]| -> (r:bool)
                    ensures
                        r == SpecMP4Matcher::has_mp4_prefix(chunk@)
                {
                    chunk.starts_with(&mp4)
                })
            }).unwrap_or(false)
        // Step 10. If the three bytes from sequence[bytes-read] to sequence[bytes-read + 2]
        // are equal to 0x6D 0x70 0x34 ("mp4"), return true.
                // // Step 10. If the three bytes from sequence[bytes-read] to sequence[bytes-read + 2]
                // // are equal to 0x6D 0x70 0x34 ("mp4"), return true.
                // .any(|chunk: &[u8]| -> (r: bool)
                //     ensures
                //         r == SpecMP4Matcher::has_mp4_prefix(chunk@)
                // { 
                //     chunk.starts_with(&mp4)
                // }
        // Step 12. Return false.
    }
}
impl MIMEChecker for Mp4Matcher {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        if self.matches(data) {
            Some("video/mp4".parse().unwrap())
        } else {
            None
        }
    }

    fn validate(&self) -> (result: Result<(), String>) {
        Ok(())
    }
}

#[cfg(not(verus_only))]
struct BinaryOrPlaintextClassifier;
#[cfg(verus_only)]
pub(crate) struct BinaryOrPlaintextClassifier;

impl BinaryOrPlaintextClassifier {
    /// <https://mimesniff.spec.whatwg.org/#rules-for-text-or-binary>
    fn classify_impl(&self, data: &[u8]) -> (result: Mime) 
        ensures
            self.classify_spec(data@) == Some(SpecMime::view(&result)),
    {
        proof {
            // Connect the slice iterator's Seq<&u8> model back to Seq<u8>.
            assert(data@.as_ref().unref() == data@);
        }
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
        // TODO: recover the original code
        // } else if data.iter().any(|&x| {
        //     x <= 0x08u8 ||
        //         x == 0x0Bu8 ||
        //         (0x0Eu8..=0x1Au8).contains(&x) ||
        //         (0x1Cu8..=0x1Fu8).contains(&x)
        } else if data.iter().any(|xp: &u8| -> (r: bool) 
            ensures
                r == SpecClassifier::is_binary_data_byte(*xp)
        {
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
    fn classify(&self, data: &[u8]) -> (r: Option<Mime>)
        ensures
            r.is_some(),
    {
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
pub(crate) trait ThreadSafeMIMEChecker: MIMEChecker + Send + Sync {
    spec fn dyn_classify_spec(&self, data: Seq<u8>) -> Option<SpecMime::MimeView>;
    spec fn dyn_validate_spec(&self) -> bool;
    proof fn bridge_validate_spec(tracked &self)
        ensures 
            self.dyn_validate_spec() == self.validate_spec();
    proof fn bridge_classify_spec(tracked &self, data: Seq<u8>)
        ensures
            self.dyn_classify_spec(data) == self.classify_spec(data);
}

// #[cfg(verus_only)]
// impl<T> ThreadSafeMIMEChecker for T
// where
//     T: MIMEChecker + Send + Sync,
// {
//     open spec fn dyn_classify_spec(&self, data: Seq<u8>) -> Option<SpecMime::MimeView> {
//         self.classify_spec(data)
//     }

//     open spec fn dyn_validate_spec(&self) -> bool {
//         self.validate_spec()
//     }
    
//     proof fn bridge_validate_spec(tracked &self) {}

//     proof fn bridge_classify_spec(tracked &self, data: Seq<u8>) {}
// }

#[cfg(verus_only)]
impl ThreadSafeMIMEChecker for ByteMatcher {
    open spec fn dyn_validate_spec(&self) -> bool {
        self.validate_spec()
    }

    open spec fn dyn_classify_spec(
        &self,
        data: Seq<u8>,
    ) -> Option<SpecMime::MimeView> {
        self.classify_spec(data)
    }

    proof fn bridge_validate_spec(tracked &self) {}
    proof fn bridge_classify_spec(tracked &self, data: Seq<u8>) {}
}

#[cfg(verus_only)]
impl ThreadSafeMIMEChecker for TagTerminatedByteMatcher {
    open spec fn dyn_validate_spec(&self) -> bool {
        self.validate_spec()
    }

    open spec fn dyn_classify_spec(
        &self,
        data: Seq<u8>,
    ) -> Option<SpecMime::MimeView> {
        self.classify_spec(data)
    }

    proof fn bridge_validate_spec(tracked &self) {}
    proof fn bridge_classify_spec(tracked &self, data: Seq<u8>) {}
}

#[cfg(verus_only)]
impl ThreadSafeMIMEChecker for Mp4Matcher {
    open spec fn dyn_validate_spec(&self) -> bool {
        self.validate_spec()
    }

    open spec fn dyn_classify_spec(
        &self,
        data: Seq<u8>,
    ) -> Option<SpecMime::MimeView> {
        self.classify_spec(data)
    }

    proof fn bridge_validate_spec(tracked &self) {}
    proof fn bridge_classify_spec(tracked &self, data: Seq<u8>) {}
}

#[cfg(verus_only)]
pub(crate) struct GroupedClassifier {
    pub(crate) byte_matchers: Vec<Box<dyn ThreadSafeMIMEChecker>>,
}


impl GroupedClassifier {
    fn image_classifer() -> (result: GroupedClassifier) 
        ensures result.validate_spec(),
    {
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
    fn audio_video_classifier() -> (result: GroupedClassifier) 
        ensures result.validate_spec(),
    {
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
    fn scriptable_classifier() -> (result: GroupedClassifier) 
        ensures result.validate_spec(),
    {
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
    fn plaintext_classifier() -> (result: GroupedClassifier) 
        ensures result.validate_spec(),
    {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::text_plain_utf_8_bom()),
                Box::new(ByteMatcher::text_plain_utf_16le_bom()),
                Box::new(ByteMatcher::text_plain_utf_16be_bom()),
                Box::new(ByteMatcher::application_postscript()),
            ],
        }
    }
    fn archive_classifier() -> (result: GroupedClassifier) 
        ensures result.validate_spec(),
    {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::application_x_gzip()),
                Box::new(ByteMatcher::application_zip()),
                Box::new(ByteMatcher::application_x_rar_compressed()),
            ],
        }
    }

    fn font_classifier() -> (result: GroupedClassifier) 
        ensures result.validate_spec(),
    {
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
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        broadcast use {
            SpecClassifier::lemma_classify_group_from_first_some,
            SpecClassifier::lemma_classify_group_from_all_none,
        };
        self.byte_matchers
            .iter()
            .find_map(
                |matcher| -> (ret: Option<Mime>)
                    requires
                        matcher.dyn_validate_spec(),
                    ensures
                        SpecMime::option_view(&ret) == matcher.dyn_classify_spec(data@),
                        matcher.dyn_classify_spec(data@) == matcher.classify_spec(data@)
            {
                proof {
                    matcher.bridge_validate_spec();
                    matcher.bridge_classify_spec(data@);
                }
                matcher.classify(data)
            })
    }

    fn validate(&self) -> Result<(), String> {
        for byte_matcher in iter: &self.byte_matchers 
            invariant
                forall |j: int| 0 <= j < iter.index() ==>
                    // #[trigger] self.byte_matchers@[j].validate_spec(),
                    #[trigger] self.byte_matchers@[j].dyn_validate_spec(),
        {
            proof {
                byte_matcher.bridge_validate_spec();
            }
            byte_matcher.validate()?
        }
        Ok(())
    }
}

// Contains hard coded byte matchers
// TODO: These should be configured and not hard coded
impl ByteMatcher {
    // A Windows Icon signature
    fn image_x_icon() -> (r: ByteMatcher) 
        ensures 
            Spec::is_image_x_icon(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\x00\x00\x01\x00", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"\x00\x00\x01\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "image/x-icon".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // A Windows Cursor signature.
    fn image_x_icon_cursor() -> (r: ByteMatcher)
        ensures
            Spec::is_image_x_icon_cursor(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\x00\x00\x02\x00", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"\x00\x00\x02\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "image/x-icon".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "BM", a BMP signature.
    fn image_bmp() -> (r: ByteMatcher) 
        ensures 
            Spec::is_image_bmp(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"BM", b"\xFF\xFF");
        ByteMatcher {
            pattern: b"BM",
            mask: b"\xFF\xFF",
            content_type: mime::IMAGE_BMP,
            leading_ignore: &[],
        }
    }
    // The string "GIF89a", a GIF signature.
    fn image_gif89a() -> (r: ByteMatcher)
        ensures 
            Spec::is_image_gif89a(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"GIF89a", b"\xFF\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"GIF89a",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_GIF,
            leading_ignore: &[],
        }
    }
    // The string "GIF87a", a GIF signature.
    fn image_gif87a() -> (r: ByteMatcher) 
        ensures 
            Spec::is_image_gif87a(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"GIF87a", b"\xFF\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"GIF87a",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_GIF,
            leading_ignore: &[],
        }
    }
    // The string "RIFF" followed by four bytes followed by the string "WEBPVP".
    fn image_webp() -> (r: ByteMatcher)
        ensures 
            Spec::is_image_webp(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(
            b"RIFF\x00\x00\x00\x00WEBPVP", 
            b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF"
        );
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00WEBPVP",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "image/webp".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // An error-checking byte followed by the string "PNG" followed by CR LF SUB LF, the PNG
    // signature.
    fn image_png() -> (r: ByteMatcher) 
        ensures 
            Spec::is_image_png(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\x89PNG\r\n\x1A\n", b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"\x89PNG\r\n\x1A\n",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_PNG,
            leading_ignore: &[],
        }
    }
    // The JPEG Start of Image marker followed by the indicator byte of another marker.
    fn image_jpeg() -> (r: ByteMatcher) 
        ensures 
            Spec::is_image_jpeg(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\xFF\xD8\xFF", b"\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"\xFF\xD8\xFF",
            mask: b"\xFF\xFF\xFF",
            content_type: mime::IMAGE_JPEG,
            leading_ignore: &[],
        }
    }
    // The WebM signature. [TODO: Use more bytes?]
    fn video_webm() -> (r: ByteMatcher) 
        ensures 
            Spec::is_video_webm(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\x1A\x45\xDF\xA3", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"\x1A\x45\xDF\xA3",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "video/webm".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string ".snd", the basic audio signature.
    fn audio_basic() -> (r: ByteMatcher) 
        ensures 
            Spec::is_audio_basic(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b".snd", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b".snd",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "audio/basic".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "FORM" followed by four bytes followed by the string "AIFF", the AIFF signature.
    fn audio_aiff() -> (r: ByteMatcher) 
        ensures 
            Spec::is_audio_aiff(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(
            b"FORM\x00\x00\x00\x00AIFF", 
            b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"
        );
        ByteMatcher {
            pattern: b"FORM\x00\x00\x00\x00AIFF",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "audio/aiff".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "ID3", the ID3v2-tagged MP3 signature.
    fn audio_mpeg() -> (r: ByteMatcher) 
        ensures 
            Spec::is_audio_mpeg(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"ID3", b"\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"ID3",
            mask: b"\xFF\xFF\xFF",
            content_type: "audio/mpeg".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "OggS" followed by NUL, the Ogg container signature.
    fn application_ogg() -> (r: ByteMatcher)
        ensures 
            Spec::is_application_ogg(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"OggS\x00", b"\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"OggS\x00",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/ogg".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "MThd" followed by four bytes representing the number 6 in 32 bits (big-endian),
    // the MIDI signature.
    fn audio_midi() -> (r: ByteMatcher) 
        ensures 
            Spec::is_audio_midi(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"MThd\x00\x00\x00\x06", b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"MThd\x00\x00\x00\x06",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "audio/midi".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "RIFF" followed by four bytes followed by the string "AVI ", the AVI signature.
    fn video_avi() -> (r: ByteMatcher) 
        ensures 
            Spec::is_video_avi(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(
            b"RIFF\x00\x00\x00\x00AVI ", 
            b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"
        );
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00AVI ",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "video/avi".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "RIFF" followed by four bytes followed by the string "WAVE", the WAVE signature.
    fn audio_wave() -> (r: ByteMatcher) 
        ensures 
            Spec::is_audio_wave(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(
            b"RIFF\x00\x00\x00\x00WAVE", 
            b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF"
        );
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00WAVE",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "audio/wave".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // doctype terminated with Tag terminating (TT) Byte
    fn text_html_doctype() -> (r: TagTerminatedByteMatcher)
        ensures 
            Spec::is_text_html_doctype(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(
            b"<!DOCTYPE HTML", 
            b"\xFF\xFF\xDF\xDF\xDF\xDF\xDF\xDF\xDF\xFF\xDF\xDF\xDF\xDF"
        );
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
    fn text_html_page() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_page(&r),
            r.validate_spec(),
    { 
        prove_valid_byte_literals!(b"<HTML", b"\xFF\xDF\xDF\xDF\xDF");
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
    fn text_html_head() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_head(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<HEAD", b"\xFF\xDF\xDF\xDF\xDF");
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
    fn text_html_script() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_script(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<SCRIPT", b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF");
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
    fn text_html_iframe() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_iframe(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<IFRAME", b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF");
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
    fn text_html_h1() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_h1(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<H1", b"\xFF\xDF\xFF");
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
    fn text_html_div() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_div(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<DIV", b"\xFF\xDF\xDF\xDF");
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
    fn text_html_font() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_font(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<FONT", b"\xFF\xDF\xDF\xDF\xDF");
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
    fn text_html_table() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_table(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<TABLE", b"\xFF\xDF\xDF\xDF\xDF\xDF");
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
    fn text_html_a() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_a(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<A", b"\xFF\xDF");
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
    fn text_html_style() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_style(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<STYLE", b"\xFF\xDF\xDF\xDF\xDF\xDF");
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
    fn text_html_title() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_title(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<TITLE", b"\xFF\xDF\xDF\xDF\xDF\xDF");
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
    fn text_html_b() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_b(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<B", b"\xFF\xDF");
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
    fn text_html_body() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_body(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<BODY", b"\xFF\xDF\xDF\xDF\xDF");
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
    fn text_html_br() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_br(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<BR", b"\xFF\xDF\xDF");
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
    fn text_html_p() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_p(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<P", b"\xFF\xDF");
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
    fn text_html_comment() -> (r: TagTerminatedByteMatcher) 
        ensures 
            Spec::is_text_html_comment(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<!--", b"\xFF\xFF\xFF\xFF");
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
    fn text_xml() -> (r: ByteMatcher) 
        ensures 
            Spec::is_text_xml(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"<?xml", b"\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"<?xml",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::TEXT_XML,
            leading_ignore: b"\t\n\x0C\r ",
        }
    }

    // The string "%PDF-", the PDF signature.
    fn application_pdf() -> (r: ByteMatcher) 
        ensures 
            Spec::is_application_pdf(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"%PDF-", b"\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"%PDF-",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::APPLICATION_PDF,
            leading_ignore: &[],
        }
    }

    // 34 bytes followed by the string "LP", the Embedded OpenType signature.
    fn application_vnd_ms_font_object() -> (r: ByteMatcher)
        ensures 
            Spec::is_application_vnd_ms_font_object(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(
            b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                \x00\x00LP", 
            b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                \x00\x00\xFF\xFF"
        );
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
    fn true_type() -> (r: ByteMatcher) 
        ensures 
            Spec::is_true_type(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\x00\x01\x00\x00", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"\x00\x01\x00\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "OTTO", the OpenType signature.
    fn open_type() -> (r: ByteMatcher) 
        ensures 
            Spec::is_open_type(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"OTTO", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"OTTO",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "ttcf", the TrueType Collection signature.
    fn true_type_collection() -> (r: ByteMatcher) 
        ensures 
            Spec::is_true_type_collection(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"ttcf", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"ttcf",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "wOFF", the Web Open Font Format signature.
    fn application_font_woff() -> (r: ByteMatcher) 
        ensures 
            Spec::is_application_font_woff(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"wOFF", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"wOFF",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-woff".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The GZIP archive signature.
    fn application_x_gzip() -> (r: ByteMatcher) 
        ensures 
            Spec::is_application_x_gzip(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\x1F\x8B\x08", b"\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"\x1F\x8B\x08",
            mask: b"\xFF\xFF\xFF",
            content_type: "application/x-gzip".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "PK" followed by ETX EOT, the ZIP archive signature.
    fn application_zip() -> (r: ByteMatcher) 
        ensures 
            Spec::is_application_zip(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"PK\x03\x04", b"\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"PK\x03\x04",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/zip".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "Rar " followed by SUB BEL NUL, the RAR archive signature.
    fn application_x_rar_compressed() -> (r: ByteMatcher) 
        ensures 
            Spec::is_application_x_rar_compressed(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"Rar \x1A\x07\x00", b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"Rar \x1A\x07\x00",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/x-rar-compressed".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "%!PS-Adobe-", the PostScript signature.
    fn application_postscript() -> (r: ByteMatcher)
        ensures 
            Spec::is_application_postscript(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"%!PS-Adobe-", b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF");
        ByteMatcher {
            pattern: b"%!PS-Adobe-",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/postscript".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // UTF-16BE BOM
    fn text_plain_utf_16be_bom() -> (r: ByteMatcher)
        ensures 
            Spec::is_text_plain_utf_16be_bom(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\xFE\xFF\x00\x00", b"\xFF\xFF\x00\x00");
        ByteMatcher {
            pattern: b"\xFE\xFF\x00\x00",
            mask: b"\xFF\xFF\x00\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
    // UTF-16LE BOM
    fn text_plain_utf_16le_bom() -> (r: ByteMatcher)
        ensures 
            Spec::is_text_plain_utf_16le_bom(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\xFF\xFE\x00\x00", b"\xFF\xFF\x00\x00");
        ByteMatcher {
            pattern: b"\xFF\xFE\x00\x00",
            mask: b"\xFF\xFF\x00\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
    // UTF-8 BOM
    fn text_plain_utf_8_bom() -> (r: ByteMatcher)
        ensures 
            Spec::is_text_plain_utf_8_bom(&r),
            r.validate_spec(),
    {
        prove_valid_byte_literals!(b"\xEF\xBB\xBF\x00", b"\xFF\xFF\xFF\x00");
        ByteMatcher {
            pattern: b"\xEF\xBB\xBF\x00",
            mask: b"\xFF\xFF\xFF\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
}

} // verus!
