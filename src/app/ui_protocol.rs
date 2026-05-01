use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum UiCommand {
    Navigate {
        url: String,
    },
    Back,
    Forward,
    NewTab {
        #[serde(default)]
        url: Option<String>,
    },
    CloseTab {
        index: usize,
    },
    SwitchTab {
        index: usize,
    },
    OpenHistory,
    OpenPdf {
        url: String,
    },
}

impl UiCommand {
    pub(crate) fn parse(message: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(message)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct TabView {
    pub(crate) label: String,
    pub(crate) url: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum UiEvent {
    UrlChanged { url: String },
    NavState { back: bool, forward: bool },
    TabsChanged { tabs: Vec<TabView>, active: usize },
}

impl UiEvent {
    pub(crate) fn to_script(&self) -> String {
        let payload = serde_json::to_string(self).expect("Falha ao serializar evento da UI");
        format!("window.__GRIMLEY_UI__.handleEvent({payload});")
    }
}
