#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfTextBlock {
    page: u32,
    text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PdfDerivedTextSession {
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

impl PdfDerivedTextSession {
    pub(crate) fn new(extracted_text: impl Into<String>, blocks: Vec<PdfTextBlock>) -> Self {
        Self {
            extracted_text: Some(extracted_text.into()),
            blocks,
        }
    }

    pub(crate) fn extracted_text(&self) -> Option<&str> {
        self.extracted_text.as_deref()
    }

    pub(crate) fn blocks(&self) -> &[PdfTextBlock] {
        &self.blocks
    }
}
