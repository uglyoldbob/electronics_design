//! This module stores general usage items.

use std::io::Write;

/// The ways design files can be saved
#[derive(strum::EnumIter, PartialEq, Debug)]
pub enum StoragePath {
    /// The design file is saved to a file on a local filesystem
    LocalFilesystem(String),
}

impl StoragePath {
    /// Convert the storage path to a user viewable representation of the option
    pub fn display(&self) -> String {
        match self {
            StoragePath::LocalFilesystem(_) => "Local Filesystem".to_string(),
        }
    }
}

impl Default for StoragePath {
    fn default() -> Self {
        StoragePath::LocalFilesystem("".to_string())
    }
}

/// The kinds of errors that can occur when saving to storage
pub enum StorageError {
    /// A filesystem error of some sort occurred.
    IoError(std::io::Error),
}

impl From<std::io::Error> for StorageError {
    fn from(value: std::io::Error) -> Self {
        StorageError::IoError(value)
    }
}

impl StorageError {
    /// Convert the error to a string
    pub fn to_string(&self) -> String {
        match self {
            StorageError::IoError(e) => e.to_string(),
        }
    }
}

impl StoragePath {
    /// Save the given data to the path
    pub fn save(&self, data: &[u8]) -> Result<(), StorageError> {
        match self {
            Self::LocalFilesystem(pathname) => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(pathname)?;
                file.write_all(data)?;
                Ok(())
            }
        }
    }
}

/// Messages that can be sent between processes
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[non_exhaustive]
pub enum IpcMessage {
    /// Create a new library editor
    NewLibrary,
    /// Create a new schematic window
    NewSchematic,
}
