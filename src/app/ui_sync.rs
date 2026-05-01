use wry::WebView;

use crate::tabs::BrowserTabs;

use super::{state::UiSnapshot, UiEvent};

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
        tabs: tabs.tab_views(),
        active_index: tabs.active_index(),
    };

    let previous = last_snapshot.as_ref();
    let mut events = Vec::new();

    if previous
        .map(|state| state.url != snapshot.url)
        .unwrap_or(true)
    {
        events.push(UiEvent::UrlChanged {
            url: snapshot.url.clone(),
        });
    }

    if previous
        .map(|state| {
            state.can_go_back != snapshot.can_go_back
                || state.can_go_forward != snapshot.can_go_forward
        })
        .unwrap_or(true)
    {
        events.push(UiEvent::NavState {
            back: snapshot.can_go_back,
            forward: snapshot.can_go_forward,
        });
    }

    if previous
        .map(|state| state.tabs != snapshot.tabs || state.active_index != snapshot.active_index)
        .unwrap_or(true)
    {
        events.push(UiEvent::TabsChanged {
            tabs: snapshot.tabs.clone(),
            active: snapshot.active_index,
        });
    }

    if events.is_empty() {
        return;
    }

    ui_webview
        .evaluate_script(
            &events
                .iter()
                .map(UiEvent::to_script)
                .collect::<Vec<_>>()
                .join(" "),
        )
        .expect("Erro ao sincronizar a interface");

    *last_snapshot = Some(snapshot);
}
