#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PdfEditMode {
    AnnotationOnly,
    DerivedText,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfTextBlock {
    page: u32,
    text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfEditSession {
    mode: PdfEditMode,
    extracted_text: Option<String>,
    blocks: Vec<PdfTextBlock>,
}

impl PdfTextBlock {
    pub(crate) fn new(page: u32, text: impl Into<String>) -> Self {
        Self {
            page,
            text: text.into(),
        }
    }

    pub(crate) fn page(&self) -> u32 {
        self.page
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }
}

impl PdfEditSession {
    pub(crate) fn annotation_only() -> Self {
        Self {
            mode: PdfEditMode::AnnotationOnly,
            extracted_text: None,
            blocks: Vec::new(),
        }
    }

    pub(crate) fn derived_text(
        extracted_text: impl Into<String>,
        blocks: Vec<PdfTextBlock>,
    ) -> Self {
        Self {
            mode: PdfEditMode::DerivedText,
            extracted_text: Some(extracted_text.into()),
            blocks,
        }
    }

    pub(crate) fn mode(&self) -> PdfEditMode {
        self.mode
    }

    pub(crate) fn extracted_text(&self) -> Option<&str> {
        self.extracted_text.as_deref()
    }

    pub(crate) fn blocks(&self) -> &[PdfTextBlock] {
        &self.blocks
    }
}
