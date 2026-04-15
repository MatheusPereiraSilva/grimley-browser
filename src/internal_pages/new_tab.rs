use super::InternalPageRenderer;

const NEW_TAB_HTML: &str = include_str!("templates/new_tab.html");

pub(crate) struct NewTabPage;

impl InternalPageRenderer for NewTabPage {
    fn render(&self) -> String {
        NEW_TAB_HTML.to_string()
    }
}

pub(crate) fn render_new_tab_html() -> String {
    let renderer = NewTabPage;
    renderer.render()
}
