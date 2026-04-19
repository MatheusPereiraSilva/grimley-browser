use std::sync::Arc;

use tao::{event::WindowEvent, event_loop::ControlFlow, window::Window};
use wry::WebView;

use crate::{
    browser::{content_bounds, is_pdf_url, normalize_url, toolbar_bounds, BrowserAction},
    internal_pages::{
        internal_page_kind_for_url, render_internal_page, VisitedPages, HISTORY_PAGE_URL,
        NEW_TAB_PAGE_URL, SHIELD_PAGE_URL,
    },
    pdf::PdfFetcherHandle,
    shield::ShieldEngineHandle,
    storage::AppStorage,
    tabs::{BrowserTabs, TabLaunch},
};

use super::{
    state::{LoadedUrls, PdfRoutes, PendingAction, UiSnapshot},
    ui_sync::sync_ui,
};

fn build_new_tab_launch(url: Option<String>) -> TabLaunch {
    match url {
        Some(value) => {
            let normalized = normalize_url(&value);
            if let Some(kind) = internal_page_kind_for_url(&normalized) {
                TabLaunch::internal(kind)
            } else if is_pdf_url(&normalized) {
                build_pdf_tab_launch(normalized)
            } else {
                TabLaunch::regular(normalized)
            }
        }
        None => TabLaunch::new_tab(),
    }
}

fn build_pdf_tab_launch(pdf_url: String) -> TabLaunch {
    TabLaunch::pdf(pdf_url)
}

pub(crate) fn handle_browser_action(
    action: BrowserAction,
    browser_tabs: &mut BrowserTabs,
    window: &Window,
    ui_webview: &WebView,
    loaded_urls: &LoadedUrls,
    pending_action: &PendingAction,
    pdf_routes: &PdfRoutes,
    pdf_fetcher: &PdfFetcherHandle,
    shield_engine: &ShieldEngineHandle,
    storage: &AppStorage,
    visited_pages: &mut VisitedPages,
    last_ui_snapshot: &mut Option<UiSnapshot>,
) {
    match action {
        BrowserAction::Navigate(url) => {
            let final_url = normalize_url(&url);
            if let Some(kind) = internal_page_kind_for_url(&final_url) {
                let tab = browser_tabs.active_tab_mut();
                tab.show_internal_page(kind);
                tab.ensure_webview(
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_action),
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
                    Arc::clone(pending_action),
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
                    Arc::clone(pending_action),
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
        BrowserAction::Back => {
            if let Some(target_url) = browser_tabs.back_active() {
                browser_tabs.active_tab_mut().ensure_webview(
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_action),
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
        BrowserAction::Forward => {
            if let Some(target_url) = browser_tabs.forward_active() {
                browser_tabs.active_tab_mut().ensure_webview(
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_action),
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
        BrowserAction::ShowHistory => {
            let active_tab = browser_tabs.active_tab_mut();
            active_tab.show_history_page();
            active_tab.ensure_webview(
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_action),
                true,
            );
            active_tab
                .webview
                .as_ref()
                .expect("WebView ausente ao abrir historico")
                .load_url(active_tab.load_target_url())
                .expect("Erro ao abrir a pagina de historico");
        }
        BrowserAction::NewTab(url) => {
            let new_index = browser_tabs.open_new_tab(
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_action),
                Arc::clone(pdf_routes),
                Arc::clone(pdf_fetcher),
                Arc::clone(shield_engine),
                storage.clone(),
                build_new_tab_launch(url),
            );
            if let Some(tab) = browser_tabs.tab_mut(new_index) {
                render_internal_page(
                    tab,
                    visited_pages,
                    Arc::clone(shield_engine),
                    storage,
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_action),
                );
            }
        }
        BrowserAction::OpenPdf(url) => {
            let pdf_url = normalize_url(&url);
            let new_index = browser_tabs.open_new_tab(
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_action),
                Arc::clone(pdf_routes),
                Arc::clone(pdf_fetcher),
                Arc::clone(shield_engine),
                storage.clone(),
                build_pdf_tab_launch(pdf_url),
            );
            if let Some(tab) = browser_tabs.tab_mut(new_index) {
                render_internal_page(
                    tab,
                    visited_pages,
                    Arc::clone(shield_engine),
                    storage,
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_action),
                );
            }
        }
        BrowserAction::SwitchTab(index) => {
            if browser_tabs.switch_tab(index) {
                render_internal_page(
                    browser_tabs.active_tab_mut(),
                    visited_pages,
                    Arc::clone(shield_engine),
                    storage,
                    window,
                    Arc::clone(loaded_urls),
                    Arc::clone(pending_action),
                );
            }
        }
        BrowserAction::CloseTab(index) => {
            browser_tabs.close_tab(
                window,
                Arc::clone(loaded_urls),
                Arc::clone(pending_action),
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
                Arc::clone(pending_action),
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
    pending_action: &PendingAction,
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
                    Arc::clone(pending_action),
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
