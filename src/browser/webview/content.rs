use std::{sync::Arc, thread};

use tao::window::Window;
use wry::{PageLoadEvent, WebView, WebViewBuilder};

use crate::{
    app::{LoadedUrls, PdfRoutes, PendingAction},
    browser::{content_bounds, is_pdf_url, BrowserAction},
};

use super::protocol::{build_pdf_protocol_response, PDF_PROTOCOL_NAME};

pub(crate) fn create_content_webview_with_url(
    window: &Window,
    url: &str,
    tab_id: usize,
    loaded_urls: LoadedUrls,
    pending_action: PendingAction,
    pdf_routes: PdfRoutes,
    visible: bool,
    allow_pdf_navigation: bool,
) -> WebView {
    WebViewBuilder::new_as_child(window)
        .with_bounds(content_bounds(window))
        .with_visible(visible)
        .with_url(url)
        .with_asynchronous_custom_protocol(PDF_PROTOCOL_NAME.into(), move |request, responder| {
            let pdf_routes = Arc::clone(&pdf_routes);

            thread::spawn(move || {
                responder.respond(build_pdf_protocol_response(request, &pdf_routes));
            });
        })
        .with_navigation_handler({
            let pending_action = Arc::clone(&pending_action);
            move |next_url| {
                if !allow_pdf_navigation && is_pdf_url(&next_url) {
                    *pending_action.lock().unwrap() = Some(BrowserAction::OpenPdf(next_url));
                    false
                } else {
                    true
                }
            }
        })
        .with_on_page_load_handler(move |event, loaded_url| {
            if matches!(event, PageLoadEvent::Finished) {
                loaded_urls.lock().unwrap().push((tab_id, loaded_url));
            }
        })
        .with_new_window_req_handler(move |new_url| {
            *pending_action.lock().unwrap() = Some(BrowserAction::NewTab(Some(new_url)));
            false
        })
        .build()
        .unwrap()
}
