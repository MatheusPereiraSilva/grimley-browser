use serde::{Deserialize, Serialize};

use crate::{
    internal_pages::InternalPageKind,
    tabs::{BrowserTabs, TabLaunch},
};

use super::{AppStorage, StorageError};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct SessionSnapshot {
    #[serde(default)]
    pub(crate) active_index: usize,
    #[serde(default)]
    pub(crate) tabs: Vec<StoredTab>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum StoredTab {
    Web { url: String },
    Internal { page: StoredInternalPage },
    Pdf { origin_url: String },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StoredInternalPage {
    NewTab,
    History,
    Shield,
}

pub(crate) fn load_session(storage: &AppStorage) -> Result<Option<SessionSnapshot>, StorageError> {
    storage.read_json::<SessionSnapshot>(&storage.session_path())
}

pub(crate) fn save_session(
    storage: &AppStorage,
    browser_tabs: &BrowserTabs,
) -> Result<(), StorageError> {
    let snapshot = SessionSnapshot {
        active_index: browser_tabs.active_index(),
        tabs: browser_tabs
            .session_launches()
            .into_iter()
            .map(stored_tab_from_launch)
            .collect(),
    };

    storage.write_json(&storage.session_path(), &snapshot)
}

pub(crate) fn restore_tab_launches(snapshot: SessionSnapshot) -> (usize, Vec<TabLaunch>) {
    let launches = snapshot
        .tabs
        .into_iter()
        .map(tab_launch_from_stored)
        .collect::<Vec<_>>();

    (snapshot.active_index, launches)
}

fn stored_tab_from_launch(launch: TabLaunch) -> StoredTab {
    match launch {
        TabLaunch::Web { url } => StoredTab::Web { url },
        TabLaunch::Internal(document) => StoredTab::Internal {
            page: stored_internal_page_from_kind(document.kind()),
        },
        TabLaunch::Pdf { origin_url, .. } => StoredTab::Pdf { origin_url },
    }
}

fn tab_launch_from_stored(tab: StoredTab) -> TabLaunch {
    match tab {
        StoredTab::Web { url } => TabLaunch::regular(url),
        StoredTab::Internal { page } => TabLaunch::internal(internal_kind_from_stored(page)),
        StoredTab::Pdf { origin_url } => TabLaunch::pdf(origin_url),
    }
}

fn stored_internal_page_from_kind(kind: InternalPageKind) -> StoredInternalPage {
    match kind {
        InternalPageKind::NewTab => StoredInternalPage::NewTab,
        InternalPageKind::History => StoredInternalPage::History,
        InternalPageKind::Shield => StoredInternalPage::Shield,
    }
}

fn internal_kind_from_stored(page: StoredInternalPage) -> InternalPageKind {
    match page {
        StoredInternalPage::NewTab => InternalPageKind::NewTab,
        StoredInternalPage::History => InternalPageKind::History,
        StoredInternalPage::Shield => InternalPageKind::Shield,
    }
}
