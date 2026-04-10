use tao::window::Window;
use wry::WebView;

use crate::browser::create_content_webview_with_url;

use super::{
    constants::{HISTORY_PAGE_URL, HOME_PAGE_URL, NEW_TAB_PAGE_URL, PDF_PAGE_URL},
    escape::escape_json,
    history::BrowserHistory,
    pages::render_new_tab_html,
    LoadedUrls, PendingAction,
};

pub(crate) struct TabLaunch {
    actual_url: String,
    initial_load_url: String,
    pending_internal_page_html: Option<String>,
    allow_pdf_navigation: bool,
    pdf_source_url: Option<String>,
    pdf_data_base64: Option<String>,
    pdf_load_error: Option<String>,
}

impl TabLaunch {
    pub(crate) fn regular(url: impl Into<String>) -> Self {
        let url = url.into();

        Self {
            actual_url: url.clone(),
            initial_load_url: url,
            pending_internal_page_html: None,
            allow_pdf_navigation: false,
            pdf_source_url: None,
            pdf_data_base64: None,
            pdf_load_error: None,
        }
    }

    pub(crate) fn new_tab() -> Self {
        Self {
            actual_url: NEW_TAB_PAGE_URL.to_string(),
            initial_load_url: NEW_TAB_PAGE_URL.to_string(),
            pending_internal_page_html: Some(render_new_tab_html()),
            allow_pdf_navigation: false,
            pdf_source_url: None,
            pdf_data_base64: None,
            pdf_load_error: None,
        }
    }

    pub(crate) fn pdf(
        pdf_url: impl Into<String>,
        pdf_data_base64: Option<String>,
        pdf_load_error: Option<String>,
        allow_pdf_navigation: bool,
    ) -> Self {
        let pdf_url = pdf_url.into();

        Self {
            actual_url: pdf_url.clone(),
            initial_load_url: PDF_PAGE_URL.to_string(),
            pending_internal_page_html: None,
            allow_pdf_navigation,
            pdf_source_url: Some(pdf_url),
            pdf_data_base64,
            pdf_load_error,
        }
    }
}

struct TabState {
    current_url: String,
    history: BrowserHistory,
}

impl TabState {
    fn new(url: &str) -> Self {
        Self {
            current_url: url.to_string(),
            history: BrowserHistory::new(url),
        }
    }

    fn navigate_to(&mut self, url: &str) -> String {
        self.current_url = url.to_string();
        self.history.navigate_to(url)
    }

    fn go_back(&mut self) -> Option<String> {
        let target = self.history.go_back()?;
        self.current_url = target.clone();
        Some(target)
    }

    fn go_forward(&mut self) -> Option<String> {
        let target = self.history.go_forward()?;
        self.current_url = target.clone();
        Some(target)
    }

    fn set_loaded_url(&mut self, url: &str) {
        self.current_url = url.to_string();
        self.history.set_current(url);
    }

    fn display_url(&self) -> String {
        if self.current_url == HISTORY_PAGE_URL {
            "grimley://historico".to_string()
        } else if self.current_url == NEW_TAB_PAGE_URL {
            "grimley://nova-aba".to_string()
        } else {
            self.current_url.clone()
        }
    }

    fn label(&self) -> String {
        if self.current_url == HISTORY_PAGE_URL {
            return "Historico".to_string();
        }
        if self.current_url == NEW_TAB_PAGE_URL {
            return "Nova aba".to_string();
        }

        let without_scheme = self
            .current_url
            .split("://")
            .nth(1)
            .unwrap_or(&self.current_url);
        let host = without_scheme.split('/').next().unwrap_or("Nova aba");
        let trimmed = if host.is_empty() { "Nova aba" } else { host };

        trimmed.chars().take(24).collect()
    }
}

