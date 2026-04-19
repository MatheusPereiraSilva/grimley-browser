pub(crate) mod annotations;
pub(crate) mod history;
pub(crate) mod session;
pub(crate) mod settings;

use std::{
    fs,
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Clone, Debug)]
pub(crate) struct AppStorage {
    root_dir: PathBuf,
}

#[derive(Debug, Error)]
pub(crate) enum StorageError {
    #[error("Nao foi possivel preparar o diretorio de storage em {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Nao foi possivel ler o arquivo {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Nao foi possivel escrever o arquivo {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Nao foi possivel interpretar JSON em {path}: {source}")]
    ParseJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("Nao foi possivel serializar JSON para {path}: {source}")]
    SerializeJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub(crate) fn create_app_storage() -> Result<AppStorage, StorageError> {
    AppStorage::new()
}

impl AppStorage {
    pub(crate) fn new() -> Result<Self, StorageError> {
        let root_dir = resolve_storage_root();

        fs::create_dir_all(&root_dir).map_err(|source| StorageError::CreateDir {
            path: root_dir.clone(),
            source,
        })?;

        Ok(Self { root_dir })
    }

    pub(crate) fn settings_path(&self) -> PathBuf {
        self.root_dir.join("settings.json")
    }

    pub(crate) fn history_path(&self) -> PathBuf {
        self.root_dir.join("history.json")
    }

    pub(crate) fn session_path(&self) -> PathBuf {
        self.root_dir.join("session.json")
    }

    pub(crate) fn annotations_path(&self) -> PathBuf {
        self.root_dir.join("pdf_annotations.json")
    }

    pub(crate) fn read_json<T>(&self, path: &std::path::Path) -> Result<Option<T>, StorageError>
    where
        T: DeserializeOwned,
    {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path).map_err(|source| StorageError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        let value = serde_json::from_str(&content).map_err(|source| StorageError::ParseJson {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(Some(value))
    }

    pub(crate) fn write_json<T>(&self, path: &std::path::Path, value: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StorageError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let content =
            serde_json::to_string_pretty(value).map_err(|source| StorageError::SerializeJson {
                path: path.to_path_buf(),
                source,
            })?;

        fs::write(path, content).map_err(|source| StorageError::WriteFile {
            path: path.to_path_buf(),
            source,
        })
    }
}

fn resolve_storage_root() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("io", "Grimley", "Grimley Browser") {
        project_dirs.data_local_dir().to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".grimley")
    }
}
