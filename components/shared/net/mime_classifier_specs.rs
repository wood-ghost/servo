#![allow(unsafe_code)]

use vstd::prelude::*;
use crate::LoadContext;

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


    pub uninterp spec fn essence_str(mt: &Mime) -> Seq<char>;
    pub assume_specification<'a>
        [Mime::essence_str]
        (mt: &'a Mime) -> (result: &'a str)
        ensures
            result@ == essence_str(mt),
    ;

    pub uninterp spec fn name_text<'a>(name: Name<'a>) -> Seq<char>;

    pub uninterp spec fn suffix_name(mt: &Mime) -> Option<Seq<char>>;
    pub assume_specification<'a>
        [Mime::suffix]
        (mt: &'a Mime) -> (result: Option<Name<'a>>)
        ensures
            match result {
                Some(name) => suffix_name(mt) == Some(name_text(name)),
                None => suffix_name(mt).is_none(),
            },
    ;

    pub uninterp spec fn type_name(mt: &Mime) -> Seq<char>;
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
        essence_str(mt) == "text/xml"@
    }

    pub(crate) open spec fn essence_is_application_xml(mt: &Mime) -> bool {
        essence_str(mt) == "application/xml"@
    }

    pub(crate) open spec fn essence_is_text_html(mt: &Mime) -> bool {
        essence_str(mt) == "text/html"@
    }

    pub(crate) uninterp spec fn is_image(mt: &Mime) -> bool;
    pub(crate) uninterp spec fn has_xml_suffix(mt: &Mime) -> bool;
    pub(crate) uninterp spec fn has_html_suffix(mt: &Mime) -> bool;
    pub(crate) uninterp spec fn is_text_plain(mt: &Mime) -> bool;
    pub(crate) uninterp spec fn is_text_plain_utf8(mt: &Mime) -> bool;

    pub uninterp spec fn dummy_mime() -> Mime;
}

pub mod SpecStd {
    use super::*;

    // pub assume_specification<T, F>
    //     [Option::<T>::or_else]
    //     (option: Option<T>, op: F) -> (result: Option<T>)
    // where
    //     F: FnOnce() -> Option<T>,
    //     requires
    //         option.is_some() || call_requires(op, ()),
    //     ensures
    //         match option {
    //             Some(value) => result == Some(value),
    //             None => call_ensures(op, (), result),
    //         },
    // ;

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

} // verus!