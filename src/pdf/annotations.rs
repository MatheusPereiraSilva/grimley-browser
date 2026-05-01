#![allow(dead_code)]

use super::model::Rect;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PdfAnnotation {
    Highlight {
        page: u32,
        rects: Vec<Rect>,
        color: String,
        note: Option<String>,
    },
    Note {
        page: u32,
        x: f32,
        y: f32,
        text: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PdfSelection {
    page: u32,
    rects: Vec<Rect>,
    text: String,
}

impl PdfAnnotation {
    pub(crate) fn page(&self) -> u32 {
        match self {
            Self::Highlight { page, .. } | Self::Note { page, .. } => *page,
        }
    }
}

impl PdfSelection {
    pub(crate) fn new(page: u32, rects: Vec<Rect>, text: impl Into<String>) -> Self {
        Self {
            page,
            rects,
            text: text.into(),
        }
    }

    pub(crate) fn page(&self) -> u32 {
        self.page
    }

    pub(crate) fn rects(&self) -> &[Rect] {
        &self.rects
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }
}

pub(crate) fn annotations_for_page<'a>(
    annotations: &'a [PdfAnnotation],
    page: u32,
) -> impl Iterator<Item = &'a PdfAnnotation> + 'a {
    annotations
        .iter()
        .filter(move |annotation| annotation.page() == page)
}
