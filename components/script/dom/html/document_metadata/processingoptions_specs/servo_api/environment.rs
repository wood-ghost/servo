/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Environment adapters, inspected 2026-10-30.
//! RequestClient is a partial Fetch client, not the HTML environment. Creation
//! URLs and API base URL belong to GlobalScope. Worker execution readiness lives
//! in WorkerGlobalScope; time origin belongs to Performance. Other observations
//! below must be supplied by the relevant navigation/worker/realm context.

#![allow(unsafe_code)]

use std::sync::Arc;
use net_traits::request::RequestClient;
use servo_url::ServoUrl;
use vstd::prelude::*;
use crate::dom::globalscope::GlobalScope;
use super::views::url_view;
use super::super::processing::types::*;

verus! {

/// Fields absent from RequestClient and from the inspected GlobalScope getters.
/// Values are explicit inputs, not assumed defaults for every environment.
pub(crate) struct EnvironmentSupplement {
    pub id: Seq<char>,
    pub top_level_origin: TopLevelOriginView,
    pub target_browsing_context: Option<BrowsingContextView>,
    pub active_service_worker: Option<ServiceWorkerView>,
    pub execution_ready_flag: bool,
}

pub(crate) struct SettingsSupplement {
    pub environment: EnvironmentSupplement,
    pub realm_execution_context: int,
    pub module_map: ModuleMapView,
    pub has_cross_site_ancestor: bool,
    pub cross_origin_isolated_capability: bool,
    pub time_origin: real,
}

// TRUSTED(scope): not verified yet; remove when GlobalScope.creation_url correspondence is verified
pub(crate) uninterp spec fn creation_url(global: &GlobalScope) -> ServoUrlView;
// TRUSTED(scope): not verified yet; remove when GlobalScope.top_level_creation_url correspondence is verified
pub(crate) uninterp spec fn top_level_creation_url(global: &GlobalScope) -> Option<ServoUrlView>;
// TRUSTED(scope): not verified yet; remove when GlobalScope::api_base_url correspondence is verified
pub(crate) uninterp spec fn api_base_url(global: &GlobalScope) -> ServoUrlView;
// TRUSTED(scope): not verified yet; remove when RequestClient.origin correspondence is verified
pub(crate) uninterp spec fn client_origin(client: &RequestClient) -> OriginView;
// TRUSTED(scope): not verified yet; remove when RequestClient.policy_container correspondence is verified
pub(crate) uninterp spec fn client_policy_container(client: &RequestClient) -> Arc<PolicyContainerView>;

pub(crate) open spec fn environment_with_context(global: &GlobalScope, context: EnvironmentSupplement) -> EnvironmentView {
    EnvironmentView {
        id: context.id,
        creation_url: creation_url(global),
        top_level_creation_url: top_level_creation_url(global),
        top_level_origin: context.top_level_origin,
        target_browsing_context: context.target_browsing_context,
        active_service_worker: context.active_service_worker,
        execution_ready_flag: context.execution_ready_flag,
    }
}

pub(crate) open spec fn settings_with_context(
    client: &RequestClient, global: &GlobalScope, context: SettingsSupplement,
) -> EnvironmentSettingsView {
    EnvironmentSettingsView {
        environment: environment_with_context(global, context.environment),
        realm_execution_context: context.realm_execution_context,
        module_map: context.module_map,
        api_base_url: api_base_url(global),
        origin: client_origin(client),
        has_cross_site_ancestor: context.has_cross_site_ancestor,
        policy_container: client_policy_container(client),
        cross_origin_isolated_capability: context.cross_origin_isolated_capability,
        time_origin: context.time_origin,
    }
}

// TRUSTED(scope): not verified yet; remove when GlobalScope::creation_url is verified
pub(crate) assume_specification[GlobalScope::creation_url](global: &GlobalScope) -> (result: ServoUrl)
    ensures url_view(&result) == creation_url(global);

// TRUSTED(scope): not verified yet; remove when GlobalScope::top_level_creation_url is verified
pub(crate) assume_specification[GlobalScope::top_level_creation_url](global: &GlobalScope) -> (result: Option<ServoUrl>)
    ensures match result {
        None => top_level_creation_url(global) is None,
        Some(url) => top_level_creation_url(global) == Some(url_view(&url)),
    };

// TRUSTED(scope): not verified yet; remove when GlobalScope::api_base_url is verified
pub(crate) assume_specification[GlobalScope::api_base_url](global: &GlobalScope) -> (result: ServoUrl)
    ensures url_view(&result) == api_base_url(global);

} // verus!
