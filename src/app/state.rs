use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{app::TabView, pdf::PdfWorkspaceState};

pub(crate) type PendingCommand = Arc<Mutex<Option<crate::app::UiCommand>>>;
pub(crate) type LoadedUrls = Arc<Mutex<Vec<(usize, String)>>>;
pub(crate) type PdfRoutes = Arc<Mutex<HashMap<usize, PdfWorkspaceState>>>;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UiSnapshot {
    pub(crate) url: String,
    pub(crate) can_go_back: bool,
    pub(crate) can_go_forward: bool,
    pub(crate) tabs: Vec<TabView>,
    pub(crate) active_index: usize,
}

pub(crate) fn create_pending_command() -> PendingCommand {
    Arc::new(Mutex::new(None))
}

pub(crate) fn create_loaded_urls() -> LoadedUrls {
    Arc::new(Mutex::new(Vec::new()))
}

pub(crate) fn create_pdf_routes() -> PdfRoutes {
    Arc::new(Mutex::new(HashMap::new()))
}
