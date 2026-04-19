use serde::{Deserialize, Serialize};

use crate::internal_pages::{HistoryVisit, VisitedPages};

use super::{AppStorage, StorageError};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct HistorySnapshot {
    #[serde(default)]
    entries: Vec<HistoryVisit>,
}

pub(crate) fn load_history(storage: &AppStorage) -> Result<VisitedPages, StorageError> {
    let snapshot = storage
        .read_json::<HistorySnapshot>(&storage.history_path())?
        .unwrap_or_default();

    Ok(VisitedPages::from_entries(snapshot.entries))
}

pub(crate) fn save_history(
    storage: &AppStorage,
    visited_pages: &VisitedPages,
) -> Result<(), StorageError> {
    let snapshot = HistorySnapshot {
        entries: visited_pages.entries().to_vec(),
    };

    storage.write_json(&storage.history_path(), &snapshot)
}
