#[derive(Clone, Debug)]
pub(crate) enum BrowserAction {
    Navigate(String),
    Back,
    Forward,
    ShowHistory,
    NewTab(Option<String>),
    OpenPdf(String),
    SwitchTab(usize),
    CloseTab(usize),
}
