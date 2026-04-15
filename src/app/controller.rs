use std::sync::Arc;

use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    browser::create_ui_webview,
    internal_pages::VisitedPages,
    tabs::BrowserTabs,
    ui::create_window,
};

use super::{
    events::{handle_browser_action, handle_loaded_urls, handle_window_event},
    state::{create_loaded_urls, create_pdf_routes, create_pending_action},
    ui_sync::sync_ui,
};

pub(crate) fn run() {
    let event_loop = EventLoop::new();
    let window = create_window(&event_loop);

    let pending_action = create_pending_action();
    let loaded_urls = create_loaded_urls();
    let pdf_routes = create_pdf_routes();
    let mut browser_tabs = BrowserTabs::new(
        &window,
        Arc::clone(&loaded_urls),
        Arc::clone(&pending_action),
        Arc::clone(&pdf_routes),
    );
    let ui_webview = create_ui_webview(&window, Arc::clone(&pending_action));
    let mut visited_pages = VisitedPages::default();
    let mut last_ui_snapshot = None;

    sync_ui(&ui_webview, &browser_tabs, &mut last_ui_snapshot);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Some(action) = pending_action.lock().unwrap().take() {
            handle_browser_action(
                action,
                &mut browser_tabs,
                &window,
                &ui_webview,
                &loaded_urls,
                &pending_action,
                &pdf_routes,
                &mut visited_pages,
                &mut last_ui_snapshot,
            );
        }

        handle_loaded_urls(
            &mut browser_tabs,
            &window,
            &ui_webview,
            &loaded_urls,
            &pending_action,
            &mut visited_pages,
            &mut last_ui_snapshot,
        );

        if let Event::WindowEvent { event, .. } = event {
            handle_window_event(&event, control_flow, &window, &ui_webview, &browser_tabs);
        }
    });
}
