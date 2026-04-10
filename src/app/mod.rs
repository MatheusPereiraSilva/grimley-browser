mod constants;
mod escape;
mod history;
mod pages;
mod pdf;
mod tabs;

use std::sync::{Arc, Mutex};

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::browser::{
    content_bounds, create_ui_webview, escape_js_string, is_pdf_url, normalize_url, toolbar_bounds,
    BrowserAction,
};
use crate::ui::create_window;

use self::{
    constants::{HISTORY_PAGE_URL, NEW_TAB_PAGE_URL, PDF_PAGE_URL},
    history::VisitedPages,
    pages::render_internal_page,
    pdf::load_pdf_state,
    tabs::{BrowserTabs, TabLaunch},
};

type PendingAction = Arc<Mutex<Option<BrowserAction>>>;
type LoadedUrls = Arc<Mutex<Vec<(usize, String)>>>;

fn sync_ui(ui_webview: &wry::WebView, tabs: &BrowserTabs) {
    let tabs_json = tabs.tabs_json();
    let active_tab = tabs.active_tab();
    let script = format!(
        "window.setUrl('{url}'); window.setNavState({back}, {forward}); window.setTabs(JSON.parse('{tabs}'), {active});",
        url = escape_js_string(&active_tab.display_url()),
        back = tabs.can_go_back(),
        forward = tabs.can_go_forward(),
        tabs = escape_js_string(&tabs_json),
        active = tabs.active_index(),
    );

    ui_webview
        .evaluate_script(&script)
        .expect("Erro ao sincronizar a interface");
}

fn build_new_tab_launch(url: Option<String>) -> TabLaunch {
    match url {
        Some(value) => {
            let normalized = normalize_url(&value);
            if is_pdf_url(&normalized) {
                build_pdf_tab_launch(normalized, true)
            } else {
                TabLaunch::regular(normalized)
            }
        }
        None => TabLaunch::new_tab(),
    }
}

fn build_pdf_tab_launch(pdf_url: String, allow_pdf_navigation: bool) -> TabLaunch {
    let (pdf_data_base64, pdf_load_error) = load_pdf_state(&pdf_url);
    TabLaunch::pdf(
        pdf_url,
        pdf_data_base64,
        pdf_load_error,
        allow_pdf_navigation,
    )
}

