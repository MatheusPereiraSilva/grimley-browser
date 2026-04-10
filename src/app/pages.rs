use tao::window::Window;

use crate::browser::escape_js_string;

use super::{
    constants::{HISTORY_PAGE_URL, NEW_TAB_PAGE_URL},
    escape::escape_html,
    history::VisitedPages,
    tabs::TabRuntime,
    LoadedUrls, PendingAction,
};

const NEW_TAB_HTML: &str = include_str!("templates/new_tab.html");
const PDF_WORKSPACE_TEMPLATE: &str = include_str!("templates/pdf_workspace.html");

pub(crate) fn render_new_tab_html() -> String {
    NEW_TAB_HTML.to_string()
}

pub(crate) fn render_internal_page(
    tab: &mut TabRuntime,
    visited_pages: &VisitedPages,
    window: &Window,
    loaded_urls: LoadedUrls,
    pending_action: PendingAction,
) {
    tab.ensure_webview(window, loaded_urls, pending_action, true);

    if !tab.needs_internal_page_render {
        return;
    }

    let html = if let Some(pdf_url) = &tab.pdf_source_url {
        render_pdf_workspace_html(
            pdf_url,
            tab.pdf_data_base64.as_deref(),
            tab.pdf_load_error.as_deref(),
        )
    } else if tab.current_url() == HISTORY_PAGE_URL {
        visited_pages.render_html()
    } else if tab.current_url() == NEW_TAB_PAGE_URL {
        tab.pending_internal_page_html
            .take()
            .unwrap_or_else(render_new_tab_html)
    } else {
        return;
    };

    let render_script = format!(
        "document.open(); document.write('{}'); document.close();",
        escape_js_string(&html)
    );
    tab.webview
        .as_ref()
        .expect("WebView ausente ao renderizar a pagina interna")
        .evaluate_script(&render_script)
        .expect("Erro ao renderizar a pagina interna");
    tab.needs_internal_page_render = false;
}

fn render_pdf_workspace_html(
    pdf_url: &str,
    pdf_data_base64: Option<&str>,
    pdf_load_error: Option<&str>,
) -> String {
    PDF_WORKSPACE_TEMPLATE
        .replace("__GRIMLEY_PDF_URL__", &escape_html(pdf_url))
        .replace("__GRIMLEY_PDF_DATA__", pdf_data_base64.unwrap_or(""))
        .replace(
            "__GRIMLEY_PDF_ERROR__",
            &pdf_load_error.map(escape_html).unwrap_or_default(),
        )
}
