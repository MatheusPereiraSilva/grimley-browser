use std::sync::Arc;

use tao::window::Window;

use crate::{
    app::{LoadedUrls, PdfRoutes, PendingCommand, TabView},
    pdf::PdfFetcherHandle,
    shield::ShieldEngineHandle,
    storage::AppStorage,
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
        pending_command: PendingCommand,
        pdf_routes: PdfRoutes,
        pdf_fetcher: PdfFetcherHandle,
        shield_engine: ShieldEngineHandle,
        storage: AppStorage,
        home_page_url: &str,
    ) -> Self {
        Self::restore(
            window,
            loaded_urls,
            pending_command,
            pdf_routes,
            pdf_fetcher,
            shield_engine,
            storage,
            vec![TabLaunch::regular(home_page_url)],
            0,
        )
    }

    pub(crate) fn restore(
        window: &Window,
        loaded_urls: LoadedUrls,
        pending_command: PendingCommand,
        pdf_routes: PdfRoutes,
        pdf_fetcher: PdfFetcherHandle,
        shield_engine: ShieldEngineHandle,
        storage: AppStorage,
        launches: Vec<TabLaunch>,
        active_index: usize,
    ) -> Self {
        let launches = if launches.is_empty() {
            vec![TabLaunch::new_tab()]
        } else {
            launches
        };
        let active_index = active_index.min(launches.len().saturating_sub(1));
        let tabs = launches
            .into_iter()
            .enumerate()
            .map(|(index, launch)| {
                TabSession::new(
                    window,
                    index,
                    Arc::clone(&loaded_urls),
                    Arc::clone(&pending_command),
                    Arc::clone(&pdf_routes),
                    Arc::clone(&pdf_fetcher),
                    Arc::clone(&shield_engine),
                    storage.clone(),
                    index == active_index,
                    launch,
                )
            })
            .collect::<Vec<_>>();

        Self {
            next_tab_id: tabs.len(),
            tabs,
            active_index,
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
        pending_command: PendingCommand,
        pdf_routes: PdfRoutes,
        pdf_fetcher: PdfFetcherHandle,
        shield_engine: ShieldEngineHandle,
        storage: AppStorage,
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
            pending_command,
            pdf_routes,
            pdf_fetcher,
            shield_engine,
            storage,
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
        pending_command: PendingCommand,
        pdf_routes: PdfRoutes,
        pdf_fetcher: PdfFetcherHandle,
        shield_engine: ShieldEngineHandle,
        storage: AppStorage,
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
                pending_command,
                pdf_routes,
                pdf_fetcher,
                shield_engine,
                storage,
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

    pub(crate) fn tab_views(&self) -> Vec<TabView> {
        self.tabs
            .iter()
            .map(|tab| TabView {
                label: tab.title().to_string(),
                url: tab.display_url(),
            })
            .collect()
    }

    pub(crate) fn session_launches(&self) -> Vec<TabLaunch> {
        self.tabs.iter().map(TabSession::session_launch).collect()
    }
}
