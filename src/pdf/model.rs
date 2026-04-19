#![allow(dead_code)]

use super::{annotations::PdfAnnotation, editor::PdfEditSession};

pub(crate) const PDF_PAGE_URL: &str = "about:blank#grimley-pdf";

pub(crate) type PdfId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PdfSource {
    RemoteUrl(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PdfWorkspaceMode {
    Workspace,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Rect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfPage {
    number: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfDocument {
    pub(crate) id: PdfId,
    pub(crate) source_url: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) pages: Vec<PdfPage>,
    pub(crate) annotations: Vec<PdfAnnotation>,
    pub(crate) edit_session: Option<PdfEditSession>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfWorkspaceState {
    document_id: PdfId,
    source: PdfSource,
    workspace_mode: PdfWorkspaceMode,
}

pub(crate) type PdfDocumentRef = PdfWorkspaceState;

impl PdfSource {
    pub(crate) fn remote_url(url: impl Into<String>) -> Self {
        Self::RemoteUrl(url.into())
    }

    pub(crate) fn as_url(&self) -> &str {
        match self {
            Self::RemoteUrl(url) => url,
        }
    }
}

impl PdfPage {
    pub(crate) fn new(number: u32) -> Self {
        Self { number }
    }

    pub(crate) fn number(&self) -> u32 {
        self.number
    }
}

impl PdfDocument {
    pub(crate) fn new(id: PdfId, source: &PdfSource, bytes: Vec<u8>) -> Self {
        Self {
            id,
            source_url: source.as_url().to_string(),
            bytes,
            pages: Vec::new(),
            annotations: Vec::new(),
            edit_session: None,
        }
    }

    pub(crate) fn with_page_count(mut self, page_count: u32) -> Self {
        self.pages = (1..=page_count).map(PdfPage::new).collect();
        self
    }
}

impl PdfWorkspaceState {
    pub(crate) fn new(
        document_id: PdfId,
        origin_url: impl Into<String>,
        workspace_mode: PdfWorkspaceMode,
    ) -> Self {
        Self {
            document_id,
            source: PdfSource::remote_url(origin_url),
            workspace_mode,
        }
    }

    pub(crate) fn document_id(&self) -> PdfId {
        self.document_id
    }

    pub(crate) fn origin_url(&self) -> &str {
        self.source.as_url()
    }

    pub(crate) fn source(&self) -> &PdfSource {
        &self.source
    }

    pub(crate) fn workspace_mode(&self) -> PdfWorkspaceMode {
        self.workspace_mode
    }

    pub(crate) fn display_url(&self) -> &str {
        self.origin_url()
    }

    pub(crate) fn title(&self) -> String {
        let filename = self.origin_url().rsplit('/').next().unwrap_or("PDF");
        filename.chars().take(24).collect()
    }
}
