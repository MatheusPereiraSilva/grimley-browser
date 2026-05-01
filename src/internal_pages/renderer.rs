use tao::window::Window;

use crate::{
    app::{LoadedUrls, PendingCommand},
    browser::escape_js_string,
    pdf::render_pdf_workspace_html,
    shield::ShieldEngineHandle,
    storage::{annotations::load_annotations_json, AppStorage},
    tabs::{TabRenderRequest, TabSession},
};

use super::{
    render_new_tab_html, HistoryPage, InternalPageKind, InternalPageRenderer, ShieldPage,
    VisitedPages,
};

pub(crate) fn render_internal_page_html(
    kind: InternalPageKind,
    visited_pages: &VisitedPages,
    shield_engine: ShieldEngineHandle,
) -> String {
    match kind {
        InternalPageKind::NewTab => render_new_tab_html(),
        InternalPageKind::History => {
            let renderer = HistoryPage::new(visited_pages.entries());
            renderer.render()
        }
        InternalPageKind::Shield => {
            let renderer = ShieldPage::new(shield_engine);
            renderer.render()
        }
    }
}

pub(crate) fn render_internal_page(
    tab: &mut TabSession,
    visited_pages: &VisitedPages,
    shield_engine: ShieldEngineHandle,
    storage: &AppStorage,
    window: &Window,
    loaded_urls: LoadedUrls,
    pending_command: PendingCommand,
) {
    tab.ensure_webview(window, loaded_urls, pending_command, true);

    let Some(render_request) = tab.take_render_request() else {
        return;
    };

    let html = match render_request {
        TabRenderRequest::Internal { kind } => {
            render_internal_page_html(kind, visited_pages, shield_engine)
        }
        TabRenderRequest::Pdf(document) => {
            let initial_annotations_json = load_annotations_json(storage, document.origin_url())
                .unwrap_or_else(|error| {
                    tracing::warn!(
                        "Falha ao carregar anotacoes persistidas do PDF {}: {error}",
                        document.origin_url()
                    );
                    "[]".to_string()
                });
            render_pdf_workspace_html(tab.id(), &document, &initial_annotations_json)
        }
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
    tab.mark_rendered();
}

#[cfg(test)]
mod tests {
    use crate::{
        internal_pages::{HistoryVisit, InternalPageKind},
        shield::create_shield_engine,
    };

    use super::*;

    #[test]
    fn renders_new_tab_internal_page() {
        let html = render_internal_page_html(
            InternalPageKind::NewTab,
            &VisitedPages::default(),
            create_shield_engine(),
        );

        assert!(html.contains("<title>Nova aba</title>"));
        assert!(html.contains("Digite uma URL na barra acima"));
    }

    #[test]
    fn renders_history_internal_page_with_escaped_urls() {
        let visited_pages = VisitedPages::from_entries(vec![HistoryVisit {
            url: "https://example.com/<grimley>?q=1&x=2".to_string(),
        }]);
        let html = render_internal_page_html(
            InternalPageKind::History,
            &visited_pages,
            create_shield_engine(),
        );

        assert!(html.contains("<title>Historico</title>"));
        assert!(html.contains("https://example.com/&lt;grimley&gt;?q=1&amp;x=2"));
    }
}
