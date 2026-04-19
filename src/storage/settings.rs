use serde::{Deserialize, Serialize};

use crate::browser::HOME_PAGE_URL;

use super::{AppStorage, StorageError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ShieldSettings {
    #[serde(default = "default_true")]
    pub(crate) observation_only: bool,
    #[serde(default)]
    pub(crate) custom_rules: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    #[serde(default = "default_home_page_url")]
    pub(crate) home_page_url: String,
    #[serde(default = "default_true")]
    pub(crate) restore_last_session: bool,
    #[serde(default)]
    pub(crate) shield: ShieldSettings,
}

impl Default for ShieldSettings {
    fn default() -> Self {
        Self {
            observation_only: true,
            custom_rules: String::new(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            home_page_url: default_home_page_url(),
            restore_last_session: true,
            shield: ShieldSettings::default(),
        }
    }
}

pub(crate) fn load_settings(storage: &AppStorage) -> Result<AppSettings, StorageError> {
    Ok(storage
        .read_json::<AppSettings>(&storage.settings_path())?
        .unwrap_or_default())
}

pub(crate) fn save_settings(
    storage: &AppStorage,
    settings: &AppSettings,
) -> Result<(), StorageError> {
    storage.write_json(&storage.settings_path(), settings)
}

fn default_home_page_url() -> String {
    HOME_PAGE_URL.to_string()
}

fn default_true() -> bool {
    true
}
