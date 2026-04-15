mod history;
mod new_tab;
mod renderer;

pub(crate) const HISTORY_PAGE_URL: &str = "about:blank#grimley-history";
pub(crate) const NEW_TAB_PAGE_URL: &str = "about:blank#grimley-new-tab";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InternalPageKind {
    NewTab,
    History,
}

pub(crate) trait InternalPageRenderer {
    fn render(&self) -> String;
}

pub(crate) use history::{HistoryPage, VisitedPages};
pub(crate) use new_tab::render_new_tab_html;
pub(crate) use renderer::render_internal_page;
