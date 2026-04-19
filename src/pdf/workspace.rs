use super::model::{PdfId, PdfWorkspaceMode, PdfWorkspaceState};

const PDF_WORKSPACE_TEMPLATE: &str = include_str!("templates/pdf_workspace.html");

pub(crate) fn open_pdf_workspace(
    document_id: PdfId,
    origin_url: impl Into<String>,
    workspace_mode: PdfWorkspaceMode,
) -> PdfWorkspaceState {
    PdfWorkspaceState::new(document_id, origin_url, workspace_mode)
}

pub(crate) fn render_pdf_workspace_html(
    tab_id: PdfId,
    pdf_document: &PdfWorkspaceState,
    initial_annotations_json: &str,
) -> String {
    let pdf_asset_url = pdf_asset_url(tab_id);
    let _workspace_mode = pdf_document.workspace_mode();
    let _document_id = pdf_document.document_id();

    PDF_WORKSPACE_TEMPLATE
        .replace("__GRIMLEY_PDF_URL__", &escape_html(pdf_document.origin_url()))
        .replace("__GRIMLEY_PDF_ASSET_URL__", &escape_html(&pdf_asset_url))
        .replace(
            "__GRIMLEY_PDF_INITIAL_ANNOTATIONS__",
            &escape_js_string(initial_annotations_json),
        )
}

fn pdf_asset_url(tab_id: PdfId) -> String {
    if cfg!(any(target_os = "windows", target_os = "android")) {
        format!("http://grimley-pdf.localhost/{tab_id}.pdf")
    } else {
        format!("grimley-pdf://localhost/{tab_id}.pdf")
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_js_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
