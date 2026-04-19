use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use super::request::{ShieldRequest, ShieldRequestContext, ShieldResourceType};

pub(crate) type ShieldDiagnosticsHandle = Arc<Mutex<ShieldDiagnosticsStore>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShieldRequestLog {
    pub(crate) sequence: u64,
    pub(crate) observed_at_ms: u128,
    pub(crate) tab_id: usize,
    pub(crate) url: String,
    pub(crate) host: Option<String>,
    pub(crate) source_url: Option<String>,
    pub(crate) context: ShieldRequestContext,
    pub(crate) resource_type: ShieldResourceType,
    pub(crate) proposed_action: String,
    pub(crate) applied_action: String,
    pub(crate) matched_rules: Vec<String>,
    pub(crate) applied_exception: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShieldDiagnosticsSnapshot {
    pub(crate) seen_count: usize,
    pub(crate) blocked_count: usize,
    pub(crate) would_block_count: usize,
    pub(crate) exception_count: usize,
    pub(crate) requests: Vec<ShieldRequestLog>,
}

#[derive(Default)]
pub(crate) struct ShieldDiagnosticsStore {
    next_sequence: u64,
    seen_count: usize,
    blocked_count: usize,
    would_block_count: usize,
    exception_count: usize,
    requests: Vec<ShieldRequestLog>,
}

pub(crate) fn create_shield_diagnostics() -> ShieldDiagnosticsHandle {
    Arc::new(Mutex::new(ShieldDiagnosticsStore::default()))
}

pub(crate) fn record_request_diagnostic(
    diagnostics: &ShieldDiagnosticsHandle,
    request: &ShieldRequest,
    proposed_action: &str,
    applied_action: &str,
    matched_rules: &[String],
    applied_exception: Option<&str>,
) {
    let mut diagnostics = diagnostics.lock().unwrap();

    diagnostics.next_sequence += 1;
    diagnostics.seen_count += 1;

    if proposed_action != "Allow" {
        diagnostics.would_block_count += 1;
    }

    if applied_action != "Allow" {
        diagnostics.blocked_count += 1;
    }

    if applied_exception.is_some() {
        diagnostics.exception_count += 1;
    }

    let sequence = diagnostics.next_sequence;
    diagnostics.requests.push(ShieldRequestLog {
        sequence,
        observed_at_ms: now_epoch_ms(),
        tab_id: request.tab_id(),
        url: request.url().to_string(),
        host: request.host().map(str::to_string),
        source_url: request.source_url().map(str::to_string),
        context: request.context(),
        resource_type: request.resource_type(),
        proposed_action: proposed_action.to_string(),
        applied_action: applied_action.to_string(),
        matched_rules: matched_rules.to_vec(),
        applied_exception: applied_exception.map(str::to_string),
    });

    if diagnostics.requests.len() > 400 {
        let overflow = diagnostics.requests.len() - 400;
        diagnostics.requests.drain(0..overflow);
    }
}

pub(crate) fn snapshot_shield_diagnostics(
    diagnostics: &ShieldDiagnosticsHandle,
) -> ShieldDiagnosticsSnapshot {
    let diagnostics = diagnostics.lock().unwrap();

    ShieldDiagnosticsSnapshot {
        seen_count: diagnostics.seen_count,
        blocked_count: diagnostics.blocked_count,
        would_block_count: diagnostics.would_block_count,
        exception_count: diagnostics.exception_count,
        requests: diagnostics.requests.clone(),
    }
}

fn now_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
