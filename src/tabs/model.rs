use std::sync::Arc;

use tao::window::Window;
use wry::WebView;

use crate::{
    app::{LoadedUrls, PdfRoutes, PendingAction},
    browser::create_content_webview_with_url,
    internal_pages::{InternalPageKind, HISTORY_PAGE_URL, NEW_TAB_PAGE_URL},
    pdf::{open_pdf_workspace, PdfDocumentRef, PdfFetcherHandle, PdfWorkspaceMode, PDF_PAGE_URL},
    shield::ShieldEngineHandle,
    storage::AppStorage,
};

use super::BrowserHistory;

pub(crate) enum TabLaunch {
    Web { url: String },
    Internal(InternalDocument),
    Pdf {
        origin_url: String,
        workspace_mode: PdfWorkspaceMode,
    },
}

impl TabLaunch {
    pub(crate) fn regular(url: impl Into<String>) -> Self {
        Self::Web { url: url.into() }
    }

    pub(crate) fn internal(kind: InternalPageKind) -> Self {
        Self::Internal(InternalDocument::from_kind(kind))
    }

    pub(crate) fn new_tab() -> Self {
        Self::Internal(InternalDocument::new_tab())
    }

    pub(crate) fn pdf(pdf_url: impl Into<String>) -> Self {
        Self::Pdf {
            origin_url: pdf_url.into(),
            workspace_mode: PdfWorkspaceMode::Workspace,
        }
    }

