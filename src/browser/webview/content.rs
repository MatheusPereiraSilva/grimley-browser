use std::{
    sync::{Arc, Mutex},
    thread,
};

use tao::window::Window;
use wry::{http::Request, PageLoadEvent, WebView, WebViewBuilder};

use crate::{
    app::{LoadedUrls, PdfRoutes, PendingAction},
    browser::{content_bounds, is_pdf_url, BrowserAction},
    internal_pages::internal_page_kind_for_url,
    pdf::PdfFetcherHandle,
    shield::ShieldEngineHandle,
    storage::{
        annotations::{save_annotations, PdfAnnotationsSaveRequest},
        AppStorage,
    },
};

use super::protocol::{build_pdf_protocol_response, PDF_PROTOCOL_NAME};

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ContentIpcMessage {
    SavePdfAnnotations {
        #[serde(rename = "pdfUrl")]
        pdf_url: String,
        annotations: Vec<crate::storage::annotations::StoredPdfAnnotation>,
    },
}

pub(crate) fn create_content_webview_with_url(
    window: &Window,
    url: &str,
    tab_id: usize,
    loaded_urls: LoadedUrls,
    pending_action: PendingAction,
    pdf_routes: PdfRoutes,
    pdf_fetcher: PdfFetcherHandle,
    shield_engine: ShieldEngineHandle,
    storage: AppStorage,
    visible: bool,
    allow_pdf_navigation: bool,
) -> WebView {
    let current_page_url = Arc::new(Mutex::new(url.to_string()));

    WebViewBuilder::new_as_child(window)
        .with_bounds(content_bounds(window))
        .with_visible(visible)
        .with_url(url)
        .with_asynchronous_custom_protocol(PDF_PROTOCOL_NAME.into(), move |request, responder| {
            let pdf_routes = Arc::clone(&pdf_routes);
            let pdf_fetcher = Arc::clone(&pdf_fetcher);

            thread::spawn(move || {
                responder.respond(build_pdf_protocol_response(
                    request,
                    &pdf_routes,
                    &pdf_fetcher,
                ));
            });
        })
        .with_ipc_handler({
            let storage = storage.clone();
            move |request: Request<String>| {
                let message = request.body().trim();
                let parsed = serde_json::from_str::<ContentIpcMessage>(message);

                match parsed {
                    Ok(ContentIpcMessage::SavePdfAnnotations {
                        pdf_url,
                        annotations,
                    }) => {
                        if let Err(error) = save_annotations(
                            &storage,
                            &PdfAnnotationsSaveRequest {
                                pdf_url,
                                annotations,
                            },
                        ) {
                            tracing::warn!("Falha ao salvar anotacoes do PDF: {error}");
                        }
                    }
                    Err(error) => {
                        tracing::warn!("Falha ao interpretar mensagem IPC do conteudo: {error}");
                    }
                }
            }
        })
        .with_navigation_handler({
            let pending_action = Arc::clone(&pending_action);
            let shield_engine = Arc::clone(&shield_engine);
            let current_page_url = Arc::clone(&current_page_url);
            move |next_url| {
                let source_url = current_page_url.lock().unwrap().clone();
                let decision =
                    shield_engine.observe_navigation(tab_id, &next_url, Some(source_url.as_str()));

                if !decision.should_allow() {
                    false
                } else if internal_page_kind_for_url(&next_url).is_some() {
                    *pending_action.lock().unwrap() = Some(BrowserAction::Navigate(next_url));
                    false
                } else if !allow_pdf_navigation && is_pdf_url(&next_url) {
                    *pending_action.lock().unwrap() = Some(BrowserAction::OpenPdf(next_url));
                    false
                } else {
                    true
                }
            }
        })
        .with_on_page_load_handler({
            let current_page_url = Arc::clone(&current_page_url);
            move |event, loaded_url| {
                if matches!(event, PageLoadEvent::Finished) {
                    *current_page_url.lock().unwrap() = loaded_url.clone();
                    loaded_urls.lock().unwrap().push((tab_id, loaded_url));
                }
            }
        })
        .with_new_window_req_handler({
            let pending_action = Arc::clone(&pending_action);
            let shield_engine = Arc::clone(&shield_engine);
            let current_page_url = Arc::clone(&current_page_url);
            move |new_url| {
                let source_url = current_page_url.lock().unwrap().clone();
                let decision =
                    shield_engine.observe_new_window(tab_id, &new_url, Some(source_url.as_str()));

                if !decision.should_allow() {
                    false
                } else {
                    *pending_action.lock().unwrap() = Some(BrowserAction::NewTab(Some(new_url)));
                    false
                }
            }
        })
        .build()
        .unwrap()
}
