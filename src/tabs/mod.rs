mod history;
mod manager;
mod model;

pub(crate) use history::BrowserHistory;
pub(crate) use manager::BrowserTabs;
pub(crate) use model::{TabLaunch, TabRenderRequest, TabSession};
