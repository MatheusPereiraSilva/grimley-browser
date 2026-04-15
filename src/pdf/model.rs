pub(crate) const PDF_PAGE_URL: &str = "about:blank#grimley-pdf";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PdfWorkspaceMode {
    Workspace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfDocumentRef {
    document_id: usize,
    origin_url: String,
    workspace_mode: PdfWorkspaceMode,
}

impl PdfDocumentRef {
    pub(crate) fn new(
        document_id: usize,
        origin_url: impl Into<String>,
        workspace_mode: PdfWorkspaceMode,
    ) -> Self {
        Self {
            document_id,
            origin_url: origin_url.into(),
            workspace_mode,
        }
    }

    pub(crate) fn document_id(&self) -> usize {
        self.document_id
    }

    pub(crate) fn origin_url(&self) -> &str {
        &self.origin_url
    }

    pub(crate) fn workspace_mode(&self) -> PdfWorkspaceMode {
        self.workspace_mode
    }

    pub(crate) fn display_url(&self) -> &str {
        self.origin_url()
    }

    pub(crate) fn title(&self) -> String {
        let filename = self.origin_url.rsplit('/').next().unwrap_or("PDF");
        filename.chars().take(24).collect()
    }
}
