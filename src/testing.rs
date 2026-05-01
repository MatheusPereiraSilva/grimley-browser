use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    browser::{is_pdf_url, normalize_url},
    internal_pages::{
        internal_page_kind_for_url, render_internal_page_html, HistoryVisit, InternalPageKind,
        VisitedPages,
    },
    pdf::{
        export_pdf_document, open_pdf_workspace, render_pdf_workspace_html, PdfAnnotation,
        PdfDerivedTextSession, PdfDocument, PdfExportFormat, PdfSource, PdfTextBlock,
        PdfWorkspaceMode,
    },
    shield::{create_shield_engine, ShieldEngineHandle},
    storage::{
        annotations::{
            load_annotations_json, save_annotations, PdfAnnotationsSaveRequest, StoredPdfAnnotation,
        },
        AppStorage,
    },
    tabs::{
        launch_for_pdf_url, launch_for_requested_url, Tab, TabContentKind, TabLaunch,
        TabRenderRequest,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarnessTabKind {
    Web,
    Internal,
    Pdf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportedDocument {
    pub filename: String,
    pub content_type: String,
    pub body: String,
}

pub struct BrowserHarness {
    tabs: Vec<Tab>,
    active_index: usize,
    next_tab_id: usize,
    visited_pages: VisitedPages,
    shield_engine: ShieldEngineHandle,
}

impl BrowserHarness {
    pub fn new(home_page_url: &str) -> Self {
        Self {
            tabs: vec![Tab::from_launch(0, TabLaunch::regular(home_page_url), true)],
            active_index: 0,
            next_tab_id: 1,
            visited_pages: VisitedPages::default(),
            shield_engine: create_shield_engine(),
        }
    }

    pub fn navigate(&mut self, raw_url: &str) -> String {
        let final_url = normalize_url(raw_url);

        if let Some(kind) = internal_page_kind_for_url(&final_url) {
            self.active_tab_mut().show_internal_page(kind);
            final_url
        } else if is_pdf_url(&final_url) && !self.active_tab().allows_pdf_navigation() {
            self.active_tab_mut().show_pdf_workspace(final_url.clone());
            final_url
        } else {
            let target = self.active_tab_mut().navigate_to_web(&final_url);
            self.record_history(&target);
            target
        }
    }

    pub fn open_new_tab(&mut self, url: Option<&str>) -> usize {
        let tab = Tab::from_launch(self.next_tab_id, launch_for_requested_url(url), true);
        self.tabs.push(tab);
        self.next_tab_id += 1;
        self.active_index = self.tabs.len() - 1;
        self.active_index
    }

    pub fn popup_new_tab(&mut self, url: &str) -> usize {
        self.open_new_tab(Some(url))
    }

    pub fn open_pdf_tab(&mut self, url: &str) -> usize {
        let tab = Tab::from_launch(self.next_tab_id, launch_for_pdf_url(url), true);
        self.tabs.push(tab);
        self.next_tab_id += 1;
        self.active_index = self.tabs.len() - 1;
        self.active_index
    }

    pub fn back(&mut self) -> Option<String> {
        let target = self.active_tab_mut().go_back();
        if let Some(url) = &target {
            self.record_history(url);
        }
        target
    }

    pub fn forward(&mut self) -> Option<String> {
        let target = self.active_tab_mut().go_forward();
        if let Some(url) = &target {
            self.record_history(url);
        }
        target
    }

    pub fn open_history(&mut self) {
        self.active_tab_mut().show_history_page();
    }

    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() || index == self.active_index {
            return false;
        }

        self.active_index = index;
        true
    }

    pub fn close_active_tab(&mut self) {
        self.close_tab(self.active_index);
    }

    pub fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }

        let was_active = index == self.active_index;
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.tabs.push(Tab::from_launch(
                self.next_tab_id,
                TabLaunch::new_tab(),
                true,
            ));
            self.next_tab_id += 1;
            self.active_index = 0;
            return;
        }

        if self.active_index > index {
            self.active_index -= 1;
        } else if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len() - 1;
        }

        if was_active && self.active_index < self.tabs.len() {
            self.tabs[self.active_index].mark_needs_render();
        }
    }

    pub fn render_active_page(&mut self) -> Option<String> {
        let active_index = self.active_index;
        let tab_id = self.tabs[active_index].id();
        let request = self.tabs[active_index].take_render_request()?;

        let html = match request {
            TabRenderRequest::Internal { kind } => {
                render_internal_page_html(kind, &self.visited_pages, self.shield_engine.clone())
            }
            TabRenderRequest::Pdf(document) => render_pdf_workspace_html(tab_id, &document, "[]"),
        };

        self.tabs[active_index].mark_rendered();
        Some(html)
    }

    pub fn active_url(&self) -> String {
        self.active_tab().display_url()
    }

    pub fn active_kind(&self) -> HarnessTabKind {
        tab_kind(self.active_tab().content_kind())
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn tab_urls(&self) -> Vec<String> {
        self.tabs.iter().map(Tab::display_url).collect()
    }

    pub fn history_len(&self) -> usize {
        self.visited_pages.entries().len()
    }

    pub fn history_urls(&self) -> Vec<String> {
        self.visited_pages
            .entries()
            .iter()
            .map(|entry| entry.url.clone())
            .collect()
    }

    fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_index]
    }

    fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_index]
    }

    fn record_history(&mut self, url: &str) {
        if !is_pdf_url(url) {
            self.visited_pages.record(url);
        }
    }
}

