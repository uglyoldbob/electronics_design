//! This module stores general usage items.

use egui_multiwin::egui;
/// interacts with ui elements (rectangles), combining them all into a single response
pub fn respond(ui: &mut egui::Ui, id: String, rects: Vec<egui::Rect>) -> egui::Response {
    let mut resp = ui.interact(
        rects[0],
        egui::Id::from(format!("{}.{}", id, 0)),
        egui::Sense {
            click: true,
            drag: true,
            focusable: true,
        },
    );
    for (num, r) in rects.iter().skip(1).enumerate() {
        let num = num + 1;
        let resp2 = ui.interact(
            *r,
            egui::Id::from(format!("{}.{}", id, num)),
            egui::Sense {
                click: true,
                drag: true,
                focusable: true,
            },
        );
        resp = resp.union(resp2);
    }
    resp
}

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
    /// Convert the storage path to a user viewable representation of the option. Used to display the kinds of options available to users.
    pub fn display(&self) -> String {
        match self {
            StoragePath::LocalFilesystem(_) => "Local Filesystem".to_string(),
        }
    }

    /// Returns a path (if applicable) for the open command
    pub fn open_path(&self) -> Option<std::path::PathBuf> {
        match self {
            StoragePath::LocalFilesystem(p) => Some(std::path::PathBuf::from(p)),
        }
    }
}

impl std::fmt::Display for StoragePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StoragePath::LocalFilesystem(p) => format!("Local Filesystem {}", p),
            }
        )
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

impl std::fmt::Display for StorageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IoError(e) => e.to_string(),
                Self::DeserializeError(e) => e.clone(),
                Self::StoragePathError(s) => s.to_string(),
            }
        )
    }
}

/// The kinds of errors that can occur connecting to a storage path
#[derive(Debug)]
pub enum StoragePathError {
    /// A generic filesystem error
    IoError(std::io::Error),
}

impl From<std::io::Error> for StoragePathError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl std::fmt::Display for StoragePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IoError(e) => e.to_string(),
            }
        )
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

impl std::fmt::Display for StorageSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IoError(e) => e.to_string(),
                Self::SerializeError(e) => e.clone(),
                Self::StoragePathError(e) => e.to_string(),
            }
        )
    }
}

impl StoragePath {
    ///Create a writer for the storage path
    pub fn writer(&self) -> Result<impl std::io::Write, StoragePathError> {
        match self {
            Self::LocalFilesystem(pathname) => {
                let file = std::fs::OpenOptions::new()
                    .truncate(true)
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
                let file = std::fs::OpenOptions::new().read(true).open(pathname)?;
                Ok(file)
            }
        }
    }
}

/// Coordinates that can be used in the program
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(tag = "type", content = "args")]
pub enum Coordinates {
    /// Imperial inches. Specified in fractional inches
    Inches(f32, f32),
    /// Metric millimeters. Units are specified in fractional millimeters
    Millimeters(f32, f32),
}

impl std::ops::Sub for Coordinates {
    type Output = Coordinates;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Self::Inches(x, y) => match rhs {
                Self::Inches(x2, y2) => Self::Inches(x - x2, y - y2),
                Self::Millimeters(x2, y2) => Self::Inches(x - x2 * 25.4, y - y2 * 25.4),
            },
            Self::Millimeters(x, y) => match rhs {
                Self::Inches(x2, y2) => Self::Millimeters(x - x2 * 25.4, y - y2 * 25.4),
                Self::Millimeters(x2, y2) => Self::Millimeters(x - x2, y - y2),
            },
        }
    }
}

impl std::ops::AddAssign for Coordinates {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::SubAssign for Coordinates {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::Add for Coordinates {
    type Output = Coordinates;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Inches(x, y) => match rhs {
                Self::Inches(x2, y2) => Self::Inches(x + x2, y + y2),
                Self::Millimeters(x2, y2) => Self::Inches(x + x2 * 25.4, y + y2 * 25.4),
            },
            Self::Millimeters(x, y) => match rhs {
                Self::Inches(x2, y2) => Self::Millimeters(x + x2 * 25.4, y + y2 * 25.4),
                Self::Millimeters(x2, y2) => Self::Millimeters(x + x2, y + y2),
            },
        }
    }
}

