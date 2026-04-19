#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use super::{
    diagnostics::{
        create_shield_diagnostics, record_request_diagnostic, snapshot_shield_diagnostics,
        ShieldDiagnosticsHandle, ShieldDiagnosticsSnapshot,
    },
    request::{is_internal_browser_url, ShieldRequest, ShieldRequestContext},
    rules::ShieldRuleset,
};

pub(crate) type ShieldEngineHandle = Arc<ShieldEngine>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShieldAction {
    Allow,
    Block,
    Redirect(String),
    CosmeticHide(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShieldDecision {
    applied_action: ShieldAction,
    proposed_action: ShieldAction,
    matched_rules: Vec<String>,
    applied_exception: Option<String>,
}

pub(crate) struct ShieldEngine {
    rules: Mutex<ShieldRuleset>,
    diagnostics: ShieldDiagnosticsHandle,
    observation_only: Mutex<bool>,
}

pub(crate) fn create_shield_engine() -> ShieldEngineHandle {
    Arc::new(ShieldEngine {
        rules: Mutex::new(ShieldRuleset::from_text("")),
        diagnostics: create_shield_diagnostics(),
        observation_only: Mutex::new(true),
    })
}

impl ShieldAction {
    fn label(&self) -> String {
        match self {
            Self::Allow => "Allow".to_string(),
            Self::Block => "Block".to_string(),
            Self::Redirect(target) => format!("Redirect -> {target}"),
            Self::CosmeticHide(selector) => format!("CosmeticHide -> {selector}"),
        }
    }
}

impl ShieldDecision {
    fn allow() -> Self {
        Self {
            applied_action: ShieldAction::Allow,
            proposed_action: ShieldAction::Allow,
            matched_rules: Vec::new(),
            applied_exception: None,
        }
    }

    pub(crate) fn should_allow(&self) -> bool {
        matches!(self.applied_action, ShieldAction::Allow)
    }
}

impl ShieldEngine {
    pub(crate) fn observe_navigation(
        &self,
        tab_id: usize,
        url: &str,
        source_url: Option<&str>,
    ) -> ShieldDecision {
        self.observe(ShieldRequest::new(
            tab_id,
            url,
            ShieldRequestContext::Navigation,
            source_url.map(str::to_string),
        ))
    }

    pub(crate) fn observe_new_window(
        &self,
        tab_id: usize,
        url: &str,
        source_url: Option<&str>,
    ) -> ShieldDecision {
        self.observe(ShieldRequest::new(
            tab_id,
            url,
            ShieldRequestContext::NewWindow,
            source_url.map(str::to_string),
        ))
    }

    pub(crate) fn diagnostics_snapshot(&self) -> ShieldDiagnosticsSnapshot {
        snapshot_shield_diagnostics(&self.diagnostics)
    }

    pub(crate) fn set_observation_only(&self, observation_only: bool) {
        *self.observation_only.lock().unwrap() = observation_only;
    }

    pub(crate) fn set_rules_text(&self, rules_text: &str) {
        *self.rules.lock().unwrap() = ShieldRuleset::from_text(rules_text);
    }

    pub(crate) fn rule_count(&self) -> usize {
        self.rules.lock().unwrap().len()
    }

    pub(crate) fn observation_only(&self) -> bool {
        *self.observation_only.lock().unwrap()
    }

    fn observe(&self, request: ShieldRequest) -> ShieldDecision {
        if is_internal_browser_url(request.url()) {
            return ShieldDecision::allow();
        }

        let matches = self.rules.lock().unwrap().find_matches(&request);
        let matched_rules = matches
            .blocked
            .iter()
            .map(|rule| rule.label().to_string())
            .collect::<Vec<_>>();
        let applied_exception = matches
            .exception
            .as_ref()
            .map(|rule| rule.label().to_string());
        let proposed_action = if applied_exception.is_some() {
            ShieldAction::Allow
        } else if matched_rules.is_empty() {
            ShieldAction::Allow
        } else {
            ShieldAction::Block
        };
        let applied_action = if self.observation_only() {
            ShieldAction::Allow
        } else {
            proposed_action.clone()
        };

        record_request_diagnostic(
            &self.diagnostics,
            &request,
            &proposed_action.label(),
            &applied_action.label(),
            &matched_rules,
            applied_exception.as_deref(),
        );

        ShieldDecision {
            applied_action,
            proposed_action,
            matched_rules,
            applied_exception,
        }
    }
}