pub(crate) struct TabRuntime {
    id: usize,
    state: TabState,
    pub(crate) webview: Option<WebView>,
    pub(crate) pending_internal_page_html: Option<String>,
    pub(crate) needs_internal_page_render: bool,
    allow_pdf_navigation: bool,
    pub(crate) pdf_source_url: Option<String>,
    pub(crate) pdf_data_base64: Option<String>,
    pub(crate) pdf_load_error: Option<String>,
}

impl TabRuntime {
    fn new(
        window: &Window,
        id: usize,
        loaded_urls: LoadedUrls,
        pending_action: PendingAction,
        visible: bool,
        launch: TabLaunch,
    ) -> Self {
        let webview = create_content_webview_with_url(
            window,
            &launch.initial_load_url,
            id,
            loaded_urls,
            pending_action,
            visible,
            launch.allow_pdf_navigation,
        );

        let needs_internal_page_render = launch.pending_internal_page_html.is_some()
            || launch.pdf_source_url.is_some()
            || launch.actual_url == HISTORY_PAGE_URL
            || launch.actual_url == NEW_TAB_PAGE_URL;

        Self {
            id,
            state: TabState::new(&launch.actual_url),
            webview: Some(webview),
            pending_internal_page_html: launch.pending_internal_page_html,
            needs_internal_page_render,
            allow_pdf_navigation: launch.allow_pdf_navigation,
            pdf_source_url: launch.pdf_source_url,
            pdf_data_base64: launch.pdf_data_base64,
            pdf_load_error: launch.pdf_load_error,
        }
    }

    pub(crate) fn current_url(&self) -> &str {
        &self.state.current_url
    }

    pub(crate) fn display_url(&self) -> String {
        if let Some(url) = &self.pdf_source_url {
            return url.clone();
        }

        self.state.display_url()
    }

    pub(crate) fn label(&self) -> String {
        if let Some(url) = &self.pdf_source_url {
            let filename = url.rsplit('/').next().unwrap_or("PDF");
            return filename.chars().take(24).collect();
        }

        self.state.label()
    }

    pub(crate) fn allows_pdf_navigation(&self) -> bool {
        self.allow_pdf_navigation
    }

    pub(crate) fn navigate_to(&mut self, url: &str) -> String {
        self.state.navigate_to(url)
    }

    pub(crate) fn go_back(&mut self) -> Option<String> {
        self.state.go_back()
    }

    pub(crate) fn go_forward(&mut self) -> Option<String> {
        self.state.go_forward()
    }

    pub(crate) fn set_loaded_url(&mut self, url: &str) {
        if self.pdf_source_url.is_some() && url == PDF_PAGE_URL {
            return;
        }

        self.state.set_loaded_url(url);
    }

    pub(crate) fn show_pdf_workspace(
        &mut self,
        pdf_url: String,
        pdf_data_base64: Option<String>,
        pdf_load_error: Option<String>,
    ) {
        self.state.navigate_to(&pdf_url);
        self.pdf_source_url = Some(pdf_url);
        self.pdf_data_base64 = pdf_data_base64;
        self.pdf_load_error = pdf_load_error;
        self.needs_internal_page_render = true;
    }

    pub(crate) fn clear_pdf_state(&mut self) {
        self.pdf_source_url = None;
        self.pdf_data_base64 = None;
        self.pdf_load_error = None;
        self.needs_internal_page_render = false;
    }

    pub(crate) fn set_pending_internal_page_html(&mut self, html: String) {
        self.pending_internal_page_html = Some(html);
        self.needs_internal_page_render = true;
    }

    fn is_internal_page(&self) -> bool {
        self.pdf_source_url.is_some()
            || self.state.current_url == HISTORY_PAGE_URL
            || self.state.current_url == NEW_TAB_PAGE_URL
    }

    fn initial_load_url(&self) -> &str {
        if self.pdf_source_url.is_some() {
            PDF_PAGE_URL
        } else if self.state.current_url == HISTORY_PAGE_URL {
            HISTORY_PAGE_URL
        } else if self.state.current_url == NEW_TAB_PAGE_URL {
            NEW_TAB_PAGE_URL
        } else {
            &self.state.current_url
        }
    }