impl Coordinates {
    /// Get the coordinates in millimeters
    pub fn get_mm(&self) -> (f32, f32) {
        match self {
            Self::Inches(x, y) => (x * 25.4, y * 25.4),
            Self::Millimeters(x, y) => (*x, *y),
        }
    }
    /// Get the coordinates in inches
    pub fn get_inches(&self) -> (f32, f32) {
        match self {
            Self::Inches(x, y) => (*x, *y),
            Self::Millimeters(x, y) => (x / 25.4, y / 25.4),
        }
    }
    /// Get coordinates according to the specified units
    pub fn get_units(&self, units: DisplayMode) -> (f32, f32) {
        match units {
            DisplayMode::Inches => self.get_inches(),
            DisplayMode::Millimeters => self.get_mm(),
        }
    }
    /// Get coordinates from screen position
    pub fn from_pos2(pos2: egui_multiwin::egui::Pos2, zoom: f32) -> Self {
        Self::Inches(pos2.x / zoom, pos2.y * -1.0 / zoom)
    }
    /// Convert coordinates to screen position in pixels
    pub fn get_pos2(
        &self,
        zoom: f32,
        zoom_center: egui_multiwin::egui::Pos2,
    ) -> egui_multiwin::egui::Pos2 {
        match self {
            Self::Inches(x, y) => {
                egui_multiwin::egui::pos2(*x * zoom, *y * -zoom) + zoom_center.to_vec2()
            }
            Self::Millimeters(x, y) => {
                egui_multiwin::egui::pos2(zoom * x / 25.4, -zoom * y / 25.4) + zoom_center.to_vec2()
            }
        }
    }
    /// Are both coordinates effectively 0?
    pub fn less_than_epsilon(&self) -> bool {
        match self {
            Self::Inches(x, y) => *x < f32::EPSILON && *y < f32::EPSILON,
            Self::Millimeters(x, y) => *x < f32::EPSILON && *y < f32::EPSILON,
        }
    }
    /// Did the x value change at all from the given value?
    pub fn changed_x(&self, p: f32) -> bool {
        match self {
            Self::Inches(x, _y) => (*x - p).abs() > f32::EPSILON,
            Self::Millimeters(x, _y) => (*x - p).abs() > f32::EPSILON,
        }
    }
    /// Did the y value change at all from the given value?
    pub fn changed_y(&self, p: f32) -> bool {
        match self {
            Self::Inches(_x, y) => (*y - p).abs() > f32::EPSILON,
            Self::Millimeters(_x, y) => (*y - p).abs() > f32::EPSILON,
        }
    }
}

/// A single dimension value of length
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Length {
    /// Imperial inches. Specified in fractional inches
    Inches(f32),
    /// Metric millimeters. Units are specified in fractional millimeters
    Millimeters(f32),
}

impl Length {
    /// Get the millimeters of this length
    pub fn get_mm(&self) -> f32 {
        match self {
            Self::Inches(i) => i * 25.4,
            Self::Millimeters(mm) => *mm,
        }
    }
    /// Get the inches of this length
    pub fn get_inches(&self) -> f32 {
        match self {
            Self::Inches(i) => *i,
            Self::Millimeters(mm) => mm / 25.4,
        }
    }
    /// Convert the length to screen units
    pub fn get_screen(&self, zoom: f32, _zoom_center: egui_multiwin::egui::Pos2) -> f32 {
        match self {
            Self::Inches(i) => *i * zoom,
            Self::Millimeters(mm) => zoom * *mm / 25.4,
        }
    }
}

/// The units mode for the program
#[derive(Copy, Clone)]
pub enum DisplayMode {
    /// Imperial inches
    Inches,
    /// Standard millimeters
    Millimeters,
}

/// The various modes of interpreting colors for the system
pub enum ColorMode {
    /// The colors for displaying on a screen in dark mode
    ScreenModeDark,
    /// The colors for displaying on a screen in light mode
    ScreenModeLight,
    /// The colors for printing to pdf
    PrintingMode,
}
