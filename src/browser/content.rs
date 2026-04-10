use std::sync::{Arc, Mutex};

use tao::window::Window;
use wry::{WebView, WebViewBuilder};

use super::{action::BrowserAction, layout::content_bounds, url::is_pdf_url};

pub fn create_content_webview_with_url(
    window: &Window,
    url: &str,
    tab_id: usize,
    loaded_urls: Arc<Mutex<Vec<(usize, String)>>>,
    pending_action: Arc<Mutex<Option<BrowserAction>>>,
    visible: bool,
    allow_pdf_navigation: bool,
) -> WebView {
    WebViewBuilder::new_as_child(window)
        .with_bounds(content_bounds(window))
        .with_visible(visible)
        .with_url(url)
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
        .with_on_page_load_handler(move |_, loaded_url| {
            loaded_urls.lock().unwrap().push((tab_id, loaded_url));
        })
        .with_new_window_req_handler(move |new_url| {
            *pending_action.lock().unwrap() = Some(BrowserAction::NewTab(Some(new_url)));
            false
        })
        .build()
        .unwrap()
}