    pub(crate) fn ensure_webview(
        &mut self,
        window: &Window,
        loaded_urls: LoadedUrls,
        pending_action: PendingAction,
        visible: bool,
    ) {
        if self.webview.is_none() {
            self.webview = Some(create_content_webview_with_url(
                window,
                self.initial_load_url(),
                self.id,
                loaded_urls,
                pending_action,
                visible,
                self.allow_pdf_navigation,
            ));
        } else if let Some(webview) = &self.webview {
            webview
                .set_visible(visible)
                .expect("Erro ao atualizar a visibilidade da aba");
        }
    }

    pub(crate) fn unload_webview(&mut self) {
        if let Some(webview) = self.webview.take() {
            webview
                .set_visible(false)
                .expect("Erro ao ocultar a aba antes de descarregar");
            drop(webview);
        }
    }

    pub(crate) fn suspend_internal_page(&mut self) {
        if !self.is_internal_page() || self.needs_internal_page_render {
            return;
        }

        if let Some(webview) = &self.webview {
            webview
                .evaluate_script("document.open(); document.write('<!DOCTYPE html><html><body style=\"margin:0;background:#ffffff;\"></body></html>'); document.close();")
                .expect("Erro ao suspender a pagina interna");
        }
        self.needs_internal_page_render = true;
    }
}

pub(crate) struct BrowserTabs {
    tabs: Vec<TabRuntime>,
    active_index: usize,
    next_tab_id: usize,
}

impl BrowserTabs {
    pub(crate) fn new(
        window: &Window,
        loaded_urls: LoadedUrls,
        pending_action: PendingAction,
    ) -> Self {
        Self {
            tabs: vec![TabRuntime::new(
                window,
                0,
                loaded_urls,
                pending_action,
                true,
                TabLaunch::regular(HOME_PAGE_URL),
            )],
            active_index: 0,
            next_tab_id: 1,
        }
    }

    pub(crate) fn active_tab(&self) -> &TabRuntime {
        &self.tabs[self.active_index]
    }

    pub(crate) fn active_tab_mut(&mut self) -> &mut TabRuntime {
        &mut self.tabs[self.active_index]
    }

    pub(crate) fn active_index(&self) -> usize {
        self.active_index
    }

    pub(crate) fn tabs(&self) -> &[TabRuntime] {
        &self.tabs
    }

    pub(crate) fn tab_mut(&mut self, index: usize) -> Option<&mut TabRuntime> {
        self.tabs.get_mut(index)
    }

    pub(crate) fn navigate_active(&mut self, url: &str) -> String {
        self.active_tab_mut().navigate_to(url)
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
        launch: TabLaunch,
    ) -> usize {
        if let Some(current) = self.tabs.get_mut(self.active_index) {
            current.suspend_internal_page();
            current.unload_webview();
        }

        self.tabs.push(TabRuntime::new(
            window,
            self.next_tab_id,
            loaded_urls,
            pending_action,
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
        index: usize,
    ) {
        if index >= self.tabs.len() {
            return;
        }

        let was_active = index == self.active_index;
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.tabs.push(TabRuntime::new(
                window,
                self.next_tab_id,
                loaded_urls,
                pending_action,
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

        if was_active {
            self.tabs[self.active_index].webview = None;
        }
    }

    pub(crate) fn set_loaded_url(&mut self, index: usize, url: &str) {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.set_loaded_url(url);
        }
    }

    pub(crate) fn find_index_by_id(&self, id: usize) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.id == id)
    }

    pub(crate) fn can_go_back(&self) -> bool {
        self.active_tab().state.history.can_go_back()
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        self.active_tab().state.history.can_go_forward()
    }

    pub(crate) fn tabs_json(&self) -> String {
        let entries = self
            .tabs
            .iter()
            .map(|tab| {
                format!(
                    r#"{{"label":"{}","url":"{}"}}"#,
                    escape_json(&tab.label()),
                    escape_json(&tab.display_url())
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("[{}]", entries)
    }
}
