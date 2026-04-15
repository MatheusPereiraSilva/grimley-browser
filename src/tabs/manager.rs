use tao::window::Window;

use crate::{
    app::{LoadedUrls, PdfRoutes, PendingAction},
    browser::HOME_PAGE_URL,
};

use super::{TabLaunch, TabSession};

pub(crate) struct BrowserTabs {
    tabs: Vec<TabSession>,
    active_index: usize,
    next_tab_id: usize,
}

impl BrowserTabs {
    pub(crate) fn new(
        window: &Window,
        loaded_urls: LoadedUrls,
        pending_action: PendingAction,
        pdf_routes: PdfRoutes,
    ) -> Self {
        Self {
            tabs: vec![TabSession::new(
                window,
                0,
                loaded_urls,
                pending_action,
                pdf_routes,
                true,
                TabLaunch::regular(HOME_PAGE_URL),
            )],
            active_index: 0,
            next_tab_id: 1,
        }
    }

    pub(crate) fn active_tab(&self) -> &TabSession {
        &self.tabs[self.active_index]
    }

    pub(crate) fn active_tab_mut(&mut self) -> &mut TabSession {
        &mut self.tabs[self.active_index]
    }

    pub(crate) fn active_index(&self) -> usize {
        self.active_index
    }

    pub(crate) fn tabs(&self) -> &[TabSession] {
        &self.tabs
    }

    pub(crate) fn tab_mut(&mut self, index: usize) -> Option<&mut TabSession> {
        self.tabs.get_mut(index)
    }

    pub(crate) fn navigate_active(&mut self, url: &str) -> String {
        self.active_tab_mut().navigate_to_web(url)
    }

    pub(crate) fn back_active(&mut self) -> Option<String> {
        self.active_tab_mut().go_back()
    }

    pub(crate) fn forward_active(&mut self) -> Option<String> {
        self.active_tab_mut().go_forward()
    }

    pub(crate) fn open_new_tab(
        &mut self,
        window: &Window,
        loaded_urls: LoadedUrls,
        pending_action: PendingAction,
        pdf_routes: PdfRoutes,
        launch: TabLaunch,
    ) -> usize {
        if let Some(current) = self.tabs.get_mut(self.active_index) {
            current.suspend_internal_page();
            current.unload_webview();
        }

        self.tabs.push(TabSession::new(
            window,
            self.next_tab_id,
            loaded_urls,
            pending_action,
            pdf_routes,
            true,
            launch,
        ));
        self.next_tab_id += 1;
        self.active_index = self.tabs.len() - 1;
        self.active_index
    }

    pub(crate) fn switch_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() || index == self.active_index {
            return false;
        }

        self.tabs[self.active_index].suspend_internal_page();
        self.tabs[self.active_index].unload_webview();
        self.active_index = index;
        true
    }

    pub(crate) fn close_tab(
        &mut self,
        window: &Window,
        loaded_urls: LoadedUrls,
        pending_action: PendingAction,
        pdf_routes: PdfRoutes,
        index: usize,
    ) {
        if index >= self.tabs.len() {
            return;
        }

        let was_active = index == self.active_index;
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.tabs.push(TabSession::new(
                window,
                self.next_tab_id,
                loaded_urls,
                pending_action,
                pdf_routes,
                true,
                TabLaunch::new_tab(),
            ));
            self.next_tab_id += 1;
            self.active_index = 0;
            return;
        }

        if self.active_index > index {
            self.active_index -= 1;
        } else if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len() - 1;
        }

        if was_active && self.active_index < self.tabs.len() {
            self.tabs[self.active_index].mark_needs_render();
        }
    }

    pub(crate) fn set_loaded_url(&mut self, index: usize, url: &str) {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.set_loaded_url(url);
        }
    }

    pub(crate) fn find_index_by_id(&self, id: usize) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.id() == id)
    }

    pub(crate) fn can_go_back(&self) -> bool {
        self.active_tab().can_go_back()
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        self.active_tab().can_go_forward()
    }

    pub(crate) fn tabs_json(&self) -> String {
        let entries = self
            .tabs
            .iter()
            .map(|tab| {
                format!(
                    r#"{{"label":"{}","url":"{}"}}"#,
                    escape_json(tab.title()),
                    escape_json(&tab.display_url())
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("[{}]", entries)
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
