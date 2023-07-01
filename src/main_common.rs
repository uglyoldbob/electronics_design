//! Includes items common to all programs created

/// The name of the overall software package
pub const PACKAGE_NAME: &str = "UglyOldBob Electronics";

lazy_static::lazy_static! {
    pub static ref DIRS: Option<directories::ProjectDirs> = directories::ProjectDirs::from("com", "UglyOldBob", "ElectronicsDesign");
}