pub fn run() {
    let event_loop = EventLoop::new();
    let window = create_window(&event_loop);

    let pending_action = Arc::new(Mutex::new(None::<BrowserAction>));
    let loaded_urls = Arc::new(Mutex::new(Vec::<(usize, String)>::new()));
    let mut browser_tabs = BrowserTabs::new(
        &window,
        Arc::clone(&loaded_urls),
        Arc::clone(&pending_action),
    );
    let ui_webview = create_ui_webview(&window, Arc::clone(&pending_action));
    let mut visited_pages = VisitedPages::default();

    sync_ui(&ui_webview, &browser_tabs);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let _ui = &ui_webview;
        let _tabs = &browser_tabs;

        if let Some(action) = pending_action.lock().unwrap().take() {
            match action {
                BrowserAction::Navigate(url) => {
                    let final_url = normalize_url(&url);
                    if is_pdf_url(&final_url) && !browser_tabs.active_tab().allows_pdf_navigation()
                    {
                        let (pdf_data_base64, pdf_load_error) = load_pdf_state(&final_url);
                        let tab = browser_tabs.active_tab_mut();
                        tab.show_pdf_workspace(final_url, pdf_data_base64, pdf_load_error);
                        tab.ensure_webview(
                            &window,
                            Arc::clone(&loaded_urls),
                            Arc::clone(&pending_action),
                            true,
                        );
                        tab.webview
                            .as_ref()
                            .expect("WebView ausente ao abrir o PDF")
                            .load_url(PDF_PAGE_URL)
                            .expect("Erro ao abrir o workspace de PDF");
                    } else {
                        browser_tabs.active_tab_mut().clear_pdf_state();
                        let target_url = browser_tabs.navigate_active(&final_url);
                        browser_tabs.active_tab_mut().ensure_webview(
                            &window,
                            Arc::clone(&loaded_urls),
                            Arc::clone(&pending_action),
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
                            &window,
                            Arc::clone(&loaded_urls),
                            Arc::clone(&pending_action),
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
                            &window,
                            Arc::clone(&loaded_urls),
                            Arc::clone(&pending_action),
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
                    let target_url = browser_tabs.navigate_active(HISTORY_PAGE_URL);
                    let active_tab = browser_tabs.active_tab_mut();
                    active_tab.set_pending_internal_page_html(visited_pages.render_html());
                    active_tab.ensure_webview(
                        &window,
                        Arc::clone(&loaded_urls),
                        Arc::clone(&pending_action),
                        true,
                    );
                    active_tab
                        .webview
                        .as_ref()
                        .expect("WebView ausente ao abrir historico")
                        .load_url(&target_url)
                        .expect("Erro ao abrir a pagina de historico");
                }
                BrowserAction::NewTab(url) => {
                    let new_index = browser_tabs.open_new_tab(
                        &window,
                        Arc::clone(&loaded_urls),
                        Arc::clone(&pending_action),
                        build_new_tab_launch(url),
                    );
                    if let Some(tab) = browser_tabs.tab_mut(new_index) {
                        render_internal_page(
                            tab,
                            &visited_pages,
                            &window,
                            Arc::clone(&loaded_urls),
                            Arc::clone(&pending_action),
                        );
                    }
                }
                BrowserAction::OpenPdf(url) => {
                    let pdf_url = normalize_url(&url);
                    let new_index = browser_tabs.open_new_tab(
                        &window,
                        Arc::clone(&loaded_urls),
                        Arc::clone(&pending_action),
                        build_pdf_tab_launch(pdf_url, true),
                    );
                    if let Some(tab) = browser_tabs.tab_mut(new_index) {
                        render_internal_page(
                            tab,
                            &visited_pages,
                            &window,
                            Arc::clone(&loaded_urls),
                            Arc::clone(&pending_action),
                        );
                    }
                }
                BrowserAction::SwitchTab(index) => {
                    if browser_tabs.switch_tab(index) {
                        render_internal_page(
                            browser_tabs.active_tab_mut(),
                            &visited_pages,
                            &window,
                            Arc::clone(&loaded_urls),
                            Arc::clone(&pending_action),
                        );
                    }
                }
                BrowserAction::CloseTab(index) => {
                    browser_tabs.close_tab(
                        &window,
                        Arc::clone(&loaded_urls),
                        Arc::clone(&pending_action),
                        index,
                    );
                    render_internal_page(
                        browser_tabs.active_tab_mut(),
                        &visited_pages,
                        &window,
                        Arc::clone(&loaded_urls),
                        Arc::clone(&pending_action),
                    );
                }
            }

            sync_ui(&ui_webview, &browser_tabs);
        }

        let pending_loaded_urls = {
            let mut queue = loaded_urls.lock().unwrap();
            std::mem::take(&mut *queue)
        };

        for (tab_id, url) in pending_loaded_urls {
            if let Some(index) = browser_tabs.find_index_by_id(tab_id) {
                if url != HISTORY_PAGE_URL && url != NEW_TAB_PAGE_URL {
                    visited_pages.record(&url);
                }

                browser_tabs.set_loaded_url(index, &url);

                if index == browser_tabs.active_index() {
                    render_internal_page(
                        browser_tabs.active_tab_mut(),
                        &visited_pages,
                        &window,
                        Arc::clone(&loaded_urls),
                        Arc::clone(&pending_action),
                    );
                    sync_ui(&ui_webview, &browser_tabs);
                }
            }
        }

        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(_) => {
                    ui_webview
                        .set_bounds(toolbar_bounds(&window))
                        .expect("Erro ao redimensionar a barra de URL");

                    for tab in browser_tabs.tabs() {
                        if let Some(webview) = &tab.webview {
                            webview
                                .set_bounds(content_bounds(&window))
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
    });
}
