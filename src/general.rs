//! This module stores general usage items.

/// The kinds of file formats that can be used for various files that are exported
#[derive(Clone)]
pub enum StorageFormat {
    /// The toml format
    Toml,
}

impl Default for StorageFormat {
    fn default() -> Self {
        Self::Toml
    }
}

impl StorageFormat {
    /// Save the object using whatever writer is given
    pub fn save<T>(
        &self,
        writer: &mut impl std::io::Write,
        object: &T,
    ) -> Result<(), StorageSaveError>
    where
        T: serde::Serialize,
    {
        match self {
            Self::Toml => match toml::to_string(object) {
                Ok(obj) => Ok(writer.write_all(obj.as_bytes())?),
                Err(e) => Err(StorageSaveError::SerializeError(e.to_string())),
            },
        }
    }

    /// Load the object using whatever reader is given
    pub fn load<T>(&self, reader: &mut impl std::io::Read) -> Result<T, StorageLoadError>
    where
        T: for<'a> serde::Deserialize<'a>,
    {
        match self {
            Self::Toml => {
                let mut buffer = vec![];
                match reader.read_to_end(&mut buffer) {
                    Ok(_len) => match std::str::from_utf8(&buffer) {
                        Ok(data) => Ok(toml::from_str(data)?),
                        Err(_) => todo!(),
                    },
                    Err(e) => Err(StorageLoadError::IoError(e)),
                }
            }
        }
    }
}

/// The ways design files can be saved
#[derive(strum::EnumIter, PartialEq, Clone)]
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

/// The kinds of errors that can occur when loading from storage
pub enum StorageLoadError {
    /// A filesystem error of some sort occurred.
    IoError(std::io::Error),
    /// An error occurred deserializing
    DeserializeError(String),
    /// A storage path error occurred
    StoragePathError(StoragePathError),
}

impl From<std::io::Error> for StorageLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<toml::de::Error> for StorageLoadError {
    fn from(value: toml::de::Error) -> Self {
        Self::DeserializeError(value.to_string())
    }
}

impl From<StoragePathError> for StorageLoadError {
    fn from(value: StoragePathError) -> Self {
        Self::StoragePathError(value)
    }
}

impl StorageLoadError {
    /// Convert the error to a string
    pub fn to_string(&self) -> String {
        match self {
            Self::IoError(e) => e.to_string(),
            Self::DeserializeError(e) => e.clone(),
            Self::StoragePathError(s) => s.to_string(),
        }
    }
}

/// The kinds of errors that can occur connecting to a storage path
pub enum StoragePathError {
    /// A generic filesystem error
    IoError(std::io::Error),
}

impl From<std::io::Error> for StoragePathError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl StoragePathError {
    /// Convert the error to a string
    pub fn to_string(&self) -> String {
        match self {
            Self::IoError(e) => e.to_string(),
        }
    }
}

/// The kinds of errors that can occur when saving to storage
pub enum StorageSaveError {
    /// A filesystem error of some sort occurred.
    IoError(std::io::Error),
    /// An error occurred serializing the data
    SerializeError(String),
    /// A storage path error occurred
    StoragePathError(StoragePathError),
}

impl From<std::io::Error> for StorageSaveError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<StoragePathError> for StorageSaveError {
    fn from(value: StoragePathError) -> Self {
        Self::StoragePathError(value)
    }
}

impl StorageSaveError {
    /// Convert the error to a string
    pub fn to_string(&self) -> String {
        match self {
            Self::IoError(e) => e.to_string(),
            Self::SerializeError(e) => e.clone(),
            Self::StoragePathError(e) => e.to_string(),
        }
    }
}

impl StoragePath {
    ///Create a writer for the storage path
    pub fn writer(&self) -> Result<impl std::io::Write, StoragePathError> {
        match self {
            Self::LocalFilesystem(pathname) => {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(pathname)?;
                Ok(file)
            }
        }
    }

    ///Create a reader for the storage path
    pub fn reader(&self) -> Result<impl std::io::Read, StoragePathError> {
        match self {
            Self::LocalFilesystem(pathname) => {
                let file = std::fs::OpenOptions::new()
                    .create(false)
                    .write(false)
                    .open(pathname)?;
                Ok(file)
            }
        }
    }
}
