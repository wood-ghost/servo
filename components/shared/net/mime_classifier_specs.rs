#![allow(unsafe_code)]

use vstd::prelude::*;
use crate::LoadContext;
use verus_state_machines_macros::state_machine;

use mime::{self, Mime, Name};

use crate::mime_classifier::{MediaType, ApacheBugFlag, NoSniffFlag};

verus! {

pub mod SpecMime {
    use super::*;

    #[verifier::external_type_specification]
    #[verifier::external_body]
    pub struct ExMime(Mime);

    #[verifier::external_type_specification]
    #[verifier::external_body]
    pub struct ExName<'a>(Name<'a>);

    #[verifier::external_type_specification]
    #[verifier::external_body]
    pub struct ExFromStrError(mime::FromStrError);
    
    // abstract Mime
    pub struct MimeView {
        pub type_: Seq<char>,
        pub subtype: Seq<char>,
        pub suffix: Option<Seq<char>>,
        pub essence: Seq<char>,
    }

    pub uninterp spec fn view(mt: &Mime) -> MimeView;

    // Constant
    pub uninterp spec fn mime_identity(mt: &Mime) -> int;

    pub uninterp spec fn text_plain_identity() -> int;
    pub(crate) assume_specification [mime::TEXT_PLAIN] -> (result: Mime)
        ensures
            mime_identity(&result) == text_plain_identity(),
    ;

    pub uninterp spec fn text_plain_utf8_identity() -> int;
    pub(crate) assume_specification [mime::TEXT_PLAIN_UTF_8] -> (result: Mime)
        ensures
            mime_identity(&result) == text_plain_utf8_identity(),
    ;

    pub uninterp spec fn application_octet_stream_identity() -> int;
    pub(crate) assume_specification [mime::APPLICATION_OCTET_STREAM] -> (result: Mime)
        ensures
            mime_identity(&result) == application_octet_stream_identity(),
    ;


    pub uninterp spec fn text_css() -> int;
    pub(crate) assume_specification [mime::TEXT_CSS] -> (result: Mime)
        ensures
            mime_identity(&result) == text_css(),
    ;

    pub uninterp spec fn text_javascript() -> int;
    pub(crate) assume_specification [mime::TEXT_JAVASCRIPT] -> (result: Mime)
        ensures
            mime_identity(&result) == text_javascript(),
    ;

    // NAME
    pub uninterp spec fn name_identity<'a>(name: &Name<'a>,) -> int;

    pub uninterp spec fn xml_identity() -> int;
    pub assume_specification [mime::XML] -> (result: Name<'static>)
        ensures
            name_identity(&result) == xml_identity(),
    ;

    pub uninterp spec fn image_identity() -> int;
    pub assume_specification [mime::IMAGE] -> (result: Name<'static>)
        ensures
            name_identity(&result) == image_identity(),
    ;

    pub uninterp spec fn audio_identity() -> int;
    pub assume_specification [mime::AUDIO] -> (result: Name<'static>)
        ensures
            name_identity(&result) == audio_identity(),
    ;

    pub uninterp spec fn video_identity() -> int;
    pub assume_specification [mime::VIDEO] -> (result: Name<'static>)
        ensures
            name_identity(&result) == video_identity(),
    ;

    pub uninterp spec fn application_identity() -> int;
    pub assume_specification [mime::APPLICATION] -> (result: Name<'static>)
        ensures
            name_identity(&result) == application_identity(),
    ;

    pub uninterp spec fn star_identity() -> int;
    pub assume_specification [mime::STAR] -> (result: Name<'static>)
        ensures
            name_identity(&result) == star_identity(),
    ;

    pub uninterp spec fn text_identity() -> int;
    pub assume_specification [mime::TEXT] -> (result: Name<'static>)
        ensures
            name_identity(&result) == text_identity(),
    ;

    pub uninterp spec fn json_identity() -> int;
    pub assume_specification [mime::JSON] -> (result: Name<'static>)
        ensures
            name_identity(&result) == json_identity(),
    ;

    pub uninterp spec fn font_identity() -> int;
    pub assume_specification [mime::FONT] -> (result: Name<'static>)
        ensures
            name_identity(&result) == font_identity(),
    ;


    pub open spec fn essence_str(mt: &Mime) -> Seq<char> {
        view(mt).essence
    }
    pub assume_specification<'a>
        [Mime::essence_str]
        (mt: &'a Mime) -> (result: &'a str)
        ensures
            result@ =~= essence_str(mt),
    ;

    pub uninterp spec fn name_text<'a>(name: Name<'a>) -> Seq<char>;

    pub open spec fn suffix_name(mt: &Mime) -> Option<Seq<char>> {
        view(mt).suffix
    }
    pub assume_specification<'a>
        [Mime::suffix]
        (mt: &'a Mime) -> (result: Option<Name<'a>>)
        ensures
            match result {
                Some(name) => suffix_name(mt) == Some(name_text(name)),
                None => suffix_name(mt).is_none(),
            },
    ;

    pub open spec fn type_name(mt: &Mime) -> Seq<char> {
        view(mt).type_
    }
    pub assume_specification<'a>
        [Mime::type_]
        (mt: &'a Mime) -> (result: Name<'a>)
        ensures
            name_text(result) == type_name(mt),
    ;

    pub uninterp spec fn subtype_name(mt: &Mime) -> Seq<char>;
    pub assume_specification<'a>
        [Mime::subtype]
        (mt: &'a Mime) -> (result: Name<'a>)
        ensures
            name_text(result) == subtype_name(mt),
    ;

    pub assume_specification<'a>
        [Name::<'a>::as_str]
        (name: &Name<'a>) -> (result: &'a str)
        ensures
            result@ == name_text(*name),
    ;

    pub(crate) open spec fn essence_is_text_xml(mt: &Mime) -> bool {
        essence_str(mt) =~= "text/xml"@
    }

    pub(crate) open spec fn essence_is_application_xml(mt: &Mime) -> bool {
        essence_str(mt) =~= "application/xml"@
    }

    pub(crate) open spec fn essence_is_text_html(mt: &Mime) -> bool {
        essence_str(mt) =~= "text/html"@
    }

    pub(crate) open spec fn essence_is_application_ogg(mt: &Mime) -> bool {
        essence_str(mt) =~= "application/ogg"@
    }

    pub open spec fn is_image(mt: &Mime) -> bool {
        type_name(mt) =~= "image"@
    }

    pub open spec fn is_audio(mt: &Mime) -> bool {
        type_name(mt) =~= "audio"@
    }
    pub open spec fn is_video(mt: &Mime) -> bool {
        type_name(mt) =~= "video"@
    }
    pub open spec fn has_xml_suffix(mt: &Mime) -> bool {
        suffix_name(mt) == Some("xml"@)
    }
    pub(crate) uninterp spec fn has_html_suffix(mt: &Mime) -> bool;
    pub uninterp spec fn is_text_plain(mt: &Mime) -> bool;
    pub uninterp spec fn is_text_plain_utf8(mt: &Mime) -> bool;

    pub uninterp spec fn dummy_mime() -> Mime;
}

pub mod SpecStd {
    use super::*;

    #[verifier::allow(undeclared_external_trait)]
    pub assume_specification<T, F>
        [Option::<T>::or_else]
        (option: Option<T>, op: F) -> (result: Option<T>)
    where
        F: FnOnce() -> Option<T> + std::marker::Destruct,
        T: std::marker::Destruct,
        requires
            option.is_some() || call_requires(op, ()),
        ensures
            match option {
                Some(value) => result == Some(value),
                None => call_ensures(op, (), result),
            },
    ;

    pub assume_specification<T>
        [<[T]>::contains]
        (slice: &[T], value: &T) -> bool
    where
        T: std::cmp::PartialEq,
    ;

    pub assume_specification<'a, T, P>
        [<std::slice::Iter<'a, T> as std::iter::Iterator>::position]
        (
            iter: &mut std::slice::Iter<'a, T>,
            predicate: P,
        ) -> Option<usize>
    where
        P: FnMut(
            <std::slice::Iter<'a, T> as std::iter::Iterator>::Item,
        ) -> bool,
        std::slice::Iter<'a, T>: Sized,
    ;

    #[verifier::reject_recursive_types(A)]
    #[verifier::reject_recursive_types(B)]
    #[verifier::external_type_specification]
    #[verifier::external_body]
    pub struct ExZip<A, B>(std::iter::Zip<A, B>);

    #[verifier::reject_recursive_types(T)]
    #[verifier::external_type_specification]
    #[verifier::external_body]
    pub struct ExChunks<'a, T: 'a>(std::slice::Chunks<'a, T>);

    pub assume_specification<'a, T>
        [<[T]>::chunks]
        (
            slice: &'a [T],
            chunk_size: usize,
        ) -> (result: std::slice::Chunks<'a, T>)
        requires
            chunk_size > 0,
    ;
    
    pub assume_specification<T>
        [<[T]>::starts_with]
        (
            slice: &[T],
            prefix: &[T],
        ) -> (result: bool)
    where
        T: std::cmp::PartialEq,
    ;

    #[verifier::allow(undeclared_external_trait)]
    pub assume_specification<T, F>
        [Option::<T>::is_some_and]
        (
            option: Option<T>,
            predicate: F,
        ) -> (result: bool)
    where
        F: FnOnce(T) -> bool + std::marker::Destruct,
        requires
            match option {
                Some(value) => call_requires(predicate, (value,)),
                None => true,
            },
        ensures
            match option {
                Some(value) => call_ensures(predicate, (value,), result),
                None => !result,
            },
    ;
}

pub mod SpecMimeClassifier {
    use super::*;
    
    // Let supplied-type be null. 
    pub(crate) open spec fn is_text_plain(mt: &Mime) -> bool {
        SpecMime::is_text_plain(mt)
    }

    // check if the resource is retrieved via HTTP
    pub(crate) open spec fn is_xml(mt: &Mime) -> bool {
        !SpecMime::is_image(mt) && (
            SpecMime::has_xml_suffix(mt)
                || SpecMime::essence_is_text_xml(mt)
                || SpecMime::essence_is_application_xml(mt))
    } 

    pub(crate) open spec fn is_html(mt: &Mime) -> bool {
        SpecMime::essence_is_text_html(mt)
    }

    pub(crate) open spec fn is_image(mt: &Mime) -> bool {
        SpecMime::is_image(mt)
    }

    pub(crate) open spec fn is_audio_video(mt: &Mime) -> bool {
        SpecMime::is_audio(mt) || SpecMime::is_video(mt) || SpecMime::essence_is_application_ogg(mt)
    }

    pub open spec fn classify_input_is_valid<'a>(
        context: LoadContext,
        no_sniff_flag: NoSniffFlag,
        apache_bug_flag: ApacheBugFlag,
        supplied_type: &Option<Mime>,
        data: &'a [u8],
    ) -> bool {
        // TODO:
        true
    }
    
    pub open spec fn classify<'a>(
        context: LoadContext,
        no_sniff_flag: NoSniffFlag,
        apache_bug_flag: ApacheBugFlag,
        supplied_type: &Option<Mime>,
        data: &'a [u8],
    ) -> Mime {
        // TODO:
        SpecMime::dummy_mime()
    }
}

pub mod SpecMimeChecker {
    use super::*;

    pub open spec fn equal_b_space_or_g(d: u8) -> bool {
        d == 0x20u8 || d == 0x3eu8
    }

}

pub mod SpecApacheBugFlag {
    use super::*;
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
}

// https://mimesniff.spec.whatwg.org/#supplied-mime-type-detection-algorithm
//TODO: change to sm
pub mod SpecSuppliedTypeDetect {
    use super::*;

    pub struct ResourceModel {
        pub context: LoadContext,
        pub no_sniff_flag: NoSniffFlag,
        pub apache_bug_flag: ApacheBugFlag,
        pub supplied_type: Option<Mime>,
        // the resource is retrieved via ...
        pub retrieval_kind: RetrievalKind,
    }

    impl ResourceModel {
        pub uninterp spec fn retrieve_from(&self) -> RetrievalKind;
        pub uninterp spec fn get_content_type_headers(&self) -> Seq<Mime>;
        pub uninterp spec fn set_supplied_type(&self, mt: Option<Mime>);
        pub uninterp spec fn get_supplied_type(&self) -> Option<Mime>;
        pub uninterp spec fn set_check_for_apache_bug_flag(&self);
        pub uninterp spec fn get_type_from_file_system(&self) -> Mime;
        pub uninterp spec fn get_type_from_protocol(&self) -> Option<Mime>;
        pub uninterp spec fn supplied_type_is_mime_type(&self) -> bool;
    }

    pub enum RetrievalKind {
        Http,
        FileSystem,
        Protocol,
        Undefined,
    }

}

/*
pub enum DetectionResult {
    Pending,
    Defined,
    Undefined,
}

state_machine! {
    SuppliedTypeDetectSM {
        //// The state definition
         
        fields {
            pub supplied_type: Option<Mime>,
            pub retrieval_kind: SpecSuppliedTypeDetect::RetrievalKind,
            pub apache_bug_flag: ApacheBugFlag,
            pub detection_result: DetectionResult,
        }

        //// The transitions

        init!{
            initialize() { // Let supplied-type be null. 
                init supplied_type = None;
                init retrieval_kind = SpecSuppliedTypeDetect::RetrievalKind::Undefined;
                init apache_bug_flag = ApacheBugFlag::Off;
                init detection_result = DetectionResult::Pending;
            }
        }

        transition!{
            get_retrieve_kind(resource: SpecSuppliedTypeDetect::ResourceModel) {
                require(pre.retrieval_kind == SpecSuppliedTypeDetect::RetrievalKind::Undefined 
                    && pre.supplied_type == None
                    && pre.detection_result == DetectionResult::Pending
                );
                update retrieval_kind = resource.retrieve_from();
            }
        }

        transition!{
            // If the resource is retrieved via HTTP
            handle_http(resource: SpecSuppliedTypeDetect::ResourceModel) {
                require(pre.retrieval_kind == SpecSuppliedTypeDetect::RetrievalKind::Http 
                    && pre.supplied_type == None
                    && pre.detection_result == DetectionResult::Pending
                );
                let content_type_headers = resource.get_content_type_headers();
                if content_type_headers.len() > 0 { // If one or more Content-Type headers are associated with the resource, execute the following steps: 
                    let last_header = content_type_headers[content_type_headers.len() - 1];
                    let new_apache_bug_flag = SpecApacheBugFlag::from_content_type(Some(&last_header));

                    // Set supplied-type to the value of the last Content-Type header associated with the resource. 
                    update supplied_type = Some(last_header);
                    // Set the check-for-apache-bug flag if supplied-type is exactly equal to one of the values in the following table:
                    update apache_bug_flag = new_apache_bug_flag;
                    update detection_result = DetectionResult::Defined;
                } else {
                    update detection_result = DetectionResult::Undefined;
                }
            }
        }

        transition!{
            // If the resource is retrieved directly from the file system
            handle_file_system(resource: SpecSuppliedTypeDetect::ResourceModel) {
                require(pre.retrieval_kind == SpecSuppliedTypeDetect::RetrievalKind::FileSystem 
                    && pre.supplied_type == None
                    && pre.detection_result == DetectionResult::Pending
                );
                let mime_type = resource.get_type_from_file_system();
                // set supplied-type to the MIME type provided by the file system. 
                update supplied_type = Some(mime_type);
                // if mime_type == None {
                    // update detection_result = DetectionResult::Undefined;
                // } else {
                    update detection_result = DetectionResult::Defined;
                // }
            }
        }

        transition!{
            // If the resource is retrieved via another protocol (such as FTP)
            handle_protocol(resource: SpecSuppliedTypeDetect::ResourceModel) {
                require(pre.retrieval_kind == SpecSuppliedTypeDetect::RetrievalKind::Protocol 
                    && pre.supplied_type == None
                    && pre.detection_result == DetectionResult::Pending
                );
                let mime_type = resource.get_type_from_protocol();
                // set supplied-type to the MIME type as determined by that protocol
                update supplied_type = mime_type;
                if mime_type.is_none() {
                    update detection_result = DetectionResult::Undefined;
                } else {
                    update detection_result = DetectionResult::Defined;
                }
            }
        }

        property!{
            supplied_type_valid() { //The supplied MIME type is supplied-type. 
                require(pre.detection_result == DetectionResult::Defined);
                assert(pre.supplied_type.is_some());
            }
        }

        property!{
            supplied_type_undefined() { //If supplied-type is not a MIME type, the supplied MIME type is undefined. Abort these steps.
                require(pre.detection_result == DetectionResult::Undefined);
                assert(pre.supplied_type.is_none());
            }
        }
     }
}
*/

} // verus!