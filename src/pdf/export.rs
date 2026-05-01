#![allow(dead_code)]

use super::{annotations::PdfAnnotation, model::PdfDocument};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PdfExportFormat {
    LayeredPdf,
    PlainText,
    GrimleyProject,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PdfExportArtifact {
    filename: String,
    content_type: String,
    bytes: Vec<u8>,
}

impl PdfExportArtifact {
    pub(crate) fn new(
        filename: impl Into<String>,
        content_type: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            filename: filename.into(),
            content_type: content_type.into(),
            bytes,
        }
    }

    pub(crate) fn filename(&self) -> &str {
        &self.filename
    }

    pub(crate) fn content_type(&self) -> &str {
        &self.content_type
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub(crate) fn export_pdf_document(
    document: &PdfDocument,
    format: PdfExportFormat,
) -> Result<PdfExportArtifact, String> {
    match format {
        PdfExportFormat::LayeredPdf => Err(
            "A exportacao em PDF com camadas adicionais sera conectada quando a etapa de anotacoes renderizadas estiver pronta."
                .to_string(),
        ),
        PdfExportFormat::PlainText => Ok(export_plain_text(document)),
        PdfExportFormat::GrimleyProject => Ok(export_grimley_project(document)),
    }
}

fn export_plain_text(document: &PdfDocument) -> PdfExportArtifact {
    let content = match document
        .derived_text_session
        .as_ref()
        .and_then(|session| session.extracted_text())
    {
        Some(text) => text.to_string(),
        None => document
            .annotations
            .iter()
            .map(annotation_to_text)
            .collect::<Vec<_>>()
            .join("\n"),
    };

    PdfExportArtifact::new(
        format!("{}.txt", filename_stem(&document.source_url)),
        "text/plain; charset=utf-8",
        content.into_bytes(),
    )
}

fn export_grimley_project(document: &PdfDocument) -> PdfExportArtifact {
    let annotations = document
        .annotations
        .iter()
        .map(annotation_to_json)
        .collect::<Vec<_>>()
        .join(",");
    let extracted_text = document
        .derived_text_session
        .as_ref()
        .and_then(|session| session.extracted_text())
        .map(|text| format!("\"{}\"", escape_json(text)))
        .unwrap_or_else(|| "null".to_string());

    let project_json = format!(
        concat!(
            "{{",
            "\"sourceUrl\":\"{}\",",
            "\"pageCount\":{},",
            "\"annotations\":[{}],",
            "\"derivedText\":{}",
            "}}"
        ),
        escape_json(&document.source_url),
        document.pages.len(),
        annotations,
        extracted_text
    );

    PdfExportArtifact::new(
        format!("{}.grimley.json", filename_stem(&document.source_url)),
        "application/json; charset=utf-8",
        project_json.into_bytes(),
    )
}

fn annotation_to_text(annotation: &PdfAnnotation) -> String {
    match annotation {
        PdfAnnotation::Highlight {
            page,
            color,
            note,
            rects,
        } => {
            let note_suffix = note
                .as_deref()
                .map(|value| format!(" - {}", value))
                .unwrap_or_default();
            format!(
                "Highlight pagina {} em {} regioes [{}]{}",
                page,
                rects.len(),
                color,
                note_suffix
            )
        }
        PdfAnnotation::Note { page, text, .. } => format!("Nota pagina {}: {}", page, text),
    }
}

fn annotation_to_json(annotation: &PdfAnnotation) -> String {
    match annotation {
        PdfAnnotation::Highlight {
            page,
            color,
            note,
            rects,
        } => {
            let note = note
                .as_deref()
                .map(|value| format!("\"{}\"", escape_json(value)))
                .unwrap_or_else(|| "null".to_string());
            format!(
                concat!(
                    "{{",
                    "\"type\":\"highlight\",",
                    "\"page\":{},",
                    "\"color\":\"{}\",",
                    "\"rectCount\":{},",
                    "\"note\":{}",
                    "}}"
                ),
                page,
                escape_json(color),
                rects.len(),
                note
            )
        }
        PdfAnnotation::Note { page, x, y, text } => format!(
            concat!(
                "{{",
                "\"type\":\"note\",",
                "\"page\":{},",
                "\"x\":{},",
                "\"y\":{},",
                "\"text\":\"{}\"",
                "}}"
            ),
            page,
            x,
            y,
            escape_json(text)
        ),
    }
}

fn filename_stem(source_url: &str) -> String {
    source_url
        .rsplit('/')
        .next()
        .unwrap_or("documento.pdf")
        .trim_end_matches(".pdf")
        .to_string()
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
