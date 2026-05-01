pub(crate) mod annotations;
pub(crate) mod editor;
pub(crate) mod export;
mod fetch;
pub(crate) mod model;
mod workspace;

pub(crate) use annotations::PdfAnnotation;
pub(crate) use editor::{PdfDerivedTextSession, PdfTextBlock};
pub(crate) use export::{export_pdf_document, PdfExportFormat};
pub(crate) use fetch::{create_pdf_fetcher, fetch_pdf_bytes, PdfFetcherHandle};
pub(crate) use model::{
    PdfDocument, PdfDocumentRef, PdfSource, PdfWorkspaceMode, PdfWorkspaceState, PDF_PAGE_URL,
};
pub(crate) use workspace::{open_pdf_workspace, render_pdf_workspace_html};