    pub(crate) fn pdf_with_mode(
        pdf_url: impl Into<String>,
        workspace_mode: PdfWorkspaceMode,
    ) -> Self {
        Self::Pdf {
            origin_url: pdf_url.into(),
            workspace_mode,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TabContentKind {
    Web,
    Internal,
    Pdf,
}

pub(crate) enum TabContent {
    Web(WebDocument),
    Internal(InternalDocument),
    Pdf(PdfDocumentRef),
}

pub(crate) struct Tab {
    id: usize,
    title: String,
    content: TabContent,
    lifecycle: TabLifecycle,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TabLifecycle {
    visible: bool,
    needs_render: bool,
    webview_attached: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct WebPermissions;

pub(crate) struct WebDocument {
    current_url: String,
    history: BrowserHistory,
    permissions: WebPermissions,
}

pub(crate) struct InternalDocument {
    kind: InternalPageKind,
}

pub(crate) enum TabRenderRequest {
    Internal { kind: InternalPageKind },
    Pdf(PdfDocumentRef),
}

pub(crate) struct TabSession {
    tab: Tab,
    pdf_routes: PdfRoutes,
    pdf_fetcher: PdfFetcherHandle,
    shield_engine: ShieldEngineHandle,
    storage: AppStorage,
    pub(crate) webview: Option<WebView>,
}

impl TabContent {
    fn kind(&self) -> TabContentKind {
        match self {
            Self::Web(_) => TabContentKind::Web,
            Self::Internal(_) => TabContentKind::Internal,
            Self::Pdf(_) => TabContentKind::Pdf,
        }
    }

    fn display_url(&self) -> String {
        match self {
            Self::Web(document) => document.display_url(),
            Self::Internal(document) => document.display_url().to_string(),
            Self::Pdf(document) => document.display_url().to_string(),
        }
    }

    fn load_target_url(&self) -> &str {
        match self {
            Self::Web(document) => document.current_url(),
            Self::Internal(document) => document.load_target_url(),
            Self::Pdf(_) => PDF_PAGE_URL,
        }
    }

    fn allows_pdf_navigation(&self) -> bool {
        matches!(self, Self::Pdf(_))
    }

    fn title(&self) -> String {
        match self {
            Self::Web(document) => document.title(),
            Self::Internal(document) => document.title().to_string(),
            Self::Pdf(document) => document.title(),
        }
    }

    fn is_suspendable_internal_page(&self) -> bool {
        matches!(self, Self::Internal(document) if document.is_suspendable())
    }
}

impl Tab {
    fn from_launch(id: usize, launch: TabLaunch, visible: bool) -> Self {
        let content = match launch {
            TabLaunch::Web { url } => TabContent::Web(WebDocument::new(url)),
            TabLaunch::Internal(document) => TabContent::Internal(document),
            TabLaunch::Pdf {
                origin_url,
                workspace_mode,
            } => TabContent::Pdf(open_pdf_workspace(id, origin_url, workspace_mode)),
        };

        let title = content.title();
        let needs_render = matches!(content.kind(), TabContentKind::Internal | TabContentKind::Pdf);

        Self {
            id,
            title,
            content,
            lifecycle: TabLifecycle {
                visible,
                needs_render,
                webview_attached: false,
            },
        }
    }

    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn content_kind(&self) -> TabContentKind {
        self.content.kind()
    }

    pub(crate) fn needs_render(&self) -> bool {
        self.lifecycle.needs_render
    }

    pub(crate) fn display_url(&self) -> String {
        self.content.display_url()
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn allows_pdf_navigation(&self) -> bool {
        self.content.allows_pdf_navigation()
    }

    pub(crate) fn can_go_back(&self) -> bool {
        matches!(&self.content, TabContent::Web(document) if document.can_go_back())
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        matches!(&self.content, TabContent::Web(document) if document.can_go_forward())
    }

    pub(crate) fn navigate_to_web(&mut self, url: &str) -> String {
        let target = match &mut self.content {
            TabContent::Web(document) => document.navigate_to(url),
            _ => {
                self.content = TabContent::Web(WebDocument::new(url.to_string()));
                url.to_string()
            }
        };

        self.lifecycle.needs_render = false;
        self.refresh_title();
        target
    }

    pub(crate) fn go_back(&mut self) -> Option<String> {
        let target = match &mut self.content {
            TabContent::Web(document) => document.go_back(),
            _ => None,
        };
        self.refresh_title();
        target
    }

    pub(crate) fn go_forward(&mut self) -> Option<String> {
        let target = match &mut self.content {
            TabContent::Web(document) => document.go_forward(),
            _ => None,
        };
        self.refresh_title();
        target
    }

    pub(crate) fn set_loaded_url(&mut self, url: &str) {
        if let TabContent::Web(document) = &mut self.content {
            document.set_loaded_url(url);
            self.refresh_title();
        }
    }

    pub(crate) fn show_history_page(&mut self) {
        self.content = TabContent::Internal(InternalDocument::history());
        self.lifecycle.needs_render = true;
        self.refresh_title();
    }

    pub(crate) fn show_internal_page(&mut self, kind: InternalPageKind) {
        self.content = TabContent::Internal(InternalDocument::from_kind(kind));
        self.lifecycle.needs_render = true;
        self.refresh_title();
    }

    pub(crate) fn show_pdf_workspace(&mut self, origin_url: String) {
        self.content = TabContent::Pdf(open_pdf_workspace(
            self.id,
            origin_url,
            PdfWorkspaceMode::Workspace,
        ));
        self.lifecycle.needs_render = true;
        self.refresh_title();
    }

    pub(crate) fn is_suspendable_internal_page(&self) -> bool {
        self.content.is_suspendable_internal_page()
    }

    pub(crate) fn mark_rendered(&mut self) {
        self.lifecycle.needs_render = false;
    }

    pub(crate) fn mark_needs_render(&mut self) {
        if matches!(self.content_kind(), TabContentKind::Internal | TabContentKind::Pdf) {
            self.lifecycle.needs_render = true;
        }
    }

    pub(crate) fn load_target_url(&self) -> &str {
        self.content.load_target_url()
    }

    pub(crate) fn take_render_request(&mut self) -> Option<TabRenderRequest> {
        if !self.needs_render() {
            return None;
        }

        match &mut self.content {
            TabContent::Web(_) => None,
            TabContent::Internal(document) => Some(TabRenderRequest::Internal {
                kind: document.kind(),
            }),
            TabContent::Pdf(document) => Some(TabRenderRequest::Pdf(document.clone())),
        }
    }

    fn refresh_title(&mut self) {
        self.title = self.content.title();
    }

    fn set_visible(&mut self, visible: bool) {
        self.lifecycle.visible = visible;
    }

    fn set_webview_attached(&mut self, webview_attached: bool) {
        self.lifecycle.webview_attached = webview_attached;
    }
}

impl WebDocument {
    fn new(url: impl Into<String>) -> Self {
        let url = url.into();

        Self {
            current_url: url.clone(),
            history: BrowserHistory::new(&url),
            permissions: WebPermissions,
        }
    }

    fn current_url(&self) -> &str {
        &self.current_url
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

    fn can_go_back(&self) -> bool {
        self.history.can_go_back()
    }

    fn can_go_forward(&self) -> bool {
        self.history.can_go_forward()
    }

    fn display_url(&self) -> String {
        self.current_url.clone()
    }

    fn title(&self) -> String {
        let _permissions = &self.permissions;
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

impl InternalDocument {
    fn from_kind(kind: InternalPageKind) -> Self {
        Self { kind }
    }

    fn new_tab() -> Self {
        Self::from_kind(InternalPageKind::NewTab)
    }

    fn history() -> Self {
        Self::from_kind(InternalPageKind::History)
    }

    pub(crate) fn kind(&self) -> InternalPageKind {
        self.kind
    }

    fn title(&self) -> &'static str {
        match self.kind {
            InternalPageKind::NewTab => "Nova aba",
            InternalPageKind::History => "Historico",
            InternalPageKind::Shield => "Grimley Shield",
        }
    }

    fn display_url(&self) -> &'static str {
        match self.kind {
            InternalPageKind::NewTab => "grimley://nova-aba",
            InternalPageKind::History => "grimley://historico",
            InternalPageKind::Shield => "grimley://shield",
        }
    }

    fn load_target_url(&self) -> &'static str {
        match self.kind {
            InternalPageKind::NewTab => NEW_TAB_PAGE_URL,
            InternalPageKind::History => HISTORY_PAGE_URL,
            InternalPageKind::Shield => crate::internal_pages::SHIELD_PAGE_URL,
        }
    }

    fn is_suspendable(&self) -> bool {
        matches!(
            self.kind,
            InternalPageKind::NewTab | InternalPageKind::History | InternalPageKind::Shield
        )
    }
}

impl TabSession {
    pub(crate) fn new(
        window: &Window,
        id: usize,
        loaded_urls: LoadedUrls,
        pending_action: PendingAction,
        pdf_routes: PdfRoutes,
        pdf_fetcher: PdfFetcherHandle,
        shield_engine: ShieldEngineHandle,
        storage: AppStorage,
        visible: bool,
        launch: TabLaunch,
    ) -> Self {
        let mut tab = Tab::from_launch(id, launch, visible);
        let webview = create_content_webview_with_url(
            window,
            tab.load_target_url(),
            id,
            loaded_urls,
            pending_action,
            Arc::clone(&pdf_routes),
            Arc::clone(&pdf_fetcher),
            Arc::clone(&shield_engine),
            storage.clone(),
            visible,
            tab.allows_pdf_navigation(),
        );

        tab.set_webview_attached(true);
        let session = Self {
            tab,
            pdf_routes,
            pdf_fetcher,
            shield_engine,
            storage,
            webview: Some(webview),
        };
        session.sync_pdf_route();
        session
    }

    pub(crate) fn id(&self) -> usize {
        self.tab.id()
    }

    pub(crate) fn content_kind(&self) -> TabContentKind {
        self.tab.content_kind()
    }

    pub(crate) fn needs_render(&self) -> bool {
        self.tab.needs_render()
    }

    pub(crate) fn display_url(&self) -> String {
        self.tab.display_url()
    }

    pub(crate) fn title(&self) -> &str {
        self.tab.title()
    }

    pub(crate) fn allows_pdf_navigation(&self) -> bool {
        self.tab.allows_pdf_navigation()
    }

    pub(crate) fn can_go_back(&self) -> bool {
        self.tab.can_go_back()
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        self.tab.can_go_forward()
    }

    pub(crate) fn navigate_to_web(&mut self, url: &str) -> String {
        let target = self.tab.navigate_to_web(url);
        self.sync_pdf_route();
        target
    }

    pub(crate) fn go_back(&mut self) -> Option<String> {
        self.tab.go_back()
    }

    pub(crate) fn go_forward(&mut self) -> Option<String> {
        self.tab.go_forward()
    }

    pub(crate) fn set_loaded_url(&mut self, url: &str) {
        self.tab.set_loaded_url(url);
    }

    pub(crate) fn show_history_page(&mut self) {
        self.tab.show_history_page();
        self.sync_pdf_route();
    }

    pub(crate) fn show_internal_page(&mut self, kind: InternalPageKind) {
        self.tab.show_internal_page(kind);
        self.sync_pdf_route();
    }

    pub(crate) fn show_pdf_workspace(&mut self, origin_url: String) {
        self.tab.show_pdf_workspace(origin_url);
        self.sync_pdf_route();
    }

    pub(crate) fn is_suspendable_internal_page(&self) -> bool {
        self.tab.is_suspendable_internal_page()
    }

    pub(crate) fn mark_needs_render(&mut self) {
        self.tab.mark_needs_render();
    }

    pub(crate) fn take_render_request(&mut self) -> Option<TabRenderRequest> {
        self.tab.take_render_request()
    }

    pub(crate) fn mark_rendered(&mut self) {
        self.tab.mark_rendered();
    }

    pub(crate) fn load_target_url(&self) -> &str {
        self.tab.load_target_url()
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
                self.load_target_url(),
                self.id(),
                loaded_urls,
                pending_action,
                Arc::clone(&self.pdf_routes),
                Arc::clone(&self.pdf_fetcher),
                Arc::clone(&self.shield_engine),
                self.storage.clone(),
                visible,
                self.allows_pdf_navigation(),
            ));
            self.tab.set_webview_attached(true);
        } else if let Some(webview) = &self.webview {
            webview
                .set_visible(visible)
                .expect("Erro ao atualizar a visibilidade da aba");
        }

        self.tab.set_visible(visible);
        self.sync_pdf_route();
    }

    pub(crate) fn unload_webview(&mut self) {
        if let Some(webview) = &self.webview {
            webview
                .set_visible(false)
                .expect("Erro ao ocultar a aba");
        }

        self.tab.set_visible(false);
    }

    pub(crate) fn suspend_internal_page(&mut self) {
        if self.content_kind() != TabContentKind::Internal
            || !self.is_suspendable_internal_page()
            || self.needs_render()
        {
            return;
        }

        if let Some(webview) = &self.webview {
            webview
                .evaluate_script("document.open(); document.write('<!DOCTYPE html><html><body style=\"margin:0;background:#ffffff;\"></body></html>'); document.close();")
                .expect("Erro ao suspender a pagina interna");
        }
        self.mark_needs_render();
    }

    fn sync_pdf_route(&self) {
        let mut routes = self.pdf_routes.lock().unwrap();

        if let Some(pdf_document) = self.pdf_route_document() {
            routes.insert(self.id(), pdf_document);
        } else {
            routes.remove(&self.id());
        }
    }

    fn pdf_route_document(&self) -> Option<PdfDocumentRef> {
        match &self.tab.content {
            TabContent::Pdf(document) => Some(document.clone()),
            _ => None,
        }
    }

    pub(crate) fn session_launch(&self) -> TabLaunch {
        match &self.tab.content {
            TabContent::Web(document) => TabLaunch::regular(document.current_url().to_string()),
            TabContent::Internal(document) => TabLaunch::internal(document.kind()),
            TabContent::Pdf(document) => {
                TabLaunch::pdf_with_mode(document.origin_url().to_string(), document.workspace_mode())
            }
        }
    }
}

impl Drop for TabSession {
    fn drop(&mut self) {
        self.pdf_routes.lock().unwrap().remove(&self.id());
    }
}
