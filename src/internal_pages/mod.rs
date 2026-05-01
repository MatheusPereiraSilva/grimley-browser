mod history;
mod new_tab;
mod renderer;
mod shield;

pub(crate) const HISTORY_PAGE_URL: &str = "about:blank#grimley-history";
pub(crate) const NEW_TAB_PAGE_URL: &str = "about:blank#grimley-new-tab";
pub(crate) const SHIELD_PAGE_URL: &str = "about:blank#grimley-shield";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InternalPageKind {
    NewTab,
    History,
    Shield,
}

pub(crate) trait InternalPageRenderer {
    fn render(&self) -> String;
}

pub(crate) use history::{HistoryPage, HistoryVisit, VisitedPages};
pub(crate) use new_tab::render_new_tab_html;
pub(crate) use renderer::{render_internal_page, render_internal_page_html};
pub(crate) use shield::ShieldPage;

pub(crate) fn internal_page_kind_for_url(url: &str) -> Option<InternalPageKind> {
    match url.trim().to_ascii_lowercase().as_str() {
        "grimley://nova-aba" => Some(InternalPageKind::NewTab),
        "grimley://historico" => Some(InternalPageKind::History),
        "grimley://shield" => Some(InternalPageKind::Shield),
        _ => None,
    }
}
