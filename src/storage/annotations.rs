use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

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
    #[serde(other)]
    Unsupported,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct StoredPdfRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl StoredPdfAnnotation {
    fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported)
    }
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
        .map(|entry| {
            entry
                .annotations
                .iter()
                .filter(|annotation| annotation.is_supported())
                .cloned()
                .collect()
        })
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
            annotations: request
                .annotations
                .iter()
                .filter(|annotation| annotation.is_supported())
                .cloned()
                .collect(),
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::storage::AppStorage;

    use super::load_annotations_json;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let path = std::env::temp_dir().join(format!(
                "{prefix}-{}-{timestamp}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("Falha ao criar diretorio temporario de teste");

            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn ignores_legacy_ink_annotations_when_loading_json() {
        let temp_dir = TempDir::new("grimley-storage-annotations");
        let storage = AppStorage::from_root_dir(temp_dir.path.clone())
            .expect("Era esperado preparar o storage temporario");
        let path = storage.annotations_path();
        let pdf_url = "https://example.com/report.pdf";

        fs::write(
            &path,
            format!(
                concat!(
                    "{{",
                    "\"documents\":{{",
                    "\"{}\":{{",
                    "\"annotations\":[",
                    "{{\"type\":\"note\",\"id\":\"note-1\",\"page\":1,\"x\":0.12,\"y\":0.34,\"text\":\"Nota valida\"}},",
                    "{{\"type\":\"ink\",\"id\":\"ink-1\",\"page\":1,\"points\":[{{\"x\":0.5,\"y\":0.6}}]}}",
                    "],",
                    "\"updated_at_ms\":1",
                    "}}",
                    "}}",
                    "}}"
                ),
                pdf_url
            ),
        )
        .expect("Era esperado gravar o snapshot de anotacoes");

        let json = load_annotations_json(&storage, pdf_url)
            .expect("Era esperado carregar apenas anotacoes suportadas");

        assert!(json.contains("\"type\":\"note\""));
        assert!(!json.contains("\"type\":\"ink\""));
        assert!(!json.contains("unsupported"));
    }
}