pub fn render_history_page(urls: &[&str]) -> String {
    let visited_pages = VisitedPages::from_entries(
        urls.iter()
            .map(|url| HistoryVisit {
                url: (*url).to_string(),
            })
            .collect(),
    );

    render_internal_page_html(
        InternalPageKind::History,
        &visited_pages,
        create_shield_engine(),
    )
}

pub fn render_pdf_workspace_with_saved_note(
    pdf_url: &str,
    note_text: &str,
) -> Result<String, String> {
    let _temp_dir = TempDir::new("grimley-annotations");
    let storage =
        AppStorage::from_root_dir(_temp_dir.path.clone()).map_err(|error| error.to_string())?;

    save_annotations(
        &storage,
        &PdfAnnotationsSaveRequest {
            pdf_url: pdf_url.to_string(),
            annotations: vec![StoredPdfAnnotation::Note {
                id: "note-1".to_string(),
                page: 1,
                x: 48.0,
                y: 96.0,
                text: note_text.to_string(),
            }],
        },
    )
    .map_err(|error| error.to_string())?;

    let annotations_json =
        load_annotations_json(&storage, pdf_url).map_err(|error| error.to_string())?;
    let workspace = open_pdf_workspace(0, pdf_url, PdfWorkspaceMode::AnnotateOriginal);

    Ok(render_pdf_workspace_html(0, &workspace, &annotations_json))
}

pub fn export_document_with_derived_text(
    source_url: &str,
    derived_text: &str,
    format: TestExportFormat,
) -> Result<ExportedDocument, String> {
    let source = PdfSource::remote_url(source_url);
    let mut document = PdfDocument::new(7, &source, vec![1, 2, 3]).with_page_count(3);
    document.annotations.push(PdfAnnotation::Note {
        page: 1,
        x: 12.0,
        y: 24.0,
        text: "Resumo original".to_string(),
    });
    document.derived_text_session = Some(PdfDerivedTextSession::new(
        derived_text,
        vec![PdfTextBlock::new(1, derived_text)],
    ));

    let export_format = match format {
        TestExportFormat::PlainText => PdfExportFormat::PlainText,
        TestExportFormat::Project => PdfExportFormat::GrimleyProject,
    };
    let artifact = export_pdf_document(&document, export_format)?;
    let body = String::from_utf8(artifact.bytes().to_vec())
        .map_err(|error| format!("Falha ao interpretar bytes exportados: {error}"))?;

    Ok(ExportedDocument {
        filename: artifact.filename().to_string(),
        content_type: artifact.content_type().to_string(),
        body,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestExportFormat {
    PlainText,
    Project,
}

fn tab_kind(kind: TabContentKind) -> HarnessTabKind {
    match kind {
        TabContentKind::Web => HarnessTabKind::Web,
        TabContentKind::Internal => HarnessTabKind::Internal,
        TabContentKind::Pdf => HarnessTabKind::Pdf,
    }
}

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
