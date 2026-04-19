use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use serde::{Deserialize, Serialize};

use super::{AppStorage, StorageError};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PdfAnnotationsSnapshot {
    #[serde(default)]
    documents: HashMap<String, StoredPdfDocumentAnnotations>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct StoredPdfDocumentAnnotations {
    #[serde(default)]
    annotations: Vec<StoredPdfAnnotation>,
    #[serde(default)]
    updated_at_ms: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PdfAnnotationsSaveRequest {
    pub(crate) pdf_url: String,
    #[serde(default)]
    pub(crate) annotations: Vec<StoredPdfAnnotation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) enum StoredPdfAnnotation {
    Highlight {
        id: String,
        page: u32,
        #[serde(default)]
        rects: Vec<StoredPdfRect>,
        #[serde(default)]
        text: String,
        #[serde(default)]
        note: String,
    },
    Note {
        id: String,
        page: u32,
        x: f32,
        y: f32,
        #[serde(default)]
        text: String,
    },
    Ink {
        id: String,
        page: u32,
        #[serde(default)]
        points: Vec<StoredPdfPoint>,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct StoredPdfRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct StoredPdfPoint {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

pub(crate) fn load_annotations_json(
    storage: &AppStorage,
    pdf_url: &str,
) -> Result<String, StorageError> {
    let annotations = load_annotations(storage, pdf_url)?;

    serde_json::to_string(&annotations).map_err(|source| StorageError::SerializeJson {
        path: storage.annotations_path(),
        source,
    })
}

pub(crate) fn load_annotations(
    storage: &AppStorage,
    pdf_url: &str,
) -> Result<Vec<StoredPdfAnnotation>, StorageError> {
    let snapshot = storage
        .read_json::<PdfAnnotationsSnapshot>(&storage.annotations_path())?
        .unwrap_or_default();

    Ok(snapshot
        .documents
        .get(pdf_url)
        .map(|entry| entry.annotations.clone())
        .unwrap_or_default())
}

pub(crate) fn save_annotations(
    storage: &AppStorage,
    request: &PdfAnnotationsSaveRequest,
) -> Result<(), StorageError> {
    let path = storage.annotations_path();
    let mut snapshot = storage
        .read_json::<PdfAnnotationsSnapshot>(&path)?
        .unwrap_or_default();

    snapshot.documents.insert(
        request.pdf_url.clone(),
        StoredPdfDocumentAnnotations {
            annotations: request.annotations.clone(),
            updated_at_ms: now_epoch_ms(),
        },
    );

    storage.write_json(&path, &snapshot)
}

fn now_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
