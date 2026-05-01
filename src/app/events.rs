use std::sync::Arc;

use tao::{event::WindowEvent, event_loop::ControlFlow, window::Window};
use wry::WebView;

use crate::{
    app::UiCommand,
    browser::{content_bounds, is_pdf_url, normalize_url, toolbar_bounds},
    internal_pages::{
        internal_page_kind_for_url, render_internal_page, VisitedPages, HISTORY_PAGE_URL,
        NEW_TAB_PAGE_URL, SHIELD_PAGE_URL,
    },
    pdf::PdfFetcherHandle,
    shield::ShieldEngineHandle,
    storage::AppStorage,
    tabs::{launch_for_pdf_url, launch_for_requested_url, BrowserTabs},
};

use super::{
    state::{LoadedUrls, PdfRoutes, PendingCommand, UiSnapshot},
    ui_sync::sync_ui,
};

pub(crate) fn handle_ui_command(
    command: UiCommand,
    browser_tabs: &mut BrowserTabs,
    window: &Window,
    ui_webview: &WebView,
    loaded_urls: &LoadedUrls,
    pending_command: &PendingCommand,
    pdf_routes: &PdfRoutes,
    pdf_fetcher: &PdfFetcherHandle,
    shield_engine: &ShieldEngineHandle,
    storage: &AppStorage,
    visited_pages: &mut VisitedPages,
    last_ui_snapshot: &mut Option<UiSnapshot>,
) {
    match command {
        UiCommand::Navigate { url } => {
            let final_url = normalize_url(&url);
            if let Some(kind) = internal_page_kind_for_url(&final_url) {
                let tab = browser_tabs.active_tab_mut();
                tab.show_internal_page(kind);
                tab.ensure_webview(
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                    true,
                );
                tab.webview
                    .as_ref()
                    .expect("WebView ausente ao abrir a pagina interna")
                    .load_url(tab.load_target_url())
                    .expect("Erro ao abrir a pagina interna");
            } else if is_pdf_url(&final_url) && !browser_tabs.active_tab().allows_pdf_navigation() {
                let tab = browser_tabs.active_tab_mut();
                tab.show_pdf_workspace(final_url);
                tab.ensure_webview(
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                    true,
                );
                tab.webview
                    .as_ref()
                    .expect("WebView ausente ao abrir o PDF")
                    .load_url(tab.load_target_url())
                    .expect("Erro ao abrir o workspace de PDF");
            } else {
                let target_url = browser_tabs.navigate_active(&final_url);
                browser_tabs.active_tab_mut().ensure_webview(
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                    true,
                );
                browser_tabs
                    .active_tab()
                    .webview
                    .as_ref()
                    .expect("WebView ausente ao navegar para a URL")
                    .load_url(&target_url)
                    .expect("Erro ao navegar para a URL");
            }
        }
        UiCommand::Back => {
            if let Some(target_url) = browser_tabs.back_active() {
                browser_tabs.active_tab_mut().ensure_webview(
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                    true,
                );
                browser_tabs
                    .active_tab()
                    .webview
                    .as_ref()
                    .expect("WebView ausente ao voltar")
                    .load_url(&target_url)
                    .expect("Erro ao voltar a pagina");
            }
        }
        UiCommand::Forward => {
            if let Some(target_url) = browser_tabs.forward_active() {
                browser_tabs.active_tab_mut().ensure_webview(
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                    true,
                );
                browser_tabs
                    .active_tab()
                    .webview
                    .as_ref()
                    .expect("WebView ausente ao avancar")
                    .load_url(&target_url)
                    .expect("Erro ao avancar a pagina");
            }
        }
        UiCommand::OpenHistory => {
            let active_tab = browser_tabs.active_tab_mut();
            active_tab.show_history_page();
            active_tab.ensure_webview(
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_command),
                true,
            );
            active_tab
                .webview
                .as_ref()
                .expect("WebView ausente ao abrir historico")
                .load_url(active_tab.load_target_url())
                .expect("Erro ao abrir a pagina de historico");
        }
        UiCommand::NewTab { url } => {
            let new_index = browser_tabs.open_new_tab(
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_command),
                Arc::clone(pdf_routes),
                Arc::clone(pdf_fetcher),
                Arc::clone(shield_engine),
                storage.clone(),
                launch_for_requested_url(url.as_deref()),
            );
            if let Some(tab) = browser_tabs.tab_mut(new_index) {
                render_internal_page(
                    tab,
                    visited_pages,
                    Arc::clone(shield_engine),
                    storage,
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                );
            }
        }
        UiCommand::OpenPdf { url } => {
            let pdf_url = normalize_url(&url);
            let new_index = browser_tabs.open_new_tab(
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_command),
                Arc::clone(pdf_routes),
                Arc::clone(pdf_fetcher),
                Arc::clone(shield_engine),
                storage.clone(),
                launch_for_pdf_url(&pdf_url),
            );
            if let Some(tab) = browser_tabs.tab_mut(new_index) {
                render_internal_page(
                    tab,
                    visited_pages,
                    Arc::clone(shield_engine),
                    storage,
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                );
            }
        }
        UiCommand::SwitchTab { index } => {
            if browser_tabs.switch_tab(index) {
                render_internal_page(
                    browser_tabs.active_tab_mut(),
                    visited_pages,
                    Arc::clone(shield_engine),
                    storage,
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                );
            }
        }
        UiCommand::CloseTab { index } => {
            browser_tabs.close_tab(
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_command),
                Arc::clone(pdf_routes),
                Arc::clone(pdf_fetcher),
                Arc::clone(shield_engine),
                storage.clone(),
                index,
            );
            render_internal_page(
                browser_tabs.active_tab_mut(),
                visited_pages,
                Arc::clone(shield_engine),
                storage,
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_command),
            );
        }
    }

    sync_ui(ui_webview, browser_tabs, last_ui_snapshot);
}

