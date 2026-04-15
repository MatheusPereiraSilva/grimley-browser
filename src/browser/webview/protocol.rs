use wry::http::{
    header::{ACCESS_CONTROL_ALLOW_ORIGIN, CACHE_CONTROL, CONTENT_TYPE},
    Request, Response, StatusCode,
};

use crate::{app::PdfRoutes, pdf::fetch_pdf_bytes};

pub(super) const PDF_PROTOCOL_NAME: &str = "grimley-pdf";

pub(super) fn build_pdf_protocol_response(
    request: Request<Vec<u8>>,
    pdf_routes: &PdfRoutes,
) -> Response<Vec<u8>> {
    let Some(tab_id) = parse_pdf_tab_id(&request) else {
        return build_text_response(
            StatusCode::BAD_REQUEST,
            "Requisicao de PDF invalida para a aba atual.",
        );
    };

    let Some(pdf_url) = pdf_routes.lock().unwrap().get(&tab_id).cloned() else {
        return build_text_response(
            StatusCode::NOT_FOUND,
            "O PDF desta aba nao esta mais disponivel.",
        );
    };

    match fetch_pdf_bytes(&pdf_url) {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/pdf")
            .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(CACHE_CONTROL, "no-store")
            .body(bytes)
            .expect("Erro ao montar a resposta do PDF"),
        Err(error) => build_text_response(StatusCode::BAD_GATEWAY, &error),
    }
}

fn build_text_response(status: StatusCode, message: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(CACHE_CONTROL, "no-store")
        .body(message.as_bytes().to_vec())
        .expect("Erro ao montar a resposta de texto")
}

fn parse_pdf_tab_id(request: &Request<Vec<u8>>) -> Option<usize> {
    let path = request.uri().path().trim_start_matches('/');
    let id = path.strip_suffix(".pdf").unwrap_or(path);
    id.parse().ok()
}
