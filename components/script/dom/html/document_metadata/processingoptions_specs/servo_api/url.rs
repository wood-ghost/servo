/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Native URL observations, inspected 2026-10-30 against components/url/lib.rs.
//! ServoUrl delegates these methods to url 2.5.8. These are correspondence
//! assumptions, not a proof of the URL parser. Password None maps to the standard's
//! empty string; port() is the explicit port, not port_or_known_default().
//! Blob entries require explicit store/environment context in the view adapter.

#![allow(unsafe_code)]

use servo_url::ServoUrl;
use vstd::prelude::*;
use super::views::url_view;
use super::super::processing::types::{HostView, UrlPathView};

verus! {

// https://url.spec.whatwg.org/#special-scheme
// Snapshot 2026-10-19, inspected 2026-10-30. Default-port column of the table:
// "ftp" 21; "file" null; "http" 80; "https" 443; "ws" 80; "wss" 443.
// Table lookup for the native origin adapter, not a numbered standard algorithm.
pub(crate) open spec fn default_port(scheme: Seq<char>) -> Option<u16> {
    if scheme == "ftp"@ { Some(21u16) }
    else if scheme == "http"@ || scheme == "ws"@ { Some(80u16) }
    else if scheme == "https"@ || scheme == "wss"@ { Some(443u16) }
    else { None }
}

// https://url.spec.whatwg.org/#concept-host-serializer
// TRUSTED(scope): not verified yet; remove when the URL host serializer is verified
pub(crate) uninterp spec fn host_serialization(host: HostView) -> Seq<char>;

// https://url.spec.whatwg.org/#url-path-serializer
// TRUSTED(scope): not verified yet; remove when the URL path serializer is verified
pub(crate) uninterp spec fn path_serialization(path: UrlPathView) -> Seq<char>;

// Public closed projections keep the standard record crate-private while
// providing contracts with the same visibility as the external Rust methods.
pub closed spec fn scheme(url: &ServoUrl) -> Seq<char> { url_view(url).scheme }
pub closed spec fn username(url: &ServoUrl) -> Seq<char> { url_view(url).username }
pub closed spec fn password(url: &ServoUrl) -> Seq<char> { url_view(url).password }
pub closed spec fn host(url: &ServoUrl) -> Option<Seq<char>> {
    match url_view(url).host { None => None, Some(host) => Some(host_serialization(host)) }
}
pub closed spec fn port(url: &ServoUrl) -> Option<u16> { url_view(url).port }
pub closed spec fn path(url: &ServoUrl) -> Seq<char> { path_serialization(url_view(url).path) }
pub closed spec fn opaque_path(url: &ServoUrl) -> bool { url_view(url).path is Opaque }
pub closed spec fn query(url: &ServoUrl) -> Option<Seq<char>> { url_view(url).query }
pub closed spec fn fragment(url: &ServoUrl) -> Option<Seq<char>> { url_view(url).fragment }
pub open spec fn optional_string(value: Option<&str>) -> Option<Seq<char>> {
    match value { None => None, Some(s) => Some(s@) }
}

// TRUSTED(scope): not verified yet; remove when ServoUrl::scheme is verified
pub assume_specification<'a>[ServoUrl::scheme](url: &'a ServoUrl) -> (result: &'a str)
    ensures result@ == scheme(url);

// TRUSTED(scope): not verified yet; remove when ServoUrl::username is verified
pub assume_specification<'a>[ServoUrl::username](url: &'a ServoUrl) -> (result: &'a str)
    ensures result@ == username(url);

// TRUSTED(scope): not verified yet; remove when ServoUrl::password is verified
pub assume_specification<'a>[ServoUrl::password](url: &'a ServoUrl) -> (result: Option<&'a str>)
    ensures password(url) == match result { None => Seq::empty(), Some(value) => value@ };

// TRUSTED(scope): not verified yet; remove when ServoUrl::host_str is verified
pub assume_specification<'a>[ServoUrl::host_str](url: &'a ServoUrl) -> (result: Option<&'a str>)
    ensures optional_string(result) == host(url);

// TRUSTED(scope): not verified yet; remove when ServoUrl::port is verified
pub assume_specification[ServoUrl::port](url: &ServoUrl) -> (result: Option<u16>)
    ensures result == port(url);

// TRUSTED(scope): not verified yet; remove when ServoUrl::path is verified
pub assume_specification<'a>[ServoUrl::path](url: &'a ServoUrl) -> (result: &'a str)
    ensures result@ == path(url);

// TRUSTED(scope): not verified yet; remove when ServoUrl::cannot_be_a_base is verified
pub assume_specification[ServoUrl::cannot_be_a_base](url: &ServoUrl) -> (result: bool)
    ensures result == opaque_path(url);

// TRUSTED(scope): not verified yet; remove when ServoUrl::query is verified
pub assume_specification<'a>[ServoUrl::query](url: &'a ServoUrl) -> (result: Option<&'a str>)
    ensures optional_string(result) == query(url);

// TRUSTED(scope): not verified yet; remove when ServoUrl::fragment is verified
pub assume_specification<'a>[ServoUrl::fragment](url: &'a ServoUrl) -> (result: Option<&'a str>)
    ensures optional_string(result) == fragment(url);

pub closed spec fn serialization(url: &ServoUrl) -> Seq<char> {
    super::super::processing::algorithms::url::serialize_url(url_view(url), false)
}

pub closed spec fn equal_excluding_fragments(a: &ServoUrl, b: &ServoUrl) -> bool {
    super::super::processing::algorithms::url::url_equals(url_view(a), url_view(b), true)
}

// TRUSTED(scope): not verified yet; remove when ServoUrl::as_str correspondence is verified
pub assume_specification<'a>[ServoUrl::as_str](url: &'a ServoUrl) -> (result: &'a str)
    ensures result@ == serialization(url);

// TRUSTED(scope): not verified yet; remove when ServoUrl::is_equal_excluding_fragments is verified
pub assume_specification[ServoUrl::is_equal_excluding_fragments](a: &ServoUrl, b: &ServoUrl) -> (result: bool)
    ensures result == equal_excluding_fragments(a, b);

} // verus!