pub(crate) fn handle_loaded_urls(
    browser_tabs: &mut BrowserTabs,
    window: &Window,
    ui_webview: &WebView,
    loaded_urls: &LoadedUrls,
    pending_command: &PendingCommand,
    visited_pages: &mut VisitedPages,
    last_ui_snapshot: &mut Option<UiSnapshot>,
    shield_engine: &ShieldEngineHandle,
    storage: &AppStorage,
) -> bool {
    let pending_loaded_urls = {
        let mut queue = loaded_urls.lock().unwrap();
        std::mem::take(&mut *queue)
    };

    let had_updates = !pending_loaded_urls.is_empty();

    for (tab_id, url) in pending_loaded_urls {
        if let Some(index) = browser_tabs.find_index_by_id(tab_id) {
            if url != HISTORY_PAGE_URL && url != NEW_TAB_PAGE_URL && url != SHIELD_PAGE_URL {
                visited_pages.record(&url);
            }

            browser_tabs.set_loaded_url(index, &url);

            if index == browser_tabs.active_index() {
                render_internal_page(
                    browser_tabs.active_tab_mut(),
                    visited_pages,
                    Arc::clone(shield_engine),
                    storage,
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_command),
                );
                sync_ui(ui_webview, browser_tabs, last_ui_snapshot);
            }
        }
    }

    had_updates
}

pub(crate) fn handle_window_event(
    event: &WindowEvent,
    control_flow: &mut ControlFlow,
    window: &Window,
    ui_webview: &WebView,
    browser_tabs: &BrowserTabs,
) {
    match event {
        WindowEvent::Resized(_) => {
            ui_webview
                .set_bounds(toolbar_bounds(window))
                .expect("Erro ao redimensionar a barra de URL");

            for tab in browser_tabs.tabs() {
                if let Some(webview) = &tab.webview {
                    webview
                        .set_bounds(content_bounds(window))
                        .expect("Erro ao redimensionar a area de conteudo");
                }
            }
        }
        WindowEvent::CloseRequested => {
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    }
}
