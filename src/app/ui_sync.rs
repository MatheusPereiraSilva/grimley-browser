use wry::WebView;

use crate::{browser::escape_js_string, tabs::BrowserTabs};

use super::state::UiSnapshot;

pub(crate) fn sync_ui(
    ui_webview: &WebView,
    tabs: &BrowserTabs,
    last_snapshot: &mut Option<UiSnapshot>,
) {
    let active_tab = tabs.active_tab();

    let snapshot = UiSnapshot {
        url: active_tab.display_url(),
        can_go_back: tabs.can_go_back(),
        can_go_forward: tabs.can_go_forward(),
        tabs_json: tabs.tabs_json(),
        active_index: tabs.active_index(),
    };

    let previous = last_snapshot.as_ref();
    let mut script_parts = Vec::new();

    if previous
        .map(|state| state.url != snapshot.url)
        .unwrap_or(true)
    {
        script_parts.push(format!(
            "window.setUrl('{url}');",
            url = escape_js_string(&snapshot.url),
        ));
    }

    if previous
        .map(|state| {
            state.can_go_back != snapshot.can_go_back
                || state.can_go_forward != snapshot.can_go_forward
        })
        .unwrap_or(true)
    {
        script_parts.push(format!(
            "window.setNavState({back}, {forward});",
            back = snapshot.can_go_back,
            forward = snapshot.can_go_forward,
        ));
    }

    if previous
        .map(|state| {
            state.tabs_json != snapshot.tabs_json || state.active_index != snapshot.active_index
        })
        .unwrap_or(true)
    {
        script_parts.push(format!(
            "window.setTabs(JSON.parse('{tabs}'), {active});",
            tabs = escape_js_string(&snapshot.tabs_json),
            active = snapshot.active_index,
        ));
    }

    if script_parts.is_empty() {
        return;
    }

    ui_webview
        .evaluate_script(&script_parts.join(" "))
        .expect("Erro ao sincronizar a interface");

    *last_snapshot = Some(snapshot);
}
